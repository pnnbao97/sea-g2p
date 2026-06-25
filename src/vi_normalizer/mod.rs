pub mod num2vi;
pub mod resources;
pub mod numerical;
pub mod datestime;
pub mod units;
pub mod technical;
pub mod misc;

use pyo3::prelude::*;
// fancy_regex only for patterns requiring look-arounds
use fancy_regex::{Regex as FRegex, Captures as FCaps};
// regex crate for simple patterns (Thompson NFA - much faster than fancy_regex backtracker)
use regex::{Regex, Captures};
use once_cell::sync::Lazy;
use unicode_normalization::UnicodeNormalization;
use crate::vi_normalizer::numerical::{normalize_number_vi, RE_MULTIPLY, expand_multiply_number};
use crate::vi_normalizer::datestime::{normalize_date, normalize_time};
use crate::vi_normalizer::units::{expand_units_and_currency, expand_compound_units, expand_scientific_notation, fix_english_style_numbers, expand_power_of_ten, expand_height_weight};
use crate::vi_normalizer::misc::{normalize_others, expand_standalone_letters, RE_ACRONYMS_EXCEPTIONS, RE_ACRONYM};
use crate::vi_normalizer::technical::{normalize_technical, normalize_emails, RE_TECHNICAL, RE_EMAIL};
use crate::vi_normalizer::resources::COMBINED_EXCEPTIONS;

