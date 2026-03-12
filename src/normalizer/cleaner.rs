use fancy_regex::{Regex, Captures, Match};
use once_cell::sync::Lazy;
use crate::normalizer::num2vi::{n2w, n2w_single};
use crate::normalizer::vi_resources::*;
use std::collections::HashMap;

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

pub static DOMAIN_SUFFIXES_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\.(com|vn|net|org|edu|gov|io|biz|info)\b").unwrap()
});

pub static RE_ACRONYM: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b(?=[A-Z0-9]*[A-Z])[A-Z0-9]{2,}\b").unwrap()
});

pub static RE_EXCEPTIONS: Lazy<Regex> = Lazy::new(|| {
    let mut keys: Vec<&str> = COMBINED_EXCEPTIONS.keys().cloned().collect();
    keys.sort_by_key(|k| std::cmp::Reverse(k.len()));
    let pattern = keys.iter().map(|k| fancy_regex::escape(k)).collect::<Vec<_>>().join("|");
    Regex::new(&format!(r"\b({})\b", pattern)).unwrap()
});

pub static RE_DOMAIN_SUFFIX: Lazy<Regex> = Lazy::new(|| {
    let mut keys: Vec<String> = DOMAIN_SUFFIX_MAP_MAP.keys().map(|k| k.to_string()).collect();
    keys.sort_by_key(|k| std::cmp::Reverse(k.len()));
    let pattern = keys.iter().map(|k| fancy_regex::escape(k)).collect::<Vec<_>>().join("|");
    Regex::new(&format!(r"(?i)\.({})\b", pattern)).unwrap()
});

pub static RE_RANGE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(\d+(?:[,.]\d+)?)\s*[–\-—]\s*(\d+(?:[,.]\d+)?)").unwrap()
});

pub static RE_DASH_TO_COMMA: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?<=\s)[–\-—](?=\s)").unwrap()
});

pub static RE_TO_SANG: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\s*(?:->|=>)\s*").unwrap()
});

pub static RE_SCIENTIFIC_NOTATION: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\b(\d+(?:[.,]\d+)?e[+-]?\d+)\b").unwrap()
});

pub static RE_ENGLISH_STYLE_NUMBERS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b\d{1,3}(?:,\d{3})+(?:\.\d+)?\b").unwrap()
});

pub static RE_POWER_OF_TEN_EXPLICIT: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\b(\d+(?:[.,]\d+)?)\s*[x*×]\s*10\^([-+]?\d+)\b").unwrap()
});

pub static RE_FULL_DATE: Lazy<Regex> = Lazy::new(|| {
    let date_sep = r"(/|-|\.)";
    Regex::new(&format!(r"(?i)\b(\d{{1,2}}){}(\d{{1,2}}){}(\d{{4}})\b", date_sep, date_sep)).unwrap()
});

pub static RE_MONTH_YEAR: Lazy<Regex> = Lazy::new(|| {
    let date_sep = r"(/|-|\.)";
    Regex::new(&format!(r"(?i)\b(\d{{1,2}}){}(\d{{4}})\b", date_sep)).unwrap()
});

pub static RE_DAY_MONTH: Lazy<Regex> = Lazy::new(|| {
    let short_date_sep = r"(/|-)";
    Regex::new(&format!(r"(?i)\b(\d{{1,2}}){}(\d{{1,2}})\b", short_date_sep)).unwrap()
});

pub static RE_FULL_TIME: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\b(\d+)(g|:|h)(\d{1,2})(p|:|m)(\d{1,2})(?:\s*(giây|s|g))?\b").unwrap()
});

pub static RE_TIME: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\b(\d+)(g|:|h)(\d{1,2})(?:\s*(phút|p|m))?\b").unwrap()
});

pub static RE_ROMAN_NUMBER: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b(?=[IVXLCDM]{2,})M{0,4}(CM|CD|D?C{0,3})(XC|XL|L?X{0,3})(IX|IV|V?I{0,3})\b").unwrap()
});

pub static RE_UNIT_POWERS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\b([a-zA-Z]+)\^([-+]?\d+)\b").unwrap()
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

pub static RE_ALPHANUMERIC: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b(\d+)([a-zA-Z])\b").unwrap()
});

pub static RE_PRIME: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(\b[a-zA-Z0-9])['’](?!\w)").unwrap()
});

pub static RE_MULTI_COMMA: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b(\d+(?:,\d+){2,})\b").unwrap()
});

pub static RE_FLOAT_WITH_COMMA: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?<![\d.])(\d+(?:\.\d{3})*),(\d+)(%)?").unwrap()
});

pub static RE_STRIP_DOT_SEP: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?<![\d.])\d+(?:\.\d{3})+(?![\d.])").unwrap()
});

pub static RE_ORDINAL: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(thứ|hạng)(\s+)(\d+)\b").unwrap()
});

pub static RE_PHONE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"((\+84|84|0|0084)(3|5|7|8|9)[0-9]{8})").unwrap()
});

pub static RE_VERSION: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b(\d+(?:\.\d+)+)\b").unwrap()
});

