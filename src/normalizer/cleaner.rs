use once_cell::sync::Lazy;
use fancy_regex::{Regex, Captures};
use std::collections::HashMap;
use aho_corasick::{AhoCorasick, MatchKind};
use crate::normalizer::num2vi::{n2w, n2w_single};
use crate::normalizer::vi_resources::*;

// Regex patterns
pub static RE_TECHNICAL: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?ix)
        \b(?:https?|ftp)://[A-Za-z0-9.\-_~:/?#\[\]@!$&\'()*+,;=]+\b
        |
        \b(?:www\.)[A-Za-z0-9.\-_~:/?#\[\]@!$&\'()*+,;=]+\b
        |
        \b[A-Za-z0-9.\-]+(?:\.com|\.vn|\.net|\.org|\.gov|\.io|\.biz|\.info)(?:/[A-Za-z0-9.\-_~:/?#\[\]@!$&\'()*+,;=]*)?\b
        |
        (?<!\w)/[a-zA-Z0-9._\-/]{2,}\b
        |
        \b[a-zA-Z]:\\[a-zA-Z0-9._\\\-]+\b
        |
        \b[a-zA-Z0-9._\-]+\.(?:txt|log|tar|gz|zip|sh|py|js|cpp|h|json|xml|yaml|yml|md|csv|pdf|docx|xlsx|exe|dll|so|config)\b
        |
        \b[a-zA-Z][a-zA-Z0-9]*(?:[._\-][a-zA-Z0-9]+){2,}\b
        |
        \b(?:[a-fA-F0-9]{1,4}:){3,7}[a-fA-F0-9]{1,4}\b
    ").unwrap()
});

pub static RE_EMAIL: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b").unwrap()
});

pub static RE_TECH_SPLIT: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"([./:?&=/_ \-\\#])").unwrap()
});

pub static RE_EMAIL_SPLIT: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"([._\-+])").unwrap()
});

pub static RE_SLASH_NUMBER: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b(\d+)/(\d+)\b").unwrap()
});

pub static RE_DOMAIN_SUFFIXES: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\.(com|vn|net|org|edu|gov|io|biz|info)\b").unwrap()
});

pub static RE_ALPHANUM_SPLIT: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"[a-zA-Z]+|\d+").unwrap()
});

pub static RE_FULL_DATE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\b(\d{1,2})(\/|-|\.)(\d{1,2})(\/|-|\.)(\d{4})\b").unwrap()
});

pub static RE_DAY_MONTH: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\b(\d{1,2})(\/|-)(\d{1,2})\b").unwrap()
});

pub static RE_MONTH_YEAR: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\b(\d{1,2})(\/|-|\.)(\d{4})\b").unwrap()
});

pub static RE_FULL_TIME: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\b(\d+)(g|:|h)(\d{1,2})(p|:|m)(\d{1,2})(?:\s*(giây|s|g))?\b").unwrap()
});

pub static RE_TIME: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\b(\d+)(g|:|h)(\d{1,2})(?:\s*(phút|p|m))?\b").unwrap()
});

pub static RE_REDUNDANT_NGAY: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\bngày\s+ngày\b").unwrap()
});

pub static RE_REDUNDANT_THANG: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\btháng\s+tháng\b").unwrap()
});

pub static RE_REDUNDANT_NAM: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\bnăm\s+năm\b").unwrap()
});

pub static RE_SCIENTIFIC: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\b(\d+(?:[.,]\d+)?e[+-]?\d+)\b").unwrap()
});

pub static RE_COMPOUND_UNIT: Lazy<Regex> = Lazy::new(|| {
    let numeric_p = r"(\d+(?:[.,]\d+)*)";
    Regex::new(&format!(r"(?i)\b{}?\s*([a-zμµ²³°]+)/([a-zμµ²³°0-9]+)\b", numeric_p)).unwrap()
});

pub static RE_CURRENCY_PREFIX_SYMBOL: Lazy<Regex> = Lazy::new(|| {
    let numeric_p = r"(\d+(?:[.,]\d+)*)";
    let magnitude_p = r"(?:\s*(tỷ|triệu|nghìn|ngàn))?";
    Regex::new(&format!(r"(?i)([$€¥£₩])\s*{}{}", numeric_p, magnitude_p)).unwrap()
});

pub static RE_CURRENCY_SUFFIX_SYMBOL: Lazy<Regex> = Lazy::new(|| {
    let numeric_p = r"(\d+(?:[.,]\d+)*)";
    let magnitude_p = r"(?:\s*(tỷ|triệu|nghìn|ngàn))?";
    Regex::new(&format!(r"(?i){}{}([$€¥£₩])", numeric_p, magnitude_p)).unwrap()
});

pub static RE_PERCENTAGE: Lazy<Regex> = Lazy::new(|| {
    let numeric_p = r"(\d+(?:[.,]\d+)*)";
    Regex::new(&format!(r"(?i){}\s*%", numeric_p)).unwrap()
});

pub static RE_UNITS_WITH_NUM: Lazy<Regex> = Lazy::new(|| {
    let numeric_p = r"(\d+(?:[.,]\d+)*)";
    let magnitude_p = r"(?:\s*(tỷ|triệu|nghìn|ngàn))?";
    let mut keys: Vec<_> = MEASUREMENT_KEY_VI.keys().cloned().collect();
    keys.extend(CURRENCY_KEY.keys().filter(|k| k.as_str() != "%").cloned());
    keys.sort_by_key(|k| std::cmp::Reverse(k.len()));
    let units_pattern = keys.iter().map(|k| regex::escape(k)).collect::<Vec<_>>().join("|");
    Regex::new(&format!(r"(?i)(?<![\d.,]){}{}\s*({})\b", numeric_p, magnitude_p, units_pattern)).unwrap()
});

pub static RE_STANDALONE_UNIT: Lazy<Regex> = Lazy::new(|| {
    let safe_standalone = ["km", "cm", "mm", "kg", "mg", "usd", "vnd", "ph"];
    let pattern = safe_standalone.iter().map(|&s| regex::escape(s)).collect::<Vec<_>>().join("|");
    Regex::new(&format!(r"(?i)(?<![\d.,])\b({})\b", pattern)).unwrap()
});

pub static RE_ROMAN_NUMBER: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b(?=[IVXLCDM]{2,})M{0,4}(CM|CD|D?C{0,3})(XC|XL|L?X{0,3})(IX|IV|V?I{0,3})\b").unwrap()
});

pub static RE_LETTER: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?i)(chữ|chữ cái|kí tự|ký tự)\s+(['"]?)([a-z])(['"]?)\b"#).unwrap()
});

pub static RE_STANDALONE_LETTER: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?<!['’])\b([a-zA-Z])\b(\.?)").unwrap()
});

