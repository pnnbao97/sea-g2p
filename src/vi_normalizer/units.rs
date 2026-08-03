//! Measurement units and currencies — pipeline stage 13.
//!
//! # Single uppercase letters are identifiers by default
//!
//! A letter glued to a number carries two competing meanings: a unit or
//! magnitude ("5M" = five million) versus a serial code ("căn hộ 51M"). Vietnamese
//! writes both constantly, so the default is the safer one — **spell the letter
//! out** — and a unit reading requires positive evidence:
//!
//!   - a **decimal point** ("3.2M"), since codes have no fractional part;
//!   - a **money cue** before or after for M/B/K ("lương 20M", "5M USD");
//!   - a **container word** before L ("chai 2L").
//!
//! Two exceptions: `W` is always watts, as no common serial uses it, and `G` is
//! never grams, because "5G"/"4G" dominate real text. Lowercase always means the
//! unit ("24h", "450g", "30m").
//!
//! # Ordering
//!
//! This stage runs after every sign and numeric cluster is settled, because each
//! rule keys on the number immediately to its left.

use fancy_regex::{Regex, Captures};
use once_cell::sync::Lazy;
use crate::vi_normalizer::num2vi::{n2w, n2w_decimal};
use crate::vi_normalizer::resources::{MEASUREMENT_KEY_VI, CURRENCY_KEY, CURRENCY_SYMBOL_MAP, VI_LETTER_NAMES};

// ── Number helpers ──────────────────────────────────────────────────────────

fn expand_scientific(num_str: &str) -> String {
    let num_lower = num_str.to_lowercase();
    let e_idx = num_lower.find('e').unwrap();
    let base = &num_str[..e_idx];
    let exp = &num_str[e_idx + 1..];

    let base_norm = if base.contains('.') {
        let parts: Vec<&str> = base.split('.').collect();
        let dec_part = parts[1].trim_end_matches('0');
        if !dec_part.is_empty() {
            format!("{} chấm {}", n2w(parts[0]), n2w_decimal(dec_part))
        } else {
            n2w(parts[0])
        }
    } else if base.contains(',') {
        let parts: Vec<&str> = base.split(',').collect();
        let dec_part = parts[1].trim_end_matches('0');
        if !dec_part.is_empty() {
            format!("{} phẩy {}", n2w(parts[0]), n2w_decimal(dec_part))
        } else {
            n2w(parts[0])
        }
    } else {
        n2w(&base.replace(',', "").replace('.', ""))
    };

    let exp_val = exp.trim_start_matches('+');
    let exp_norm = if exp_val.starts_with('-') {
        format!("trừ {}", n2w(&exp_val[1..]))
    } else {
        n2w(exp_val)
    };
    format!("{} nhân mười mũ {}", base_norm, exp_norm)
}

fn expand_mixed_sep(num_str: &str) -> String {
    let parts_owned: Vec<String>;
    let r_dot = num_str.rfind('.').unwrap_or(0);
    let r_comma = num_str.rfind(',').unwrap_or(0);

    if r_dot > r_comma {
        parts_owned = num_str.replace(',', "").split('.').map(|s: &str| s.to_string()).collect();
    } else {
        parts_owned = num_str.replace('.', "").split(',').map(|s: &str| s.to_string()).collect();
    }

    if parts_owned.len() < 2 { return n2w(&num_str.replace(',', "").replace('.', "")); }
    let dec_part = parts_owned[1].trim_end_matches('0');
    if dec_part.is_empty() {
        n2w(&parts_owned[0])
    } else {
        format!("{} phẩy {}", n2w(&parts_owned[0]), n2w_decimal(dec_part))
    }
}

fn expand_single_sep(num_str: &str) -> String {
    if num_str.contains(',') {
        let parts: Vec<&str> = num_str.split(',').collect();
        if parts.len() > 2 || (parts.len() == 2 && parts[1].len() == 3) {
            return n2w(&num_str.replace(',', ""));
        }
        let dec_part = parts[1].trim_end_matches('0');
        if dec_part.is_empty() {
            return n2w(parts[0]);
        }
        return format!("{} phẩy {}", n2w(parts[0]), n2w_decimal(dec_part));
    }

    let parts: Vec<&str> = num_str.split('.').collect();
    if parts.len() > 2 || (parts.len() == 2 && parts[1].len() == 3) {
        return n2w(&num_str.replace('.', ""));
    }
    let dec_part = parts[1].trim_end_matches('0');
    if dec_part.is_empty() {
        return n2w(parts[0]);
    }
    return format!("{} chấm {}", n2w(parts[0]), n2w_decimal(dec_part));
}

