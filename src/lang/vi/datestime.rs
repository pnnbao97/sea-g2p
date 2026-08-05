//! Dates and clock times — pipeline stage 10.
//!
//! # The central ambiguity
//!
//! `d/m` and a fraction are written identically. "1/8" is the first of August
//! or one eighth, and both readings are common, so the decision rests entirely
//! on context. Three signals are used, in decreasing order of confidence:
//!
//!   1. **A leading zero** ("01/8") — only ever written for dates.
//!   2. **A lead word immediately before** ("chiều 17/10", "trước 30/4"), via
//!      [`DATE_LEAD_WORDS`]. Adjacency matters: "chiều dài 3/4 mét" must stay a
//!      fraction, and it does because "dài" sits in between.
//!   3. **A date keyword within three words** ("cuộc họp vào ngày 15/3"), via
//!      [`DATE_KEYWORDS`]. Wider reach, so it accepts only well-formed dates.
//!
//! Mathematical context ([`MATH_KEYWORDS`], nearby operators) forces the
//! fraction reading regardless.
//!
//! # Ranges
//!
//! In "từ 1/8 đến hết 31/8" the second date has no lead word of its own, so
//! `RE_DATE_RANGE` matches the whole span and converts both halves together.
//!
//! # Duplicate lead words
//!
//! Expansion inserts "ngày", which collides with a "ngày" the author already
//! wrote. The collapse rule fires **only when a number follows**, because
//! reduplication is ordinary Vietnamese: "ngày ngày vẫn đông khách" must keep
//! both words. Getting this wrong deletes a word from the sentence with nothing
//! audible to signal the loss — see `tests/test_invariants.py`.

use fancy_regex::{Regex, Captures};
use once_cell::sync::Lazy;
use crate::lang::vi::num2vi::{n2w, n2w_decimal};
use crate::lang::vi::resources::{DATE_KEYWORDS, DATE_LEAD_WORDS, MATH_KEYWORDS};

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

// "quý 1/2025" -> "quý một năm hai nghìn…". Runs before RE_MONTH_YEAR, which
// would otherwise read "1/2025" as a month and year. Roman quarters are matched
// here too ("quý III/2027") because the general Roman-numeral pass comes after
// the date stage and so arrives too late.
static RE_QUY_YEAR: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\b(quý)\s+(\d{1,2}|IV|III|II|I)\s*/\s*(\d{4})\b").unwrap()
});

// Full time like 10:30:15 or 10g30p15s
static RE_FULL_TIME: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\b(\d+)(g|:|h)(\d{1,2})(p|:|m)(\d{1,2})(?:\s*(giây|s|g))?\b").unwrap()
});

// Plain clock times: 10:30, 14h30, "10:20 phút".
// Captured groups: 1:hour, 2:separator, 3:minute, 4:suffix (optional)
static RE_TIME: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?ix)
        \b(\d+)(g|h|:)(\d{1,2})(?:\s*(phút|p|m|giây|s|g))?\b(?![.,]\d)
    ").unwrap()
});

static RE_HOUR_CONTEXT: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\b(\d+)g\s*(sáng|trưa|chiều|tối|khuya)\b").unwrap()
});

static RE_LUC_HOUR: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\blúc\s*(\d+)g\b").unwrap()
});

// Date expansion can produce "ngày ngày …" when the author already wrote the
// lead word ("ngày 15/3" becomes "ngày" + "ngày mười lăm…"). Collapse the pair
// **only when a number follows**; otherwise it is genuine reduplication
// ("ngày ngày năn nỉ", "suốt năm năm trời", "tháng tháng đóng tiền") and
// deleting one half silently removes a word from the sentence.
pub static NUM_WORDS: Lazy<std::collections::HashSet<&'static str>> = Lazy::new(|| {
    ["không", "một", "hai", "ba", "bốn", "năm", "sáu", "bảy", "tám", "chín",
     "mười", "mươi", "mốt", "lăm", "tư", "trăm", "nghìn", "ngàn", "triệu", "tỷ"]
        .into_iter().collect()
});

fn dedup_lead(text: &str, re: &Regex, keep: &str) -> String {
    re.replace_all(text, |caps: &Captures| {
        let next = caps.get(1).unwrap().as_str();
        if NUM_WORDS.contains(next.to_lowercase().as_str()) {
            format!("{} {}", keep, next)
        } else {
            caps.get(0).unwrap().as_str().to_string()
        }
    }).into_owned()
}

// "từ 1/8 đến hết 31/8", "ngày 20/11 - 25/11": the whole span is a date range.
static RE_DATE_RANGE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\b(từ|ngày|hôm)\s+(\d{1,2})/(\d{1,2})\s*(đến hết|đến|tới|-|–)\s*(\d{1,2})/(\d{1,2})\b").unwrap()
});