pub fn normalize_technical(text: &str) -> String {
    RE_TECHNICAL.replace_all(text, |caps: &Captures| {
        let orig = caps.get(0).unwrap().as_str();
        let mut rest = orig;
        let mut res = Vec::new();

        if let Some(p_idx) = orig.to_lowercase().find("://") {
            let protocol = &orig[..p_idx];
            let mut p_norm = protocol.to_lowercase();
            if (protocol.chars().all(|c: char| c.is_uppercase()) && protocol.len() <= 4) || protocol.len() <= 3 {
                p_norm = p_norm.chars().map(|c: char| c.to_string()).collect::<Vec<String>>().join(" ");
            }
            res.push(format!("__start_en__{}__end_en__", p_norm));
            rest = &orig[p_idx + 3..];
        } else if orig.starts_with('/') {
            res.push("gạch".to_string());
            rest = &orig[1..];
        }

        let segments = split_keep_delimiters(&RE_TECH_SPLIT, rest);
        let mut idx = 0;
        while idx < segments.len() {
            let s = segments[idx].as_str();
            if s.is_empty() {
                idx += 1;
                continue;
            }

            match s {
                "." => {
                    let mut next_seg = "";
                    for j in idx + 1..segments.len() {
                        let sj = segments[j].as_str();
                        if !sj.is_empty() && !r"./:?&=/_ -\".contains(sj) {
                            next_seg = sj;
                            break;
                        }
                    }
                    if !next_seg.is_empty() && DOMAIN_SUFFIX_MAP_MAP.contains_key(next_seg.to_lowercase().as_str()) {
                        res.push("chấm".to_string());
                        res.push((*DOMAIN_SUFFIX_MAP_MAP.get(next_seg.to_lowercase().as_str()).unwrap()).to_string());
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
                _ => {
                    if let Some(mapped) = DOMAIN_SUFFIX_MAP_MAP.get(s.to_lowercase().as_str()) {
                        res.push(mapped.to_string());
                    } else if s.chars().all(|c: char| c.is_alphanumeric() && c.is_ascii()) {
                        if s.chars().all(|c: char| c.is_ascii_digit()) {
                            res.push(s.chars().map(|c: char| n2w_single(&c.to_string())).collect::<Vec<String>>().join(" "));
                        } else {
                            let re_sub = Regex::new(r"[a-zA-Z]+|\d+").unwrap();
                            let sub_tokens: Vec<&str> = re_sub.find_iter(s).map(|m| m.unwrap().as_str()).collect();
                            if sub_tokens.len() > 1 {
                                for t in sub_tokens {
                                    if t.chars().all(|c: char| c.is_ascii_digit()) {
                                        res.push(t.chars().map(|c: char| n2w_single(&c.to_string())).collect::<Vec<String>>().join(" "));
                                    } else {
                                        let mut val = t.to_lowercase();
                                        if (t.chars().all(|c: char| c.is_uppercase()) && t.len() <= 4) || t.len() <= 2 {
                                            val = val.chars().map(|c: char| c.to_string()).collect::<Vec<String>>().join(" ");
                                        }
                                        res.push(format!("__start_en__{}__end_en__", val));
                                    }
                                }
                            } else {
                                if s.chars().all(|c: char| c.is_ascii_digit()) {
                                    res.push(s.chars().map(|c: char| n2w_single(&c.to_string())).collect::<Vec<String>>().join(" "));
                                } else {
                                    let mut val = s.to_lowercase();
                                    if (s.chars().all(|c: char| c.is_uppercase()) && s.len() <= 4) || val.len() <= 2 {
                                        val = val.chars().map(|c: char| c.to_string()).collect::<Vec<String>>().join(" ");
                                    }
                                    res.push(format!("__start_en__{}__end_en__", val));
                                }
                            }
                        }
                    } else {
                        for char in s.to_lowercase().chars() {
                            if char.is_alphanumeric() {
                                if char.is_ascii_digit() {
                                    res.push(n2w_single(&char.to_string()));
                                } else {
                                    let cs = char.to_string();
                                    res.push(VI_LETTER_NAMES_MAP.get(cs.as_str()).cloned().unwrap_or(cs.as_str()).to_string());
                                }
                            } else {
                                res.push(char.to_string());
                            }
                        }
                    }
                }
            }
            idx += 1;
        }
        res.join(" ").replace("  ", " ").trim().to_string()
    }).into_owned()
}

pub fn split_keep_delimiters(re: &Regex, text: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut last = 0;
    for mat in re.find_iter(text) {
        let m: Match = mat.unwrap();
        if m.start() > last {
            result.push(text[last..m.start()].to_string());
        }
        result.push(m.as_str().to_string());
        last = m.end();
    }
    if last < text.len() {
        result.push(text[last..].to_string());
    }
    result
}

pub fn normalize_emails(text: &str) -> String {
    RE_EMAIL.replace_all(text, |caps: &Captures| {
        let email = caps.get(0).unwrap().as_str();
        let parts: Vec<&str> = email.split('@').collect();
        if parts.len() != 2 {
            return email.to_string();
        }

        let user_part = parts[0];
        let domain_part = parts[1];

        let user_norm = process_email_part(user_part, false);

        let domain_part_lower = domain_part.to_lowercase();
        let domain_norm = if let Some(norm) = COMMON_EMAIL_DOMAINS_MAP.get(domain_part_lower.as_str()) {
            norm.to_string()
        } else {
            process_email_part(domain_part, true)
        };

        format!("{} a còng {}", user_norm, domain_norm).replace("  ", " ").trim().to_string()
    }).into_owned()
}

fn process_email_part(p: &str, is_domain: bool) -> String {
    let segments = split_keep_delimiters(&RE_EMAIL_SPLIT, p);
    let mut res = Vec::new();
    let mut idx = 0;
    while idx < segments.len() {
        let s = segments[idx].as_str();
        if s.is_empty() {
            idx += 1;
            continue;
        }
        match s {
            "." => {
                if is_domain {
                    let mut next_seg = None;
                    let mut peek_idx = -1;
                    for j in idx + 1..segments.len() {
                        let sj = segments[j].as_str();
                        if !sj.is_empty() && !"._-+".contains(sj) {
                            next_seg = Some(sj);
                            peek_idx = j as i32;
                            break;
                        }
                    }
                    if let Some(ns) = next_seg {
                        if let Some(mapped) = DOMAIN_SUFFIX_MAP_MAP.get(ns.to_lowercase().as_str()) {
                            res.push("chấm".to_string());
                            res.push(mapped.to_string());
                            idx = peek_idx as usize + 1;
                            continue;
                        }
                    }
                }
                res.push("chấm".to_string());
            }
            "_" => res.push("gạch dưới".to_string()),
            "-" => res.push("gạch ngang".to_string()),
            "+" => res.push("cộng".to_string()),
            _ => {
                res.push(norm_email_segment(s));
            }
        }
        idx += 1;
    }
    res.join(" ")
}

fn norm_email_segment(s: &str) -> String {
    if s.is_empty() { return "".to_string(); }
    if s.chars().all(|c: char| c.is_ascii_digit()) { return n2w(s); }
    if s.chars().all(|c: char| c.is_alphanumeric() && c.is_ascii()) {
        let re_sub = Regex::new(r"[a-zA-Z]+|\d+").unwrap();
        let sub_tokens: Vec<&str> = re_sub.find_iter(s).map(|m| m.unwrap().as_str()).collect();
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
    for char in s.to_lowercase().chars() {
        if char.is_alphanumeric() {
            if char.is_ascii_digit() {
                res.push(n2w_single(&char.to_string()));
            } else {
                let cs = char.to_string();
                res.push(VI_LETTER_NAMES_MAP.get(cs.as_str()).cloned().unwrap_or(cs.as_str()).to_string());
            }
        } else {
            res.push(char.to_string());
        }
    }
    res.join(" ")
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
    }).into_owned()
}

