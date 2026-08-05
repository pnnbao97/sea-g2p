//! Indonesian text normalization.
//!
//! # Architecture: a staged pipeline
//!
//! Same contract as the Vietnamese and Thai modules — the order encodes what
//! each stage assumes the earlier ones have already resolved.
//!
//! | # | Stage | Does | Why it sits here |
//! |---|-------|------|------------------|
//! | 1 | `abbreviations` | yg, dgn, tdk, DPR, Rp | Before numbers and dates: the money and date forms embed digits these would claim |
//! | 2 | `datetime` | 17/8/1945, 14:30 | After abbreviations supplied month names; before generic numbers |
//! | 3 | `money` | Rp1.250.000 | Before numbers: Indonesian groups thousands with a PERIOD, so the generic pass would read it as a decimal |
//! | 4 | `numbers` | decimals, thousands, cardinals | Once the specialised forms are consumed |
//! | 5 | `symbols` | %, °, mathematical signs | Anything left |
//! | 6 | `residual` | punctuation, whitespace | Must be last |
//!
//! # The separator trap
//!
//! Indonesian writes 1.250,75 where English writes 1,250.75 — period for
//! thousands, comma for the decimal mark. Reading it with the English
//! convention turns one and a quarter thousand into "one point two five".
//! The money and number stages therefore read a period between digit groups
//! as a separator, and a comma as the decimal point.
//!
//! # Invariant: nothing disappears in silence
//!
//! [`audit_unmapped`] reports characters that would reach the residual stage
//! and be deleted without becoming any word, the same guard the other two
//! languages carry.

use once_cell::sync::Lazy;
use regex::{Captures, Regex};

use super::num2id::{n2w, n2w_decimal, n2w_single};
use super::resources::{ID_ABBREV, ID_LETTER_NAMES, ID_MONTHS, ID_SYMBOLS};
use crate::core::abbrev::Reading;

// ── Stage 1: abbreviations ──────────────────────────────────────────────────

static RE_ABBREV: Lazy<Regex> = Lazy::new(|| {
    let mut keys: Vec<&str> = ID_ABBREV.keys().collect();
    keys.sort_by_key(|k| std::cmp::Reverse(k.len()));
    let alts: Vec<String> = keys.iter().map(|k| regex::escape(k)).collect();
    // word boundaries, so "yg" does not fire inside "bayg..."
    Regex::new(&format!(r"(?i)\b(?:{})\b\.?", alts.join("|"))).unwrap()
});

fn stage_abbreviations(text: &str) -> String {
    RE_ABBREV
        .replace_all(text, |c: &Captures| {
            let key = c[0].trim_end_matches('.').to_lowercase();
            let raw = c[0].trim_end_matches('.');
            match ID_ABBREV.get(&key).or_else(|| ID_ABBREV.get(raw)) {
                Some(Reading::Expand(v)) | Some(Reading::Fixed(v)) => format!(" {} ", v),
                // Spelled initialisms are rewritten into their letter names
                // here rather than left for the G2P stage, which would see
                // a vowel-less string and read it as one word.
                Some(Reading::LettersNative) => {
                    let names: Vec<&str> = raw
                        .to_lowercase()
                        .chars()
                        .filter_map(|ch| ID_LETTER_NAMES.get(&ch).copied())
                        .collect();
                    format!(" {} ", names.join(" "))
                }
                _ => c[0].to_string(),
            }
        })
        .into_owned()
}

// ── Stage 2: datetime ───────────────────────────────────────────────────────

static RE_DATE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b(\d{1,2})[/-](\d{1,2})[/-](\d{4})\b").unwrap()
});
/// The cue word is captured so it is not emitted twice: "pukul 14:30" was
/// coming out as "pukul pukul empat belas".
static RE_TIME: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\b(?:(pukul|jam)\s+)?(\d{1,2}):(\d{2})\b").unwrap()
});

fn stage_datetime(text: &str) -> String {
    let out = RE_DATE.replace_all(text, |c: &Captures| {
        let d: u32 = c[1].parse().unwrap_or(0);
        let m: usize = c[2].parse().unwrap_or(0);
        if d == 0 || d > 31 || m == 0 || m > 12 {
            return c[0].to_string();
        }
        format!(" {} {} {} ", n2w(&c[1]), ID_MONTHS[m - 1], n2w(&c[3]))
    });
    RE_TIME
        .replace_all(&out, |c: &Captures| {
            let (h, mi): (u32, u32) = (c[2].parse().unwrap_or(99), c[3].parse().unwrap_or(99));
            if h > 23 || mi > 59 {
                return c[0].to_string();
            }
            let cue = c.get(1).map(|m| m.as_str()).unwrap_or("pukul");
            if mi == 0 {
                format!(" {} {} ", cue, n2w(&c[2]))
            } else {
                format!(" {} {} lewat {} menit ", cue, n2w(&c[2]), n2w(&c[3]))
            }
        })
        .into_owned()
}

