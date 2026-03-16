pub mod num2vi;
pub mod resources;
pub mod numerical;
pub mod datestime;
pub mod units;
pub mod technical;
pub mod misc;

use pyo3::prelude::*;
use fancy_regex::{Regex, Captures};
use once_cell::sync::Lazy;
use unicode_normalization::UnicodeNormalization;
use crate::normalizer::numerical::{normalize_number_vi, RE_MULTIPLY, expand_multiply_number};
use crate::normalizer::datestime::{normalize_date, normalize_time};
use crate::normalizer::units::{expand_units_and_currency, expand_compound_units, expand_scientific_notation, fix_english_style_numbers, expand_power_of_ten};
use crate::normalizer::misc::{normalize_others, expand_standalone_letters, RE_ACRONYMS_EXCEPTIONS};
use crate::normalizer::technical::{normalize_technical, normalize_emails, RE_TECHNICAL, RE_EMAIL};
use crate::normalizer::resources::COMBINED_EXCEPTIONS;

static RE_EXTRA_SPACES: Lazy<Regex> = Lazy::new(|| Regex::new(r"[ \t\xA0]+").unwrap());
static RE_EXTRA_COMMAS: Lazy<Regex> = Lazy::new(|| Regex::new(r",\s*,").unwrap());
static RE_COMMA_BEFORE_PUNCT: Lazy<Regex> = Lazy::new(|| Regex::new(r",\s*([.!?;])").unwrap());
static RE_SPACE_BEFORE_PUNCT: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s+([,.!?;:])").unwrap());
static RE_MISSING_SPACE_AFTER_PUNCT: Lazy<Regex> = Lazy::new(|| Regex::new(r"([.,!?;:])(?=[^\s\d<])").unwrap());
static RE_INTERNAL_EN_TAG: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)(__start_en__.*?__end_en__|<en>.*?</en>)").unwrap());
static RE_DOT_BETWEEN_DIGITS: Lazy<Regex> = Lazy::new(|| Regex::new(r"(\d+)\.(\d+)").unwrap());

fn cleanup_whitespace(text: &str) -> String {
    let mut res = RE_EXTRA_SPACES.replace_all(text, " ").to_string();
    res = RE_EXTRA_COMMAS.replace_all(&res, ",").to_string();
    res = RE_COMMA_BEFORE_PUNCT.replace_all(&res, "$1").to_string();
    res = RE_SPACE_BEFORE_PUNCT.replace_all(&res, "$1").to_string();
    res = RE_MISSING_SPACE_AFTER_PUNCT.replace_all(&res, "$1 ").to_string();
    res.trim().trim_matches(',').to_string()
}