pub fn expand_number_with_sep(num_str: &str) -> String {
    if num_str.is_empty() { return String::new(); }
    if num_str.to_lowercase().contains('e') {
        return expand_scientific(num_str);
    }
    if num_str.contains(',') && num_str.contains('.') {
        return expand_mixed_sep(num_str);
    }
    if num_str.contains(',') || num_str.contains('.') {
        return expand_single_sep(num_str);
    }
    n2w(num_str)
}

// ── Unit / currency maps ────────────────────────────────────────────────────

static ALL_UNITS_MAP: Lazy<std::collections::HashMap<String, String>> = Lazy::new(|| {
    let mut m = std::collections::HashMap::new();
    for (k, v) in MEASUREMENT_KEY_VI.iter() {
        m.insert(k.to_lowercase(), v.to_string());
    }
    for (k, v) in CURRENCY_KEY.iter() {
        if *k != "%" {
            m.insert(k.to_lowercase(), v.to_string());
        }
    }
    m.insert("m".to_string(), "mét".to_string());
    m
});

static UNITS_RE_PATTERN: Lazy<String> = Lazy::new(|| {
    let mut keys: Vec<String> = MEASUREMENT_KEY_VI.keys().map(|&k: &&str| k.to_string()).collect();
    for &k in CURRENCY_KEY.keys() {
        if k != "%" { keys.push(k.to_string()); }
    }
    keys.sort_by_key(|b: &String| std::cmp::Reverse(b.len()));
    keys.iter().map(|k: &String| regex::escape(k)).collect::<Vec<String>>().join("|")
});

const NUMERIC_P: &str = r"(\d+(?:[.,]\d+)*)";
const MAGNITUDE_P: &str = r"(?:\s*(tỷ|triệu|nghìn|ngàn))?";

static RE_COMPOUND_UNIT: Lazy<Regex> = Lazy::new(|| {
    Regex::new(&format!(r"(?i)\b{}?\s*([a-zμµ²³°]+)/([a-zμµ²³°0-9]+)\b", NUMERIC_P)).unwrap()
});

static RE_UNITS_WITH_NUM: Lazy<Regex> = Lazy::new(|| {
    Regex::new(&format!(r"(?i)(?<![a-zA-Z\d.,]){}{}\s*({})\b", NUMERIC_P, MAGNITUDE_P, *UNITS_RE_PATTERN)).unwrap()
});

static RE_STANDALONE_UNIT: Lazy<Regex> = Lazy::new(|| {
    let safe = ["km", "cm", "mm", "kg", "mg", "usd", "vnd", "ph"];
    Regex::new(&format!(r"(?i)(?<![\d.,])\b({})\b", safe.join("|"))).unwrap()
});

static RE_CURRENCY_PREFIX_SYMBOL: Lazy<Regex> = Lazy::new(|| {
    // English magnitude suffixes written flush against the number: "$5M" is
    // millions, "$1.5B" billions, "$99K" thousands. Requiring adjacency plus a
    // word boundary stops the rule eating the first letter of the next word
    // ("₩1000 mỗi…" must not lose its "m").
    Regex::new(&format!(r"(?i)([$€¥£₩₫])\s*{}{}(?:([MBK])\b)?", NUMERIC_P, MAGNITUDE_P)).unwrap()
});

static RE_CURRENCY_SUFFIX_SYMBOL: Lazy<Regex> = Lazy::new(|| {
    Regex::new(&format!(r"(?i){}{}([$€¥£₩₫])", NUMERIC_P, MAGNITUDE_P)).unwrap()
});

static RE_PERCENTAGE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(&format!(r"(?i){}\s*%", NUMERIC_P)).unwrap()
});

static RE_ENGLISH_STYLE_NUMBERS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\b\d{1,3}(?:,\d{3})+(?:\.\d+)?\b").unwrap()
});

static RE_POWER_OF_TEN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\b(\d+(?:[.,]\d+)?)\s*[x*×]\s*10\^([-+]?\d+)\b").unwrap()
});