// ── Tier 1: regex crate (Thompson NFA, much faster for simple patterns) ────
static RE_EXTRA_SPACES: Lazy<Regex> = Lazy::new(|| Regex::new(r"[ \t\xA0]+").unwrap());
static RE_EXTRA_COMMAS: Lazy<Regex> = Lazy::new(|| Regex::new(r",\s*,").unwrap());
// Ellipsis một-ký-tự: ․(U+2024) ‥(U+2025) …(U+2026) -> "." để đi chung đường
// với "..." (RE_MULTI_DOT gộp tiếp về một dấu chấm).
static RE_ELLIPSIS: Lazy<Regex> = Lazy::new(|| Regex::new(r"[\u{2024}\u{2025}\u{2026}]").unwrap());
static RE_MULTI_DOT: Lazy<Regex> = Lazy::new(|| Regex::new(r"\.[\s.]*\.").unwrap());
static RE_COMMA_BEFORE_PUNCT: Lazy<Regex> = Lazy::new(|| Regex::new(r",\s*([.!?;])").unwrap());
static RE_SPACE_BEFORE_PUNCT: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s+([,.!?;:])").unwrap());
// Rewritten to avoid lookahead: capture the following char so regex crate can handle it
static RE_MISSING_SPACE_AFTER_PUNCT: Lazy<Regex> = Lazy::new(|| Regex::new(r"([.,!?;:])([^\s\d<])").unwrap());
static RE_INTERNAL_EN_TAG: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)(?s)(__start_en__.*?__end_en__|<en>.*?</en>)").unwrap());
static RE_DOT_BETWEEN_DIGITS: Lazy<Regex> = Lazy::new(|| Regex::new(r"(\d+)\.(\d+)").unwrap());
static RE_ENTOKEN: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)ENTOKEN\d+").unwrap());
static RE_EN_TAG: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?si)<en>.*?</en>").unwrap());
// L\u01b0u \u00fd: c\u00e1c t\u1eeb ng\u1eef c\u1ea3nh ph\u1ea3i l\u00e0 k\u00fd t\u1ef1 Vi\u1ec7t th\u1eadt. KH\u00d4NG d\u00f9ng byte-escape
// `\xHH` trong raw string `r"..."` \u2014 raw string kh\u00f4ng x\u1eed l\u00fd escape n\u00ean engine
// hi\u1ec3u `\xe1` l\u00e0 codepoint U+00E1, bi\u1ebfn "b\u1eb1ng" th\u00e0nh mojibake kh\u00f4ng bao gi\u1edd kh\u1edbp.
static RE_CONTEXT_TRU: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)\b(b\u1eb1ng|t\u00ednh|k\u1ebft qu\u1ea3)\s+(\d+(?:[.,]\d+)?)\s*[-\u2013\u2014]\s*(\d+(?:[.,]\d+)?)\b").unwrap());
static RE_CONTEXT_TRU_POST: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)\b(\d+(?:[.,]\d+)?)\s*[-\u2013\u2014]\s*(\d+(?:[.,]\d+)?)\s+(b\u1eb1ng|t\u00ednh|k\u1ebft qu\u1ea3)\b").unwrap());
static RE_CONTEXT_DEN: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)\b(t\u1eeb|kho\u1ea3ng|trong)\s+(\d+(?:[.,]\d+)?)\s*[-\u2013\u2014]\s*(\d+(?:[.,]\d+)?)\b").unwrap());
static RE_EQ_MINUS: Lazy<Regex> = Lazy::new(|| Regex::new(r"([\d./]+)\s*[-\u2013\u2014]\s*([\d./]+)\s*=").unwrap());
static RE_EQ_NEG: Lazy<Regex> = Lazy::new(|| Regex::new(r"=\s*[-\u2013\u2014](\d+(?:[./]\d+)?)").unwrap());
static RE_PHONE_WITH_DASH: Lazy<Regex> = Lazy::new(|| Regex::new(r"\b(0\d{2,3})[\u2013\-\u2014](\d{3,4})[\u2013\-\u2014](\d{4})\b").unwrap());
static RE_POWER_OF_TEN_IMPLICIT: Lazy<Regex> = Lazy::new(|| Regex::new(r"\b10\^([-+]?\d+)\b").unwrap());
static RE_TO_SANG: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s*(?:->|=>)\s*").unwrap());
static RE_MULTI_COMMA: Lazy<Regex> = Lazy::new(|| Regex::new(r"\b(\d+(?:,\d+){2,})\b").unwrap());
static RE_NUMERIC_DASH_GROUPS: Lazy<Regex> = Lazy::new(|| Regex::new(r"\b\d+(?:[\u2013\-\u2014]\d+){2,}\b").unwrap());
static RE_PHONE_SPACE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\b0\d{2,3}(?:\s\d{3}){2}\b").unwrap());
// Khoảng phần trăm "5-7%" -> "5 đến 7%" (dấu % xác nhận đây là KHOẢNG, không phải
// phân số/ngày-tháng). Chạy trước normalize_date để không bị đọc thành "trên".
static RE_RANGE_PCT: Lazy<Regex> = Lazy::new(|| Regex::new(r"(\d+(?:[.,]\d+)?)\s*[-–—]\s*(\d+(?:[.,]\d+)?)\s*%").unwrap());
// Tổng đài 1800/1900: đọc rời từng chữ số thay vì gộp thành số lớn.
static RE_HOTLINE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\b(1800|1900)[\s.]?(\d{3,6})\b").unwrap());

// ── Tier 2: fancy_regex (REQUIRED for look-around assertions) ────────────────
// RE_COMBINED_TECH_EMAIL removed — two separate passes are faster (mirrors Python)
static RE_RANGE: Lazy<FRegex> = Lazy::new(|| FRegex::new(r"(?<!\d)(?<!\d[,.])(?<![a-zA-Z])(\d{1,15}(?:[,.]\d{1,15})?)(\s*)[\u2013\-\u2014](\s*)(\d{1,15}(?:[,.]\d{1,15})?)(?!\d)(?![.,]\d)").unwrap());
static RE_DASH_TO_COMMA: Lazy<FRegex> = Lazy::new(|| FRegex::new(r"(?<=\s)[\u2013\-\u2014](?=\s)").unwrap());
static RE_FLOAT_WITH_COMMA: Lazy<FRegex> = Lazy::new(|| FRegex::new(r"(?<![\d.])(\d+(?:\.\d{3})*),(\d+)(%)?").unwrap());
static RE_STRIP_DOT_SEP: Lazy<FRegex> = Lazy::new(|| FRegex::new(r"(?<![\d.])\d+(?:\.\d{3})+(?![\d.])").unwrap());
static RE_LONG_NUM: Lazy<FRegex> = Lazy::new(|| FRegex::new(r"(?<!\d)(?<!\d[,.])([-–—]?)(\d{7,})(?!\d)(?![.,]\d)").unwrap());
static RE_CAMEL_CASE: Lazy<FRegex> = Lazy::new(|| FRegex::new(r"(?<=[a-z])(?=[A-Z])|(?<=[A-Z])(?=[A-Z][a-z])").unwrap());
static RE_POTENTIAL_CONCAT: Lazy<Regex> = Lazy::new(|| Regex::new(r"\b[a-zA-Z]{3,}\b").unwrap());

