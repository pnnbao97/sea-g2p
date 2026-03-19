use fancy_regex::{Regex, Captures};
use once_cell::sync::Lazy;
use crate::vi_normalizer::num2vi::{n2w, n2w_decimal};
use crate::vi_normalizer::resources::{DATE_KEYWORDS, MATH_KEYWORDS};

const DAY_IN_MONTH: [i32; 12] = [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
const DATE_SEP: &str = r"(\/|-|\.)";
const SHORT_DATE_SEP: &str = r"(\/|-)";

static RE_FULL_DATE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(&format!(r"(?<![a-zA-Z\d])(?<![a-zA-Z\d][.,])(\d{{1,2}}){}{{{}}}(\d{{1,2}}){}{{{}}}(\d{{4}})(?!\d|[.,]\d)", DATE_SEP, 1, DATE_SEP, 1)).unwrap()
});

static RE_YYYY_MM_DD: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?<![a-zA-Z\d])(?<![a-zA-Z\d][.,])(\d{4})-(\d{2})-(\d{2})(?!\d|[.,]\d)").unwrap()
});

static RE_ISO_FIX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(\d{2})T(\d{2})|(\d{2})Z\b").unwrap()
});


static RE_DAY_MONTH: Lazy<Regex> = Lazy::new(|| {
    Regex::new(&format!(r"(?<![a-zA-Z\d])(?<![a-zA-Z\d][.,])(\d{{1,2}}){}{{{}}}(\d{{1,2}})(?!\d|[.,]\d)", SHORT_DATE_SEP, 1)).unwrap()
});

static RE_MONTH_YEAR: Lazy<Regex> = Lazy::new(|| {
    Regex::new(&format!(r"(?<![a-zA-Z\d])(?<![a-zA-Z\d][.,])(\d{{1,2}}){}{{{}}}(\d{{4}})(?!\d|[.,]\d)", DATE_SEP, 1)).unwrap()
});

static RE_PERIOD_YEAR: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\b([a-zA-Z]\d*)/(\d{4})\b").unwrap()
});

static RE_FULL_TIME: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\b(\d+)(g|:|h)(\d{1,2})(p|:|m)(\d{1,2})(?:\s*(giây|s|g))?\b").unwrap()
});

static RE_TIME: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\b(\d+)(g|:|h)(\d{1,2})(?:\s*(phút|p|m))?\b").unwrap()
});

static RE_HOUR_CONTEXT: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\b(\d+)g\s*(sáng|trưa|chiều|tối|khuya)\b").unwrap()
});

static RE_LUC_HOUR: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\blúc\s*(\d+)g\b").unwrap()
});

static RE_REDUNDANT_NGAY: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\bngày\s+ngày\b").unwrap()
});

static RE_REDUNDANT_THANG: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\btháng\s+tháng\b").unwrap()
});

static RE_REDUNDANT_NAM: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\bnăm\s+năm\b").unwrap()
});

fn is_valid_date(day: &str, month: &str) -> bool {
    let day: i32 = day.parse().unwrap_or(0);
    let month: i32 = month.parse().unwrap_or(0);
    if month >= 1 && month <= 12 {
        return day >= 1 && day <= DAY_IN_MONTH[month as usize - 1];
    }
    false
}

fn get_context_words(text: &str, start: usize, end: usize, window_size: usize) -> Vec<String> {
    let left_part: Vec<&str> = text[..start].split_whitespace().collect();
    let right_part: Vec<&str> = text[end..].split_whitespace().collect();

    let mut context = Vec::new();
    let left_start = if left_part.len() > window_size { left_part.len() - window_size } else { 0 };
    for &w in &left_part[left_start..] {
        context.push(w.trim_matches(|c: char| ",.!?;()[]{}".contains(c)).to_lowercase());
    }
    for &w in &right_part[..std::cmp::min(right_part.len(), window_size)] {
        context.push(w.trim_matches(|c: char| ",.!?;()[]{}".contains(c)).to_lowercase());
    }
    context
}

fn norm_time_part(s: &str) -> &str {
    if s == "00" { "0" } else { s }
}

