//! Email addresses, URLs and file paths: spans that must be read as a unit.
//!
//! These are the one class of input where the *later* stages are the danger.
//! A URL contains slashes, periods, colons and hyphens that the symbol and
//! numeric passes each have a legitimate claim on, so left alone
//! `https://www.google.com` came out of the Thai pipeline as
//! "https, ทับ ทับ www.google.com" — the punctuation voiced, the domain
//! never read at all.
//!
//! The Vietnamese module solves this with a dedicated `protect_spans` stage
//! that masks such spans before anything else runs. This is the same idea,
//! shared: one set of patterns, one reader, and per-language words for the
//! separators.
//!
//! # Ordering
//!
//! [`expand`] must run **first**, before any stage that voices punctuation
//! or claims digits. It replaces each span with its spoken form, so nothing
//! downstream sees the original characters.

use once_cell::sync::Lazy;
use regex::{Captures, Regex};

/// What one language calls the pieces of an address.
pub struct SpanWords {
    /// `@` — "a còng", "แอท", "at".
    pub at: &'static str,
    /// `.` inside a domain — "chấm", "จุด", "titik".
    pub dot: &'static str,
    /// `/` inside a path — "gạch chéo", "ทับ", "garis miring".
    pub slash: &'static str,
    /// `-` inside a domain or path — "gạch ngang", "ขีด", "strip".
    pub dash: &'static str,
    /// `_` — "gạch dưới", "ขีดล่าง", "garis bawah".
    pub underscore: &'static str,
}

static RE_EMAIL: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\b[a-z0-9._%+-]+@[a-z0-9.-]+\.[a-z]{2,}\b").unwrap()
});
static RE_URL: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\b(?:https?://|www\.)[a-z0-9.\-/_%?=&#]+").unwrap()
});

/// Domain suffixes read as words rather than spelled: everyone says "com",
/// nobody says "see oh em".
const WORD_SUFFIXES: &[&str] = &[
    "com", "net", "org", "edu", "gov", "info", "biz", "io", "co", "id",
    "th", "vn", "my", "sg", "ph", "asia", "shop", "site", "app", "dev",
];

/// Expand every address-like span. `read_word` phonemises an ordinary word,
/// letting a caller decide whether an unknown domain label is spelled or
/// read; passing `None` leaves labels as they are for the G2P stage.
pub fn expand(text: &str, w: &SpanWords) -> String {
    let out = RE_EMAIL.replace_all(text, |c: &Captures| {
        let span = &c[0];
        match span.split_once('@') {
            Some((user, host)) => {
                format!(" {} {} {} ", read_label(user, w), w.at, read_host(host, w))
            }
            None => span.to_string(),
        }
    });
    RE_URL
        .replace_all(&out, |c: &Captures| {
            let span = c[0].trim_end_matches(|ch: char| matches!(ch, '.' | ',' | ')' | '!' | '?'));
            // The scheme is noise in speech: nobody dictates "h t t p s colon
            // slash slash" when reading an address aloud.
            let rest = span
                .trim_start_matches("https://")
                .trim_start_matches("http://");
            let (host, path) = match rest.split_once('/') {
                Some((h, p)) => (h, Some(p)),
                None => (rest, None),
            };
            let mut out = format!(" {} ", read_host(host, w));
            if let Some(p) = path.filter(|p| !p.is_empty()) {
                for piece in p.split('/') {
                    if !piece.is_empty() {
                        out.push_str(&format!("{} {} ", w.slash, read_label(piece, w)));
                    }
                }
            }
            out
        })
        .into_owned()
}

/// A host: labels joined by the word for ".", with `www` kept as written.
fn read_host(host: &str, w: &SpanWords) -> String {
    host.split('.')
        .filter(|l| !l.is_empty())
        .map(|l| read_label(l, w))
        .collect::<Vec<_>>()
        .join(&format!(" {} ", w.dot))
}

/// One label, with its internal separators voiced. A known domain suffix is
/// left as a word; anything else is handed on for the G2P stage to read.
fn read_label(label: &str, w: &SpanWords) -> String {
    if WORD_SUFFIXES.contains(&label.to_lowercase().as_str()) {
        return label.to_lowercase();
    }
    let mut out = String::with_capacity(label.len());
    for c in label.chars() {
        match c {
            '-' => out.push_str(&format!(" {} ", w.dash)),
            '_' => out.push_str(&format!(" {} ", w.underscore)),
            '.' => out.push_str(&format!(" {} ", w.dot)),
            '/' => out.push_str(&format!(" {} ", w.slash)),
            '%' | '?' | '=' | '&' | '#' | '+' => out.push(' '),
            _ => out.push(c),
        }
    }
    out.trim().to_string()
}

/// Characters this module consumes, for the silent-deletion audits.
pub fn handled_chars() -> &'static str {
    "@:/_%?=&#+"
}