// ── Stage 3: money ──────────────────────────────────────────────────────────

static RE_RUPIAH: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\bRp\.?\s*(\d[\d.]*(?:,\d+)?)").unwrap()
});

fn stage_money(text: &str) -> String {
    RE_RUPIAH
        .replace_all(text, |c: &Captures| format!(" {} rupiah ", read_number(&c[1])))
        .into_owned()
}

// ── Stage 4: numbers ────────────────────────────────────────────────────────

/// A period only groups thousands when exactly three digits follow it, and a
/// comma is the decimal mark. This is the reverse of the English convention.
static RE_NUMBER: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\d+(?:\.\d{3})*(?:,\d+)?").unwrap()
});

fn read_number(s: &str) -> String {
    let (int_part, frac) = match s.split_once(',') {
        Some((a, b)) => (a, Some(b)),
        None => (s, None),
    };
    let digits: String = int_part.chars().filter(|c| c.is_ascii_digit()).collect();
    match frac {
        Some(f) => n2w_decimal(&digits, f),
        None => {
            if digits.len() > 1 && digits.starts_with('0') {
                n2w_single(&digits) // a written leading zero marks an identifier
            } else if digits.len() > 6 && !int_part.contains('.') {
                n2w_single(&digits)
            } else {
                n2w(&digits)
            }
        }
    }
}

fn stage_numbers(text: &str) -> String {
    RE_NUMBER
        .replace_all(text, |c: &Captures| format!(" {} ", read_number(&c[0])))
        .into_owned()
}

// ── Stage 5: symbols ────────────────────────────────────────────────────────

static RE_MARKUP_RUN: Lazy<Regex> = Lazy::new(|| Regex::new(r"[=\-*_#]{2,}").unwrap());

fn stage_symbols(text: &str) -> String {
    let text = RE_MARKUP_RUN.replace_all(text, " ");
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match ID_SYMBOLS.get(&c) {
            Some(w) => out.push_str(w),
            None => out.push(c),
        }
    }
    out
}

// ── Stage 6: residual ───────────────────────────────────────────────────────

static RE_PAUSE: Lazy<Regex> = Lazy::new(|| Regex::new(r"[;:]").unwrap());
static RE_ELLIPSIS: Lazy<Regex> = Lazy::new(|| Regex::new(r"[…‥․]+|\.{2,}").unwrap());
static RE_DROP: Lazy<Regex> = Lazy::new(|| Regex::new(r"[^A-Za-z0-9\s,.!?'-]").unwrap());
static RE_SPACES: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s+").unwrap());

fn stage_residual(text: &str) -> String {
    let out = RE_PAUSE.replace_all(text, ",");
    let out = RE_ELLIPSIS.replace_all(&out, ".");
    let out = RE_DROP.replace_all(&out, " ");
    RE_SPACES.replace_all(&out, " ").trim().to_string()
}

/// Normalize Indonesian text into a form the G2P stage can read.
pub fn normalize(text: &str) -> String {
    let mut s = stage_abbreviations(text);
    s = stage_datetime(&s);
    s = stage_money(&s);
    s = stage_numbers(&s);
    s = stage_symbols(&s);
    stage_residual(&s)
}

/// Punctuation and formatting whose removal changes nothing audible.
const INTENTIONALLY_DROPPED: &str = "\"“”‘’()[]{}«»–—_|\\*#…:;\u{200B}\u{FEFF}";

/// Characters of `text` that would be deleted without becoming a word.
pub fn audit_unmapped(text: &str) -> Vec<char> {
    let mut out = Vec::new();
    for c in text.chars() {
        if c.is_alphanumeric()
            || c.is_whitespace()
            || matches!(c, ',' | '.' | '!' | '?' | '\'' | '-')
            || ID_SYMBOLS.contains_key(&c)
            || INTENTIONALLY_DROPPED.contains(c)
        {
            continue;
        }
        if !out.contains(&c) {
            out.push(c);
        }
    }
    out
}