pub fn expand_scientific(num_str: &str) -> String {
    let num_lower = num_str.to_lowercase();
    if let Some(e_idx) = num_lower.find('e') {
        let base = &num_str[..e_idx];
        let exp = &num_str[e_idx + 1..];

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
    } else {
        num_str.to_string()
    }
}

pub fn expand_mixed_sep(num_str: &str) -> String {
    if num_str.rfind('.').unwrap_or(0) > num_str.rfind(',').unwrap_or(0) {
        let parts: Vec<&str> = num_str.split('.').collect();
        let int_part = parts[0].replace(',', "");
        let dec_part = parts[1].trim_end_matches('0');
        if dec_part.is_empty() {
            n2w(&int_part)
        } else {
            format!("{} phẩy {}", n2w(&int_part), n2w_single(dec_part))
        }
    } else {
        let parts: Vec<&str> = num_str.split(',').collect();
        let int_part = parts[0].replace('.', "");
        let dec_part = parts[1].trim_end_matches('0');
        if dec_part.is_empty() {
            n2w(&int_part)
        } else {
            format!("{} phẩy {}", n2w(&int_part), n2w_single(dec_part))
        }
    }
}

pub fn expand_single_sep(num_str: &str) -> String {
    if num_str.contains(',') {
        let parts: Vec<&str> = num_str.split(',').collect();
        if parts.len() > 2 || (parts.len() == 2 && parts[1].len() == 3) {
            return n2w(&num_str.replace(',', ""));
        }
        let dec_part = parts[1].trim_end_matches('0');
        if dec_part.is_empty() {
            n2w(parts[0])
        } else {
            format!("{} phẩy {}", n2w(parts[0]), n2w_single(dec_part))
        }
    } else if num_str.contains('.') {
        let parts: Vec<&str> = num_str.split('.').collect();
        if parts.len() > 2 || (parts.len() == 2 && parts[1].len() == 3) {
            return n2w(&num_str.replace('.', ""));
        }
        let dec_part = parts[1].trim_end_matches('0');
        if dec_part.is_empty() {
            n2w(parts[0])
        } else {
            format!("{} chấm {}", n2w(parts[0]), n2w_single(dec_part))
        }
    } else {
        n2w(num_str)
    }
}

