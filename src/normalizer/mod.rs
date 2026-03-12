pub mod vi_resources;
pub mod num2vi;
pub mod cleaner;

use fancy_regex::Regex;
use unicode_normalization::UnicodeNormalization;
use crate::normalizer::cleaner::*;

pub struct Normalizer {
    pub lang: String,
}

impl Normalizer {
    pub fn new(lang: &str) -> Self {
        if lang != "vi" {
            // In a real library we might use log crate here
            eprintln!("Language '{}' is not fully supported for normalization yet. Falling back to 'vi'.", lang);
        }
        Normalizer {
            lang: lang.to_string(),
        }
    }

    pub fn normalize(&self, text: &str) -> String {
        if text.is_empty() {
            return "".to_string();
        }

        // Pre-normalization: Ensure NFC format
        let text_nfc: String = text.nfc().collect();

        // Step 1: Detect and protect EN tags
        let mut en_contents = Vec::new();
        let placeholder_prefix = "ENTOKEN";

        let re_en = Regex::new(r"(?i)<en>.*?</en>").unwrap();

        let mut last_end = 0;
        let mut new_text = String::new();
        for mat in re_en.find_iter(&text_nfc) {
            let m = mat.unwrap();
            new_text.push_str(&text_nfc[last_end..m.start()]);
            en_contents.push(m.as_str().to_string());
            new_text.push_str(&format!("{}{}", placeholder_prefix, en_contents.len().saturating_sub(1)));
            last_end = m.end();
        }
        new_text.push_str(&text_nfc[last_end..]);
        let text_protected = new_text;

        // Step 2: Core Normalization
        let mut text_norm = clean_vietnamese_text(&text_protected);

        // Final cleanup
        text_norm = text_norm.to_lowercase();
        let re_spaces = Regex::new(r"[ \t\xA0]+").unwrap();
        text_norm = re_spaces.replace_all(&text_norm, " ").trim().to_string();

        // Step 3: Restore EN tags
        for (idx, en_content) in en_contents.iter().enumerate() {
            let placeholder = format!("{}{}", placeholder_prefix, idx).to_lowercase();
            text_norm = text_norm.replace(&placeholder, en_content);
        }

        // Final whitespace cleanup
        text_norm = re_spaces.replace_all(&text_norm, " ").trim().to_string();

        text_norm
    }
}

pub fn clean_vietnamese_text(text: &str) -> String {
    let mut mask_map = std::collections::HashMap::new();
    let mut current_text = text.to_string();

    let re_entoken = Regex::new(r"(?i)ENTOKEN\d+").unwrap();

    // Manual protection for placeholder
    {
        let mut last_end = 0;
        let mut new_text = String::new();
        let original_text = current_text.clone();
        for mat in re_entoken.find_iter(&original_text) {
            let m = mat.unwrap();
            new_text.push_str(&original_text[last_end..m.start()]);

            let matched_text = m.as_str();

            let idx = mask_map.len();
            let mask = format!("mask{:0>4}mask", idx)
                .chars()
                .map(|c| if c.is_ascii_digit() { (c as u8 - b'0' + b'a') as char } else { c })
                .collect::<String>();

            mask_map.insert(mask.clone(), matched_text.to_string());
            new_text.push_str(&mask);
            last_end = m.end();
        }
        new_text.push_str(&original_text[last_end..]);
        current_text = new_text;
    }

    let re_email = Regex::new(r"(?i)\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b").unwrap();
    let re_technical = crate::normalizer::cleaner::RE_TECHNICAL.clone();

    // Email first
    {
        let mut last_end = 0;
        let mut new_text = String::new();
        let text_for_email = current_text.clone();
        for mat in re_email.find_iter(&text_for_email) {
            let m = mat.unwrap();
            new_text.push_str(&text_for_email[last_end..m.start()]);
            let processed = normalize_emails(m.as_str());
            let mask = format!("mask{:0>4}mask", mask_map.len()).chars().map(|c| if c.is_ascii_digit() { (c as u8 - b'0' + b'a') as char } else { c }).collect::<String>();
            mask_map.insert(mask.clone(), processed);
            new_text.push_str(&mask);
            last_end = m.end();
        }
        new_text.push_str(&text_for_email[last_end..]);
        current_text = new_text;
    }

    // Technical
    {
        let mut last_end = 0;
        let mut new_text = String::new();
        let text_for_tech = current_text.clone();
        for mat in re_technical.find_iter(&text_for_tech) {
            let m = mat.unwrap();
            new_text.push_str(&text_for_tech[last_end..m.start()]);

            let matched = m.as_str();
            let processed = if let Some(normed) = crate::normalizer::vi_resources::COMBINED_EXCEPTIONS.get(matched) {
                normed.to_string()
            } else {
                normalize_technical(matched)
            };

            let mask = format!("mask{:0>4}mask", mask_map.len()).chars().map(|c| if c.is_ascii_digit() { (c as u8 - b'0' + b'a') as char } else { c }).collect::<String>();
            mask_map.insert(mask.clone(), processed);
            new_text.push_str(&mask);
            last_end = m.end();
        }
        new_text.push_str(&text_for_tech[last_end..]);
        current_text = new_text;
    }

    current_text = normalize_pre_number(&current_text);
    current_text = normalize_units_currency(&current_text);
    current_text = normalize_post_number(&current_text);

    let re_internal_en = Regex::new(r"(?i)(__start_en__.*?__end_en__|<en>.*?</en>)").unwrap();

    // Protect internal EN
    {
        let mut last_end = 0;
        let mut new_text = String::new();
        let original_text = current_text.clone();
        for mat in re_internal_en.find_iter(&original_text) {
            let m = mat.unwrap();
            new_text.push_str(&original_text[last_end..m.start()]);

            let idx = mask_map.len();
            let mask = format!("mask{:0>4}mask", idx)
                .chars()
                .map(|c| if c.is_ascii_digit() { (c as u8 - b'0' + b'a') as char } else { c })
                .collect::<String>();

            mask_map.insert(mask.clone(), m.as_str().to_string());
            new_text.push_str(&mask);
            last_end = m.end();
        }
        new_text.push_str(&original_text[last_end..]);
        current_text = new_text;
    }

    current_text = expand_standalone_letters(&current_text);

    let re_dot_digits = Regex::new(r"(\d+)\.(\d+)").unwrap();
    while re_dot_digits.is_match(&current_text).unwrap_or(false) {
        current_text = re_dot_digits.replace_all(&current_text, "$1 chấm $2").into_owned();
    }

    // Sort masks by length descending to avoid partial replacement
    let mut masks: Vec<String> = mask_map.keys().cloned().collect();
    masks.sort_by_key(|m: &String| std::cmp::Reverse(m.len()));

    for mask in masks {
        let original = mask_map.get(&mask).unwrap();
        current_text = current_text.replace(&mask, original);
        current_text = current_text.replace(&mask.to_lowercase(), original);
    }

    current_text = current_text.replace("__start_en__", "<en>").replace("__end_en__", "</en>");
    current_text = current_text.replace("_", " ");

    cleanup_whitespace(&current_text)
}

