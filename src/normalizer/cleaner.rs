use aho_corasick::AhoCorasick;
use fancy_regex::{Regex as FancyRegex, Captures as FancyCaptures};
use regex::{Regex as StdRegex, Captures as StdCaptures};
use unicode_normalization::UnicodeNormalization;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use super::num2vi::{n2w, n2w_single, get_unit_word};
use super::vi_resources::*;

// Standard Regex patterns (no look-around needed)
static RE_POWER_OF_TEN_EXPLICIT: Lazy<StdRegex> = Lazy::new(|| StdRegex::new(r"(?i)\b(\d+(?:[.,]\d+)?)\s*[x*×]\s*10\^([-+]?\d+)\b").unwrap());
static RE_POWER_OF_TEN_IMPLICIT: Lazy<StdRegex> = Lazy::new(|| StdRegex::new(r"(?i)\b10\^([-+]?\d+)\b").unwrap());
static RE_RANGE: Lazy<StdRegex> = Lazy::new(|| StdRegex::new(r"\b(\d+(?:[,.]\d+)?)\s*[–\-—]\s*(\d+(?:[,.]\d+)?)\b").unwrap());
static RE_TO_SANG: Lazy<StdRegex> = Lazy::new(|| StdRegex::new(r"(?i)\s*(?:->|=>)\s*").unwrap());
static RE_ENGLISH_STYLE_NUMBERS: Lazy<StdRegex> = Lazy::new(|| StdRegex::new(r"\b\d{1,3}(?:,\d{3})+(?:\.\d+)?\b").unwrap());
static RE_MULTI_COMMA: Lazy<StdRegex> = Lazy::new(|| StdRegex::new(r"\b(\d+(?:,\d+){2,})\b").unwrap());
static RE_EXTRA_SPACES: Lazy<StdRegex> = Lazy::new(|| StdRegex::new(r"[ \t\xA0]+").unwrap());
static RE_EXTRA_COMMAS: Lazy<StdRegex> = Lazy::new(|| StdRegex::new(r",\s*,").unwrap());
static RE_COMMA_BEFORE_PUNCT: Lazy<StdRegex> = Lazy::new(|| StdRegex::new(r",\s*([.!?;])").unwrap());
static RE_SPACE_BEFORE_PUNCT: Lazy<StdRegex> = Lazy::new(|| StdRegex::new(r"\s+([,.!?;:])").unwrap());
static RE_ENTOKEN: Lazy<StdRegex> = Lazy::new(|| StdRegex::new(r"(?i)ENTOKEN\d+").unwrap());
static RE_INTERNAL_EN_TAG: Lazy<StdRegex> = Lazy::new(|| StdRegex::new(r"(?i)(__start_en__.*?__end_en__|<en>.*?</en>)").unwrap());
static RE_DOT_BETWEEN_DIGITS: Lazy<StdRegex> = Lazy::new(|| StdRegex::new(r"(\d+)\.(\d+)").unwrap());
static RE_EMAIL: Lazy<StdRegex> = Lazy::new(|| StdRegex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b").unwrap());
static RE_ORDINAL: Lazy<StdRegex> = Lazy::new(|| StdRegex::new(r"(?i)(thứ|hạng)(\s+)(\d+)\b").unwrap());
static RE_MULTIPLY: Lazy<StdRegex> = Lazy::new(|| StdRegex::new(r"(\d+)(x|\sx\s)(\d+)").unwrap());
static RE_PHONE: Lazy<StdRegex> = Lazy::new(|| StdRegex::new(r"((\+84|84|0|0084)(3|5|7|8|9)[0-9]{8})").unwrap());
static RE_COMPOUND_UNIT: Lazy<StdRegex> = Lazy::new(|| StdRegex::new(r"(?i)\b(\d+(?:[.,]\d+)*)?\s*([a-zμµ²³°]+)/([a-zμµ²³°0-9]+)\b").unwrap());
static RE_CURRENCY_PREFIX_SYMBOL: Lazy<StdRegex> = Lazy::new(|| StdRegex::new(&format!(r"(?i)([$€¥£₩])\s*{}{}", NUMERIC_P, MAGNITUDE_P)).unwrap());
static RE_CURRENCY_SUFFIX_SYMBOL: Lazy<StdRegex> = Lazy::new(|| StdRegex::new(&format!(r"(?i){}{}([$€¥£₩])", NUMERIC_P, MAGNITUDE_P)).unwrap());
static RE_PERCENTAGE: Lazy<StdRegex> = Lazy::new(|| StdRegex::new(&format!(r"(?i){}\s*%", NUMERIC_P)).unwrap());
static RE_LETTER: Lazy<StdRegex> = Lazy::new(|| StdRegex::new(r#"(?i)(chữ|chữ cái|kí tự|ký tự|kỳ tự)\s+(['"]?)([a-z])(['"]?)\b"#).unwrap());
static RE_SENTENCE_SPLIT: Lazy<StdRegex> = Lazy::new(|| StdRegex::new(r"([.!?]+(?:\s+|$))").unwrap());
static RE_ALPHANUMERIC: Lazy<StdRegex> = Lazy::new(|| StdRegex::new(r"\b(\d+)([a-zA-Z])\b").unwrap());
static RE_BRACKETS: Lazy<StdRegex> = Lazy::new(|| StdRegex::new(r"[\(\[\{]\s*(.*?)\s*[\)\]\}]").unwrap());
static RE_STRIP_BRACKETS: Lazy<StdRegex> = Lazy::new(|| StdRegex::new(r"[\[\]\(\)\{\}]").unwrap());
static RE_TEMP_C_NEG: Lazy<StdRegex> = Lazy::new(|| StdRegex::new(r"(?i)-(\d+(?:[.,]\d+)?)\s*°\s*c\b").unwrap());
static RE_TEMP_F_NEG: Lazy<StdRegex> = Lazy::new(|| StdRegex::new(r"(?i)-(\d+(?:[.,]\d+)?)\s*°\s*f\b").unwrap());
static RE_TEMP_C: Lazy<StdRegex> = Lazy::new(|| StdRegex::new(r"(?i)(\d+(?:[.,]\d+)?)\s*°\s*c\b").unwrap());
static RE_TEMP_F: Lazy<StdRegex> = Lazy::new(|| StdRegex::new(r"(?i)(\d+(?:[.,]\d+)?)\s*°\s*f\b").unwrap());
static RE_DEGREE: Lazy<StdRegex> = Lazy::new(|| StdRegex::new(r"°").unwrap());
static RE_VERSION: Lazy<StdRegex> = Lazy::new(|| StdRegex::new(r"\b(\d+(?:\.\d+)+)\b").unwrap());
static RE_CLEAN_OTHERS: Lazy<StdRegex> = Lazy::new(|| StdRegex::new(r"[^a-zA-Z0-9\sàáảãạăắằẳẵặâấầẩẫậèéẻẽẹêềếểễệìíỉĩịòóỏõọôốồổỗộơớờởỡợùúủũụưứừửữựỳýỷỹỵđÀÁẢÃẠĂẮẰẲẴẶÂẤẦẨẪẬÈÉẺẼẸÊẾỀỂỄỆÌÍỈĨỊÒÓỎÕỌÔỐỒỔỖỘƠỚỜỞỠỢÙÚỦŨỤƯỨỪỬỮỰỲÝỶỸỴĐ.,!?_\'’]").unwrap());
static RE_CLEAN_QUOTES: Lazy<StdRegex> = Lazy::new(|| StdRegex::new(r#"["“”"]"#).unwrap());
static RE_COLON_SEMICOLON: Lazy<StdRegex> = Lazy::new(|| StdRegex::new(r"[:;]").unwrap());
static RE_UNIT_POWERS: Lazy<StdRegex> = Lazy::new(|| StdRegex::new(r"\b([a-zA-Z]+)\^([-+]?\d+)\b").unwrap());
static RE_TECH_SPLIT: Lazy<StdRegex> = Lazy::new(|| StdRegex::new(r"([./:?&=/_ \-\\#])").unwrap());
static RE_EMAIL_SPLIT: Lazy<StdRegex> = Lazy::new(|| StdRegex::new(r"([._\-+])").unwrap());
static RE_SLASH_NUMBER: Lazy<StdRegex> = Lazy::new(|| StdRegex::new(r"\b(\d+)/(\d+)\b").unwrap());
static RE_DOMAIN_SUFFIXES: Lazy<StdRegex> = Lazy::new(|| StdRegex::new(r"(?i)\.(com|vn|net|org|edu|gov|io|biz|info)\b").unwrap());

// Fancy Regex patterns (look-around support needed)
static RE_DASH_TO_COMMA: Lazy<FancyRegex> = Lazy::new(|| FancyRegex::new(r"(?<= )[–\-—](?= )").unwrap());
static RE_FLOAT_WITH_COMMA: Lazy<FancyRegex> = Lazy::new(|| FancyRegex::new(r"(?<![\d.])(\d+(?:\.\d{3})*),(\d+)(%)?").unwrap());
static RE_STRIP_DOT_SEP_RE: Lazy<FancyRegex> = Lazy::new(|| FancyRegex::new(r"(?<![\d.])\d+(?:\.\d{3})+(?![\d.])").unwrap());
static RE_MISSING_SPACE_AFTER_PUNCT: Lazy<FancyRegex> = Lazy::new(|| FancyRegex::new(r"([.,!?;:])(?=[^\s\d<])").unwrap());
static RE_TECHNICAL: Lazy<FancyRegex> = Lazy::new(|| FancyRegex::new(r"(?ix)
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
").unwrap());
static RE_NUMBER_START: Lazy<FancyRegex> = Lazy::new(|| FancyRegex::new(r"(?m)^(-{1})?(\d+(?:[.,]\d+)*)(?!\d)").unwrap());
static RE_NUMBER: Lazy<FancyRegex> = Lazy::new(|| FancyRegex::new(r"(\D)(-{1})?(\d+(?:[.,]\d+)*)(?!\d)").unwrap());
static RE_UNITS_WITH_NUM: Lazy<FancyRegex> = Lazy::new(|| {
    let mut keys: Vec<String> = MEASUREMENT_KEY_VI.iter().map(|(k, _)| regex::escape(k)).collect();
    for (k, _) in CURRENCY_KEY { if *k != "%" { keys.push(regex::escape(k)); } }
    keys.sort_by_key(|k| std::cmp::Reverse(k.len()));
    let pattern = format!(r"(?i)(?<![\d.,]){}{}\s*({})\b", NUMERIC_P, MAGNITUDE_P, keys.join("|"));
    FancyRegex::new(&pattern).unwrap()
});
static RE_STANDALONE_UNIT: Lazy<FancyRegex> = Lazy::new(|| {
    let safe = ["km", "cm", "mm", "kg", "mg", "usd", "vnd", "ph"];
    let pattern = format!(r"(?i)(?<![\d.,])\b({})\b", safe.join("|"));
    FancyRegex::new(&pattern).unwrap()
});
static RE_ROMAN_NUMBER: Lazy<FancyRegex> = Lazy::new(|| FancyRegex::new(r"\b(?=[IVXLCDM]{2,})M{0,4}(CM|CD|D?C{0,3})(XC|XL|L?X{0,3})(IX|IV|V?I{0,3})\b").unwrap());
static RE_STANDALONE_LETTER: Lazy<FancyRegex> = Lazy::new(|| FancyRegex::new(r"(?<![\'’])\b([a-zA-Z])\b(\.?)").unwrap());
static RE_ACRONYM: Lazy<FancyRegex> = Lazy::new(|| FancyRegex::new(r"\b(?=[A-Z0-9]*[A-Z])[A-Z0-9]{2,}\b").unwrap());
static RE_PRIME: Lazy<FancyRegex> = Lazy::new(|| FancyRegex::new(r"(\b[a-zA-Z0-9])['’](?!\w)").unwrap());
static RE_CLEAN_QUOTES_EDGES: Lazy<FancyRegex> = Lazy::new(|| FancyRegex::new(r"(^|\s)['’]+|['’]+($|\s)").unwrap());

static MAGNITUDE_P: &str = r"(?:\s*(tỷ|triệu|nghìn|ngàn))?";
static NUMERIC_P: &str = r"(\d+(?:[.,]\d+)*)";

static RE_ACRONYMS_EXCEPTIONS_AC: Lazy<AhoCorasick> = Lazy::new(|| {
    let mut keys: Vec<String> = COMBINED_EXCEPTIONS.keys().map(|k| k.to_string()).collect();
    keys.sort_by_key(|k| std::cmp::Reverse(k.len()));
    aho_corasick::AhoCorasickBuilder::new()
        .match_kind(aho_corasick::MatchKind::LeftmostLongest)
        .build(keys)
        .unwrap()
});

static ABBRS_AC: Lazy<AhoCorasick> = Lazy::new(|| {
    let mut keys: Vec<String> = ABBRS_MAP.keys().map(|k| k.to_string()).collect();
    keys.sort_by_key(|k| std::cmp::Reverse(k.len()));
    aho_corasick::AhoCorasickBuilder::new()
        .match_kind(aho_corasick::MatchKind::LeftmostLongest)
        .build(keys)
        .unwrap()
});

static SYMBOLS_AC: Lazy<AhoCorasick> = Lazy::new(|| {
    let keys: Vec<String> = SYMBOLS_MAP_LAZY.keys().map(|k| k.to_string()).collect();
    AhoCorasick::new(keys).unwrap()
});

// Helper for Python-like split behavior
fn py_split<'a>(re: &StdRegex, text: &'a str) -> Vec<&'a str> {
    let mut result = Vec::new();
    let mut last = 0;
    for mat in re.find_iter(text) {
        if mat.start() > last {
            result.push(&text[last..mat.start()]);
        }
        result.push(mat.as_str());
        last = mat.end();
    }
    if last < text.len() {
        result.push(&text[last..]);
    }
    result
}

// Helper functions
fn expand_number_with_sep(num_str: &str) -> String {
    if num_str.is_empty() { return String::new(); }
    let num_lower = num_str.to_lowercase();
    if num_lower.contains('e') {
        let e_idx = num_lower.find('e').unwrap();
        let base = &num_str[..e_idx];
        let exp = &num_str[e_idx+1..];

        let base_norm = if base.contains('.') {
            let parts: Vec<&str> = base.split('.').collect();
            let dec = parts[1].trim_end_matches('0');
            if dec.is_empty() { n2w(parts[0]) } else { format!("{} chấm {}", n2w(parts[0]), n2w_single(dec)) }
        } else if base.contains(',') {
            let parts: Vec<&str> = base.split(',').collect();
            let dec = parts[1].trim_end_matches('0');
            if dec.is_empty() { n2w(parts[0]) } else { format!("{} phẩy {}", n2w(parts[0]), n2w_single(dec)) }
        } else {
            n2w(&base.replace(',', "").replace('.', ""))
        };

        let exp_val = exp.trim_start_matches('+');
        let exp_norm = if exp_val.starts_with('-') { format!("trừ {}", n2w(&exp_val[1..])) } else { n2w(exp_val) };
        return format!("{} nhân mười mũ {}", base_norm, exp_norm);
    }

    if num_str.contains(',') && num_str.contains('.') {
        let r_dot = num_str.rfind('.').unwrap();
        let r_comma = num_str.rfind(',').unwrap();
        if r_dot > r_comma { // English
            let s = num_str.replace(',', "");
            let parts: Vec<&str> = s.split('.').collect();
            let dec = parts[1].trim_end_matches('0');
            if dec.is_empty() { return n2w(&parts[0]); }
            return format!("{} phẩy {}", n2w(&parts[0]), n2w_single(dec));
        } else { // Vietnamese
            let s = num_str.replace('.', "");
            let parts: Vec<&str> = s.split(',').collect();
            let dec = parts[1].trim_end_matches('0');
            if dec.is_empty() { return n2w(&parts[0]); }
            return format!("{} phẩy {}", n2w(&parts[0]), n2w_single(dec));
        }
    }

    if num_str.contains(',') {
        let parts: Vec<&str> = num_str.split(',').collect();
        if parts.len() > 2 || (parts.len() == 2 && parts[1].len() == 3) {
            return n2w(&num_str.replace(',', ""));
        }
        let dec = parts[1].trim_end_matches('0');
        if dec.is_empty() { return n2w(parts[0]); }
        return format!("{} phẩy {}", n2w(parts[0]), n2w_single(dec));
    }

    if num_str.contains('.') {
        let parts: Vec<&str> = num_str.split('.').collect();
        if parts.len() > 2 || (parts.len() == 2 && parts[1].len() == 3) {
            return n2w(&num_str.replace('.', ""));
        }
        let dec = parts[1].trim_end_matches('0');
        if dec.is_empty() { return n2w(parts[0]); }
        return format!("{} chấm {}", n2w(parts[0]), n2w_single(dec));
    }

    n2w(num_str)
}

fn normalize_technical(text: &str) -> String {
    RE_TECHNICAL.replace_all(text, |cap: &FancyCaptures| {
        let orig = cap.get(0).unwrap().as_str();
        let mut rest = orig;
        let mut res = Vec::new();

        if let Some(p_idx) = orig.to_lowercase().find("://") {
            let protocol = &orig[..p_idx];
            let p_norm = if (protocol.chars().all(|c: char| c.is_uppercase()) && protocol.len() <= 4) || protocol.len() <= 3 {
                protocol.chars().map(|c: char| c.to_lowercase().to_string()).collect::<Vec<String>>().join(" ")
            } else {
                protocol.to_lowercase()
            };
            res.push(format!("__start_en__{}__end_en__", p_norm));
            rest = &orig[p_idx+3..];
        } else if orig.starts_with('/') {
            res.push("gạch".to_string());
            rest = &orig[1..];
        }

        let seg_vec = py_split(&RE_TECH_SPLIT, rest);
        let mut idx = 0;
        while idx < seg_vec.len() {
            let s = seg_vec[idx];
            if s.is_empty() { idx += 1; continue; }

            match s {
                "." => {
                    let mut next_seg = "";
                    for j in (idx + 1)..seg_vec.len() {
                        if !seg_vec[j].is_empty() && !matches!(seg_vec[j], "." | "/" | ":" | "?" | "&" | "=" | "_" | "-" | "\\" | "#") {
                            next_seg = seg_vec[j];
                            break;
                        }
                    }
                    if !next_seg.is_empty() && DOMAIN_SUFFIX_MAP_LAZY.contains_key(&next_seg.to_lowercase()) {
                        res.push("chấm".to_string());
                        res.push(DOMAIN_SUFFIX_MAP_LAZY.get(&next_seg.to_lowercase()).unwrap().clone());
                        idx += 1;
                        while idx < seg_vec.len() && (seg_vec[idx].is_empty() || seg_vec[idx].to_lowercase() != next_seg.to_lowercase()) {
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
                _ => {
                    let lower = s.to_lowercase();
                    if let Some(v) = DOMAIN_SUFFIX_MAP_LAZY.get(&lower) {
                        res.push(v.clone());
                    } else if s.chars().all(|c: char| c.is_alphanumeric() && c.is_ascii()) {
                        if s.chars().all(|c: char| c.is_ascii_digit()) {
                            res.push(n2w_single(s));
                        } else {
                            let std_re_sub = regex::Regex::new(r"[a-zA-Z]+|\d+").unwrap();
                            let sub_tokens: Vec<&str> = std_re_sub.find_iter(s).map(|m| m.as_str()).collect();
                            if sub_tokens.len() > 1 {
                                for t in sub_tokens {
                                    if t.chars().all(|c: char| c.is_ascii_digit()) {
                                        res.push(n2w_single(t));
                                    } else {
                                        let mut val = t.to_lowercase();
                                        if (t.chars().all(|c: char| c.is_uppercase()) && t.len() <= 4) || t.len() <= 2 {
                                            val = val.chars().map(|c: char| c.to_lowercase().to_string()).collect::<Vec<String>>().join(" ");
                                        }
                                        res.push(format!("__start_en__{}__end_en__", val));
                                    }
                                }
                            } else {
                                let mut val = s.to_lowercase();
                                if (s.chars().all(|c: char| c.is_uppercase()) && s.len() <= 4) || s.len() <= 2 {
                                    val = val.chars().map(|c: char| c.to_lowercase().to_string()).collect::<Vec<String>>().join(" ");
                                }
                                res.push(format!("__start_en__{}__end_en__", val));
                            }
                        }
                    } else {
                        for char in s.chars() {
                            if char.is_alphanumeric() {
                                if char.is_ascii_digit() { res.push(n2w_single(&char.to_string())); }
                                else { res.push(LETTER_KEY_VI.get(&char.to_lowercase().to_string()).cloned().unwrap_or(char.to_string())); }
                            } else { res.push(char.to_string()); }
                        }
                    }
                }
            }
            idx += 1;
        }
        res.join(" ").replace("  ", " ").trim().to_string()
    }).to_string()
}

fn normalize_emails(text: &str) -> String {
    RE_EMAIL.replace_all(text, |cap: &StdCaptures| {
        let email = cap.get(0).unwrap().as_str();
        let parts: Vec<&str> = email.split('@').collect();
        if parts.len() != 2 { return email.to_string(); }

        let user_part = parts[0];
        let domain_part = parts[1];

        fn norm_segment(s: &str) -> String {
            if s.is_empty() { return String::new(); }
            if s.chars().all(|c: char| c.is_ascii_digit()) { return n2w(s); }
            if s.chars().all(|c: char| c.is_alphanumeric() && c.is_ascii()) {
                let std_re_sub = StdRegex::new(r"[a-zA-Z]+|\d+").unwrap();
                let sub_tokens: Vec<&str> = std_re_sub.find_iter(s).map(|m| m.as_str()).collect();
                if sub_tokens.len() > 1 {
                    let mut res_parts = Vec::new();
                    for t in sub_tokens {
                        if t.chars().all(|c: char| c.is_ascii_digit()) { res_parts.push(n2w(t)); }
                        else { res_parts.push(format!("__start_en__{}__end_en__", t.to_lowercase())); }
                    }
                    return res_parts.join(" ");
                }
                return format!("__start_en__{}__end_en__", s.to_lowercase());
            }
            let mut res = Vec::new();
            for char in s.chars() {
                if char.is_alphanumeric() {
                    if char.is_ascii_digit() { res.push(n2w_single(&char.to_string())); }
                    else { res.push(LETTER_KEY_VI.get(&char.to_lowercase().to_string()).cloned().unwrap_or(char.to_string())); }
                } else { res.push(char.to_string()); }
            }
            res.join(" ")
        }

        fn process_part(p: &str, is_domain: bool) -> String {
            let seg_vec = py_split(&RE_EMAIL_SPLIT, p);
            let mut res = Vec::new();
            let mut idx = 0;
            while idx < seg_vec.len() {
                let s = seg_vec[idx];
                if s.is_empty() { idx += 1; continue; }
                match s {
                    "." => {
                        if is_domain {
                            let mut next_seg = "";
                            let mut peek_idx = -1;
                            for j in (idx+1)..seg_vec.len() {
                                if !seg_vec[j].is_empty() && !matches!(seg_vec[j], "." | "_" | "-" | "+") {
                                    next_seg = seg_vec[j];
                                    peek_idx = j as i32;
                                    break;
                                }
                            }
                            if !next_seg.is_empty() && DOMAIN_SUFFIX_MAP_LAZY.contains_key(&next_seg.to_lowercase()) {
                                res.push("chấm".to_string());
                                res.push(DOMAIN_SUFFIX_MAP_LAZY.get(&next_seg.to_lowercase()).unwrap().clone());
                                idx = (peek_idx + 1) as usize;
                                continue;
                            }
                        }
                        res.push("chấm".to_string());
                    }
                    "_" => res.push("gạch dưới".to_string()),
                    "-" => res.push("gạch ngang".to_string()),
                    "+" => res.push("cộng".to_string()),
                    _ => res.push(norm_segment(s)),
                }
                idx += 1;
            }
            res.join(" ")
        }

        let user_norm = process_part(user_part, false);
        let domain_norm = if let Some(v) = EMAIL_DOMAINS_MAP.get(&domain_part.to_lowercase()) {
            v.clone()
        } else {
            process_part(domain_part, true)
        };

        format!("{} a còng {}", user_norm, domain_norm).replace("  ", " ").trim().to_string()
    }).to_string()
}

fn normalize_pre_number(mut text: String) -> String {
    text = RE_POWER_OF_TEN_EXPLICIT.replace_all(&text, |cap: &StdCaptures| {
        let base = cap.get(1).unwrap().as_str();
        let exp = cap.get(2).unwrap().as_str();
        let base_norm = normalize_others(base.to_string());
        let exp_val = exp.replace('+', "");
        let exp_norm = if exp_val.starts_with('-') { format!("trừ {}", n2w(&exp_val[1..])) } else { n2w(&exp_val) };
        format!(" {} nhân mười mũ {} ", base_norm.trim(), exp_norm)
    }).to_string();

    text = RE_POWER_OF_TEN_IMPLICIT.replace_all(&text, |cap: &StdCaptures| {
        let exp = cap.get(1).unwrap().as_str();
        let exp_val = exp.replace('+', "");
        let exp_norm = if exp_val.starts_with('-') { format!("trừ {}", n2w(&exp_val[1..])) } else { n2w(&exp_val) };
        format!("mười mũ {}", exp_norm)
    }).to_string();

    // Use Aho-Corasick for abbreviations
    let mut new_text = String::with_capacity(text.len());
    let mut last_match = 0;
    for mat in ABBRS_AC.find_iter(&text) {
        new_text.push_str(&text[last_match..mat.start()]);
        let key = &text[mat.start()..mat.end()];
        new_text.push_str(ABBRS_MAP.get(key).unwrap());
        last_match = mat.end();
    }
    new_text.push_str(&text[last_match..]);
    text = new_text;

    text = normalize_date(text);
    text = normalize_time(text);

    text = RE_RANGE.replace_all(&text, |cap: &StdCaptures| {
        let n1_raw = cap.get(1).unwrap().as_str();
        let n2_raw = cap.get(2).unwrap().as_str();
        let n1 = n1_raw.replace(',', "").replace('.', "");
        let n2 = n2_raw.replace(',', "").replace('.', "");
        if (n1.len() as i32 - n2.len() as i32).abs() <= 1 {
            format!("{} đến {}", n1_raw, n2_raw)
        } else {
            format!("{} - {}", n1_raw, n2_raw)
        }
    }).to_string();

    text = RE_DASH_TO_COMMA.replace_all(&text, |_: &FancyCaptures| ",").to_string();
    text = RE_TO_SANG.replace_all(&text, |_: &StdCaptures| " sang ").to_string();

    text
}

fn normalize_date(mut text: String) -> String {
    let is_valid_date = |d: &str, m: &str| {
        let day_in_month = [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
        if let (Ok(dv), Ok(mv)) = (d.parse::<usize>(), m.parse::<usize>()) {
            mv >= 1 && mv <= 12 && dv >= 1 && dv <= day_in_month[mv - 1]
        } else { false }
    };

    static RE_FULL_DATE_STD: Lazy<StdRegex> = Lazy::new(|| StdRegex::new(r"(?i)\b(\d{1,2})(/|-|\.)(\d{1,2})(/|-|\.)(\d{4})\b").unwrap());
    static RE_DAY_MONTH_STD: Lazy<StdRegex> = Lazy::new(|| StdRegex::new(r"(?i)\b(\d{1,2})(/|-)(\d{1,2})\b").unwrap());
    static RE_MONTH_YEAR_STD: Lazy<StdRegex> = Lazy::new(|| StdRegex::new(r"(?i)\b(\d{1,2})(/|-|\.)(\d{4})\b").unwrap());

    text = RE_FULL_DATE_STD.replace_all(&text, |cap: &StdCaptures| {
        let d = cap.get(1).unwrap().as_str();
        let m = cap.get(3).unwrap().as_str();
        let y = cap.get(5).unwrap().as_str();
        if is_valid_date(d, m) {
            let m_val = if m.parse::<i32>().unwrap() == 4 { "tư".to_string() } else { n2w(&m.parse::<i32>().unwrap().to_string()) };
            format!("ngày {} tháng {} năm {}", n2w(&d.parse::<i32>().unwrap().to_string()), m_val, n2w(y))
        } else { cap.get(0).unwrap().as_str().to_string() }
    }).to_string();

    text = RE_MONTH_YEAR_STD.replace_all(&text, |cap: &StdCaptures| {
        let m = cap.get(1).unwrap().as_str();
        let y = cap.get(3).unwrap().as_str();
        let m_val = if m.parse::<i32>().unwrap() == 4 { "tư".to_string() } else { n2w(&m.parse::<i32>().unwrap().to_string()) };
        format!("tháng {} năm {}", m_val, n2w(y))
    }).to_string();

    text = RE_DAY_MONTH_STD.replace_all(&text, |cap: &StdCaptures| {
        let d = cap.get(1).unwrap().as_str();
        let m = cap.get(3).unwrap().as_str();
        if is_valid_date(d, m) {
            let m_val = if m.parse::<i32>().unwrap() == 4 { "tư".to_string() } else { n2w(&m.parse::<i32>().unwrap().to_string()) };
            format!("ngày {} tháng {}", n2w(&d.parse::<i32>().unwrap().to_string()), m_val)
        } else { cap.get(0).unwrap().as_str().to_string() }
    }).to_string();

    text = StdRegex::new(r"(?i)\bngày\s+ngày\b").unwrap().replace_all(&text, "ngày").to_string();
    text = StdRegex::new(r"(?i)\btháng\s+tháng\b").unwrap().replace_all(&text, "tháng").to_string();
    text = StdRegex::new(r"(?i)\bnăm\s+năm\b").unwrap().replace_all(&text, "năm").to_string();
    text
}

fn normalize_time(mut text: String) -> String {
    static RE_FULL_TIME_STD: Lazy<StdRegex> = Lazy::new(|| StdRegex::new(r"(?i)\b(\d+)(g|:|h)(\d{1,2})(p|:|m)(\d{1,2})(?:\s*(giây|s|g))?\b").unwrap());
    static RE_TIME_SHORT_STD: Lazy<StdRegex> = Lazy::new(|| StdRegex::new(r"(?i)\b(\d+)(g|:|h)(\d{1,2})(?:\s*(phút|p|m))?\b").unwrap());

    text = RE_FULL_TIME_STD.replace_all(&text, |cap: &StdCaptures| {
        let h = cap.get(1).unwrap().as_str();
        let m = cap.get(3).unwrap().as_str();
        let s = cap.get(5).unwrap().as_str();
        format!("{} giờ {} phút {} giây", n2w(if h == "00" { "0" } else { h }), n2w(if m == "00" { "0" } else { m }), n2w(if s == "00" { "0" } else { s }))
    }).to_string();

    text = RE_TIME_SHORT_STD.replace_all(&text, |cap: &StdCaptures| {
        let h = cap.get(1).unwrap().as_str();
        let sep = cap.get(2).unwrap().as_str();
        let m = cap.get(3).unwrap().as_str();
        if let (Ok(hv), Ok(mv)) = (h.parse::<i32>(), m.parse::<i32>()) {
            if mv >= 0 && mv < 60 {
                let h_norm = if h == "00" { "0" } else { h };
                let m_norm = if m == "00" { "0" } else { m };
                if sep == ":" {
                    if hv < 24 { format!("{} giờ {} phút", n2w(h_norm), n2w(m_norm)) }
                    else { format!("{} phút {} giây", n2w(h), n2w(m_norm)) }
                } else {
                    format!("{} giờ {} phút", n2w(h_norm), n2w(m_norm))
                }
            } else { cap.get(0).unwrap().as_str().to_string() }
        } else { cap.get(0).unwrap().as_str().to_string() }
    }).to_string();

    text
}

fn normalize_units_currency(mut text: String) -> String {
    text = StdRegex::new(r"(?i)\b(\d+(?:[.,]\d+)?e[+-]?\d+)\b").unwrap().replace_all(&text, |cap: &StdCaptures| {
        expand_number_with_sep(cap.get(1).unwrap().as_str())
    }).to_string();

    text = RE_COMPOUND_UNIT.replace_all(&text, |cap: &StdCaptures| {
        let num_str = cap.get(1).map(|m| m.as_str()).unwrap_or("");
        if num_str.is_empty() { return cap.get(0).unwrap().as_str().to_string(); }
        let num_norm = expand_number_with_sep(num_str);
        let u1 = cap.get(2).unwrap().as_str();
        let u2 = cap.get(3).unwrap().as_str();
        let full1 = if u1 == "M" { "triệu" } else if u1 == "m" { "mét" } else { ALL_UNITS_MAP.get(&u1.to_lowercase()).map(|s| s.as_str()).unwrap_or(u1) };
        let full2 = if u2 == "M" { "triệu" } else if u2 == "m" { "mét" } else { ALL_UNITS_MAP.get(&u2.to_lowercase()).map(|s| s.as_str()).unwrap_or(u2) };
        format!("{} {} trên {} ", num_norm, full1, full2)
    }).to_string();

    text = RE_UNITS_WITH_NUM.replace_all(&text, |cap: &FancyCaptures| {
        let num = cap.get(1).unwrap().as_str();
        let mag = cap.get(2).map(|m| m.as_str()).unwrap_or("");
        let unit = cap.get(3).unwrap().as_str();
        let full = if unit == "M" { "triệu" } else if unit == "m" { "mét" } else { ALL_UNITS_MAP.get(&unit.to_lowercase()).map(|s| s.as_str()).unwrap_or(unit) };
        format!("{} {} {}", expand_number_with_sep(num), mag, full).replace("  ", " ").trim().to_string()
    }).to_string();

    text = RE_STANDALONE_UNIT.replace_all(&text, |cap: &FancyCaptures| {
        let unit = cap.get(1).unwrap().as_str().to_lowercase();
        format!(" {} ", ALL_UNITS_MAP.get(&unit).cloned().unwrap_or(unit))
    }).to_string();

    let repl_symbol = |cap: &StdCaptures, is_prefix: bool| {
        let symbol = if is_prefix { cap.get(1).unwrap().as_str() } else { cap.get(3).unwrap().as_str() };
        let num = if is_prefix { cap.get(2).unwrap().as_str() } else { cap.get(1).unwrap().as_str() };
        let mag = if is_prefix { cap.get(3).map(|m| m.as_str()).unwrap_or("") } else { cap.get(2).map(|m| m.as_str()).unwrap_or("") };
        let full = CURRENCY_SYMBOL_MAP_LAZY.get(symbol).cloned().unwrap_or_default();
        format!("{} {} {}", expand_number_with_sep(num), mag, full).replace("  ", " ").trim().to_string()
    };

    text = RE_CURRENCY_PREFIX_SYMBOL.replace_all(&text, |cap: &StdCaptures| repl_symbol(cap, true)).to_string();
    text = RE_CURRENCY_SUFFIX_SYMBOL.replace_all(&text, |cap: &StdCaptures| repl_symbol(cap, false)).to_string();
    text = RE_PERCENTAGE.replace_all(&text, |cap: &StdCaptures| format!("{} phần trăm", expand_number_with_sep(cap.get(1).unwrap().as_str()))).to_string();

    text = RE_ENGLISH_STYLE_NUMBERS.replace_all(&text, |cap: &StdCaptures| {
        let val = cap.get(0).unwrap().as_str();
        let has_comma = val.contains(',');
        let has_dot = val.contains('.');
        if val.matches(',').count() > 1 || (has_comma && has_dot && val.find(',').unwrap() < val.find('.').unwrap()) {
            if has_dot { val.replace(',', "").replace('.', ",") } else { val.replace(',', "") }
        } else if has_comma && has_dot {
            val.replace(',', "").replace('.', ",")
        } else {
            val.to_string()
        }
    }).to_string();

    text = RE_MULTI_COMMA.replace_all(&text, |cap: &StdCaptures| {
        cap.get(1).unwrap().as_str().split(',').map(|s| n2w_single(s)).collect::<Vec<String>>().join(" phẩy ")
    }).to_string();

    text = RE_FLOAT_WITH_COMMA.replace_all(&text, |cap: &FancyCaptures| {
        let int_part = n2w(&cap.get(1).unwrap().as_str().replace('.', ""));
        let dec_part = cap.get(2).unwrap().as_str().trim_end_matches('0');
        let mut res = if dec_part.is_empty() { int_part } else { format!("{} phẩy {}", int_part, n2w_single(dec_part)) };
        if cap.get(3).is_some() { res.push_str(" phần trăm"); }
        format!(" {} ", res)
    }).to_string();

    text = RE_STRIP_DOT_SEP_RE.replace_all(&text, |cap: &FancyCaptures| cap.get(0).unwrap().as_str().replace('.', "")).to_string();

    text
}

fn normalize_post_number(mut text: String) -> String {
    text = normalize_others(text.clone());
    text = normalize_number_vi(text);
    text
}

fn normalize_others(mut text: String) -> String {
    // 1. Aho-Corasick for exceptions
    let mut new_text = String::with_capacity(text.len());
    let mut last_match = 0;
    for mat in RE_ACRONYMS_EXCEPTIONS_AC.find_iter(&text) {
        new_text.push_str(&text[last_match..mat.start()]);
        let key = &text[mat.start()..mat.end()];
        let replacement = COMBINED_EXCEPTIONS.get(key).unwrap();

        if mat.start() > 0 && !new_text.is_empty() && !new_text.ends_with(|c: char| c.is_whitespace()) && !replacement.starts_with(|c: char| c.is_whitespace()) {
            new_text.push(' ');
        }

        new_text.push_str(replacement);
        last_match = mat.end();
    }
    new_text.push_str(&text[last_match..]);
    text = new_text;

    text = RE_SLASH_NUMBER.replace_all(&text, |cap: &StdCaptures| {
        let n1 = cap.get(1).unwrap().as_str();
        let n2 = cap.get(2).unwrap().as_str();
        if n1.len() > 2 || n1.parse::<i32>().unwrap_or(0) > 31 { format!("{} xẹt {}", n2w(n1), n2w(n2)) }
        else { format!("{} trên {}", n2w(n1), n2w(n2)) }
    }).to_string();

    text = RE_DOMAIN_SUFFIXES.replace_all(&text, |cap: &StdCaptures| {
        format!(" chấm {} ", DOMAIN_SUFFIX_MAP_LAZY.get(&cap.get(1).unwrap().as_str().to_lowercase()).cloned().unwrap_or(cap.get(1).unwrap().as_str().to_string()))
    }).to_string();

    text = RE_ROMAN_NUMBER.replace_all(&text, |cap: &FancyCaptures| {
        let num = cap.get(0).unwrap().as_str().to_uppercase();
        let mut result = 0;
        let chars: Vec<char> = num.chars().collect();
        for i in 0..chars.len() {
            let v = *ROMAN_NUMERALS_MAP.get(&chars[i]).unwrap();
            if i + 1 < chars.len() && v < *ROMAN_NUMERALS_MAP.get(&chars[i+1]).unwrap() { result -= v; }
            else { result += v; }
        }
        format!(" {} ", n2w(&result.to_string()))
    }).to_string();

    text = RE_LETTER.replace_all(&text, |cap: &StdCaptures| {
        let prefix = cap.get(1).unwrap().as_str();
        let c = cap.get(3).unwrap().as_str().to_lowercase();
        if let Some(v) = LETTER_KEY_VI.get(&c) { format!("{} {} ", prefix, v) }
        else { cap.get(0).unwrap().as_str().to_string() }
    }).to_string();

    text = RE_ALPHANUMERIC.replace_all(&text, |cap: &StdCaptures| {
        let num = cap.get(1).unwrap().as_str();
        let c = cap.get(2).unwrap().as_str().to_lowercase();
        if let Some(v) = LETTER_KEY_VI.get(&c) {
            let mut pron = v.clone();
            let lower = text.to_lowercase();
            if c == "d" && (lower.contains("quốc lộ") || lower.contains("ql")) { pron = "đê".to_string(); }
            format!("{} {}", num, pron)
        } else { cap.get(0).unwrap().as_str().to_string() }
    }).to_string();

    text = RE_PRIME.replace_all(&text, |cap: &FancyCaptures| {
        let val = cap.get(1).unwrap().as_str().to_lowercase();
        format!("{} phẩy", if val.chars().all(|c: char| c.is_ascii_digit()) { n2w_single(&val) } else { LETTER_KEY_VI.get(&val).cloned().unwrap_or(val.to_string()) })
    }).to_string();

    text = RE_UNIT_POWERS.replace_all(&text, |cap: &StdCaptures| {
        let base = cap.get(1).unwrap().as_str();
        let power = cap.get(2).unwrap().as_str();
        let p_norm = if power.starts_with('-') { format!("trừ {}", n2w(&power[1..])) } else { n2w(&power.replace('+', "")) };
        let b_lower = base.to_lowercase();
        let full_b = ALL_UNITS_MAP.get(&b_lower).cloned().unwrap_or(base.to_string());
        format!(" {} mũ {} ", full_b, p_norm)
    }).to_string();

    text = RE_CLEAN_QUOTES.replace_all(&text, "").to_string();
    text = RE_CLEAN_QUOTES_EDGES.replace_all(&text, |cap: &FancyCaptures| {
        let g1 = cap.get(1).map(|m| m.as_str()).unwrap_or("");
        let g2 = cap.get(2).map(|m| m.as_str()).unwrap_or("");
        format!("{}{}", g1, g2)
    }).to_string();

    // Use Aho-Corasick for symbols
    let mut new_text = String::with_capacity(text.len());
    let mut last_match = 0;
    for mat in SYMBOLS_AC.find_iter(&text) {
        new_text.push_str(&text[last_match..mat.start()]);
        let key = &text[mat.start()..mat.end()];
        new_text.push_str(SYMBOLS_MAP_LAZY.get(key).unwrap());
        last_match = mat.end();
    }
    new_text.push_str(&text[last_match..]);
    text = new_text;

    text = RE_BRACKETS.replace_all(&text, ", $1, ").to_string();
    text = RE_STRIP_BRACKETS.replace_all(&text, " ").to_string();

    text = RE_TEMP_C_NEG.replace_all(&text, "âm $1 độ xê").to_string();
    text = RE_TEMP_F_NEG.replace_all(&text, "âm $1 độ ép").to_string();
    text = RE_TEMP_C.replace_all(&text, "$1 độ xê").to_string();
    text = RE_TEMP_F.replace_all(&text, "$1 độ ép").to_string();
    text = RE_DEGREE.replace_all(&text, " độ ").to_string();

    text = normalize_acronyms(&text);

    text = RE_VERSION.replace_all(&text, |cap: &StdCaptures| {
        cap.get(1).unwrap().as_str().split('.').map(|s| n2w_single(s)).collect::<Vec<String>>().join(" chấm ")
    }).to_string();

    text = RE_COLON_SEMICOLON.replace_all(&text, ",").to_string();
    RE_CLEAN_OTHERS.replace_all(&text, " ").to_string()
}

fn normalize_acronyms(text: &str) -> String {
    let caps = py_split(&RE_SENTENCE_SPLIT, text);
    let mut processed = Vec::new();
    for i in (0..caps.len()).step_by(2) {
        let mut s = caps[i].to_string();
        let sep = if i+1 < caps.len() { caps[i+1] } else { "" };
        if s.is_empty() { processed.push(sep.to_string()); continue; }

        s = RE_ACRONYM.replace_all(&s, |cap: &FancyCaptures| {
            let word = cap.get(0).unwrap().as_str();
            if word.chars().all(|c: char| c.is_ascii_digit()) { return word.to_string(); }
            if WORD_LIKE_ACRONYMS.contains(&word) { return format!("__start_en__{}__end_en__", word.to_lowercase()); }

            let mut is_numeric_acronym = false;
            if word.chars().any(|c: char| c.is_ascii_digit()) {
                let letters = word.chars().filter(|c| c.is_alphabetic()).count();
                let digits = word.chars().filter(|c| c.is_ascii_digit()).count();
                if letters > 0 && digits > 0 && word.len() <= 10 {
                    is_numeric_acronym = true;
                }
            }

            if is_numeric_acronym {
                return word.chars().map(|c: char| if c.is_ascii_digit() { get_unit_word(c).to_string() } else { LETTER_KEY_VI.get(&c.to_lowercase().to_string()).cloned().unwrap_or(c.to_string()) }).collect::<Vec<String>>().join(" ");
            }
            let spaced: String = word.chars().filter(|c: &char| c.is_alphanumeric()).map(|c: char| c.to_lowercase().to_string()).collect::<Vec<String>>().join(" ");
            if spaced.is_empty() { word.to_string() } else { format!("__start_en__{}__end_en__", spaced) }
        }).to_string();
        processed.push(format!("{}{}", s, sep));
    }
    processed.join("")
}

fn normalize_number_vi(mut text: String) -> String {
    text = RE_ORDINAL.replace_all(&text, |cap: &StdCaptures| {
        let prefix = cap.get(1).unwrap().as_str();
        let space = cap.get(2).unwrap().as_str();
        let num = cap.get(3).unwrap().as_str();
        if num == "1" { format!("{}{}{}", prefix, space, "nhất") }
        else if num == "4" { format!("{}{}{}", prefix, space, "tư") }
        else { format!("{}{}{}", prefix, space, n2w(num)) }
    }).to_string();

    text = RE_MULTIPLY.replace_all(&text, |cap: &StdCaptures| {
        format!("{} nhân {}", n2w(cap.get(1).unwrap().as_str()), n2w(cap.get(3).unwrap().as_str()))
    }).to_string();

    text = RE_PHONE.replace_all(&text, |cap: &StdCaptures| n2w_single(cap.get(0).unwrap().as_str().trim())).to_string();

    text = RE_NUMBER_START.replace_all(&text, |cap: &FancyCaptures| {
        let neg = cap.get(1).is_some();
        let num = cap.get(2).unwrap().as_str();
        num_to_words(num, neg)
    }).to_string();

    text = RE_NUMBER.replace_all(&text, |cap: &FancyCaptures| {
        let prefix = cap.get(1).unwrap().as_str();
        let neg = cap.get(2).is_some();
        let num = cap.get(3).unwrap().as_str();
        format!("{} {}", prefix, num_to_words(num, neg))
    }).to_string();

    text
}

fn num_to_words(num: &str, negative: bool) -> String {
    let clean = num.replace('.', "").replace(' ', "");
    let mut res = if clean.contains(',') {
        let parts: Vec<&str> = clean.split(',').collect();
        format!("{} phẩy {}", n2w(parts[0]), n2w(parts[1]))
    } else {
        n2w(&clean)
    };
    if negative { res = format!("âm {}", res); }
    res
}

fn cleanup_whitespace(mut text: String) -> String {
    text = RE_EXTRA_SPACES.replace_all(&text, " ").to_string();
    text = RE_EXTRA_COMMAS.replace_all(&text, ",").to_string();
    text = RE_COMMA_BEFORE_PUNCT.replace_all(&text, "$1").to_string();
    text = RE_SPACE_BEFORE_PUNCT.replace_all(&text, "$1").to_string();
    text = RE_MISSING_SPACE_AFTER_PUNCT.replace_all(&text, |cap: &FancyCaptures| format!("{} ", cap.get(1).unwrap().as_str())).to_string();
    text.trim().trim_matches(',').to_string()
}

pub fn clean_vietnamese_text(mut text: String) -> String {
    let mut mask_map = HashMap::new();
    let mut mask_counter = 0;

    let mut protect = |content: &str| {
        let mask = format!("mask{:04}mask", mask_counter).chars().map(|c: char| {
            if c.is_ascii_digit() { (c as u8 - b'0' + b'a') as char } else { c }
        }).collect::<String>();
        mask_map.insert(mask.clone(), content.to_string());
        mask_counter += 1;
        mask
    };

    text = RE_ENTOKEN.replace_all(&text, |cap: &StdCaptures| protect(cap.get(0).unwrap().as_str())).to_string();

    let mut protect_url_email = |orig: &str| {
        if orig.contains('@') { return protect(&normalize_emails(orig)); }
        if let Some(v) = COMBINED_EXCEPTIONS.get(orig) { return protect(v); }
        protect(&normalize_technical(orig))
    };

    text = RE_EMAIL.replace_all(&text, |cap: &StdCaptures| protect_url_email(cap.get(0).unwrap().as_str())).to_string();
    text = RE_TECHNICAL.replace_all(&text, |cap: &FancyCaptures| protect_url_email(cap.get(0).unwrap().as_str())).to_string();

    text = normalize_pre_number(text);
    text = normalize_units_currency(text);
    text = normalize_post_number(text);

    text = RE_INTERNAL_EN_TAG.replace_all(&text, |cap: &StdCaptures| protect(cap.get(0).unwrap().as_str())).to_string();

    text = RE_STANDALONE_LETTER.replace_all(&text, |cap: &FancyCaptures| {
        let c = cap.get(1).unwrap().as_str();
        let dot = cap.get(2).map(|m| m.as_str()).unwrap_or("");
        let lower = c.to_lowercase();
        if let Some(v) = LETTER_KEY_VI.get(&lower) {
            if c.chars().next().unwrap().is_uppercase() && dot == "." { format!(" {} ", v) }
            else if dot == "." { format!(" {}. ", v) }
            else { format!(" {} ", v) }
        } else { cap.get(0).unwrap().as_str().to_string() }
    }).to_string();

    while RE_DOT_BETWEEN_DIGITS.is_match(&text) {
        text = RE_DOT_BETWEEN_DIGITS.replace_all(&text, |cap: &StdCaptures| format!("{} chấm {}", cap.get(1).unwrap().as_str(), cap.get(2).unwrap().as_str())).to_string();
    }

    for (mask, original) in &mask_map {
        text = text.replace(mask, original);
        text = text.replace(&mask.to_lowercase(), original);
    }

    text = text.replace("__start_en__", "<en>").replace("__end_en__", "</en>").replace('_', " ");
    let cleaned = cleanup_whitespace(text).to_lowercase();
    cleaned.nfc().collect::<String>()
}