pub fn expand_number_with_sep(num_str: &str) -> String {
    if num_str.is_empty() { return "".to_string(); }
    if num_str.to_lowercase().contains('e') { return expand_scientific(num_str); }
    if num_str.contains(',') && num_str.contains('.') { return expand_mixed_sep(num_str); }
    if num_str.contains(',') || num_str.contains('.') { return expand_single_sep(num_str); }
    n2w(num_str)
}

pub static RE_COMPOUND_UNIT: Lazy<Regex> = Lazy::new(|| {
    Regex::new(&format!(r"(?i)\b{}\s*([a-zμµ²³°]+)/([a-zμµ²³°0-9]+)\b", r"(\d+(?:[.,]\d+)*)?")).unwrap()
});

pub static ALL_UNITS_MAP: Lazy<HashMap<String, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();
    for (k, v) in MEASUREMENT_KEY_VI {
        m.insert(k.to_lowercase(), *v);
    }
    for (k, v) in CURRENCY_KEY {
        if *k != "%" {
            m.insert(k.to_lowercase(), *v);
        }
    }
    m.insert("m".to_string(), "mét");
    m
});

pub static SORTED_UNITS_KEYS: Lazy<Vec<String>> = Lazy::new(|| {
    let mut keys: Vec<String> = MEASUREMENT_KEY_VI.iter().map(|(k, _)| k.to_string()).collect();
    keys.extend(CURRENCY_KEY.iter().filter(|(k, _)| *k != "%").map(|(k, _)| k.to_string()));
    keys.sort_by_key(|k: &String| std::cmp::Reverse(k.len()));
    keys
});

pub static UNITS_RE_PATTERN: Lazy<String> = Lazy::new(|| {
    SORTED_UNITS_KEYS.iter().map(|k| fancy_regex::escape(k)).collect::<Vec<_>>().join("|")
});

pub static RE_UNITS_WITH_NUM: Lazy<Regex> = Lazy::new(|| {
    let numeric_p = r"(\d+(?:[.,]\d+)*)";
    let magnitude_p = r"(?:\s*(tỷ|triệu|nghìn|ngàn))?";
    Regex::new(&format!(r"(?i)(?<![\d.,]){}{}\s*({})\b", numeric_p, magnitude_p, *UNITS_RE_PATTERN)).unwrap()
});

pub static SAFE_STANDALONE: &[&str] = &["km", "cm", "mm", "kg", "mg", "usd", "vnd", "ph"];

