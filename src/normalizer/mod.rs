pub mod vi_resources;
pub mod num2vi;
pub mod cleaner;

use unicode_normalization::UnicodeNormalization;

pub struct Normalizer {
}

impl Normalizer {
    pub fn new() -> Self {
        Self {}
    }

    pub fn normalize(&self, text: &str) -> String {
        if text.is_empty() {
            return String::new();
        }

        // Pre-normalization: Ensure NFC format for Vietnamese characters
        let text_nfc: String = text.nfc().collect();

        // 1. Detect and protect EN tags (matches Python's Normalizer.normalize)
        let mut en_contents = Vec::new();
        let mut text_masked = text_nfc;

        static RE_EN: Lazy<fancy_regex::Regex> = Lazy::new(|| fancy_regex::Regex::new(r"(?i)<en>.*?</en>").unwrap());

        while let Ok(Some(mat)) = RE_EN.find(&text_masked) {
            let start = mat.start();
            let end = mat.end();
            let content = mat.as_str().to_string();
            let placeholder = format!("ENTOKEN{}", en_contents.len());
            en_contents.push(content);
            text_masked.replace_range(start..end, &placeholder);
        }

        // 2. Core Normalization
        let mut normalized = cleaner::clean_vietnamese_text(&text_masked);

        // 3. Final cleanup - preserve newlines
        normalized = normalized.to_lowercase();
        normalized = cleaner::cleanup_whitespace(&normalized);

        // 4. Restore EN tags
        for (idx, content) in en_contents.iter().enumerate() {
            let placeholder = format!("entoken{}", idx);
            normalized = normalized.replace(&placeholder, content);
        }

        // Final whitespace cleanup
        cleaner::cleanup_whitespace(&normalized)
    }
}

use once_cell::sync::Lazy;
