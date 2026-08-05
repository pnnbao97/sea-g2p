//! Generic abbreviation table shared by every language module.
//!
//! One table per language maps an abbreviation, matched on word boundaries,
//! to an explicit *reading mode* — the design decision being that how an
//! abbreviation is read (expanded phrase, fixed pronunciation, English word,
//! spelled letter-by-letter in either letter-name system) is data, not
//! something a heuristic should guess. Heuristics remain only as fallback
//! for tokens absent from the table.
//!
//! The Vietnamese instance is built in `lang::vi::resources`; a Thai
//! instance will carry รธน./ม.ค.-style entries with the same type.

use std::collections::HashMap;

/// How an abbreviation is read aloud.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Reading {
    /// Replace with target-language words: "UBND" -> "uỷ ban nhân dân".
    Expand(&'static str),
    /// Fixed pronunciation, kept verbatim (may carry language tags):
    /// "JSON" -> "__start_en__j son__end_en__", "CUDA" -> "cu đa".
    Fixed(&'static str),
    /// Read as one English word: "NASA" -> tagged lowercase form.
    WordEn,
    /// Spell out with Vietnamese letter names ("xê mờ nờ đê").
    LettersVi,
    /// Spell out with English letter names, tagged English.
    LettersEn,
}

pub struct AbbrevTable {
    map: HashMap<&'static str, Reading>,
}

impl AbbrevTable {
    pub fn new() -> Self {
        Self { map: HashMap::new() }
    }

    pub fn insert(&mut self, key: &'static str, reading: Reading) {
        self.map.insert(key, reading);
    }

    pub fn get(&self, key: &str) -> Option<&Reading> {
        self.map.get(key)
    }

    pub fn contains(&self, key: &str) -> bool {
        self.map.contains_key(key)
    }

    /// Keys whose reading replaces the token with plain text (Expand/Fixed) —
    /// the set the normalizer builds its boundary-matching regex from.
    pub fn replacement_keys(&self) -> impl Iterator<Item = &'static str> + '_ {
        self.map.iter().filter_map(|(k, r)| match r {
            Reading::Expand(_) | Reading::Fixed(_) => Some(*k),
            _ => None,
        })
    }

    /// The replacement text for Expand/Fixed keys, `None` for other modes.
    pub fn replacement(&self, key: &str) -> Option<&'static str> {
        match self.map.get(key) {
            Some(Reading::Expand(s)) | Some(Reading::Fixed(s)) => Some(s),
            _ => None,
        }
    }
}

impl Default for AbbrevTable {
    fn default() -> Self {
        Self::new()
    }
}