static RE_SCIENTIFIC_NOTATION: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)([-\u2013\u2014])?(\d+(?:[.,]\d+)?e[+-]?\d+)").unwrap()
});

// Vietnamese height notation: "1m75" -> "một mét bảy mươi lăm", "1m8" -> "một
// mét tám". The 'm' is lowercase-only, with no (?i), so "1M" (one million) is
// not swallowed. The tail is any two digits, or one digit OTHER than 2 or 3,
// which leaves "m2" and "m3" to the area and volume units.
static RE_HEIGHT: Lazy<Regex> = Lazy::new(|| {
    // The lookahead blocks [.,] only when a digit follows, i.e. a real decimal
    // ("1m75.5"). A sentence-final period ("Cao 1m75.") must still match.
    Regex::new(r"(?<![\d.,a-zA-Z])(\d{1,2})m(\d{2}|[01456789])(?!\d|[.,]\d)").unwrap()
});

// Vietnamese weight notation: "1kg2" -> "một ki lô gam hai", where the trailing
// 2 counts hectograms ("lạng").
static RE_WEIGHT: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?<![\d.,a-zA-Z])(\d{1,2})kg(\d{1,2})(?!\d|[.,]\d)").unwrap()
});

pub fn expand_height_weight(text: &str) -> String {
    let res = RE_WEIGHT.replace_all(text, |caps: &Captures| {
        format!(" {} ki l\u{f4} gam {} ", n2w(caps.get(1).unwrap().as_str()), n2w(caps.get(2).unwrap().as_str()))
    }).into_owned();
    RE_HEIGHT.replace_all(&res, |caps: &Captures| {
        format!(" {} m\u{e9}t {} ", n2w(caps.get(1).unwrap().as_str()), n2w(caps.get(2).unwrap().as_str()))
    }).into_owned()
}

// ── Context for a single uppercase letter after a number ──────────────────
// By default such a letter ("51M", "12B", "5S", "100K") marks an identifier and
// is spelled out. A unit or magnitude reading requires one of the cues below.
static MONEY_LEAD_WORDS: Lazy<std::collections::HashSet<&'static str>> = Lazy::new(|| {
    // The LAST word of a money phrase sitting right before the number
    // ("doanh thu" -> "thu", "thu nhập" -> "nhập", "đầu tư" -> "tư").
    ["giá", "vé", "phí", "lương", "thưởng", "vốn", "quỹ", "thu", "chi", "lãi",
     "lỗ", "nợ", "tiền", "cọc", "combo", "thầu", "tư", "nhập", "trả", "giảm",
     "tặng", "đơn", "phạt", "ship",
     // Verbs of price movement immediately before the number
     // ("giá … lên 92M", "còn 500K").
     "lên", "xuống", "còn", "đạt", "chạm", "về", "hết", "tốn", "tầm", "khoảng",
     "gần", "hơn", "tới"].into_iter().collect()
});
static MONEY_AFTER_WORDS: Lazy<std::collections::HashSet<&'static str>> = Lazy::new(|| {
    // A word right after the number-letter pair confirming money or bulk count.
    ["usd", "vnd", "vnđ", "usdt", "eur", "euro", "gbp", "jpy", "cny", "đồng",
     "đô", "lượt", "view", "views", "follower", "followers", "sub", "subs",
     "người"].into_iter().collect()
});
static CONTAINER_LEAD_WORDS: Lazy<std::collections::HashSet<&'static str>> = Lazy::new(|| {
    // A container or measuring verb before the number makes "L" mean litres
    // ("chai 2L", "bình 20L").
    ["chai", "bình", "thùng", "can", "xô", "bồn", "két", "lu", "ấm", "nồi",
     "tích", "chứa", "đựng", "đổ", "uống"].into_iter().collect()
});

fn last_word_before(text: &str, pos: usize) -> String {
    text[..pos].split_whitespace().next_back()
        .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()).to_lowercase())
        .unwrap_or_default()
}

fn first_word_after(text: &str, pos: usize) -> String {
    text[pos..].split_whitespace().next()
        .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()).to_lowercase())
        .unwrap_or_default()
}

