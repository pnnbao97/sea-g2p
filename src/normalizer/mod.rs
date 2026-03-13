pub mod cleaner;
pub mod num2vi;
pub mod vi_resources;

use unicode_normalization::UnicodeNormalization;
use fancy_regex::Regex;
use once_cell::sync::Lazy;

pub struct Normalizer {
    pub lang: String,
}

static RE_EXTRA_SPACES: Lazy<Regex> = Lazy::new(|| Regex::new(r"[ \t\xA0]+").unwrap());

impl Normalizer {
    pub fn new(lang: &str) -> Self {
        Self {
            lang: lang.to_string(),
        }
    }

    pub fn normalize(&self, text: &str) -> String {
        if text.is_empty() {
            return String::new();
        }

        // Pre-normalization: Ensure NFC format
        let normalized_text: String = text.nfc().collect();

        // Step 1: Detect and protect EN tags
        let mut en_contents = Vec::new();
        let placeholder_pattern = "entoken{}";

        let re_en = Regex::new(r"(?i)<en>.*?</en>").unwrap();
        let mut text_with_placeholders = String::new();
        let mut last_end = 0;

        for mat_res in re_en.find_iter(&normalized_text) {
            if let Ok(mat) = mat_res {
                text_with_placeholders.push_str(&normalized_text[last_end..mat.start()]);
                let placeholder = placeholder_pattern.replace("{}", &en_contents.len().to_string());
                text_with_placeholders.push_str(&placeholder);
                en_contents.push(mat.as_str().to_string());
                last_end = mat.end();
            }
        }
        text_with_placeholders.push_str(&normalized_text[last_end..]);

        // Step 2: Core Normalization
        let mut result = cleaner::clean_vietnamese_text(text_with_placeholders);

        // Final cleanup - preserve newlines
        result = result.to_lowercase();
        result = RE_EXTRA_SPACES.replace_all(&result, " ").to_string();
        result = result.trim().to_string();

        // Step 3: Restore EN tags
        for (idx, en_content) in en_contents.iter().enumerate() {
            let placeholder = placeholder_pattern.replace("{}", &idx.to_string());
            result = result.replace(&placeholder, en_content);
        }

        // Final whitespace cleanup
        result = RE_EXTRA_SPACES.replace_all(&result, " ").to_string();
        result.trim().to_string()
    }
}