pub fn clean_vietnamese_text(text: &str) -> String {
    let mut mask_map: Vec<(String, String)> = Vec::new();
    let mut current_text = text.to_string();

    let protect = |original: String, map: &mut Vec<(String, String)>| -> String {
        let idx = map.len();
        let mask = format!("mask{:0>4}mask", idx).chars().map(|c: char| {
            if c.is_ascii_digit() {
                ((c as u8 - b'0') + b'a') as char
            } else {
                c
            }
        }).collect::<String>();
        map.push((mask.clone(), original));
        mask
    };

    // Protect ENTOKEN placeholders (if any)
    let re_entoken = Regex::new(r"(?i)ENTOKEN\d+").unwrap();
    let temp_text = current_text.clone();
    current_text = re_entoken.replace_all(&temp_text, |caps: &Captures| {
        let orig = caps.get(0).unwrap().as_str();
        protect(orig.to_lowercase(), &mut mask_map)
    }).to_string();

    // Protect URLs, emails, and technical strings
    let combined_tech_email = Regex::new(&format!(r"{}|{}", RE_EMAIL.as_str(), RE_TECHNICAL.as_str())).unwrap();
    let temp_text2 = current_text.clone();
    current_text = combined_tech_email.replace_all(&temp_text2, |caps: &Captures| {
        let orig = caps.get(0).unwrap().as_str();
        let val = if RE_EMAIL.is_match(orig).unwrap_or(false) {
            normalize_emails(orig)
        } else if RE_ACRONYMS_EXCEPTIONS.is_match(orig).unwrap_or(false) {
             COMBINED_EXCEPTIONS.get(orig).cloned().unwrap_or(orig.to_string())
        } else {
             normalize_technical(orig)
        };
        protect(val, &mut mask_map)
    }).to_string();

    // Core normalization passes
    current_text = expand_power_of_ten(&current_text);
    current_text = RE_MULTIPLY.replace_all(&current_text, |caps: &Captures| {
        expand_multiply_number(caps.get(0).unwrap().as_str())
    }).to_string();

    let re_context_tru = Regex::new(r"(?i)\b(bằng|tính|kết quả)\s+(\d+(?:[.,]\d+)?)\s*[-–—]\s*(\d+(?:[.,]\d+)?)\b").unwrap();
    current_text = re_context_tru.replace_all(&current_text, " $1 $2 trừ $3 ").to_string();
    let re_context_tru_post = Regex::new(r"(?i)\b(\d+(?:[.,]\d+)?)\s*[-–—]\s*(\d+(?:[.,]\d+)?)\s+(bằng|tính|kết quả)\b").unwrap();
    current_text = re_context_tru_post.replace_all(&current_text, " $1 trừ $2 $3 ").to_string();
    let re_context_den = Regex::new(r"(?i)\b(từ|khoảng|trong)\s+(\d+(?:[.,]\d+)?)\s*[-–—]\s*(\d+(?:[.,]\d+)?)\b").unwrap();
    current_text = re_context_den.replace_all(&current_text, " $1 $2 đến $3 ").to_string();

    let re_phone_with_dash = Regex::new(r"\b(0\d{2,3})[–\-—](\d{3,4})[–\-—](\d{4})\b").unwrap();
    current_text = re_phone_with_dash.replace_all(&current_text, |caps: &Captures| {
        format!(" {} ", crate::normalizer::num2vi::n2w_single(&format!("{}{}{}", caps.get(1).unwrap().as_str(), caps.get(2).unwrap().as_str(), caps.get(3).unwrap().as_str())))
    }).to_string();

    let re_power_of_ten_implicit = Regex::new(r"\b10\^([-+]?\d+)\b").unwrap();
    current_text = re_power_of_ten_implicit.replace_all(&current_text, |caps: &Captures| {
        let exp = caps.get(1).unwrap().as_str();
        if exp.starts_with('-') {
            format!("mười mũ trừ {}", crate::normalizer::num2vi::n2w(&exp[1..]))
        } else {
            format!("mười mũ {}", crate::normalizer::num2vi::n2w(&exp.replace('+', "")))
        }
    }).to_string();

    current_text = crate::normalizer::misc::expand_abbreviations(&current_text);
    current_text = normalize_date(&current_text);
    current_text = normalize_time(&current_text);

    let re_range = Regex::new(r"(?<![\d.,])(\d{1,15}(?:[,.]\d{1,15})?)(\s*)[–\-—](\s*)(\d{1,15}(?:[,.]\d{1,15})?)(?![\d.,])").unwrap();
    current_text = re_range.replace_all(&current_text, |caps: &Captures| {
        let n1_raw = caps.get(1).unwrap().as_str();
        let s1 = caps.get(2).unwrap().as_str();
        let s2 = caps.get(3).unwrap().as_str();
        let n2_raw = caps.get(4).unwrap().as_str();
        if !s1.is_empty() && s2.is_empty() {
            return caps.get(0).unwrap().as_str().to_string();
        }
        let n1 = n1_raw.replace(',', "").replace('.', "");
        let n2 = n2_raw.replace(',', "").replace('.', "");
        if (n1.len() as i32 - n2.len() as i32).abs() <= 1 {
            format!(" {} đến {} ", n1_raw, n2_raw)
        } else {
            format!(" {} {} ", n1_raw, n2_raw)
        }
    }).to_string();

    let re_dash_to_comma = Regex::new(r"(?<=\s)[–\-—](?=\s)").unwrap();
    current_text = re_dash_to_comma.replace_all(&current_text, ",").to_string();
    let re_to_sang = Regex::new(r"\s*(?:->|=>)\s*").unwrap();
    current_text = re_to_sang.replace_all(&current_text, " sang ").to_string();

    current_text = expand_scientific_notation(&current_text);
    current_text = expand_compound_units(&current_text);
    current_text = expand_units_and_currency(&current_text);
    current_text = fix_english_style_numbers(&current_text);

    let re_multi_comma = Regex::new(r"\b(\d+(?:,\d+){2,})\b").unwrap();
    current_text = re_multi_comma.replace_all(&current_text, |caps: &Captures| {
        caps.get(1).unwrap().as_str().split(',').map(|s: &str| crate::normalizer::num2vi::n2w_decimal(s)).collect::<Vec<String>>().join(" phẩy ")
    }).to_string();

    let re_float_with_comma = Regex::new(r"(?<![\d.])(\d+(?:\.\d{3})*),(\d+)(%)?").unwrap();
    current_text = re_float_with_comma.replace_all(&current_text, |caps: &Captures| {
        let int_part = crate::normalizer::num2vi::n2w(&caps.get(1).unwrap().as_str().replace('.', ""));
        let dec_part = caps.get(2).unwrap().as_str().trim_end_matches('0');
        let mut res = if dec_part.is_empty() { int_part } else { format!("{} phẩy {}", int_part, crate::normalizer::num2vi::n2w_decimal(dec_part)) };
        if caps.get(3).is_some() { res.push_str(" phần trăm"); }
        format!(" {} ", res)
    }).to_string();

    let re_strip_dot_sep = Regex::new(r"(?<![\d.])\d+(?:\.\d{3})+(?![\d.])").unwrap();
    current_text = re_strip_dot_sep.replace_all(&current_text, |caps: &Captures| {
        caps.get(0).unwrap().as_str().replace('.', "")
    }).to_string();

    current_text = normalize_others(&current_text);
    current_text = normalize_number_vi(&current_text);

    let temp_text3 = current_text.clone();
    current_text = RE_INTERNAL_EN_TAG.replace_all(&temp_text3, |caps: &Captures| {
        protect(caps.get(0).unwrap().as_str().to_string(), &mut mask_map)
    }).to_string();

    current_text = expand_standalone_letters(&current_text);

    if current_text.contains('.') {
        while let Ok(Some(m)) = RE_DOT_BETWEEN_DIGITS.find(&current_text) {
             let caps = RE_DOT_BETWEEN_DIGITS.captures(&current_text).unwrap().unwrap();
             let start = m.start();
             let end = m.end();
             let replacement = format!("{} chấm {}", caps.get(1).unwrap().as_str(), caps.get(2).unwrap().as_str());
             current_text.replace_range(start..end, &replacement);
        }
    }

    for (mask, original) in mask_map {
        current_text = current_text.replace(&mask, &original);
        current_text = current_text.replace(&mask.to_lowercase(), &original);
    }

    current_text = current_text.replace("__start_en__", "<en>").replace("__end_en__", "</en>");
    current_text = current_text.replace('_', " ").replace('-', " ");
    current_text = cleanup_whitespace(&current_text);
    current_text.to_lowercase()
}

