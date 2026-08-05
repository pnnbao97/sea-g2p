//! Roman numerals, which are only numerals when something says they are.
//!
//! `IX`, `II` and `XXI` are also ordinary letter sequences, so expanding them
//! on sight turns "CD" into four hundred and "MC" into eleven hundred. The
//! Vietnamese module solves this by requiring a **cue word** in front — "thế
//! kỷ XXI", "chương IV" — and this is the same rule, shared, with each
//! language supplying its own cues.
//!
//! Thai needs it more than Vietnamese does: reign names are written this way
//! constantly (`รัชกาลที่ IX`), and left alone the numeral reached the G2P
//! stage as three unread Latin letters.

use once_cell::sync::Lazy;
use regex::{Captures, Regex};

/// A cue is a word that can only be followed by a number: "century",
/// "chapter", "reign", "World War". Matching is case-insensitive and the cue
/// may be followed by an ordinal marker the language uses ("ที่" in Thai,
/// "ke-" in Indonesian).
pub struct RomanCues {
    /// Words that license the reading, lowercase.
    pub words: &'static [&'static str],
}

static RE_ROMAN: Lazy<Regex> = Lazy::new(|| {
    // A well-formed numeral, at least two characters so a lone "I" or "V" —
    // almost always an initial — is never claimed.
    Regex::new(
        r"(?i)\b(M{0,4}(?:CM|CD|D?C{0,3})(?:XC|XL|L?X{0,3})(?:IX|IV|V?I{0,3}))\b",
    )
    .unwrap()
});

fn value(s: &str) -> Option<u32> {
    let digits: Vec<u32> = s
        .to_uppercase()
        .chars()
        .map(|c| match c {
            'I' => Some(1),
            'V' => Some(5),
            'X' => Some(10),
            'L' => Some(50),
            'C' => Some(100),
            'D' => Some(500),
            'M' => Some(1000),
            _ => None,
        })
        .collect::<Option<Vec<u32>>>()?;
    if digits.is_empty() {
        return None;
    }
    let mut total = 0;
    for (i, d) in digits.iter().enumerate() {
        match digits.get(i + 1) {
            Some(next) if d < next => total -= *d as i64 as u32,
            _ => total += d,
        }
    }
    Some(total)
}

/// Expand Roman numerals that a cue word licenses.
///
/// `cardinal` renders the value in the target language. Text with no cue is
/// left exactly as it was — an unlicensed `CD` stays "CD".
pub fn expand(text: &str, cues: &RomanCues, cardinal: fn(&str) -> String) -> String {
    RE_ROMAN
        .replace_all(text, |c: &Captures| {
            let numeral = &c[1];
            if numeral.len() < 2 {
                return numeral.to_string();
            }
            let start = c.get(1).map(|m| m.start()).unwrap_or(0);
            if !preceded_by_cue(text, start, cues) {
                return numeral.to_string();
            }
            match value(numeral) {
                Some(v) if v > 0 => format!("{} ", cardinal(&v.to_string())),
                _ => numeral.to_string(),
            }
        })
        .into_owned()
}

/// Does a cue word appear in the few characters before `pos`?
///
/// The window is generous because a language may insert an ordinal marker
/// between the cue and the numeral: Thai writes "รัชกาลที่ IX", Indonesian
/// "Perang Dunia II".
fn preceded_by_cue(text: &str, pos: usize, cues: &RomanCues) -> bool {
    let before = text[..pos].trim_end().to_lowercase();
    cues.words.iter().any(|w| before.ends_with(w))
}
