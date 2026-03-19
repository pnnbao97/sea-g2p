use fancy_regex::{Regex, Captures};
use once_cell::sync::Lazy;
use crate::vi_normalizer::num2vi::{n2w, n2w_single};
use crate::vi_normalizer::resources::{VI_LETTER_NAMES, COMMON_EMAIL_DOMAINS, DOMAIN_SUFFIX_MAP};

static RE_TECH_SPLIT: Lazy<regex::Regex> = Lazy::new(|| regex::Regex::new(r"([./:?&=/_ \-\\#])").unwrap());
static RE_EMAIL_SPLIT: Lazy<regex::Regex> = Lazy::new(|| regex::Regex::new(r"([._\-+])").unwrap());
static RE_SUB_TOKENS: Lazy<regex::Regex> = Lazy::new(|| regex::Regex::new(r"[a-zA-Z]+|\d+").unwrap());

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

pub static RE_SLASH_NUMBER: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?<![a-zA-Z\d,.])(\d+)/(\d+)(?![\d,.])").unwrap()
});

static RE_NEG_FRAC: Lazy<regex::Regex> = Lazy::new(|| {
    regex::Regex::new(r"(?:=|\s)-((\d+)/(\d+))").unwrap()
});

pub fn normalize_technical(text: &str) -> String {
    RE_TECHNICAL.replace_all(text, |caps: &Captures| {
        let orig = caps.get(0).unwrap().as_str();
        let mut rest = orig;
        let mut res = Vec::new();

        if let Some(p_idx) = orig.to_lowercase().find("://") {
            let protocol = &orig[..p_idx];
            let p_norm = if (protocol.chars().all(|c: char| c.is_uppercase()) && protocol.len() <= 4) || protocol.len() <= 3 {
                protocol.to_lowercase().chars().map(|c: char| c.to_string()).collect::<Vec<String>>().join(" ")
            } else {
                protocol.to_lowercase()
            };
            res.push(format!("__start_en__{}__end_en__", p_norm));
            rest = &orig[p_idx + 3..];
        } else if orig.starts_with('/') {
            res.push("gạch".to_string());
            rest = &orig[1..];
        }

        let re_split = &*RE_TECH_SPLIT;
        let mut segments_vec = Vec::new();
        let mut last = 0;
        for mat in re_split.find_iter(rest) {
            segments_vec.push(&rest[last..mat.start()]);
            segments_vec.push(mat.as_str());
            last = mat.end();
        }
        segments_vec.push(&rest[last..]);

        let mut idx = 0;
        while idx < segments_vec.len() {
            let s = segments_vec[idx];
            if s.is_empty() { idx += 1; continue; }

            match s {
                "." => {
                    let mut next_seg = "";
                    for j in idx + 1..segments_vec.len() {
                        let sj = segments_vec[j];
                        if !sj.is_empty() && !("./:?&=/_ -\\".contains(sj)) {
                            next_seg = sj;
                            break;
                        }
                    }
                    if !next_seg.is_empty() && DOMAIN_SUFFIX_MAP.contains_key(next_seg.to_lowercase().as_str()) {
                        res.push("chấm".to_string());
                        res.push(DOMAIN_SUFFIX_MAP.get(next_seg.to_lowercase().as_str()).unwrap().to_string());
                        idx += 1;
                        while idx < segments_vec.len() && (segments_vec[idx].is_empty() || segments_vec[idx].to_lowercase() != next_seg.to_lowercase()) {
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
                    if let Some(suffix) = DOMAIN_SUFFIX_MAP.get(s.to_lowercase().as_str()) {
                        res.push(suffix.to_string());
                    } else if s.chars().all(|c: char| c.is_alphanumeric() && c.is_ascii()) {
                        if s.chars().all(|c: char| c.is_ascii_digit()) {
                            res.push(s.chars().map(|c: char| n2w_single(&c.to_string())).collect::<Vec<String>>().join(" "));
                        } else {
                            let re_sub = &*RE_SUB_TOKENS;
                            let sub_tokens: Vec<&str> = re_sub.find_iter(s).map(|m: regex::Match| m.as_str()).collect();
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
                                let mut val = s.to_lowercase();
                                if (s.chars().all(|c: char| c.is_uppercase()) && s.len() <= 4) || s.len() <= 2 {
                                    val = val.chars().map(|c: char| c.to_string()).collect::<Vec<String>>().join(" ");
                                }
                                res.push(format!("__start_en__{}__end_en__", val));
                            }
                        }
                    } else {
                        for char in s.to_lowercase().chars() {
                            if char.is_alphanumeric() {
                                if char.is_ascii_digit() {
                                    res.push(n2w_single(&char.to_string()));
                                } else {
                                    res.push(VI_LETTER_NAMES.get(char.to_string().as_str()).cloned().unwrap_or(char.to_string().as_str()).to_string());
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
    }).to_string()
}

pub fn normalize_emails(text: &str) -> String {
    RE_EMAIL.replace_all(text, |caps: &Captures| {
        let email = caps.get(0).unwrap().as_str();
        let parts: Vec<&str> = email.split('@').collect();
        if parts.len() != 2 { return email.to_string(); }

        let user_part = parts[0];
        let domain_part = parts[1];

        let norm_segment = |s: &str| {
            if s.is_empty() { return String::new(); }
            if s.chars().all(|c: char| c.is_ascii_digit()) { return n2w(s); }
            if s.chars().all(|c: char| c.is_alphanumeric() && c.is_ascii()) {
                let re_sub = &*RE_SUB_TOKENS;
                let sub_tokens: Vec<&str> = re_sub.find_iter(s).map(|m: regex::Match| m.as_str()).collect();
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
                        res.push(VI_LETTER_NAMES.get(char.to_string().as_str()).cloned().unwrap_or(char.to_string().as_str()).to_string());
                    }
                } else {
                    res.push(char.to_string());
                }
            }
            res.join(" ")
        };

        let process_part = |p: &str, is_domain: bool| {
            let re_split = &*RE_EMAIL_SPLIT;
            let mut segments_vec = Vec::new();
            let mut last = 0;
            for mat in re_split.find_iter(p) {
                segments_vec.push(&p[last..mat.start()]);
                segments_vec.push(mat.as_str());
                last = mat.end();
            }
            segments_vec.push(&p[last..]);

            let mut res = Vec::new();
            let mut idx = 0;
            while idx < segments_vec.len() {
                let s = segments_vec[idx];
                if s.is_empty() { idx += 1; continue; }
                match s {
                    "." => {
                        if is_domain {
                            let mut next_seg = "";
                            let mut peek_idx = -1;
                            for j in idx + 1..segments_vec.len() {
                                let sj = segments_vec[j];
                                if !sj.is_empty() && !("._-+".contains(sj)) {
                                    next_seg = sj;
                                    peek_idx = j as i32;
                                    break;
                                }
                            }
                            if !next_seg.is_empty() && DOMAIN_SUFFIX_MAP.contains_key(next_seg.to_lowercase().as_str()) {
                                res.push("chấm".to_string());
                                res.push(DOMAIN_SUFFIX_MAP.get(next_seg.to_lowercase().as_str()).unwrap().to_string());
                                idx = peek_idx as usize + 1;
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
        };

        let user_norm = process_part(user_part, false);
        let domain_part_lower = domain_part.to_lowercase();
        let domain_norm = if let Some(dn) = COMMON_EMAIL_DOMAINS.get(domain_part_lower.as_str()) {
            dn.to_string()
        } else {
            process_part(domain_part, true)
        };

        format!("{} a còng {}", user_norm, domain_norm).replace("  ", " ").trim().to_string()
    }).to_string()
}

pub fn normalize_slashes(text: &str) -> String {
    let res = RE_NEG_FRAC.replace_all(text, |caps: &regex::Captures| {
        let matched = caps.get(0).unwrap().as_str();
        let frac = caps.get(1).unwrap().as_str();
        let prefix = if matched.starts_with('=') { "= âm " } else { " âm " };
        format!("{}{}", prefix, frac)
    }).into_owned();

    RE_SLASH_NUMBER.replace_all(&res, |caps: &Captures| {
        let n1 = caps.get(1).unwrap().as_str();
        let n2 = caps.get(2).unwrap().as_str();
        format!("{} trên {}", n2w(n1), n2w(n2))
    }).to_string()
}