#[pyclass]
pub struct Normalizer {
    #[pyo3(get)]
    pub lang: String,
}

#[pymethods]
impl Normalizer {
    #[new]
    #[pyo3(signature = (lang="vi"))]
    pub fn new(lang: &str) -> Self {
        Normalizer { lang: lang.to_string() }
    }

    pub fn normalize(&self, text: &str) -> String {
        if text.is_empty() { return String::new(); }

        let nfc_text: String = text.nfc().collect();
        let mut current_text = nfc_text;

        let mut en_contents = Vec::new();
        let placeholder_pattern = "ENTOKEN{}";

        let re_en = Regex::new(r"(?i)<en>.*?</en>").unwrap();
        let temp_text = current_text.clone();
        current_text = re_en.replace_all(&temp_text, |caps: &Captures| {
            en_contents.push(caps.get(0).unwrap().as_str().to_string());
            placeholder_pattern.replace("{}", &en_contents.len().saturating_sub(1).to_string())
        }).to_string();

        current_text = clean_vietnamese_text(&current_text);

        current_text = RE_EXTRA_SPACES.replace_all(&current_text, " ").trim().to_string();

        if !en_contents.is_empty() {
            for (idx, content) in en_contents.iter().enumerate() {
                let placeholder = placeholder_pattern.replace("{}", &idx.to_string()).to_lowercase();
                current_text = current_text.replace(&placeholder, content);
            }
        }

        RE_EXTRA_SPACES.replace_all(&current_text, " ").trim().to_string()
    }
}