pub static RE_SENTENCE_SPLIT: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"([.!?]+(?:\s+|$))").unwrap()
});

pub static RE_ACRONYM: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b(?=[A-Z0-9]*[A-Z])[A-Z0-9]{2,}\b").unwrap()
});

pub static RE_ALPHANUMERIC: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b(\d+)([a-zA-Z])\b").unwrap()
});

pub static RE_BRACKETS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"[\(\[\{]\s*(.*?)\s*[\)\]\}]").unwrap()
});

pub static RE_STRIP_BRACKETS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"[\[\]\(\)\{\}]").unwrap()
});

pub static RE_TEMP_C_NEG: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)-(\d+(?:[.,]\d+)?)\s*°\s*c\b").unwrap()
});

pub static RE_TEMP_F_NEG: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)-(\d+(?:[.,]\d+)?)\s*°\s*f\b").unwrap()
});

pub static RE_TEMP_C: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(\d+(?:[.,]\d+)?)\s*°\s*c\b").unwrap()
});

pub static RE_TEMP_F: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(\d+(?:[.,]\d+)?)\s*°\s*f\b").unwrap()
});

pub static RE_DEGREE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"°").unwrap()
});

pub static RE_VERSION: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b(\d+(?:\.\d+)+)\b").unwrap()
});

pub static RE_CLEAN_OTHERS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"[^a-zA-Z0-9\sàáảãạăắằẳẵặâấầẩẫậèéẻẽẹêếềểễệìíỉĩịòóỏõọôốồổỗộơớờởỡợùúủũụưứừửữựỳýỷỹỵđÀÁẢÃẠĂẮẰẲẴẶÂẤẦẨẪẬÈÉẺẼẸÊẾỀỂỄỆÌÍỈĨỊÒÓỎÕỌÔỐỒỔỖỘƠỚỜỞỠỢÙÚỦŨỤƯỨỪỬỮỰỲÝỶỸỴĐ.,!?_\'’]").unwrap()
});

pub static RE_CLEAN_QUOTES: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"["“”"]"#).unwrap()
});

pub static RE_CLEAN_QUOTES_EDGES: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(^|\s)['’]+|['’]+($|\s)").unwrap()
});

pub static RE_PRIME: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(\b[a-zA-Z0-9])['’](?!\w)").unwrap()
});

pub static RE_COLON_SEMICOLON: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"[:;]").unwrap()
});

pub static RE_EXTRA_SPACES: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"[ \t\xA0]+").unwrap()
});

pub static RE_EXTRA_COMMAS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r",\s*,").unwrap()
});

pub static RE_COMMA_BEFORE_PUNCT: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r",\s*([.!?;])").unwrap()
});

pub static RE_SPACE_BEFORE_PUNCT: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\s+([,.!?;:])").unwrap()
});

pub static RE_MISSING_SPACE_AFTER_PUNCT: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"([.,!?;:])(?=[^\s\d<])").unwrap()
});

pub static RE_ENTOKEN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)ENTOKEN\d+").unwrap()
});

pub static RE_INTERNAL_EN_TAG: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(__start_en__.*?__end_en__|<en>.*?</en>)").unwrap()
});

pub static RE_DOT_BETWEEN_DIGITS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(\d+)\.(\d+)").unwrap()
});

pub static RE_UNIT_POWERS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b([a-zA-Z]+)\^([-+]?\d+)\b").unwrap()
});

pub static RE_ACRONYMS_EXCEPTIONS: Lazy<Regex> = Lazy::new(|| {
    let mut keys: Vec<_> = COMBINED_EXCEPTIONS.keys().cloned().collect();
    keys.sort_by_key(|k| std::cmp::Reverse(k.len()));
    let pattern = keys.iter().map(|k| format!(r"\b{}\b", regex::escape(k))).collect::<Vec<_>>().join("|");
    Regex::new(&pattern).unwrap()
});

pub static RE_ORDINAL: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(thứ|hạng)(\s+)(\d+)\b").unwrap()
});

pub static RE_MULTIPLY: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(\d+)(x|\sx\s)(\d+)").unwrap()
});

pub static RE_PHONE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"((\+84|84|0|0084)(3|5|7|8|9)[0-9]{8})").unwrap()
});

pub static RE_DOT_SEP: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\d+(\.\d{3})+").unwrap()
});

pub static RE_NUMBER: Lazy<Regex> = Lazy::new(|| {
    let num_combined = r"(\d+[,]\d+|\d+(\.\d{3}){1,3}|\d+(\s\d{3}){1,3}|\d+)";
    Regex::new(&format!(r"(\D)(-)?{}(?!\d)", num_combined)).unwrap()
});

pub static RE_NUMBER_START: Lazy<Regex> = Lazy::new(|| {
    let num_combined = r"(\d+[,]\d+|\d+(\.\d{3}){1,3}|\d+(\s\d{3}){1,3}|\d+)";
    Regex::new(&format!(r"(?m)^(-)?{}(?!\d)", num_combined)).unwrap()
});

pub static RE_FLOAT_WITH_COMMA: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?<![\d.])(\d+(?:\.\d{3})*),(\d+)(%)?").unwrap()
});

pub static RE_MULTI_COMMA: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b(\d+(?:,\d+){2,})\b").unwrap()
});

pub static RE_STRIP_DOT_SEP: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?<![\d.])\d+(?:\.\d{3})+(?![\d.])").unwrap()
});

pub static RE_POWER_OF_TEN_EXPLICIT: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\b(\d+(?:[.,]\d+)?)\s*[x*×]\s*10\^([-+]?\d+)\b").unwrap()
});

pub static RE_POWER_OF_TEN_IMPLICIT: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b10\^([-+]?\d+)\b").unwrap()
});

pub static AC_ABBRS: Lazy<(AhoCorasick, Vec<&'static str>)> = Lazy::new(|| {
    let mut sorted_abbrs: Vec<_> = ABBRS.iter().collect();
    sorted_abbrs.sort_by_key(|(k, _)| std::cmp::Reverse(k.len()));
    let keys: Vec<_> = sorted_abbrs.iter().map(|(k, _)| k.as_str()).collect();
    let values: Vec<_> = sorted_abbrs.iter().map(|(_, &v)| v).collect();
    let ac = AhoCorasick::builder()
        .match_kind(MatchKind::LeftmostLongest)
        .build(&keys)
        .unwrap();
    (ac, values)
});