static RE_REDUNDANT_NGAY: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\bngày\s+ngày\s+(\S+)").unwrap()
});

static RE_REDUNDANT_THANG: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\btháng\s+tháng\s+(\S+)").unwrap()
});

static RE_REDUNDANT_NAM: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\bnăm\s+năm\s+(\S+)").unwrap()
});

// Same principle as dedup_lead: drop "ngày" only when the date pass inserted it,
// i.e. when a number follows. "Hôm ngày lễ" and "mùng ngày rằm" must survive.
static RE_REDUNDANT_HOM_NGAY: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\bhôm\s+ngày\s+(\S+)").unwrap()
});

static RE_REDUNDANT_MUNG_NGAY: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\b(mùng|mồng)\s+ngày\s+(\S+)").unwrap()
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
    let trimmed = s.trim_start_matches('0');
    if trimmed.is_empty() { "0" } else { trimmed }
}

pub fn normalize_date(text: &str) -> String {
    let mut result = text.to_string();

    result = RE_QUY_YEAR.replace_all(&result, |caps: &Captures| {
        let q = caps.get(2).unwrap().as_str();
        let q_num = if q.chars().all(|c| c.is_ascii_digit()) {
            q.to_string()
        } else {
            crate::lang::vi::misc::roman_to_int(&q.to_uppercase()).to_string()
        };
        format!("quý {} năm {}",
            n2w(&q_num),
            n2w(caps.get(3).unwrap().as_str()))
    }).to_string();

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
                } else if let Some(name) = crate::lang::vi::resources::VI_LETTER_NAMES.get(cl.as_str()) {
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
            // An impossible date (30/2/2024) is read as numbers joined by "trên",
            // which consumes the whole cluster. Leaving parts behind would let
            // the month-year rule recombine them into nonsense.
            format!(" {} trên {} trên {} ", n2w(day), n2w(month), n2w(year))
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

    // Date ranges are handled as one unit so the second half also becomes a
    // date. Treating "đến" as a lead word on its own would be too weak a signal
    // and would start turning fractions into dates.
    result = RE_DATE_RANGE.replace_all(&result, |caps: &Captures| {
        let d1 = caps.get(2).unwrap().as_str();
        let m1 = caps.get(3).unwrap().as_str();
        let d2 = caps.get(5).unwrap().as_str();
        let m2 = caps.get(6).unwrap().as_str();
        if !is_valid_date(d1, m1) || !is_valid_date(d2, m2) {
            return caps.get(0).unwrap().as_str().to_string();
        }
        let mon = |m: &str| {
            let v = m.parse::<i32>().unwrap_or(0);
            if v == 4 { "tư".to_string() } else { n2w(&v.to_string()) }
        };
        let day = |d: &str| n2w(&d.parse::<i32>().unwrap_or(0).to_string());
        format!("{} ngày {} tháng {} {} ngày {} tháng {}",
            caps.get(1).unwrap().as_str(), day(d1), mon(m1),
            caps.get(4).unwrap().as_str(), day(d2), mon(m2))
    }).to_string();

    let current_text = result.clone();
    result = RE_DAY_MONTH.replace_all(&current_text, |caps: &Captures| {
        let full_match = caps.get(0).unwrap();
        let day_str = caps.get(1).unwrap().as_str();
        let month_str = caps.get(3).unwrap().as_str();
        let sep = caps.get(2).unwrap().as_str();
        // Absent date context, a dash between numbers leans towards a range
        // ("5-10" -> "năm đến mười") while a slash leans towards a fraction
        // ("3/5" -> "ba trên năm").
        let joiner = if sep == "-" { "đến" } else { "trên" };
        let a = day_str.parse::<i32>().unwrap_or(0);
        let b = month_str.parse::<i32>().unwrap_or(0);

        let context_words = get_context_words(&current_text, full_match.start(), full_match.end(), 3);
        let math_symbols = ["+", "-", "*", "x", "×", "/", "=", ">", "<", "≥", "≤", "≈", "±"];
        let is_valid = is_valid_date(day_str, month_str);

        let month_has_leading_zero = month_str.starts_with('0') && month_str.len() > 1;
        let day_has_leading_zero = day_str.starts_with('0') && day_str.len() > 1;
        if is_valid && (month_has_leading_zero || day_has_leading_zero) {
            let m_val = if b == 4 { "tư".to_string() } else { n2w(&b.to_string()) };
            return format!("ngày {} tháng {}", n2w(&a.to_string()), m_val);
        }

        // Lead word immediately before ("chiều 17/10", "từ 1/8", "trước 30/4").
        let lead_word = current_text[..full_match.start()]
            .split_whitespace()
            .next_back()
            .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()).to_lowercase())
            .unwrap_or_default();
        if is_valid && DATE_LEAD_WORDS.contains(lead_word.as_str()) {
            let m_val = if b == 4 { "tư".to_string() } else { n2w(&b.to_string()) };
            return format!("ngày {} tháng {}", n2w(&a.to_string()), m_val);
        }

        if is_valid && context_words.iter().any(|w: &String| DATE_KEYWORDS.contains(w.as_str())) {
            let m_val = if b == 4 { "tư".to_string() } else { n2w(&b.to_string()) };
            return format!("ngày {} tháng {}", n2w(&a.to_string()), m_val);
        }

        if context_words.iter().any(|w: &String| MATH_KEYWORDS.contains(w.as_str())) ||
           context_words.iter().any(|w: &String| math_symbols.contains(&w.as_str())) {
            return format!("{} trên {}", n2w(day_str), n2w(month_str));
        }

        if !is_valid {
            if day_str.starts_with('0') || month_str.starts_with('0') {
                return format!("{} {} {}", n2w(day_str), joiner, n2w(month_str));
            }
            return full_match.as_str().to_string();
        }

        format!("{} {} {}", n2w(day_str), joiner, n2w(month_str))
    }).to_string();

    result = dedup_lead(&result, &RE_REDUNDANT_NGAY, "ngày");
    result = dedup_lead(&result, &RE_REDUNDANT_THANG, "tháng");
    result = dedup_lead(&result, &RE_REDUNDANT_NAM, "năm");
    result = dedup_lead(&result, &RE_REDUNDANT_HOM_NGAY, "hôm");
    result = RE_REDUNDANT_MUNG_NGAY.replace_all(&result, |caps: &Captures| {
        let next = caps.get(2).unwrap().as_str();
        if NUM_WORDS.contains(next.to_lowercase().as_str()) {
            format!("{} {}", caps.get(1).unwrap().as_str(), next)
        } else {
            caps.get(0).unwrap().as_str().to_string()
        }
    }).into_owned();

    result
}