pub fn normalize_date(text: &str) -> String {
    let mut result = text.to_string();

    result = RE_ISO_FIX.replace_all(&result, |caps: &Captures| {
        if let Some(m) = caps.get(1) {
            format!("{} T {}", m.as_str(), caps.get(2).unwrap().as_str())
        } else {
            format!("{} Z ", caps.get(3).unwrap().as_str())
        }
    }).to_string();

    result = RE_YYYY_MM_DD.replace_all(&result, |caps: &Captures| {
        let year = caps.get(1).unwrap().as_str();
        let month = caps.get(2).unwrap().as_str();
        let day = caps.get(3).unwrap().as_str();
        if is_valid_date(day, month) {
            let m_val = if month.parse::<i32>().unwrap_or(0) == 4 { "tư".to_string() } else { n2w(&month.parse::<i32>().unwrap_or(0).to_string()) };
            format!("ngày {} tháng {} năm {}", n2w(&day.parse::<i32>().unwrap_or(0).to_string()), m_val, n2w(year))
        } else {
            caps.get(0).unwrap().as_str().to_string()
        }
    }).to_string();

    result = RE_PERIOD_YEAR.replace_all(&result, |caps: &Captures| {
        let code = caps.get(1).unwrap().as_str();
        let year = caps.get(2).unwrap().as_str();
        let code_lower = code.to_lowercase();
        
        let prefix = if code_lower.starts_with('q') && code.len() <= 2 {
            let q_num = &code[1..];
            format!("quý {}", if q_num.is_empty() { "".to_string() } else { n2w(q_num) })
        } else {
            let mut parts = Vec::new();
            for c in code.chars() {
                let cl = c.to_lowercase().to_string();
                if c.is_ascii_digit() {
                    parts.push(n2w(&c.to_string()));
                } else if let Some(name) = crate::vi_normalizer::resources::VI_LETTER_NAMES.get(cl.as_str()) {
                    parts.push(name.to_string());
                } else {
                    parts.push(cl);
                }
            }
            parts.join(" ")
        };

        format!("{} {}", prefix.trim(), n2w_decimal(year))
    }).to_string();

    result = RE_FULL_DATE.replace_all(&result, |caps: &Captures| {
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

    result = RE_MONTH_YEAR.replace_all(&result, |caps: &Captures| {
        let month_str = caps.get(1).unwrap().as_str();
        let year_str = caps.get(3).unwrap().as_str();
        let m = month_str.parse::<i32>().unwrap_or(0);
        let y = year_str.parse::<i32>().unwrap_or(0);
        if m >= 1 && m <= 12 && y <= 2500 {
            let m_val = if m == 4 { "tư".to_string() } else { n2w(&m.to_string()) };
            format!("tháng {} năm {}", m_val, n2w(&y.to_string()))
        } else {
            caps.get(0).unwrap().as_str().to_string()
        }
    }).to_string();

    let current_text = result.clone();
    result = RE_DAY_MONTH.replace_all(&current_text, |caps: &Captures| {
        let full_match = caps.get(0).unwrap();
        let day_str = caps.get(1).unwrap().as_str();
        let month_str = caps.get(3).unwrap().as_str();
        let a = day_str.parse::<i32>().unwrap_or(0);
        let b = month_str.parse::<i32>().unwrap_or(0);

        let context_words = get_context_words(&current_text, full_match.start(), full_match.end(), 3);
        let math_symbols = ["+", "-", "*", "x", "×", "/", "=", ">", "<", "≥", "≤", "≈", "±"];
        let is_valid = is_valid_date(day_str, month_str);

        if context_words.iter().any(|w: &String| MATH_KEYWORDS.contains(w.as_str())) ||
           context_words.iter().any(|w: &String| math_symbols.contains(&w.as_str())) {
            return format!("{} trên {}", n2w(day_str), n2w(month_str));
        }

        let month_has_leading_zero = month_str.starts_with('0') && month_str.len() > 1;
        let day_has_leading_zero = day_str.starts_with('0') && day_str.len() > 1;
        if is_valid && (month_has_leading_zero || day_has_leading_zero) {
            let m_val = if b == 4 { "tư".to_string() } else { n2w(&b.to_string()) };
            return format!("ngày {} tháng {}", n2w(&a.to_string()), m_val);
        }

        if is_valid && context_words.iter().any(|w: &String| DATE_KEYWORDS.contains(w.as_str())) {
            let m_val = if b == 4 { "tư".to_string() } else { n2w(&b.to_string()) };
            return format!("ngày {} tháng {}", n2w(&a.to_string()), m_val);
        }

        if !is_valid {
            if day_str.starts_with('0') || month_str.starts_with('0') {
                return format!("{} trên {}", n2w(day_str), n2w(month_str));
            }
            return full_match.as_str().to_string();
        }

        format!("{} trên {}", n2w(day_str), n2w(month_str))
    }).to_string();

    result = RE_REDUNDANT_NGAY.replace_all(&result, "ngày").into_owned();
    result = RE_REDUNDANT_THANG.replace_all(&result, "tháng").into_owned();
    result = RE_REDUNDANT_NAM.replace_all(&result, "năm").into_owned();

    result
}

pub fn normalize_time(text: &str) -> String {
    let mut result = RE_FULL_TIME.replace_all(text, |caps: &Captures| {
        format!("{} giờ {} phút {} giây",
            n2w(norm_time_part(caps.get(1).unwrap().as_str())),
            n2w(norm_time_part(caps.get(3).unwrap().as_str())),
            n2w(norm_time_part(caps.get(5).unwrap().as_str()))
        )
    }).to_string();

    result = RE_TIME.replace_all(&result, |caps: &Captures| {
        let full_match = caps.get(0).unwrap().as_str();
        let h_str = caps.get(1).unwrap().as_str();
        let sep = caps.get(2).unwrap().as_str();
        let m_str = caps.get(3).unwrap().as_str();

        let h_int = h_str.parse::<i32>().unwrap_or(-1);
        let m_int = m_str.parse::<i32>().unwrap_or(-1);

        // Heuristic: If separator is uppercase H or G, it's likely chemistry (e.g., 2H2)
        // unless it's followed by a time unit (phút, p, m).
        if (sep == "H" || sep == "G") && !full_match.to_lowercase().contains("phút") && !full_match.to_lowercase().contains(" p") && !full_match.to_lowercase().contains(" m") {
            return full_match.to_string();
        }

        if m_int >= 0 && m_int < 60 {
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
            full_match.to_string()
        }
    }).to_string();

    result = RE_HOUR_CONTEXT.replace_all(&result, |caps: &Captures| {
        format!("{} giờ {}", n2w(caps.get(1).unwrap().as_str()), caps.get(2).unwrap().as_str())
    }).to_string();

    result = RE_LUC_HOUR.replace_all(&result, |caps: &Captures| {
        format!("lúc {} giờ", n2w(caps.get(1).unwrap().as_str()))
    }).to_string();

    result
}
