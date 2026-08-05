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
    /// `:` — "hai chấm", "ทวิภาค", "titik dua".
    pub colon: &'static str,
    /// Spell a run of Latin letters with this language's letter names.
    /// Vietnamese reads "https" as "hát tê tê phê ét", not as an English
    /// word, and the other languages should match rather than leave the
    /// scheme for the G2P stage to guess at.
    pub spell: fn(&str) -> String,
}

/// `<math>…</math>` and `<en>…</en>`: markup the caller wrote deliberately.
/// Reading the tag itself turned `<math>b²</math>` into "less than math
/// greater than b squared", so the delimiters are stripped and the content
/// kept — the symbol stage inside it then voices the operators.
static RE_TAG: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)</?(?:math|en)>").unwrap()
});

/// Identifier-shaped runs: licence plates, model numbers, order codes. The
/// digits in them are not quantities — a Thai plate `กก 1234` is four
/// figures, not one thousand two hundred and thirty-four — so they are read
/// out one by one.
static RE_PLATE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?i)([A-Za-z\u{0E01}-\u{0E4E}]{1,3})\s?(\d{2,5})(?:\s?([A-Za-z]{1,3}))?\b",
    )
    .unwrap()
});

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
    let text = RE_TAG.replace_all(text, " ");
    let out = RE_EMAIL.replace_all(&text, |c: &Captures| {
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
            // The scheme is READ, not dropped. Text that says "https://"
            // means it, and silently removing it is the same defect class
            // this module was written to fix — the listener has no way to
            // know an address was a secure one, or that a scheme was there
            // at all.
            let mut prefix = String::new();
            let rest = if let Some(r) = span.strip_prefix("https://") {
                prefix = format!("{} {} {} {} ", (w.spell)("https"), w.colon, w.slash, w.slash);
                r
            } else if let Some(r) = span.strip_prefix("http://") {
                prefix = format!("{} {} {} {} ", (w.spell)("http"), w.colon, w.slash, w.slash);
                r
            } else {
                span
            };
            let (host, path) = match rest.split_once('/') {
                Some((h, p)) => (h, Some(p)),
                None => (rest, None),
            };
            let mut out = format!(" {}{} ", prefix, read_host(host, w));
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
    let lower = label.to_lowercase();
    if WORD_SUFFIXES.contains(&lower.as_str()) {
        return lower;
    }
    // www is an initialism, not a word: Vietnamese reads it "vê kép" three
    // times over, and the other languages spell it too.
    if lower == "www" {
        return (w.spell)("www");
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

/// Read an identifier-shaped run figure by figure: `B 1234 XYZ`, `กก 1234`.
///
/// `spell_digits` renders each figure separately and `spell` the letters, so
/// the caller decides both. Returns `None` when nothing in `text` looks like
/// an identifier, letting the caller skip the pass.
pub fn expand_identifiers(
    text: &str,
    w: &SpanWords,
    spell_digits: fn(&str) -> String,
    cues: &[&str],
) -> String {
    RE_PLATE
        .replace_all(text, |c: &Captures| {
            let start = c.get(0).map(|m| m.start()).unwrap_or(0);
            let before = text[..start].trim_end().to_lowercase();
            if !cues.iter().any(|cue| before.ends_with(cue)) {
                return c[0].to_string();
            }
            let head = (w.spell)(&c[1]);
            let digits = spell_digits(&c[2]);
            match c.get(3) {
                Some(tail) => format!(" {} {} {} ", head, digits, (w.spell)(tail.as_str())),
                None => format!(" {} {} ", head, digits),
            }
        })
        .into_owned()
}

/// Characters this module consumes, for the silent-deletion audits.
pub fn handled_chars() -> &'static str {
    "@:/_%?=&#+"
}