fn normalize_pre_number(text: &str) -> String {
    let mut t = expand_power_of_ten(text);
    t = expand_abbreviations(&t);
    t = normalize_date(&t);
    t = normalize_time(&t);

    let re_range = crate::normalizer::cleaner::RE_RANGE.clone();
    t = re_range.replace_all(&t, |caps: &fancy_regex::Captures| {
        let n1_raw = caps.get(1).unwrap().as_str();
        let n2_raw = caps.get(2).unwrap().as_str();
        let n1 = n1_raw.replace(',', "").replace('.', "");
        let n2 = n2_raw.replace(',', "").replace('.', "");
        if (n1.len() as i32 - n2.len() as i32).abs() <= 1 {
            format!("{} đến {}", n1_raw, n2_raw)
        } else {
            format!("{} {}", n1_raw, n2_raw)
        }
    }).into_owned();

    t = crate::normalizer::cleaner::RE_DASH_TO_COMMA.replace_all(&t, ",").into_owned();
    t = crate::normalizer::cleaner::RE_TO_SANG.replace_all(&t, " sang ").into_owned();
    t
}

fn normalize_units_currency(text: &str) -> String {
    let mut t = expand_scientific_notation(text);
    t = expand_compound_units(&t);
    t = expand_measurement_currency(&t);
    t = expand_currency_symbols(&t);
    t = fix_english_style_numbers(&t);

    let re_multi_comma = crate::normalizer::cleaner::RE_MULTI_COMMA.clone();
    t = re_multi_comma.replace_all(&t, |caps: &fancy_regex::Captures| {
        let val = caps.get(1).unwrap().as_str();
        let parts: Vec<String> = val.split(',')
            .map(|s: &str| s.chars().map(|c: char| crate::normalizer::num2vi::n2w_single(&c.to_string())).collect::<Vec<String>>().join(" "))
            .collect();
        parts.join(" phẩy ")
    }).into_owned();

    let re_float_with_comma = crate::normalizer::cleaner::RE_FLOAT_WITH_COMMA.clone();
    t = re_float_with_comma.replace_all(&t, |caps: &fancy_regex::Captures| {
        let int_part = crate::normalizer::num2vi::n2w(&caps.get(1).unwrap().as_str().replace('.', ""));
        let dec_part = caps.get(2).unwrap().as_str().trim_end_matches('0');
        let mut res = if dec_part.is_empty() {
            int_part
        } else {
            format!("{} phẩy {}", int_part, crate::normalizer::num2vi::n2w_single(dec_part))
        };
        if caps.get(3).is_some() {
            res.push_str(" phần trăm");
        }
        format!(" {} ", res)
    }).into_owned();

    let re_strip_dot_sep = crate::normalizer::cleaner::RE_STRIP_DOT_SEP.clone();
    t = re_strip_dot_sep.replace_all(&t, |caps: &fancy_regex::Captures| {
        caps.get(0).unwrap().as_str().replace('.', "")
    }).into_owned();

    t
}

fn normalize_post_number(text: &str) -> String {
    let mut t = normalize_others(text);
    t = normalize_number_vi(&t);
    t
}

fn cleanup_whitespace(text: &str) -> String {
    let mut t = Regex::new(r"[ \t\xA0]+").unwrap().replace_all(text, " ").into_owned();
    t = Regex::new(r",\s*,").unwrap().replace_all(&t, ",").into_owned();
    t = Regex::new(r",\s*([.!?;])").unwrap().replace_all(&t, "$1").into_owned();
    t = Regex::new(r"\s+([,.!?;:])").unwrap().replace_all(&t, "$1").into_owned();
    t = Regex::new(r"([.,!?;:])(?=[^\s\d<])").unwrap().replace_all(&t, "$1 ").into_owned();
    t.trim().trim_matches(',').to_string()
}