pub fn expand_units_and_currency(text: &str) -> String {
    let mut result = text.to_string();

    result = RE_CURRENCY_PREFIX_SYMBOL.replace_all(&result, |caps: &Captures| {
        let symbol = caps.get(1).unwrap().as_str();
        let num = caps.get(2).unwrap().as_str();
        let mag = caps.get(3).map_or("", |m: fancy_regex::Match| m.as_str());
        let suffix = caps.get(4).map_or("", |m: fancy_regex::Match| m.as_str());
        let mag_word = match suffix.to_uppercase().as_str() {
            "M" => "triệu", "B" => "tỷ", "K" => "nghìn", _ => mag,
        };
        let full = CURRENCY_SYMBOL_MAP.get(symbol).copied().unwrap_or("");
        format!("{} {} {}", expand_number_with_sep(num), mag_word, full).replace("  ", " ").trim().to_string()
    }).to_string();

    result = RE_CURRENCY_SUFFIX_SYMBOL.replace_all(&result, |caps: &Captures| {
        let num = caps.get(1).unwrap().as_str();
        let mag = caps.get(2).map_or("", |m: fancy_regex::Match| m.as_str());
        let symbol = caps.get(3).unwrap().as_str();
        let full = CURRENCY_SYMBOL_MAP.get(symbol).copied().unwrap_or("");
        format!("{} {} {}", expand_number_with_sep(num), mag, full).replace("  ", " ").trim().to_string()
    }).to_string();

    result = RE_PERCENTAGE.replace_all(&result, |caps: &Captures| {
        format!("{} phần trăm", expand_number_with_sep(caps.get(1).unwrap().as_str()))
    }).to_string();

    // Single uppercase letter after a number: an identifier by default, so leave
    // it for the letter-name pass ("51M" -> "mờ", "12B" -> "bê", "5S" -> "ét").
    // It becomes a unit only with context:
    //   - a DECIMAL number ("3.2M", "1.25B", "1.5L"), since identifiers have no
    //     fractional part;
    //   - M/B/K with a money cue before ("lương 20M") or after ("5M USD",
    //     "2B đồng") -> million / billion / thousand;
    //   - L with a container word before it ("chai 2L") -> litres.
    // Two exceptions: "W" is always watts, as no common serial uses it, and "G"
    // is never grams because "5G"/"4G" dominate real text. Lowercase always
    // keeps the unit meaning ("24h", "450g", "30m", "15s").
    let units_src = result.clone();
    result = RE_UNITS_WITH_NUM.replace_all(&units_src, |caps: &Captures| {
        let m0 = caps.get(0).unwrap();
        let num = caps.get(1).unwrap().as_str();
        let mag = caps.get(2).map_or("", |m: fancy_regex::Match| m.as_str());
        let unit = caps.get(3).unwrap().as_str();

        let is_upper_single = unit.len() == 1
            && unit.chars().next().unwrap().is_ascii_uppercase();
        if is_upper_single && unit != "W" {
            let is_decimal = num.contains('.') || num.contains(',');
            let lead = last_word_before(&units_src, m0.start());
            let after = first_word_after(&units_src, m0.end());
            let money_ok = matches!(unit, "M" | "B" | "K")
                && (MONEY_LEAD_WORDS.contains(lead.as_str())
                    || MONEY_AFTER_WORDS.contains(after.as_str()));
            let liter_ok = unit == "L" && CONTAINER_LEAD_WORDS.contains(lead.as_str());
            if unit == "G" || !(is_decimal || money_ok || liter_ok) {
                return m0.as_str().to_string();
            }
        }

        // A one-letter unit followed by another single letter ("2 b c", left
        // behind after the formula stage split "2bc") is a run of variables, not
        // a measurement. "x" is excluded because it marks multiplication, so
        // "5 m x 20 m" still reads as metres.
        if unit.chars().count() == 1 {
            let after = first_word_after(&units_src, m0.end());
            if after.chars().count() == 1
                && after.chars().all(|c| c.is_ascii_alphabetic())
                && after != "x"
            {
                return m0.as_str().to_string();
            }
        }

        let full = if unit == "M" {
            "triệu"
        } else if unit == "B" {
            "tỷ"
        } else if unit == "K" {
            "nghìn"
        } else if unit == "m" {
            "mét"
        } else {
            ALL_UNITS_MAP.get(&unit.to_lowercase()).map(|s: &String| s.as_str()).unwrap_or(unit)
        };
        format!("{} {} {}", expand_number_with_sep(num), mag, full).replace("  ", " ").trim().to_string()
    }).to_string();

    result = RE_STANDALONE_UNIT.replace_all(&result, |caps: &Captures| {
        let unit = caps.get(1).unwrap().as_str();
        format!(" {} ", ALL_UNITS_MAP.get(&unit.to_lowercase()).map(|s: &String| s.as_str()).unwrap_or(unit))
    }).to_string();

    result
}