fn cleanup_whitespace(text: &str) -> String {
    let mut res = RE_MULTI_DOT.replace_all(text, ".").into_owned();
    res = RE_EXTRA_SPACES.replace_all(&res, " ").into_owned();
    res = RE_EXTRA_COMMAS.replace_all(&res, ",").into_owned();
    res = RE_COMMA_BEFORE_PUNCT.replace_all(&res, "$1").into_owned();
    res = RE_SPACE_BEFORE_PUNCT.replace_all(&res, "$1").into_owned();
    // Pattern now captures the char after punct; replace with "$1 $2" (insert space)
    res = RE_MISSING_SPACE_AFTER_PUNCT.replace_all(&res, "$1 $2").into_owned();
    res.trim().trim_matches(',').to_string()
}

fn split_concatenated_terms(text: &str) -> String {
    let re_potential = &*RE_POTENTIAL_CONCAT;
    let re_camel = &*RE_CAMEL_CASE;
    let re_acronym = &*RE_ACRONYM;

    re_potential.replace_all(text, |caps: &Captures| {
        let word = caps.get(0).unwrap().as_str();
        if re_acronym.is_match(word).unwrap_or(false) {
            word.to_string()
        } else {
            re_camel.replace_all(word, " ").into_owned()
        }
    }).into_owned()
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
    current_text = RE_ENTOKEN.replace_all(&current_text, |caps: &Captures| {
        let orig = caps.get(0).unwrap().as_str();
        protect(orig.to_lowercase(), &mut mask_map)
    }).into_owned();

    // Protect emails first (simple pattern, matches @)
    let temp_email = current_text.clone();
    current_text = RE_EMAIL.replace_all(&temp_email, |caps: &FCaps| {
        let orig = caps.get(0).unwrap().as_str();
        let val = normalize_emails(orig);
        protect(val, &mut mask_map)
    }).to_string();

    // Protect technical strings (URLs, paths, etc.) separately
    let temp_tech = current_text.clone();
    current_text = RE_TECHNICAL.replace_all(&temp_tech, |caps: &FCaps| {
        let orig = caps.get(0).unwrap().as_str();
        // Cụm từ tiếng Anh nối bằng gạch ngang (text-to-speech, state-of-the-art,
        // plug-and-play): chỉ gồm chữ cái ASCII + '-' và có ít nhất một chữ thường
        // -> KHÔNG phải định danh kỹ thuật. Tách thành các từ (gạch -> space) để
        // G2P đọc như tiếng Anh, không đọc "gạch ngang" hay spell "to" -> "t o".
        if orig.contains('-')
            && orig.chars().all(|c: char| c.is_ascii_alphabetic() || c == '-')
            && orig.chars().any(|c: char| c.is_ascii_lowercase())
        {
            return orig.replace('-', " ");
        }
        let val = if RE_ACRONYMS_EXCEPTIONS.is_match(orig) {
            COMBINED_EXCEPTIONS.get(orig).cloned().unwrap_or(orig.to_string())
        } else {
            normalize_technical(orig)
        };
        protect(val, &mut mask_map)
    }).to_string();

    current_text = split_concatenated_terms(&current_text);

    // Core normalization passes
    current_text = expand_power_of_ten(&current_text);
    current_text = RE_MULTIPLY.replace_all(&current_text, |caps: &FCaps| {
        expand_multiply_number(caps.get(0).unwrap().as_str())
    }).to_string();

    current_text = RE_CONTEXT_TRU.replace_all(&current_text, " $1 $2 trừ $3 ").into_owned();
    current_text = RE_CONTEXT_TRU_POST.replace_all(&current_text, " $1 trừ $2 $3 ").into_owned();
    current_text = RE_CONTEXT_DEN.replace_all(&current_text, " $1 $2 đến $3 ").into_owned();

    current_text = RE_EQ_MINUS.replace_all(&current_text, |caps: &Captures| {
        format!("{} trừ {} =", caps.get(1).unwrap().as_str(), caps.get(2).unwrap().as_str())
    }).into_owned();

    current_text = RE_EQ_NEG.replace_all(&current_text, |caps: &Captures| {
        format!("= âm {}", caps.get(1).unwrap().as_str())
    }).into_owned();

    current_text = crate::vi_normalizer::misc::expand_abbreviations(&current_text);
    current_text = expand_scientific_notation(&current_text);

    current_text = RE_RANGE_PCT.replace_all(&current_text, "$1 đến $2%").into_owned();

    current_text = normalize_date(&current_text);
    current_text = normalize_time(&current_text);

    current_text = RE_NUMERIC_DASH_GROUPS.replace_all(&current_text, |caps: &Captures| {
        let matched = caps.get(0).unwrap().as_str();
        let parts: Vec<&str> = matched.split(&['-', '\u{2013}', '\u{2014}'][..]).collect();
        parts.iter()
            .map(|&p| crate::vi_normalizer::num2vi::n2w_single(p))
            .collect::<Vec<String>>()
            .join(", ")
    }).into_owned();

    current_text = RE_PHONE_SPACE.replace_all(&current_text, |caps: &Captures| {
        let matched = caps.get(0).unwrap().as_str();
        let parts: Vec<&str> = matched.split_whitespace().collect();
        parts.iter()
            .map(|&p| crate::vi_normalizer::num2vi::n2w_single(p))
            .collect::<Vec<String>>()
            .join(", ")
    }).into_owned();

    current_text = RE_PHONE_WITH_DASH.replace_all(&current_text, |caps: &Captures| {
        let p1 = caps.get(1).unwrap().as_str();
        let p2 = caps.get(2).unwrap().as_str();
        let p3 = caps.get(3).unwrap().as_str();
        format!(" {}, {}, {} ",
            crate::vi_normalizer::num2vi::n2w_single(p1),
            crate::vi_normalizer::num2vi::n2w_single(p2),
            crate::vi_normalizer::num2vi::n2w_single(p3)
        )
    }).into_owned();

    current_text = RE_HOTLINE.replace_all(&current_text, |caps: &Captures| {
        format!(" {} {} ",
            crate::vi_normalizer::num2vi::n2w_single(caps.get(1).unwrap().as_str()),
            crate::vi_normalizer::num2vi::n2w_single(caps.get(2).unwrap().as_str())
        )
    }).into_owned();

    current_text = RE_POWER_OF_TEN_IMPLICIT.replace_all(&current_text, |caps: &Captures| {
        let exp = caps.get(1).unwrap().as_str();
        if exp.starts_with('-') {
            format!("mười mũ trừ {}", crate::vi_normalizer::num2vi::n2w(&exp[1..]))
        } else {
            format!("mười mũ {}", crate::vi_normalizer::num2vi::n2w(&exp.replace('+', "")))
        }
    }).into_owned();

    current_text = RE_RANGE.replace_all(&current_text, |caps: &FCaps| {
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

    current_text = RE_DASH_TO_COMMA.replace_all(&current_text, ",").into_owned();
    current_text = RE_TO_SANG.replace_all(&current_text, " sang ").into_owned();

    current_text = expand_scientific_notation(&current_text);
    current_text = expand_height_weight(&current_text);
    current_text = crate::vi_normalizer::misc::expand_size_labels(&current_text);
    current_text = expand_compound_units(&current_text);
    current_text = expand_units_and_currency(&current_text);
    current_text = RE_LONG_NUM.replace_all(&current_text, |caps: &FCaps| {
        let neg = caps.get(1).unwrap().as_str();
        let num_str = caps.get(2).unwrap().as_str();
        let neg_prefix = if !neg.is_empty() { "âm " } else { "" };
        format!(" {}{} ", neg_prefix, crate::vi_normalizer::num2vi::n2w_single(num_str))
    }).to_string();

    current_text = fix_english_style_numbers(&current_text);

    current_text = RE_MULTI_COMMA.replace_all(&current_text, |caps: &Captures| {
        caps.get(1).unwrap().as_str().split(',').map(|s: &str| crate::vi_normalizer::num2vi::n2w_decimal(s)).collect::<Vec<String>>().join(" phẩy ")
    }).into_owned();

    current_text = RE_FLOAT_WITH_COMMA.replace_all(&current_text, |caps: &FCaps| {
        let int_part = crate::vi_normalizer::num2vi::n2w(&caps.get(1).unwrap().as_str().replace('.', ""));
        let dec_part = caps.get(2).unwrap().as_str().trim_end_matches('0');
        let mut res = if dec_part.is_empty() { int_part } else { format!("{} phẩy {}", int_part, crate::vi_normalizer::num2vi::n2w_decimal(dec_part)) };
        if caps.get(3).is_some() { res.push_str(" phần trăm"); }
        format!(" {} ", res)
    }).to_string();

    current_text = RE_STRIP_DOT_SEP.replace_all(&current_text, |caps: &FCaps| {
        caps.get(0).unwrap().as_str().replace('.', "")
    }).to_string();

    current_text = normalize_others(&current_text);
    current_text = normalize_number_vi(&current_text);

    let temp_text3 = current_text.clone();
    current_text = RE_INTERNAL_EN_TAG.replace_all(&temp_text3, |caps: &Captures| {
        protect(caps.get(0).unwrap().as_str().to_string(), &mut mask_map)
    }).into_owned();

    current_text = expand_standalone_letters(&current_text);

    if current_text.contains('.') {
        current_text = RE_DOT_BETWEEN_DIGITS.replace_all(&current_text, |caps: &Captures| {
            format!("{} chấm {}", caps.get(1).unwrap().as_str(), caps.get(2).unwrap().as_str())
        }).into_owned();
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

    #[pyo3(signature = (text, punc_norm=false))]
    pub fn normalize(&self, text: &str, punc_norm: bool) -> String {
        if text.is_empty() { return String::new(); }

        let nfc_text: String = text.nfc().collect();
        // Quy ellipsis (… ‥ ․) về "." NGAY ĐẦU để nó theo cùng đường xử lý với
        // "...". Nếu để muộn, "…" bị các pass trước đó nuốt thành dấu phân tách.
        let mut current_text = RE_ELLIPSIS.replace_all(&nfc_text, ".").into_owned();

        let mut en_contents = Vec::new();
        let placeholder_pattern = "ENTOKEN{}";

        let temp_text = current_text.clone();
        current_text = RE_EN_TAG.replace_all(&temp_text, |caps: &Captures| {
            en_contents.push(caps.get(0).unwrap().as_str().to_string());
            placeholder_pattern.replace("{}", &en_contents.len().saturating_sub(1).to_string())
        }).into_owned();

        current_text = clean_vietnamese_text(&current_text);

        current_text = RE_EXTRA_SPACES.replace_all(&current_text, " ").trim().to_string();

        if !en_contents.is_empty() {
            for (idx, content) in en_contents.iter().enumerate() {
                let placeholder = placeholder_pattern.replace("{}", &idx.to_string()).to_lowercase();
                current_text = current_text.replace(&placeholder, content);
            }
        }

        let result = RE_EXTRA_SPACES.replace_all(&current_text, " ").trim().to_string();

        if punc_norm {
            crate::punc::apply_punc_norm(&result)
        } else {
            result
        }
    }

    #[pyo3(signature = (texts, punc_norm=false))]
    pub fn normalize_batch(&self, py: Python<'_>, texts: Vec<String>, punc_norm: bool) -> PyResult<Vec<String>> {
        py.allow_threads(|| {
            use rayon::prelude::*;
            Ok(texts.into_par_iter().map(|t| self.normalize(&t, punc_norm)).collect())
        })
    }
}
