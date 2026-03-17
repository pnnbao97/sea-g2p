use fancy_regex::{Regex as FRegex, Captures as FCaps};
use regex::{Regex, Captures};
use once_cell::sync::Lazy;
use crate::vi_normalizer::num2vi::{n2w, n2w_single};
use crate::vi_normalizer::resources::{
    VI_LETTER_NAMES, DOMAIN_SUFFIX_MAP,
    ROMAN_NUMERALS, ABBRS, SYMBOLS_MAP, WORD_LIKE_ACRONYMS, MEASUREMENT_KEY_VI,
    CURRENCY_KEY, COMBINED_EXCEPTIONS
};
use crate::vi_normalizer::technical::normalize_slashes;
use crate::vi_normalizer::numerical::num_to_words;

const VI_UPPER: &str = "ĐĂÂÊÔƠƯ";

// ─ Patterns requiring look-arounds ───────────────────────────────────────
static RE_ROMAN_NUMBER: Lazy<FRegex> = Lazy::new(|| {
    FRegex::new(r"\b(?=[IVXLCDM]{2,})(?:M{0,4}(?:CM|CD|D?C{0,3})(?:XC|XL|L?X{0,3})(?:IX|IV|V?I{0,3}))(?<=[IVXLCDM])\b").unwrap()
});
static RE_STANDALONE_LETTER: Lazy<FRegex> = Lazy::new(|| {
    FRegex::new(r"(?<![\''])\b([a-zA-Z])\b(\.?)").unwrap()
});
static RE_ACRONYM: Lazy<FRegex> = Lazy::new(|| {
    FRegex::new(&format!(r"\b(?=[A-Z{}0-9]*[A-Z])[A-Z{}0-9]{{2,}}\b", VI_UPPER, VI_UPPER)).unwrap()
});
static RE_VERSION: Lazy<FRegex> = Lazy::new(|| {
    FRegex::new(r"(?<![-\u2013\u2014])\b(\d+(?:\.\d+){2,})\b").unwrap()
});
static RE_PRIME: Lazy<FRegex> = Lazy::new(|| {
    FRegex::new(r"(\b[a-zA-Z0-9])['\u2019](?!\w)").unwrap()
});

// ─ Simple patterns (regex crate — Thompson NFA, fast) ──────────────────
static RE_LETTER: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?i)(chữ|chữ cái|kí tự|ký tự)\s+(['"]?)([a-z])(['"]?)\b"#).unwrap()
});
static RE_ALPHANUMERIC: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b(\d+)([a-zA-Z])\b").unwrap()
});
static RE_BRACKETS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"[\(\[\{]\s*(.*?)\s*[\)\]\}]").unwrap()
});
static RE_STRIP_BRACKETS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"[\[\]\(\)\{\}]").unwrap()
});
static RE_TEMP_C_NEG: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)-(\d+(?:[.,]\d+)?)\s*°\s*c\b").unwrap()
});
static RE_TEMP_F_NEG: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)-(\d+(?:[.,]\d+)?)\s*°\s*f\b").unwrap()
});
static RE_TEMP_C: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(\d+(?:[.,]\d+)?)\s*°\s*c\b").unwrap()
});
static RE_TEMP_F: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(\d+(?:[.,]\d+)?)\s*°\s*f\b").unwrap()
});
static RE_DEGREE: Lazy<Regex> = Lazy::new(|| Regex::new(r"°").unwrap());
static RE_STANDARD_COLON: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b(\d+):(\d+)\b").unwrap()
});
static RE_CLEAN_OTHERS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"[^a-zA-Z0-9\sàáảãạăắằẳẵặâấầẩẫậèéẻẽẹêếềểễệìíỉĩịòóỏõọôốồổỗộơớờởỡợùúủũụưứừửữựỳýỷỹỵđÀÁẢÃẠĂẮẰẲẴẶÂẤẦẨẪẬÈÉẺẼẸÊẾỀỂỄỆÌÍỈĨỊÒÓỎÕỌÔỐỒỔỖỘƠỚỜỞỠỢÙÚỦŨỤƯỨỪỬỮỰỲÝỶỸỴĐ.,!?_\'\'-]").unwrap()
});
static RE_CLEAN_QUOTES: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"[“”„]"#).unwrap()
});
static RE_CLEAN_QUOTES_EDGES: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(^|\s)[\u2018\u2019']+|[\u2018\u2019']+($|\s)").unwrap()
});
static RE_COLON_SEMICOLON: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"[:;]").unwrap()
});
static RE_UNIT_POWERS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b([a-zA-Z]+)\^([-+]?\d+)\b").unwrap()
});
pub static RE_ACRONYMS_EXCEPTIONS: Lazy<Regex> = Lazy::new(|| {
    let mut keys: Vec<String> = COMBINED_EXCEPTIONS.keys().map(|k: &String| k.to_string()).collect();
    keys.sort_by_key(|b: &String| std::cmp::Reverse(b.len()));
    let pattern = keys.iter().map(|k: &String| format!(r"\b{}\b", regex::escape(k))).collect::<Vec<String>>().join("|");
    Regex::new(&pattern).unwrap()
});
pub static DOMAIN_SUFFIXES_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\.(com|vn|net|org|edu|gov|io|biz|info)\b").unwrap()
});
static RE_ACRONYMS_SPLIT: Lazy<regex::Regex> = Lazy::new(|| {
    regex::Regex::new(r"([.!?]+(?:\s+|$))").unwrap()
});