pub static AC_SYMBOLS: Lazy<(AhoCorasick, Vec<&'static str>)> = Lazy::new(|| {
    let mut sorted_symbols: Vec<_> = SYMBOLS_MAP.iter().collect();
    sorted_symbols.sort_by_key(|(k, _)| std::cmp::Reverse(k.to_string().len()));
    let keys: Vec<_> = sorted_symbols.iter().map(|(k, _)| k.to_string()).collect();
    let values: Vec<_> = sorted_symbols.iter().map(|(_, &v)| v).collect();
    let ac = AhoCorasick::builder()
        .match_kind(MatchKind::LeftmostLongest)
        .build(&keys)
        .unwrap();
    (ac, values)
});

const DAY_IN_MONTH: [i32; 12] = [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

fn is_valid_date(day: &str, month: &str) -> bool {
    let day_val = day.parse::<i32>().unwrap_or(0);
    let month_val = month.parse::<i32>().unwrap_or(0);
    month_val >= 1 && month_val <= 12 && day_val >= 1 && day_val <= DAY_IN_MONTH[(month_val - 1) as usize]
}

pub fn normalize_date(text: &str) -> String {
    let mut text = RE_FULL_DATE.replace_all(text, |caps: &Captures| {
        let day = caps.get(1).unwrap().as_str();
        let month = caps.get(3).unwrap().as_str();
        let year = caps.get(5).unwrap().as_str();
        if is_valid_date(day, month) {
            let m_val = if month.parse::<i32>().unwrap_or(0) == 4 { "tư".to_string() } else { n2w(&month.parse::<i32>().unwrap_or(0).to_string()) };
            format!("ngày {} tháng {} năm {}", n2w(&day.parse::<i32>().unwrap_or(0).to_string()), m_val, n2w(year))
        } else {
            caps.get(0).unwrap().as_str().to_string()
        }
    }).to_string();

    text = RE_MONTH_YEAR.replace_all(&text, |caps: &Captures| {
        let month = caps.get(1).unwrap().as_str();
        let year = caps.get(3).unwrap().as_str();
        let m_val = if month.parse::<i32>().unwrap_or(0) == 4 { "tư".to_string() } else { n2w(&month.parse::<i32>().unwrap_or(0).to_string()) };
        format!("tháng {} năm {}", m_val, n2w(year))
    }).to_string();

    text = RE_DAY_MONTH.replace_all(&text, |caps: &Captures| {
        let day = caps.get(1).unwrap().as_str();
        let month = caps.get(3).unwrap().as_str();
        if is_valid_date(day, month) {
            let m_val = if month.parse::<i32>().unwrap_or(0) == 4 { "tư".to_string() } else { n2w(&month.parse::<i32>().unwrap_or(0).to_string()) };
            format!("ngày {} tháng {}", n2w(&day.parse::<i32>().unwrap_or(0).to_string()), m_val)
        } else {
            caps.get(0).unwrap().as_str().to_string()
        }
    }).to_string();

    text = RE_REDUNDANT_NGAY.replace_all(&text, "ngày").to_string();
    text = RE_REDUNDANT_THANG.replace_all(&text, "tháng").to_string();
    text = RE_REDUNDANT_NAM.replace_all(&text, "năm").to_string();
    text
}

fn norm_time_part(s: &str) -> &str {
    if s == "00" { "0" } else { s }
}

pub fn normalize_time(text: &str) -> String {
    let mut text = RE_FULL_TIME.replace_all(text, |caps: &Captures| {
        let h = caps.get(1).unwrap().as_str();
        let m = caps.get(3).unwrap().as_str();
        let s = caps.get(5).unwrap().as_str();
        format!("{} giờ {} phút {} giây", n2w(norm_time_part(h)), n2w(norm_time_part(m)), n2w(norm_time_part(s)))
    }).to_string();

    text = RE_TIME.replace_all(&text, |caps: &Captures| {
        let h = caps.get(1).unwrap().as_str();
        let sep = caps.get(2).unwrap().as_str();
        let m = caps.get(3).unwrap().as_str();
        let h_int = h.parse::<i32>().unwrap_or(0);
        let m_int = m.parse::<i32>().unwrap_or(0);

        if m_int >= 0 && m_int < 60 {
            if sep == ":" {
                if h_int < 24 {
                    format!("{} giờ {} phút", n2w(norm_time_part(h)), n2w(norm_time_part(m)))
                } else {
                    format!("{} phút {} giây", n2w(h), n2w(norm_time_part(m)))
                }
            } else {
                format!("{} giờ {} phút", n2w(norm_time_part(h)), n2w(norm_time_part(m)))
            }
        } else {
            caps.get(0).unwrap().as_str().to_string()
        }
    }).to_string();

    text
}

pub fn expand_scientific(num_str: &str) -> String {
    let num_lower = num_str.to_lowercase();
    let e_idx = num_lower.find('e').unwrap();
    let base = &num_str[..e_idx];
    let exp = &num_str[e_idx+1..];

    let base_norm = if base.contains('.') {
        let parts: Vec<&str> = base.split('.').collect();
        let dec_part = parts[1].trim_end_matches('0');
        if !dec_part.is_empty() {
            format!("{} chấm {}", n2w(parts[0]), n2w_single(dec_part))
        } else {
            n2w(parts[0])
        }
    } else if base.contains(',') {
        let parts: Vec<&str> = base.split(',').collect();
        let dec_part = parts[1].trim_end_matches('0');
        if !dec_part.is_empty() {
            format!("{} phẩy {}", n2w(parts[0]), n2w_single(dec_part))
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

pub fn expand_number_with_sep(num_str: &str) -> String {
    if num_str.is_empty() { return String::new(); }
    if num_str.to_lowercase().contains('e') { return expand_scientific(num_str); }

    if num_str.contains(',') && num_str.contains('.') {
        let (p0, p1) = if num_str.rfind('.').unwrap() > num_str.rfind(',').unwrap() {
            let s = num_str.replace(',', "");
            let idx = s.rfind('.').unwrap();
            let p1 = s[idx + 1..].to_string();
            let p0 = s[..idx].to_string();
            (p0, p1)
        } else {
            let s = num_str.replace('.', "");
            let idx = s.rfind(',').unwrap();
            let p1 = s[idx + 1..].to_string();
            let p0 = s[..idx].to_string();
            (p0, p1)
        };
        let dec_part = p1.trim_end_matches('0');
        if dec_part.is_empty() { return n2w(&p0); }
        return format!("{} phẩy {}", n2w(&p0), n2w_single(dec_part));
    }

    if num_str.contains(',') || num_str.contains('.') {
        let sep = if num_str.contains(',') { ',' } else { '.' };
        let parts: Vec<&str> = num_str.split(sep).collect();
        if parts.len() > 2 {
            return n2w(&num_str.replace(sep, ""));
        }
        if parts.len() == 2 {
            if sep == '.' && parts[1].len() == 3 {
                 return n2w(&num_str.replace(sep, ""));
            }
            // For single comma, favor decimal (phẩy)
            let dec_part = parts[1].trim_end_matches('0');
            if dec_part.is_empty() { return n2w(parts[0]); }
            let sep_word = if sep == ',' { "phẩy" } else { "chấm" };
            return format!("{} {} {}", n2w(parts[0]), sep_word, n2w_single(dec_part));
        }
    }

    n2w(num_str)
}

pub fn expand_measurement_currency(text: &str) -> String {
    let mut text = RE_UNITS_WITH_NUM.replace_all(text, |caps: &Captures| {
        let num = caps.get(1).unwrap().as_str();
        let mag = caps.get(2).map_or("", |m| m.as_str());
        let unit = caps.get(3).unwrap().as_str();

        let full = match unit {
            "M" => "triệu",
            "m" => "mét",
            _ => MEASUREMENT_KEY_VI.get(unit.to_lowercase().as_str())
                    .or_else(|| CURRENCY_KEY.get(unit.to_lowercase().as_str()))
                    .unwrap_or(&unit),
        };
        format!("{} {} {}", expand_number_with_sep(num), mag, full).replace("  ", " ").trim().to_string()
    }).to_string();

    text = RE_STANDALONE_UNIT.replace_all(&text, |caps: &Captures| {
        let unit = caps.get(1).unwrap().as_str();
        let unit_low = unit.to_lowercase();
        format!(" {} ", MEASUREMENT_KEY_VI.get(unit_low.as_str()).or_else(|| CURRENCY_KEY.get(unit_low.as_str())).unwrap_or(&unit))
    }).to_string();

    text = RE_CURRENCY_PREFIX_SYMBOL.replace_all(&text, |caps: &Captures| {
        let symbol = caps.get(1).unwrap().as_str();
        let num = caps.get(2).unwrap().as_str();
        let mag = caps.get(3).map_or("", |m| m.as_str());
        let full = CURRENCY_SYMBOL_MAP.get(symbol).unwrap_or(&"");
        format!("{} {} {}", expand_number_with_sep(num), mag, full).replace("  ", " ").trim().to_string()
    }).to_string();

    text = RE_CURRENCY_SUFFIX_SYMBOL.replace_all(&text, |caps: &Captures| {
        let num = caps.get(1).unwrap().as_str();
        let mag = caps.get(2).map_or("", |m| m.as_str());
        let symbol = caps.get(3).unwrap().as_str();
        let full = CURRENCY_SYMBOL_MAP.get(symbol).unwrap_or(&"");
        format!("{} {} {}", expand_number_with_sep(num), mag, full).replace("  ", " ").trim().to_string()
    }).to_string();

    text = RE_PERCENTAGE.replace_all(&text, |caps: &Captures| {
        format!("{} phần trăm", expand_number_with_sep(caps.get(1).unwrap().as_str()))
    }).to_string();

    text
}

pub fn expand_compound_units(text: &str) -> String {
    RE_COMPOUND_UNIT.replace_all(text, |caps: &Captures| {
        let num_str = caps.get(1).map_or("", |m| m.as_str());
        if num_str.is_empty() { return caps.get(0).unwrap().as_str().to_string(); }

        let num = expand_number_with_sep(num_str);
        let u1_raw = caps.get(2).unwrap().as_str();
        let u2_raw = caps.get(3).unwrap().as_str();

        fn get_unit(u_raw: &str) -> &str {
            match u_raw {
                "M" => "triệu",
                "m" => "mét",
                _ => MEASUREMENT_KEY_VI.get(u_raw.to_lowercase().as_str())
                        .or_else(|| CURRENCY_KEY.get(u_raw.to_lowercase().as_str()))
                        .unwrap_or(&u_raw),
            }
        }

        format!("{} {} trên {}", num, get_unit(u1_raw), get_unit(u2_raw))
    }).to_string()
}

pub fn expand_roman(text: &str) -> String {
    RE_ROMAN_NUMBER.replace_all(text, |caps: &Captures| {
        let num = caps.get(0).unwrap().as_str().to_uppercase();
        let mut result = 0;
        let chars: Vec<char> = num.chars().collect();
        for i in 0..chars.len() {
            let val = *ROMAN_NUMERALS.get(&chars[i]).unwrap();
            if i + 1 < chars.len() && val < *ROMAN_NUMERALS.get(&chars[i+1]).unwrap() {
                result -= val;
            } else {
                result += val;
            }
        }
        format!(" {} ", n2w(&result.to_string()))
    }).to_string()
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
            .or_else(|| CURRENCY_KEY.get(base_lower.as_str()))
            .cloned()
            .unwrap_or(base);
        format!(" {} mũ {} ", full_base, power_norm)
    }).to_string()
}

pub fn expand_abbreviations(text: &str) -> String {
    let (ac, values) = &*AC_ABBRS;
    ac.replace_all(text, values)
}

pub fn expand_symbols(text: &str) -> String {
    let (ac, values) = &*AC_SYMBOLS;
    ac.replace_all(text, values)
}

pub fn normalize_acronyms(text: &str) -> String {
    let mut result = Vec::new();
    // Re-implementing split because RE_SENTENCE_SPLIT might not work as expected with split
    let mut split_parts = Vec::new();
    let mut last_pos = 0;
    for mat in RE_SENTENCE_SPLIT.find_iter(text) {
        let mat = mat.unwrap();
        split_parts.push(text[last_pos..mat.start()].to_string());
        split_parts.push(mat.as_str().to_string());
        last_pos = mat.end();
    }
    if last_pos < text.len() {
        split_parts.push(text[last_pos..].to_string());
    }

    for i in (0..split_parts.len()).step_by(2) {
        let s = &split_parts[i];
        let sep = if i + 1 < split_parts.len() { &split_parts[i+1] } else { "" };
        if s.is_empty() {
            result.push(sep.to_string());
            continue;
        }

        let words: Vec<&str> = s.split_whitespace().collect();
        let alpha_words: Vec<&str> = words.iter().filter(|w| w.chars().any(|c: char| c.is_alphabetic())).cloned().collect();
        let is_all_caps = !alpha_words.is_empty() && alpha_words.iter().all(|w| w.chars().all(|c: char| c.is_uppercase()));

        if !is_all_caps {
            let processed_s = RE_ACRONYM.replace_all(s, |caps: &Captures| {
                let word = caps.get(0).unwrap().as_str();
                if word.chars().all(|c: char| c.is_ascii_digit()) { return word.to_string(); }
                if WORD_LIKE_ACRONYMS.contains(word) {
                    return format!("__start_en__{}__end_en__", word.to_lowercase());
                }
                if word.chars().any(|c: char| c.is_ascii_digit()) {
                    let res: Vec<String> = word.to_lowercase().chars().map(|c: char| {
                        if c.is_ascii_digit() { n2w_single(&c.to_string()) }
                        else { (*VI_LETTER_NAMES.get(c.to_string().as_str()).unwrap_or(&c.to_string().as_str())).to_string() }
                    }).collect();
                    return res.join(" ");
                }
                let spaced_word: String = word.chars().filter(|c: &char| c.is_alphanumeric()).map(|c: char| c.to_lowercase().to_string()).collect::<Vec<_>>().join(" ");
                if !spaced_word.is_empty() {
                    format!("__start_en__{}__end_en__", spaced_word)
                } else {
                    word.to_string()
                }
            }).to_string();
            result.push(format!("{}{}", processed_s, sep));
        } else {
            result.push(format!("{}{}", s, sep));
        }
    }
    result.join("")
}

pub fn expand_standalone_letters(text: &str) -> String {
    RE_STANDALONE_LETTER.replace_all(text, |caps: &Captures| {
        let char_raw = caps.get(1).unwrap().as_str();
        let char_low = char_raw.to_lowercase();
        let dot = caps.get(2).map_or("", |m| m.as_str());
        if let Some(&name) = VI_LETTER_NAMES.get(char_low.as_str()) {
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

pub fn expand_alphanumeric(text: &str) -> String {
    RE_ALPHANUMERIC.replace_all(text, |caps: &Captures| {
        let num = caps.get(1).unwrap().as_str();
        let char_low = caps.get(2).unwrap().as_str().to_lowercase();
        if let Some(&name) = VI_LETTER_NAMES.get(char_low.as_str()) {
            let mut pronunciation = name.to_string();
            if char_low == "d" && (text.to_lowercase().contains("quốc lộ") || text.to_lowercase().contains("ql")) {
                pronunciation = "đê".to_string();
            }
            format!("{} {}", num, pronunciation)
        } else {
            caps.get(0).unwrap().as_str().to_string()
        }
    }).to_string()
}

pub fn expand_prime(text: &str) -> String {
    RE_PRIME.replace_all(text, |caps: &Captures| {
        let val = caps.get(1).unwrap().as_str().to_lowercase();
        let pron = if val.chars().all(|c: char| c.is_ascii_digit()) {
            n2w_single(&val)
        } else {
            (*VI_LETTER_NAMES.get(val.as_str()).unwrap_or(&val.as_str())).to_string()
        };
        format!("{} phẩy", pron)
    }).to_string()
}

pub fn expand_temperatures(text: &str) -> String {
    let mut text = RE_TEMP_C_NEG.replace_all(text, "âm $1 độ xê").to_string();
    text = RE_TEMP_F_NEG.replace_all(&text, "âm $1 độ ép").to_string();
    text = RE_TEMP_C.replace_all(&text, "$1 độ xê").to_string();
    text = RE_TEMP_F.replace_all(&text, "$1 độ ép").to_string();
    RE_DEGREE.replace_all(&text, " độ ").to_string()
}

pub fn num_to_words(number: &str, negative: bool) -> String {
    let mut num = number.to_string();
    if RE_DOT_SEP.is_match(&num).unwrap_or(false) {
        num = num.replace(".", "");
    }
    num = num.replace(" ", "");

    if num.contains(',') {
        let parts: Vec<&str> = num.split(',').collect();
        if num.matches(',').count() == 1 {
            // In Vietnamese context for these tests, single comma is often decimal
            // 1,299 -> "một phẩy hai chín chín"
            return format!("{} phẩy {}", n2w(parts[0]), n2w_single(parts[1]));
        }
        return n2w(&num.replace(',', ""));
    }

    if negative {
        format!("âm {}", n2w(&num))
    } else {
        n2w(&num)
    }
}

pub fn normalize_number_vi(text: &str) -> String {
    let mut text_mut = RE_ORDINAL.replace_all(text, |caps: &Captures| {
        let prefix = caps.get(1).unwrap().as_str();
        let space = caps.get(2).unwrap().as_str();
        let num = caps.get(3).unwrap().as_str();
        if num == "1" { format!("{}{}nhất", prefix, space) }
        else if num == "4" { format!("{}{}tư", prefix, space) }
        else { format!("{}{}{}", prefix, space, n2w(num)) }
    }).to_string();

    text_mut = RE_MULTIPLY.replace_all(&text_mut, |caps: &Captures| {
        format!("{} nhân {}", n2w(caps.get(1).unwrap().as_str()), n2w(caps.get(3).unwrap().as_str()))
    }).to_string();

    text_mut = RE_PHONE.replace_all(&text_mut, |caps: &Captures| {
        n2w_single(caps.get(0).unwrap().as_str().trim())
    }).to_string();

    let temp_text = text_mut.clone();
    text_mut = RE_NUMBER_START.replace_all(&temp_text, |caps: &Captures| {
        let negative = caps.get(1).is_some();
        let num = caps.get(2).unwrap().as_str();
        format!("{} ", num_to_words(num, negative))
    }).to_string();

    let temp_text2 = text_mut.clone();
    text_mut = RE_NUMBER.replace_all(&temp_text2, |caps: &Captures| {
        let prefix = caps.get(1).unwrap().as_str();
        let negative = caps.get(2).is_some();
        let num = caps.get(3).unwrap().as_str();

        // Check if prefix is letter. If so, don't insert space after prefix if it might be an alphanumeric acronym part
        // but the current regex RE_NUMBER uses (\D) which includes spaces.
        if prefix.chars().all(|c| c.is_alphabetic()) && prefix.trim().len() == prefix.len() {
             format!("{} {} ", prefix, num_to_words(num, negative))
        } else {
             format!("{} {} ", prefix, num_to_words(num, negative))
        }
    }).to_string();

    text_mut
}

pub fn normalize_technical(text: &str) -> String {
    RE_TECHNICAL.replace_all(text, |caps: &Captures| {
        let orig = caps.get(0).unwrap().as_str();
        let mut rest = orig;
        let mut res = Vec::new();

        if let Some(p_idx) = orig.to_lowercase().find("://") {
            let protocol = &orig[..p_idx];
            let p_norm = if (protocol.chars().all(|c: char| c.is_uppercase()) && protocol.len() <= 4) || protocol.len() <= 3 {
                protocol.to_lowercase().chars().map(|c| c.to_string()).collect::<Vec<_>>().join(" ")
            } else {
                protocol.to_lowercase()
            };
            res.push(format!("__start_en__{}__end_en__", p_norm));
            rest = &orig[p_idx + 3..];
        } else if orig.starts_with('/') {
            res.push("gạch".to_string());
            rest = &orig[1..];
        }

        let mut segments = Vec::new();
        let mut last_pos = 0;
        for mat in RE_TECH_SPLIT.find_iter(rest) {
            let mat = mat.unwrap();
            segments.push(rest[last_pos..mat.start()].to_string());
            segments.push(mat.as_str().to_string());
            last_pos = mat.end();
        }
        if last_pos < rest.len() {
            segments.push(rest[last_pos..].to_string());
        }

        let mut idx = 0;
        while idx < segments.len() {
            let s = &segments[idx];
            if s.is_empty() {
                idx += 1;
                continue;
            }

            match s.as_str() {
                "." => {
                    let mut next_seg = "";
                    for j in (idx + 1)..segments.len() {
                        let sj = &segments[j];
                        if !sj.is_empty() && !("./:?&=/_ -\\".contains(sj)) {
                            next_seg = sj;
                            break;
                        }
                    }
                    if !next_seg.is_empty() && DOMAIN_SUFFIX_MAP.contains_key(next_seg.to_lowercase().as_str()) {
                        res.push("chấm".to_string());
                        res.push((*DOMAIN_SUFFIX_MAP.get(next_seg.to_lowercase().as_str()).unwrap()).to_string());
                        idx += 1;
                        while idx < segments.len() && (segments[idx].is_empty() || segments[idx].to_lowercase() != next_seg.to_lowercase()) {
                            idx += 1;
                        }
                        idx += 1;
                        continue;
                    }
                    res.push("chấm".to_string());
                }
                "/" | "\\" => res.push("gạch".to_string()),
                "-" => res.push("gạch ngang".to_string()),
                "_" => res.push("gạch dưới".to_string()),
                ":" => res.push("hai chấm".to_string()),
                "?" => res.push("hỏi".to_string()),
                "&" => res.push("và".to_string()),
                "=" => res.push("bằng".to_string()),
                "#" => res.push("thăng".to_string()),
                _ if DOMAIN_SUFFIX_MAP.contains_key(s.to_lowercase().as_str()) => {
                    res.push((*DOMAIN_SUFFIX_MAP.get(s.to_lowercase().as_str()).unwrap()).to_string());
                }
                _ if s.chars().all(|c: char| c.is_alphanumeric() && c.is_ascii()) => {
                    if s.chars().all(|c: char| c.is_ascii_digit()) {
                        res.push(s.chars().map(|c: char| n2w_single(&c.to_string())).collect::<Vec<_>>().join(" "));
                    } else {
                        let sub_tokens: Vec<&str> = RE_ALPHANUM_SPLIT.find_iter(s).map(|m| m.unwrap().as_str()).collect();
                        if sub_tokens.len() > 1 {
                            for t in sub_tokens {
                                if t.chars().all(|c: char| c.is_ascii_digit()) {
                                    res.push(t.chars().map(|c: char| n2w_single(&c.to_string())).collect::<Vec<_>>().join(" "));
                                } else {
                                    let mut val = t.to_lowercase();
                                    if (t.chars().all(|c: char| c.is_uppercase()) && t.len() <= 4) || t.len() <= 2 {
                                        val = val.chars().map(|c| c.to_string()).collect::<Vec<_>>().join(" ");
                                    }
                                    res.push(format!("__start_en__{}__end_en__", val));
                                }
                            }
                        } else {
                            let mut val = s.to_lowercase();
                            if (s.chars().all(|c: char| c.is_uppercase()) && s.len() <= 4) || s.len() <= 2 {
                                val = val.chars().map(|c| c.to_string()).collect::<Vec<_>>().join(" ");
                            }
                            res.push(format!("__start_en__{}__end_en__", val));
                        }
                    }
                }
                _ => {
                    for c in s.to_lowercase().chars() {
                        if c.is_alphanumeric() {
                            if c.is_ascii_digit() {
                                res.push(n2w_single(&c.to_string()));
                            } else {
                                res.push((*VI_LETTER_NAMES.get(c.to_string().as_str()).unwrap_or(&c.to_string().as_str())).to_string());
                            }
                        } else {
                            res.push(c.to_string());
                        }
                    }
                }
            }
            idx += 1;
        }
        res.join(" ").replace("  ", " ").trim().to_string()
    }).to_string()
}

pub fn normalize_emails(text: &str) -> String {
    RE_EMAIL.replace_all(text, |caps: &Captures| {
        let email = caps.get(0).unwrap().as_str();
        let parts: Vec<&str> = email.split('@').collect();
        if parts.len() != 2 { return email.to_string(); }

        let user_part = parts[0];
        let domain_part = parts[1];

        fn norm_segment(s: &str) -> String {
            if s.is_empty() { return String::new(); }
            if s.chars().all(|c: char| c.is_ascii_digit()) { return n2w(s); }
            if s.chars().all(|c: char| c.is_alphanumeric() && c.is_ascii()) {
                let sub_tokens: Vec<&str> = RE_ALPHANUM_SPLIT.find_iter(s).map(|m| m.unwrap().as_str()).collect();
                if sub_tokens.len() > 1 {
                    let mut res_parts = Vec::new();
                    for t in sub_tokens {
                        if t.chars().all(|c: char| c.is_ascii_digit()) {
                            res_parts.push(n2w(t));
                        } else {
                            res_parts.push(format!("__start_en__{}__end_en__", t.to_lowercase()));
                        }
                    }
                    return res_parts.join(" ");
                }
                return format!("__start_en__{}__end_en__", s.to_lowercase());
            }

            let mut res = Vec::new();
            for c in s.to_lowercase().chars() {
                if c.is_alphanumeric() {
                    if c.is_ascii_digit() {
                        res.push(n2w_single(&c.to_string()));
                    } else {
                        res.push((*VI_LETTER_NAMES.get(c.to_string().as_str()).unwrap_or(&c.to_string().as_str())).to_string());
                    }
                } else {
                    res.push(c.to_string());
                }
            }
            res.join(" ")
        }

        fn process_part(p: &str, is_domain: bool) -> String {
            let mut segments = Vec::new();
            let mut last_pos = 0;
            for mat in RE_EMAIL_SPLIT.find_iter(p) {
                let mat = mat.unwrap();
                segments.push(p[last_pos..mat.start()].to_string());
                segments.push(mat.as_str().to_string());
                last_pos = mat.end();
            }
            if last_pos < p.len() {
                segments.push(p[last_pos..].to_string());
            }

            let mut res = Vec::new();
            let mut idx = 0;
            while idx < segments.len() {
                let s = &segments[idx];
                if s.is_empty() {
                    idx += 1;
                    continue;
                }
                if s == "." {
                    if is_domain {
                        let mut next_seg = "";
                        let mut peek_idx = -1;
                        for j in (idx + 1)..segments.len() {
                            let sj = &segments[j];
                            if !sj.is_empty() && !("._-+".contains(sj)) {
                                next_seg = sj;
                                peek_idx = j as i32;
                                break;
                            }
                        }

                        if !next_seg.is_empty() && DOMAIN_SUFFIX_MAP.contains_key(next_seg.to_lowercase().as_str()) {
                            res.push("chấm".to_string());
                            res.push((*DOMAIN_SUFFIX_MAP.get(next_seg.to_lowercase().as_str()).unwrap()).to_string());
                            idx = peek_idx as usize + 1;
                            continue;
                        }
                    }
                    res.push("chấm".to_string());
                } else if s == "_" { res.push("gạch dưới".to_string()); }
                else if s == "-" { res.push("gạch ngang".to_string()); }
                else if s == "+" { res.push("cộng".to_string()); }
                else {
                    res.push(norm_segment(s));
                }
                idx += 1;
            }
            res.join(" ")
        }

        let user_norm = process_part(user_part, false);
        let domain_norm = if let Some(norm) = COMMON_EMAIL_DOMAINS.get(domain_part.to_lowercase().as_str()) {
            (*norm).to_string()
        } else {
            process_part(domain_part, true)
        };

        format!("{} a còng {}", user_norm, domain_norm).replace("  ", " ").trim().to_string()
    }).to_string()
}

pub fn normalize_slashes(text: &str) -> String {
    RE_SLASH_NUMBER.replace_all(text, |caps: &Captures| {
        let n1 = caps.get(1).unwrap().as_str();
        let n2 = caps.get(2).unwrap().as_str();
        if n1.len() > 2 || n1.parse::<i32>().unwrap_or(0) > 31 {
            format!("{} xẹt {}", n2w(n1), n2w(n2))
        } else {
            format!("{} trên {}", n2w(n1), n2w(n2))
        }
    }).to_string()
}

pub fn expand_power_of_ten(text: &str) -> String {
    let mut text = RE_POWER_OF_TEN_EXPLICIT.replace_all(text, |caps: &Captures| {
        let base = caps.get(1).unwrap().as_str();
        let exp = caps.get(2).unwrap().as_str();
        let base_norm = normalize_others(base);
        let exp_val = exp.replace('+', "");
        let exp_norm = if exp_val.starts_with('-') {
            format!("trừ {}", n2w(&exp_val[1..]))
        } else {
            n2w(&exp_val)
        };
        format!(" {} nhân mười mũ {} ", base_norm.trim(), exp_norm)
    }).to_string();

    text = RE_POWER_OF_TEN_IMPLICIT.replace_all(&text, |caps: &Captures| {
        let exp = caps.get(1).unwrap().as_str();
        let exp_val = exp.replace('+', "");
        let exp_norm = if exp_val.starts_with('-') {
            format!("trừ {}", n2w(&exp_val[1..]))
        } else {
            n2w(&exp_val)
        };
        format!("mười mũ {}", exp_norm)
    }).to_string();

    text
}

pub fn normalize_others(text: &str) -> String {
    let mut text = RE_ACRONYMS_EXCEPTIONS.replace_all(text, |caps: &Captures| {
        (*COMBINED_EXCEPTIONS.get(caps.get(0).unwrap().as_str()).unwrap()).to_string()
    }).to_string();

    text = normalize_slashes(&text);
    text = RE_DOMAIN_SUFFIXES.replace_all(&text, |caps: &Captures| {
        format!(" chấm {}", DOMAIN_SUFFIX_MAP.get(caps.get(1).unwrap().as_str().to_lowercase().as_str()).unwrap_or(&caps.get(1).unwrap().as_str()))
    }).to_string();

    text = expand_roman(&text);
    text = RE_LETTER.replace_all(&text, |caps: &Captures| {
        let prefix = caps.get(1).unwrap().as_str();
        let c = caps.get(3).unwrap().as_str().to_lowercase();
        if let Some(&name) = VI_LETTER_NAMES.get(c.as_str()) {
            format!("{} {} ", prefix, name)
        } else {
            caps.get(0).unwrap().as_str().to_string()
        }
    }).to_string();

    text = expand_alphanumeric(&text);
    text = expand_prime(&text);
    text = expand_unit_powers(&text);
    text = RE_CLEAN_QUOTES.replace_all(&text, "").to_string();
    text = RE_CLEAN_QUOTES_EDGES.replace_all(&text, "$1 $2").to_string();
    text = expand_symbols(&text);
    text = RE_BRACKETS.replace_all(&text, ", $1, ").to_string();
    text = RE_STRIP_BRACKETS.replace_all(&text, " ").to_string();
    text = expand_temperatures(&text);
    text = normalize_acronyms(&text);

    text = RE_VERSION.replace_all(&text, |caps: &Captures| {
        let v = caps.get(1).unwrap().as_str();
        let parts: Vec<String> = v.split('.').map(|s| n2w_single(s)).collect();
        parts.join(" chấm ")
    }).to_string();

    text = RE_COLON_SEMICOLON.replace_all(&text, ",").to_string();
    RE_CLEAN_OTHERS.replace_all(&text, " ").to_string()
}

pub fn cleanup_whitespace(text: &str) -> String {
    let mut text = RE_EXTRA_SPACES.replace_all(text, " ").to_string();
    text = RE_EXTRA_COMMAS.replace_all(&text, ",").to_string();
    text = RE_COMMA_BEFORE_PUNCT.replace_all(&text, "$1").to_string();
    text = RE_SPACE_BEFORE_PUNCT.replace_all(&text, "$1").to_string();
    text = RE_MISSING_SPACE_AFTER_PUNCT.replace_all(&text, "$1 ").to_string();
    text.trim().trim_matches(',').to_string()
}

pub fn clean_vietnamese_text(text: &str) -> String {
    let mut mask_map = HashMap::new();
    let mut text_mut = text.to_string();

    text_mut = RE_ENTOKEN.replace_all(&text_mut, |caps: &Captures| {
        let orig = caps.get(0).unwrap().as_str();
        let idx = mask_map.len();
        let mut mask = format!("mask{:0>4}mask", idx);
        let trans: Vec<(char, char)> = "0123456789".chars().zip("abcdefghij".chars()).collect();
        for (from, to) in trans {
            mask = mask.replace(from, to.to_string().as_str());
        }
        mask_map.insert(mask.clone(), orig.to_string());
        mask
    }).to_string();

    let mut temp_text = text_mut.clone();
    text_mut = RE_EMAIL.replace_all(&temp_text, |caps: &Captures| {
        let orig = caps.get(0).unwrap().as_str();
        let normed = normalize_emails(orig);
        let idx = mask_map.len();
        let mut mask = format!("mask{:0>4}mask", idx);
        let trans: Vec<(char, char)> = "0123456789".chars().zip("abcdefghij".chars()).collect();
        for (from, to) in trans {
            mask = mask.replace(from, to.to_string().as_str());
        }
        mask_map.insert(mask.clone(), normed);
        mask
    }).to_string();

    temp_text = text_mut.clone();
    text_mut = RE_TECHNICAL.replace_all(&temp_text, |caps: &Captures| {
        let orig = caps.get(0).unwrap().as_str();
        if let Some(normed_ex) = COMBINED_EXCEPTIONS.get(orig) {
            let idx = mask_map.len();
            let mut mask = format!("mask{:0>4}mask", idx);
            let trans: Vec<(char, char)> = "0123456789".chars().zip("abcdefghij".chars()).collect();
            for (from, to) in trans {
                mask = mask.replace(from, to.to_string().as_str());
            }
            mask_map.insert(mask.clone(), normed_ex.to_string());
            return mask;
        }
        let normed = normalize_technical(orig);
        let idx = mask_map.len();
        let mut mask = format!("mask{:0>4}mask", idx);
        let trans: Vec<(char, char)> = "0123456789".chars().zip("abcdefghij".chars()).collect();
        for (from, to) in trans {
            mask = mask.replace(from, to.to_string().as_str());
        }
        mask_map.insert(mask.clone(), normed);
        mask
    }).to_string();

    text_mut = expand_power_of_ten(&text_mut);
    text_mut = expand_abbreviations(&text_mut);
    text_mut = normalize_date(&text_mut);
    text_mut = normalize_time(&text_mut);

    // Range handling
    static RE_RANGE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(\d+(?:[,.]\d+)?)\s*[–\-—]\s*(\d+(?:[,.]\d+)?)").unwrap());
    static RE_DASH_TO_COMMA: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?<= )[–\-—](?= )").unwrap());
    static RE_TO_SANG: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s*(?:->|=>)\s*").unwrap());

    temp_text = text_mut.clone();
    text_mut = RE_RANGE.replace_all(&temp_text, |caps: &Captures| {
        let n1 = caps.get(1).unwrap().as_str().replace(',', "").replace('.', "");
        let n2 = caps.get(2).unwrap().as_str().replace(',', "").replace('.', "");
        if (n1.len() as i32 - n2.len() as i32).abs() <= 1 {
            format!("{} đến {}", caps.get(1).unwrap().as_str(), caps.get(2).unwrap().as_str())
        } else {
            format!("{} {}", caps.get(1).unwrap().as_str(), caps.get(2).unwrap().as_str())
        }
    }).to_string();
    temp_text = text_mut.clone();
    text_mut = RE_DASH_TO_COMMA.replace_all(&temp_text, ",").to_string();
    temp_text = text_mut.clone();
    text_mut = RE_TO_SANG.replace_all(&temp_text, " sang ").to_string();

    temp_text = text_mut.clone();
    text_mut = RE_SCIENTIFIC.replace_all(&temp_text, |caps: &Captures| expand_number_with_sep(caps.get(1).unwrap().as_str())).to_string();
    text_mut = expand_compound_units(&text_mut);
    text_mut = expand_measurement_currency(&text_mut);

    // English style numbers fix
    static RE_ENGLISH_STYLE_NUMBERS: Lazy<Regex> = Lazy::new(|| Regex::new(r"\b\d{1,3}(?:,\d{3})+(?:\.\d+)?\b").unwrap());
    temp_text = text_mut.clone();
    text_mut = RE_ENGLISH_STYLE_NUMBERS.replace_all(&temp_text, |caps: &Captures| {
        let val = caps.get(0).unwrap().as_str();
        if val.contains(',') && val.contains('.') {
            val.replace(',', "").replace('.', ",")
        } else {
            val.replace(',', "")
        }
    }).to_string();

    temp_text = text_mut.clone();
    text_mut = RE_MULTI_COMMA.replace_all(&temp_text, |caps: &Captures| {
        let s = caps.get(1).unwrap().as_str();
        let parts: Vec<String> = s.split(',').map(|p| {
            p.chars().map(|c| n2w_single(&c.to_string())).collect::<Vec<_>>().join(" ")
        }).collect();
        parts.join(" phẩy ")
    }).to_string();

    temp_text = text_mut.clone();
    text_mut = RE_FLOAT_WITH_COMMA.replace_all(&temp_text, |caps: &Captures| {
        let int_part = n2w(&caps.get(1).unwrap().as_str().replace('.', ""));
        let dec_part = caps.get(2).unwrap().as_str().trim_end_matches('0');
        let mut res = if dec_part.is_empty() {
            int_part
        } else {
            format!("{} phẩy {}", int_part, n2w_single(dec_part))
        };
        if caps.get(3).is_some() {
            res.push_str(" phần trăm");
        }
        format!(" {} ", res)
    }).to_string();

    temp_text = text_mut.clone();
    text_mut = RE_STRIP_DOT_SEP.replace_all(&temp_text, |caps: &Captures| {
        caps.get(0).unwrap().as_str().replace('.', "")
    }).to_string();

    text_mut = normalize_others(&text_mut);

    // Mask English tags before normalizing other numbers
    temp_text = text_mut.clone();
    text_mut = RE_INTERNAL_EN_TAG.replace_all(&temp_text, |caps: &Captures| {
        let orig = caps.get(0).unwrap().as_str();
        let idx = mask_map.len();
        let mut mask = format!("mask{:0>4}mask", idx);
        let trans: Vec<(char, char)> = "0123456789".chars().zip("abcdefghij".chars()).collect();
        for (from, to) in trans {
            mask = mask.replace(from, to.to_string().as_str());
        }
        mask_map.insert(mask.clone(), orig.to_string());
        mask
    }).to_string();

    text_mut = normalize_number_vi(&text_mut);

    text_mut = expand_standalone_letters(&text_mut);

    while let Ok(Some(_)) = RE_DOT_BETWEEN_DIGITS.captures(&text_mut) {
        temp_text = text_mut.clone();
        text_mut = RE_DOT_BETWEEN_DIGITS.replace_all(&temp_text, "$1 chấm $2").to_string();
    }

    for (mask, original) in mask_map {
        text_mut = text_mut.replace(&mask, &original);
        text_mut = text_mut.replace(&mask.to_lowercase(), &original);
    }

    text_mut = text_mut.replace("__start_en__", "<en>").replace("__end_en__", "</en>").replace('_', " ");
    cleanup_whitespace(&text_mut).to_lowercase()
}