pub fn expand_compound_units(text: &str) -> String {
    RE_COMPOUND_UNIT.replace_all(text, |caps: &Captures| {
        let num_str = caps.get(1).map_or("", |m: fancy_regex::Match| m.as_str());
        let u1_raw = caps.get(2).unwrap().as_str();
        let u2_raw = caps.get(3).unwrap().as_str();

        let get_unit = |u: &str| {
            if u == "M" { return "triệu".to_string(); }
            if u == "m" { return "mét".to_string(); }
            ALL_UNITS_MAP.get(&u.to_lowercase()).map(|s: &String| s.to_string()).unwrap_or(u.to_string())
        };

        if num_str.is_empty() {
            let u1_lower = u1_raw.to_lowercase();
            let u2_lower = u2_raw.to_lowercase();
            let u1_is_unit = ALL_UNITS_MAP.contains_key(&u1_lower);
            let u2_is_unit = ALL_UNITS_MAP.contains_key(&u2_lower);

            // Special heuristic for literal ratios like P/E
            if u1_raw.len() == 1 && u2_raw.len() == 1 && (!u1_is_unit || !u2_is_unit) {
                let l1 = VI_LETTER_NAMES.get(u1_lower.as_str()).cloned().unwrap_or(u1_raw).to_string();
                let l2 = VI_LETTER_NAMES.get(u2_lower.as_str()).cloned().unwrap_or(u2_raw).to_string();
                format!(" {} trên {} ", l1, l2)
            } else if u1_is_unit && u2_is_unit {
                format!(" {} trên {} ", get_unit(u1_raw), get_unit(u2_raw))
            } else {
                // No number in front and not a genuine unit pair ("ML/AI",
                // "TP/HCM"): leave it to the acronym pass and SYMBOLS_MAP, which
                // reads "/" as "trên". Otherwise "ML" would become "mi li lít".
                caps.get(0).unwrap().as_str().to_string()
            }
        } else {
            let num = expand_number_with_sep(num_str);
            format!("{} {} trên {} ", num, get_unit(u1_raw), get_unit(u2_raw))
        }
    }).to_string()
}

pub fn fix_english_style_numbers(text: &str) -> String {
    RE_ENGLISH_STYLE_NUMBERS.replace_all(text, |caps: &Captures| {
        let val = caps.get(0).unwrap().as_str();
        let has_comma = val.contains(',');
        let has_dot = val.contains('.');
        if val.chars().filter(|&c: &char| c == ',').count() > 1 || (has_comma && has_dot && val.find(',').unwrap() < val.find('.').unwrap()) {
             if has_dot { val.replace(',', "").replace('.', ",") } else { val.replace(',', "") }
        } else if has_comma && has_dot {
             val.replace(',', "").replace('.', ",")
        } else {
             val.to_string()
        }
    }).to_string()
}

pub fn expand_power_of_ten(text: &str) -> String {
    RE_POWER_OF_TEN.replace_all(text, |caps: &Captures| {
        let base = caps.get(1).unwrap().as_str();
        let exp = caps.get(2).unwrap().as_str();

        let base_norm = expand_number_with_sep(base);
        let exp_val = exp.replace('+', "");
        let exp_norm = if exp_val.starts_with('-') {
            format!("trừ {}", n2w(&exp_val[1..]))
        } else {
            n2w(&exp_val)
        };
        format!(" {} nhân mười mũ {} ", base_norm, exp_norm)
    }).to_string()
}

pub fn expand_scientific_notation(text: &str) -> String {
    RE_SCIENTIFIC_NOTATION.replace_all(text, |caps: &Captures| {
        let neg = caps.get(1).map(|m: fancy_regex::Match| m.as_str()).unwrap_or("");
        let num_str = caps.get(2).unwrap().as_str();
        let expanded = expand_number_with_sep(num_str);
        if !neg.is_empty() {
            format!(" âm {} ", expanded)
        } else {
            format!(" {} ", expanded)
        }
    }).to_string()
}