pub fn expand_roman(match_str: &str) -> String {
    if match_str.is_empty() {
        return String::new();
    }
    let num = match_str.to_uppercase();
    let mut result = 0;
    let chars: Vec<char> = num.chars().collect();
    for i in 0..chars.len() {
        let val = *ROMAN_NUMERALS.get(&chars[i]).unwrap_or(&0);
        if i + 1 < chars.len() && val < *ROMAN_NUMERALS.get(&chars[i+1]).unwrap_or(&0) {
            result -= val;
        } else {
            result += val;
        }
    }
    if result == 0 {
        return match_str.to_string();
    }
    format!(" {} ", n2w(&result.to_string()))
}

pub fn expand_unit_powers(text: &str) -> String {
    RE_UNIT_POWERS.replace_all(text, |caps: &Captures| {
        let base = caps.get(1).unwrap().as_str();
        let power = caps.get(2).unwrap().as_str();
        let power_norm = if power.starts_with('-') {
            format!("trừ {}", n2w(&power[1..]))
        } else {
            n2w(&power.replace('+', ""))
        };
        let base_lower = base.to_lowercase();
        let full_base = MEASUREMENT_KEY_VI.get(base_lower.as_str())
            .or(CURRENCY_KEY.get(base_lower.as_str()))
            .copied()
            .unwrap_or(base);
        format!(" {} mũ {} ", full_base, power_norm)
    }).to_string()
}

pub fn expand_letter(text: &str) -> String {
    RE_LETTER.replace_all(text, |caps: &Captures| {
        let prefix = caps.get(1).unwrap().as_str();
        let char = caps.get(3).unwrap().as_str();
        if let Some(name) = VI_LETTER_NAMES.get(char.to_lowercase().as_str()) {
            format!("{} {} ", prefix, name)
        } else {
            caps.get(0).unwrap().as_str().to_string()
        }
    }).to_string()
}

pub fn expand_abbreviations(text: &str) -> String {
    let mut result = text.to_string();
    for (k, v) in ABBRS.iter() {
        result = result.replace(k, v);
    }
    result
}

pub fn expand_standalone_letters(text: &str) -> String {
    RE_STANDALONE_LETTER.replace_all(text, |caps: &FCaps| {
        let char_raw = caps.get(1).unwrap().as_str();
        let char_lower = char_raw.to_lowercase();
        let dot = caps.get(2).unwrap().as_str();
        if let Some(name) = VI_LETTER_NAMES.get(char_lower.as_str()) {
            if char_raw.chars().next().unwrap().is_uppercase() && dot == "." {
                format!(" {} ", name)
            } else {
                format!(" {}{} ", name, dot)
            }
        } else {
            caps.get(0).unwrap().as_str().to_string()
        }
    }).to_string()
}

pub fn normalize_acronyms(text: &str) -> String {
    let mut result = Vec::new();
    let re_split = &*RE_ACRONYMS_SPLIT;

    let mut last = 0;
    let mut final_parts = Vec::new();
    for mat in re_split.find_iter(text) {
        final_parts.push(&text[last..mat.start()]);
        final_parts.push(mat.as_str());
        last = mat.end();
    }
    final_parts.push(&text[last..]);

    for i in (0..final_parts.len()).step_by(2) {
        let s = final_parts[i];
        let sep = if i + 1 < final_parts.len() { final_parts[i+1] } else { "" };
        if s.is_empty() {
            result.push(sep.to_string());
            continue;
        }

        let words: Vec<&str> = s.split_whitespace().collect();
        let is_all_caps = !words.is_empty() && words.iter().all(|w: &&str| w.chars().any(|c: char| c.is_alphabetic()) && w.chars().all(|c: char| c.is_uppercase()));

        let mut processed_s = s.to_string();
        if !is_all_caps {
            processed_s = RE_ACRONYM.replace_all(&processed_s, |caps: &FCaps| {
                let word = caps.get(0).unwrap().as_str();
                if word.chars().all(|c: char| c.is_ascii_digit()) { return word.to_string(); }
                if WORD_LIKE_ACRONYMS.contains(word) {
                    return format!("__start_en__{}__end_en__", word.to_lowercase());
                }

                let has_vi_letter = word.chars().any(|c: char| !c.is_ascii() && c.is_alphabetic());
                if has_vi_letter {
                    let mut parts = Vec::new();
                    for c in word.chars() {
                        let cl = c.to_lowercase().to_string();
                        if c.is_ascii_digit() { parts.push(n2w_single(&c.to_string())); }
                        else if let Some(name) = VI_LETTER_NAMES.get(cl.as_str()) { parts.push(name.to_string()); }
                        else if c.is_alphabetic() { parts.push(cl); }
                    }
                    return parts.join(" ");
                }

                if word.chars().any(|c: char| c.is_ascii_digit()) {
                    let res: Vec<String> = word.chars().map(|c: char| {
                        if c.is_ascii_digit() { n2w_single(&c.to_string()) }
                        else { VI_LETTER_NAMES.get(c.to_lowercase().to_string().as_str()).cloned().unwrap_or(c.to_string().as_str()).to_string() }
                    }).collect();
                    return res.join(" ");
                }

                let spaced_word = word.chars().filter(|c: &char| c.is_alphanumeric()).map(|c: char| c.to_lowercase().to_string()).collect::<Vec<String>>().join(" ");
                if !spaced_word.is_empty() { format!("__start_en__{}__end_en__", spaced_word) } else { word.to_string() }
            }).to_string();
        }
        result.push(processed_s + sep);
    }
    result.join("")
}