pub fn normalize_time(text: &str) -> String {
    let mut result = RE_FULL_TIME.replace_all(text, |caps: &Captures| {
        let h = caps.get(1).unwrap().as_str();
        let sep1 = caps.get(2).unwrap().as_str();
        let m = caps.get(3).unwrap().as_str();
        let sep2 = caps.get(4).unwrap().as_str();
        let s = caps.get(5).unwrap().as_str();
        // An all-colon form is a time only when minutes and seconds are both two
        // digits (01:02:03). Single-digit parts ("1:2:3") mean a ratio, so leave
        // it for the ratio pass.
        if sep1 == ":" && sep2 == ":" && (m.len() != 2 || s.len() != 2) {
            return caps.get(0).unwrap().as_str().to_string();
        }
        format!("{} giờ {} phút {} giây",
            n2w(norm_time_part(h)),
            n2w(norm_time_part(m)),
            n2w(norm_time_part(s))
        )
    }).to_string();

    result = RE_TIME.replace_all(&result, |caps: &Captures| {
        let full_match = caps.get(0).unwrap().as_str();
        let h_str = caps.get(1).unwrap().as_str();
        let sep = caps.get(2).unwrap().as_str();
        let m_str = caps.get(3).unwrap().as_str();
        let suffix = caps.get(4).map(|m| m.as_str().to_lowercase()).unwrap_or_default();

        let h_int = h_str.parse::<i32>().unwrap_or(-1);
        let m_int = m_str.parse::<i32>().unwrap_or(-1);

        // Strictness for ":" (xx:xx): Require exactly 2-digit minutes unless a suffix exists.
        if sep == ":" && m_str.len() != 2 && suffix.is_empty() {
            return full_match.to_string();
        }

        // Chemistry/Measurement hint: Skip alphabetic separators (H/G) if no time suffix and single-digit minute.
        if (sep == "H" || sep == "G" || sep == "g" || sep == "h") && suffix.is_empty() && m_str.len() == 1 {
             return full_match.to_string();
        }

        if m_int >= 0 && m_int < 60 {
            let is_min_sec = sep == ":" && h_int >= 24;
            let h_words = if is_min_sec { n2w(h_str) } else { n2w(norm_time_part(h_str)) };
            let m_words = n2w(norm_time_part(m_str));
            
            let h_unit = if is_min_sec { "phút" } else { "giờ" };
            let m_unit = if is_min_sec { "giây" } else { "phút" };
            
            format!("{} {} {} {}", h_words, h_unit, m_words, m_unit)
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