pub static RE_STANDALONE_UNIT: Lazy<Regex> = Lazy::new(|| {
    let pattern = SAFE_STANDALONE.iter().map(|u| fancy_regex::escape(u)).collect::<Vec<_>>().join("|");
    Regex::new(&format!(r"(?i)(?<![\d.,])\b({})\b", pattern)).unwrap()
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

pub fn expand_measurement_currency(text: &str) -> String {
    let text = RE_UNITS_WITH_NUM.replace_all(text, |caps: &Captures| {
        let num = caps.get(1).unwrap().as_str();
        let mag = caps.get(2).map(|m: Match| m.as_str()).unwrap_or("");
        let unit = caps.get(3).unwrap().as_str();

        let full = if unit == "M" {
            "triệu"
        } else if unit == "m" {
            "mét"
        } else {
            ALL_UNITS_MAP.get(&unit.to_lowercase()).cloned().unwrap_or(unit)
        };

        let expanded_num = expand_number_with_sep(num);
        format!("{} {} {}", expanded_num, mag, full).replace("  ", " ").trim().to_string()
    }).into_owned();

    let text = RE_STANDALONE_UNIT.replace_all(&text, |caps: &Captures| {
        let unit = caps.get(1).unwrap().as_str().to_lowercase();
        format!(" {} ", ALL_UNITS_MAP.get(&unit).cloned().unwrap_or(&unit))
    }).into_owned();

    text
}

pub fn expand_currency_symbols(text: &str) -> String {
    let text = RE_CURRENCY_PREFIX_SYMBOL.replace_all(text, |caps: &Captures| {
        let symbol = caps.get(1).unwrap().as_str();
        let num = caps.get(2).unwrap().as_str();
        let mag = caps.get(3).map(|m: Match| m.as_str()).unwrap_or("");
        let full = CURRENCY_SYMBOL_MAP_MAP.get(symbol).cloned().unwrap_or("");
        let expanded_num = expand_number_with_sep(num);
        format!("{} {} {}", expanded_num, mag, full).replace("  ", " ").trim().to_string()
    }).into_owned();

    let text = RE_CURRENCY_SUFFIX_SYMBOL.replace_all(&text, |caps: &Captures| {
        let num = caps.get(1).unwrap().as_str();
        let mag = caps.get(2).map(|m: Match| m.as_str()).unwrap_or("");
        let symbol = caps.get(3).unwrap().as_str();
        let full = CURRENCY_SYMBOL_MAP_MAP.get(symbol).cloned().unwrap_or("");
        let expanded_num = expand_number_with_sep(num);
        format!("{} {} {}", expanded_num, mag, full).replace("  ", " ").trim().to_string()
    }).into_owned();

    let text = RE_PERCENTAGE.replace_all(&text, |caps: &Captures| {
        format!("{} phần trăm", expand_number_with_sep(caps.get(1).unwrap().as_str()))
    }).into_owned();

    text
}

pub fn expand_compound_units(text: &str) -> String {
    RE_COMPOUND_UNIT.replace_all(text, |caps: &Captures| {
        let num_str = caps.get(1).map(|m: Match| m.as_str()).unwrap_or("");
        if num_str.is_empty() {
            return caps.get(0).unwrap().as_str().to_string();
        }

        let num = expand_number_with_sep(num_str);
        let u1_raw = caps.get(2).unwrap().as_str();
        let u2_raw = caps.get(3).unwrap().as_str();

        let get_unit = |u_raw: &str| {
            if u_raw == "M" { return "triệu".to_string(); }
            if u_raw == "m" { return "mét".to_string(); }
            ALL_UNITS_MAP.get(&u_raw.to_lowercase()).cloned().map(|s| s.to_string()).unwrap_or(u_raw.to_string())
        };

        let full1 = get_unit(u1_raw);
        let full2 = get_unit(u2_raw);
        let mut res = format!(" {} trên {} ", full1, full2);
        if !num.is_empty() {
            res = format!("{} {}", num, res);
        }
        res
    }).into_owned()
}

pub fn fix_english_style_numbers(text: &str) -> String {
    let re = Regex::new(r"\b\d{1,3}(?:,\d{3})+(?:\.\d+)?\b").unwrap();
    re.replace_all(text, |caps: &Captures| {
        let val = caps.get(0).unwrap().as_str();
        let has_comma = val.contains(',');
        let has_dot = val.contains('.');
        if val.matches(',').count() > 1 || (has_comma && has_dot && val.find(',').unwrap() < val.find('.').unwrap()) {
            if has_dot {
                val.replace(',', "").replace('.', ",")
            } else {
                val.replace(',', "")
            }
        } else if has_comma && has_dot {
            val.replace(',', "").replace('.', ",")
        } else {
            val.to_string()
        }
    }).into_owned()
}

pub fn expand_power_of_ten(text: &str) -> String {
    RE_POWER_OF_TEN_EXPLICIT.replace_all(text, |caps: &Captures| {
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
    }).into_owned()
}

pub fn expand_scientific_notation(text: &str) -> String {
    RE_SCIENTIFIC_NOTATION.replace_all(text, |caps: &Captures| {
        expand_number_with_sep(caps.get(1).unwrap().as_str())
    }).into_owned()
}

pub static DAY_IN_MONTH: [u32; 12] = [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

pub fn is_valid_date(day: &str, month: &str) -> bool {
    if let (Ok(d), Ok(m)) = (day.parse::<u32>(), month.parse::<u32>()) {
        m >= 1 && m <= 12 && d >= 1 && d <= DAY_IN_MONTH[m as usize - 1]
    } else {
        false
    }
}

pub fn expand_full_date(text: &str) -> String {
    RE_FULL_DATE.replace_all(text, |caps: &Captures| {
        let day = caps.get(1).unwrap().as_str();
        let month = caps.get(3).unwrap().as_str();
        let year = caps.get(5).unwrap().as_str();
        if is_valid_date(day, month) {
            let m_val = if month.parse::<u32>().unwrap() == 4 { "tư".to_string() } else { n2w(&month.parse::<u32>().unwrap().to_string()) };
            format!("ngày {} tháng {} năm {}", n2w(&day.parse::<u32>().unwrap().to_string()), m_val, n2w(year))
        } else {
            caps.get(0).unwrap().as_str().to_string()
        }
    }).into_owned()
}

pub fn expand_month_year(text: &str) -> String {
    RE_MONTH_YEAR.replace_all(text, |caps: &Captures| {
        let month = caps.get(1).unwrap().as_str();
        let year = caps.get(3).unwrap().as_str();
        let m_int = month.parse::<u32>().unwrap_or(0);
        if m_int >= 1 && m_int <= 12 {
            let m_val = if m_int == 4 { "tư".to_string() } else { n2w(&m_int.to_string()) };
            format!("tháng {} năm {}", m_val, n2w(year))
        } else {
            caps.get(0).unwrap().as_str().to_string()
        }
    }).into_owned()
}

pub fn expand_day_month(text: &str) -> String {
    RE_DAY_MONTH.replace_all(text, |caps: &Captures| {
        let day = caps.get(1).unwrap().as_str();
        let month = caps.get(3).unwrap().as_str();
        if is_valid_date(day, month) {
            let m_val = if month.parse::<u32>().unwrap() == 4 { "tư".to_string() } else { n2w(&month.parse::<u32>().unwrap().to_string()) };
            format!("ngày {} tháng {}", n2w(&day.parse::<u32>().unwrap().to_string()), m_val)
        } else {
            caps.get(0).unwrap().as_str().to_string()
        }
    }).into_owned()
}

pub fn norm_time_part(s: &str) -> &str {
    if s == "00" { "0" } else { s }
}

pub fn expand_full_time(text: &str) -> String {
    RE_FULL_TIME.replace_all(text, |caps: &Captures| {
        let h = caps.get(1).unwrap().as_str();
        let m = caps.get(3).unwrap().as_str();
        let s = caps.get(5).unwrap().as_str();
        format!("{} giờ {} phút {} giây", n2w(norm_time_part(h)), n2w(norm_time_part(m)), n2w(norm_time_part(s)))
    }).into_owned()
}

pub fn expand_time(text: &str) -> String {
    RE_TIME.replace_all(text, |caps: &Captures| {
        let h_str = caps.get(1).unwrap().as_str();
        let sep = caps.get(2).unwrap().as_str();
        let m_str = caps.get(3).unwrap().as_str();
        if let (Ok(h_int), Ok(m_int)) = (h_str.parse::<u32>(), m_str.parse::<u32>()) {
            if m_int < 60 {
                if sep == ":" {
                    if h_int < 24 {
                        format!("{} giờ {} phút", n2w(norm_time_part(h_str)), n2w(norm_time_part(m_str)))
                    } else {
                        format!("{} phút {} giây", n2w(h_str), n2w(norm_time_part(m_str)))
                    }
                } else {
                    format!("{} giờ {} phút", n2w(norm_time_part(h_str)), n2w(norm_time_part(m_str)))
                }
            } else {
                caps.get(0).unwrap().as_str().to_string()
            }
        } else {
            caps.get(0).unwrap().as_str().to_string()
        }
    }).into_owned()
}

pub fn normalize_date(text: &str) -> String {
    let mut t = expand_full_date(text);
    t = expand_month_year(&t);
    t = expand_day_month(&t);
    t = Regex::new(r"(?i)\bngày\s+ngày\b").unwrap().replace_all(&t, "ngày").into_owned();
    t = Regex::new(r"(?i)\btháng\s+tháng\b").unwrap().replace_all(&t, "tháng").into_owned();
    t = Regex::new(r"(?i)\bnăm\s+năm\b").unwrap().replace_all(&t, "năm").into_owned();
    t
}

pub fn normalize_time(text: &str) -> String {
    let mut t = expand_full_time(text);
    t = expand_time(&t);
    t
}

pub fn expand_roman(text: &str) -> String {
    RE_ROMAN_NUMBER.replace_all(text, |caps: &Captures| {
        let num = caps.get(0).unwrap().as_str().to_uppercase();
        let mut result = 0;
        let chars: Vec<char> = num.chars().collect();
        for i in 0..chars.len() {
            let val = *ROMAN_NUMERALS_MAP.get(&chars[i]).unwrap();
            if (i + 1) == chars.len() || val >= *ROMAN_NUMERALS_MAP.get(&chars[i + 1]).unwrap() {
                result += val;
            } else {
                result -= val;
            }
        }
        format!(" {} ", n2w(&result.to_string()))
    }).into_owned()
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
        let full_base = ALL_UNITS_MAP.get(&base_lower).cloned().unwrap_or(base);
        format!(" {} mũ {} ", full_base, power_norm)
    }).into_owned()
}

pub fn expand_letter(text: &str) -> String {
    RE_LETTER.replace_all(text, |caps: &Captures| {
        let prefix = caps.get(1).unwrap().as_str();
        let char = caps.get(3).unwrap().as_str().to_lowercase();
        if let Some(val) = VI_LETTER_NAMES_MAP.get(char.as_str()) {
            format!("{} {} ", prefix, val)
        } else {
            caps.get(0).unwrap().as_str().to_string()
        }
    }).into_owned()
}

pub fn expand_abbreviations(text: &str) -> String {
    let mut result = text.to_string();
    for (k, v) in ABBRS {
        result = result.replace(k, v);
    }
    result
}

pub fn expand_standalone_letters(text: &str) -> String {
    RE_STANDALONE_LETTER.replace_all(text, |caps: &Captures| {
        let char_raw = caps.get(1).unwrap().as_str();
        let char = char_raw.to_lowercase();
        let dot = caps.get(2).unwrap().as_str();
        if let Some(val) = VI_LETTER_NAMES_MAP.get(char.as_str()) {
            if char_raw.chars().next().unwrap().is_uppercase() && dot == "." {
                format!(" {} ", val)
            } else {
                format!(" {}{} ", val, dot)
            }
        } else {
            caps.get(0).unwrap().as_str().to_string()
        }
    }).into_owned()
}

pub fn normalize_acronyms(text: &str) -> String {
    let sentences = split_keep_delimiters(&RE_SENTENCE_SPLIT, text);
    let mut processed = Vec::new();

    for i in (0..sentences.len()).step_by(2) {
        let s = &sentences[i];
        let sep = if i + 1 < sentences.len() { &sentences[i+1] } else { "" };
        if s.is_empty() {
            processed.push(sep.to_string());
            continue;
        }

        let words: Vec<&str> = s.split_whitespace().collect();
        let alpha_words: Vec<&str> = words.iter().filter(|&&w| w.chars().any(|c| c.is_alphabetic())).copied().collect();
        let is_all_caps = !alpha_words.is_empty() && alpha_words.iter().all(|w| w.chars().all(|c| c.is_uppercase()));

        if !is_all_caps {
            let s_norm = RE_ACRONYM.replace_all(s, |caps: &Captures| {
                let word = caps.get(0).unwrap().as_str();
                if word.chars().all(|c: char| c.is_ascii_digit()) { return word.to_string(); }
                if WORD_LIKE_ACRONYMS_SET.contains(word) {
                    return format!("__start_en__{}__end_en__", word.to_lowercase());
                }
                if word.chars().any(|c: char| c.is_ascii_digit()) {
                    let mut res = Vec::new();
                    for c in word.to_lowercase().chars() {
                        if c.is_ascii_digit() {
                            res.push(n2w_single(&c.to_string()));
                        } else {
                            let cs = c.to_string();
                            res.push(VI_LETTER_NAMES_MAP.get(cs.as_str()).cloned().map(|v| v.to_string()).unwrap_or(cs));
                        }
                    }
                    return res.join(" ");
                }
                let spaced_word = word.chars().filter(|&c| c.is_alphanumeric()).map(|c| c.to_lowercase().to_string()).collect::<Vec<String>>().join(" ");
                if !spaced_word.is_empty() {
                    format!("__start_en__{}__end_en__", spaced_word)
                } else {
                    word.to_string()
                }
            }).into_owned();
            processed.push(format!("{}{}", s_norm, sep));
        } else {
            processed.push(format!("{}{}", s, sep));
        }
    }
    processed.join("")
}

pub fn expand_alphanumeric(text: &str) -> String {
    RE_ALPHANUMERIC.replace_all(text, |caps: &Captures| {
        let num = caps.get(1).unwrap().as_str();
        let char = caps.get(2).unwrap().as_str().to_lowercase();
        if let Some(pronunciation) = VI_LETTER_NAMES_MAP.get(char.as_str()) {
            let text_lower = text.to_lowercase();
            let mut p = pronunciation.to_string();
            if char == "d" && (text_lower.contains("quốc lộ") || text_lower.contains("ql")) {
                p = "đê".to_string();
            }
            format!("{} {}", num, p)
        } else {
            caps.get(0).unwrap().as_str().to_string()
        }
    }).into_owned()
}

pub fn expand_symbols(text: &str) -> String {
    let mut result = text.to_string();
    for (s, v) in SYMBOLS_MAP {
        result = result.replace(s, v);
    }
    result
}

pub fn expand_prime(text: &str) -> String {
    RE_PRIME.replace_all(text, |caps: &Captures| {
        let val = caps.get(1).unwrap().as_str().to_lowercase();
        let expanded = if val.chars().all(|c: char| c.is_ascii_digit()) {
            n2w_single(&val)
        } else {
            VI_LETTER_NAMES_MAP.get(val.as_str()).cloned().unwrap_or(&val).to_string()
        };
        format!("{} phẩy", expanded)
    }).into_owned()
}

pub fn expand_temperatures(text: &str) -> String {
    let re_c_neg = Regex::new(r"(?i)-(\d+(?:[.,]\d+)?)\s*°\s*c\b").unwrap();
    let text = re_c_neg.replace_all(text, "âm $1 độ xê").into_owned();
    let re_f_neg = Regex::new(r"(?i)-(\d+(?:[.,]\d+)?)\s*°\s*f\b").unwrap();
    let text = re_f_neg.replace_all(&text, "âm $1 độ ép").into_owned();
    let re_c = Regex::new(r"(?i)(\d+(?:[.,]\d+)?)\s*°\s*c\b").unwrap();
    let text = re_c.replace_all(&text, "$1 độ xê").into_owned();
    let re_f = Regex::new(r"(?i)(\d+(?:[.,]\d+)?)\s*°\s*f\b").unwrap();
    let text = re_f.replace_all(&text, "$1 độ ép").into_owned();
    let re_degree = Regex::new(r"°").unwrap();
    re_degree.replace_all(&text, " độ ").into_owned()
}

pub fn normalize_others(text: &str) -> String {
    let mut text = RE_EXCEPTIONS.replace_all(text, |caps: &Captures| {
        COMBINED_EXCEPTIONS.get(caps.get(1).unwrap().as_str()).unwrap().to_string()
    }).into_owned();

    text = normalize_slashes(&text);

    text = RE_DOMAIN_SUFFIX.replace_all(&text, |caps: &Captures| {
        format!(" chấm {}", DOMAIN_SUFFIX_MAP_MAP.get(caps.get(1).unwrap().as_str().to_lowercase().as_str()).unwrap())
    }).into_owned();

    text = expand_roman(&text);
    text = expand_letter(&text);
    text = expand_alphanumeric(&text);

    text = expand_prime(&text);
    text = expand_unit_powers(&text);

    text = Regex::new(r#"["“”"]"#).unwrap().replace_all(&text, "").into_owned();
    text = Regex::new(r"(^|\s)['’]+|['’]+($|\s)").unwrap().replace_all(&text, "$1 $2").into_owned();
    text = expand_symbols(&text);

    text = Regex::new(r"[\(\[\{]\s*(.*?)\s*[\)\]\}]").unwrap().replace_all(&text, ", $1, ").into_owned();
    text = Regex::new(r"[\[\]\(\)\{\}]").unwrap().replace_all(&text, " ").into_owned();

    text = expand_temperatures(&text);
    text = normalize_acronyms(&text);

    text = RE_VERSION.replace_all(&text, |caps: &Captures| {
        let parts: Vec<String> = caps.get(1).unwrap().as_str().split('.')
            .map(|s: &str| s.chars().map(|c: char| n2w_single(&c.to_string())).collect::<Vec<String>>().join(" "))
            .collect();
        parts.join(" chấm ")
    }).into_owned();

    text = Regex::new(r"[:;]").unwrap().replace_all(&text, ",").into_owned();

    Regex::new(r"(?i)[^a-zA-Z0-9\sàáảãạăắằẳẵặâấầẩẫậèéẻẽẹêếềểễệìíỉĩịòóỏõọôốồổỗộơớờởỡợùúủũụưứừửữựỳýỷỹỵđ.,!?_\'’]").unwrap().replace_all(&text, " ").into_owned()
}

pub fn num_to_words(number: &str, negative: bool) -> String {
    let re_dot_sep = Regex::new(r"\d+(\.\d{3})+").unwrap();
    let num_cleaned = if re_dot_sep.is_match(number).unwrap() {
        number.replace('.', "")
    } else {
        number.to_string()
    }.replace(' ', "");

    if num_cleaned.contains(',') {
        let parts: Vec<&str> = num_cleaned.split(',').collect();
        format!("{} phẩy {}", n2w(parts[0]), n2w(parts[1]))
    } else if negative {
        format!("âm {}", n2w(&num_cleaned))
    } else {
        n2w(&num_cleaned)
    }
}

pub fn normalize_number_vi(text: &str) -> String {
    let normal_number_re = r"[\d]+";
    let float_number_re = r"[\d]+[,]{1}[\d]+";
    let number_with_one_dot = r"[\d]+[.]{1}[\d]{3}";
    let number_with_two_dot = r"[\d]+[.]{1}[\d]{3}[.]{1}[\d]{3}";
    let number_with_three_dot = r"[\d]+[.]{1}[\d]{3}[.]{1}[\d]{3}[.]{1}[\d]{3}";
    let number_with_one_space = r"[\d]+[\s]{1}[\d]{3}";
    let number_with_two_space = r"[\d]+[\s]{1}[\d]{3}[\s]{1}[\d]{3}";
    let number_with_three_space = r"[\d]+[\s]{1}[\d]{3}[\s]{1}[\d]{3}[\s]{1}[\d]{3}";

    let number_combined = format!("({}|{}|{}|{}|{}|{}|{}|{})",
        float_number_re, number_with_three_dot, number_with_two_dot, number_with_one_dot,
        number_with_three_space, number_with_two_space, number_with_one_space, normal_number_re
    );

    let mut text = RE_ORDINAL.replace_all(text, |caps: &Captures| {
        let prefix = caps.get(1).unwrap().as_str();
        let space = caps.get(2).unwrap().as_str();
        let number = caps.get(3).unwrap().as_str();
        if number == "1" { format!("{}{}nhất", prefix, space) }
        else if number == "4" { format!("{}{}tư", prefix, space) }
        else { format!("{}{}{}", prefix, space, n2w(number)) }
    }).into_owned();

    let re_multiply = Regex::new(&format!(r"({})([x]|\s[x]\s)({})", normal_number_re, normal_number_re)).unwrap();
    text = re_multiply.replace_all(&text, |caps: &Captures| {
        format!("{} nhân {}", n2w(caps.get(1).unwrap().as_str()), n2w(caps.get(3).unwrap().as_str()))
    }).into_owned();

    text = RE_PHONE.replace_all(&text, |caps: &Captures| {
        n2w_single(caps.get(0).unwrap().as_str().trim())
    }).into_owned();

    let re_number_start = Regex::new(&format!(r"(?m)^(-{{1}})?{}(?!\d)", number_combined)).unwrap();
    text = re_number_start.replace_all(&text, |caps: &Captures| {
        let negative = caps.get(1).is_some();
        let number = caps.get(2).unwrap().as_str();
        format!("{} ", num_to_words(number, negative))
    }).into_owned();

    let re_number = Regex::new(&format!(r"(\D)(-{{1}})?{}(?!\d)", number_combined)).unwrap();
    re_number.replace_all(&text, |caps: &Captures| {
        let prefix = caps.get(1).unwrap().as_str();
        let negative = caps.get(2).is_some();
        let number = caps.get(3).unwrap().as_str();
        format!("{} {} ", prefix, num_to_words(number, negative))
    }).into_owned()
}