pub fn expand_alphanumeric(text: &str) -> String {
    RE_ALPHANUMERIC.replace_all(text, |caps: &Captures| {
        let num = caps.get(1).unwrap().as_str();
        let char = caps.get(2).unwrap().as_str().to_lowercase();
        if let Some(name) = VI_LETTER_NAMES.get(char.as_str()) {
            let mut pronunciation = name.to_string();
            if char == "d" && (text.to_lowercase().contains("quốc lộ") || text.to_lowercase().contains("ql")) {
                pronunciation = "đê".to_string();
            }
            format!("{} {}", num, pronunciation)
        } else {
            caps.get(0).unwrap().as_str().to_string()
        }
    }).into_owned()
}

pub fn expand_symbols(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    for c in text.chars() {
        if let Some(v) = SYMBOLS_MAP.get(&c) {
            result.push_str(v);
        } else {
            result.push(c);
        }
    }
    result
}

pub fn expand_prime(text: &str) -> String {
    RE_PRIME.replace_all(text, |caps: &FCaps| {
        let val = caps.get(1).unwrap().as_str().to_lowercase();
        let name = if val.chars().next().unwrap().is_ascii_digit() {
            n2w_single(&val)
        } else {
            VI_LETTER_NAMES.get(val.as_str()).cloned().unwrap_or(&val).to_string()
        };
        format!("{} phẩy", name)
    }).to_string()
}

pub fn expand_temperatures(text: &str) -> String {
    let mut res = RE_TEMP_C_NEG.replace_all(text, "âm $1 độ xê").into_owned();
    res = RE_TEMP_F_NEG.replace_all(&res, "âm $1 độ ép").into_owned();
    res = RE_TEMP_C.replace_all(&res, "$1 độ xê").into_owned();
    res = RE_TEMP_F.replace_all(&res, "$1 độ ép").into_owned();
    RE_DEGREE.replace_all(&res, " độ ").into_owned()
}

pub fn normalize_others(text: &str) -> String {
    let mut res = RE_ACRONYMS_EXCEPTIONS.replace_all(text, |caps: &Captures| {
        COMBINED_EXCEPTIONS.get(caps.get(0).unwrap().as_str()).cloned().unwrap_or(caps.get(0).unwrap().as_str().to_string())
    }).into_owned();

    res = normalize_slashes(&res);
    res = DOMAIN_SUFFIXES_RE.replace_all(&res, |caps: &Captures| {
        let suffix = DOMAIN_SUFFIX_MAP.get(caps.get(1).unwrap().as_str().to_lowercase().as_str()).copied().unwrap_or("");
        format!(" chấm {} ", if suffix.is_empty() { caps.get(1).unwrap().as_str() } else { suffix })
    }).into_owned();

    res = RE_ROMAN_NUMBER.replace_all(&res, |caps: &FCaps| {
        expand_roman(caps.get(0).unwrap().as_str())
    }).to_string();

    res = expand_letter(&res);
    res = expand_alphanumeric(&res);
    res = expand_prime(&res);
    res = expand_unit_powers(&res);
    res = RE_CLEAN_QUOTES.replace_all(&res, "").into_owned();
    res = RE_CLEAN_QUOTES_EDGES.replace_all(&res, "$1 $2").into_owned();
    res = expand_symbols(&res);
    res = RE_BRACKETS.replace_all(&res, ", $1, ").into_owned();
    res = RE_STRIP_BRACKETS.replace_all(&res, " ").into_owned();
    res = expand_temperatures(&res);
    res = normalize_acronyms(&res);

    res = RE_VERSION.replace_all(&res, |caps: &FCaps| {
        let parts: Vec<String> = caps.get(1).unwrap().as_str().split('.').map(|s: &str| {
            s.chars().map(|c: char| n2w_single(&c.to_string())).collect::<Vec<String>>().join(" ")
        }).collect();
        parts.join(" chấm ")
    }).to_string();

    res = RE_STANDARD_COLON.replace_all(&res, |caps: &Captures| {
        format!(" {} hai chấm {} ", num_to_words(caps.get(1).unwrap().as_str(), false), num_to_words(caps.get(2).unwrap().as_str(), false))
    }).into_owned();

    res = RE_COLON_SEMICOLON.replace_all(&res, ",").into_owned();
    RE_CLEAN_OTHERS.replace_all(&res, " ").into_owned()
}
