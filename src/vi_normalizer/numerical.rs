use fancy_regex::{Regex as FRegex, Captures as FCaps};
use regex::{Regex, Captures};
use once_cell::sync::Lazy;
use crate::vi_normalizer::num2vi::{n2w, n2w_single, n2w_decimal};

static RE_NUMBER: Lazy<FRegex> = Lazy::new(|| {
    FRegex::new(r"(?<!\d)(?P<neg>[-–—])?(\d+(?:,\d+|(?:\.\d{3})+(?!\d)|\.\d+|(?:\s\d{3})+(?!\d))?)(?!\d)").unwrap()
});

pub static RE_MULTIPLY: Lazy<FRegex> = Lazy::new(|| {
    // Greedier version to catch multi-factor chains like 10x20x30.
    // Matches a number (with optional unit) followed by one or more (x + number + unit) sequences.
    FRegex::new(r"(?i)\d+(?:\s*[a-zA-Zμµ²³°]+\d*)?(?:\s*[x×]\s*\d+(?:\s*[a-zA-Zμµ²³°]+\d*)?)+").unwrap()
});

static RE_EXPAND_MULTIPLY: Lazy<FRegex> = Lazy::new(|| {
    // Helper to expand multiplication symbols. 
    // Uses fixed-width lookbehind for maximum compatibility.
    FRegex::new(r"(?i)(?<=\d|[a-zA-Zμµ²³°])\s*[x×]\s*(?=\d)").unwrap()
});

static RE_ORDINAL: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(thứ|hạng)(\s+)(\d+)\b").unwrap()
});

static RE_PHONE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"((\+84|84|0|0084)(3|5|7|8|9)[0-9]{8})").unwrap()
});

static RE_DOT_SEP: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\d+(\.\d{3})+").unwrap()
});

fn normalize_dot_sep(number: &str) -> String {
    if RE_DOT_SEP.is_match(number) && number.chars().filter(|&c: &char| c == '.').count() > 0 {
         if let Some(m) = RE_DOT_SEP.find(number) {
             if m.as_str() == number {
                 return number.replace(".", "");
             }
         }
    }
    number.to_string()
}

pub fn num_to_words(number: &str, negative: bool) -> String {
    if !number.contains('.') && !number.contains(',') && !number.contains(' ') && number.starts_with('0') && number.len() > 1 {
        let neg_prefix = if negative { "âm " } else { "" };
        return format!("{}{}", neg_prefix, n2w_single(number)).trim().to_string();
    }

    if number.contains('.') {
        let is_thousands = if let Some(m) = RE_DOT_SEP.find(number) {
            m.as_str() == number
        } else {
            false
        };

        if !is_thousands {
            let parts: Vec<&str> = number.split('.').collect();
            if parts.len() == 2 {
                let neg_prefix = if negative { "âm " } else { "" };
                return format!("{}{} chấm {}", neg_prefix, n2w(parts[0]), n2w_decimal(parts[1])).trim().to_string();
            }
        }
    }

    let number_no_dots = normalize_dot_sep(number).replace(" ", "");
    if number_no_dots.contains(',') {
        let parts: Vec<&str> = number_no_dots.split(',').collect();
        let neg_prefix = if negative { "âm " } else { "" };
        return format!("{}{} phẩy {}", neg_prefix, n2w(parts[0]), n2w_decimal(parts[1])).trim().to_string();
    } else if negative {
        return format!("âm {}", n2w(&number_no_dots)).trim().to_string();
    }
    n2w(&number_no_dots)
}

pub fn expand_multiply_number(text: &str) -> String {
    RE_EXPAND_MULTIPLY.replace_all(text, " nhân ").into_owned()
}

pub fn normalize_number_vi(text: &str) -> String {
    let mut result = text.to_string();

    result = RE_ORDINAL.replace_all(&result, |caps: &Captures| {
        let prefix = caps.get(1).unwrap().as_str();
        let space = caps.get(2).unwrap().as_str();
        let number = caps.get(3).unwrap().as_str();
        if number == "1" {
            format!("{}{}nhất", prefix, space)
        } else if number == "4" {
            format!("{}{}tư", prefix, space)
        } else {
            format!("{}{}{}", prefix, space, n2w(number))
        }
    }).to_string();

    result = RE_PHONE.replace_all(&result, |caps: &Captures| {
        n2w_single(caps.get(0).unwrap().as_str())
    }).into_owned();

    let temp_result = result.clone();
    result = RE_NUMBER.replace_all(&temp_result, |caps: &FCaps| {
        let full_match = caps.get(0).unwrap();
        let start = full_match.start();
        let prefix_char = if start > 0 { temp_result.as_bytes().get(start - 1).map(|&b: &u8| b as char) } else { None };

        let neg_symbol = caps.name("neg").map(|m: fancy_regex::Match| m.as_str());
        let number_str = caps.get(2).unwrap().as_str();

        let mut is_neg = false;
        if let Some(_neg) = neg_symbol {
            if prefix_char.is_none() || prefix_char.unwrap().is_whitespace() || "([;,. ".contains(prefix_char.unwrap()) {
                is_neg = true;
            }
        }

        let word = num_to_words(number_str, is_neg);
        if neg_symbol.is_some() && !is_neg {
            format!("{}{}", neg_symbol.unwrap(), word)
        } else {
            format!(" {} ", word)
        }
    }).to_string();

    result
}
