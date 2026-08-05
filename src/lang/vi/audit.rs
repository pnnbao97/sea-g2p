//! Detection of **silent deletion**: meaningful characters that the pipeline
//! discards without turning them into any spoken word.
//!
//! # The problem
//!
//! At the end of `normalize_others`, `RE_CLEAN_OTHERS` strips every character
//! outside a whitelist (Vietnamese letters, digits, whitespace, a few
//! punctuation marks). That safety net is necessary — but it also means a new
//! symbol whose reading nobody declared simply **vanishes without a trace**.
//! The output still reads fluently, so listeners cannot notice the loss.
//!
//! This bug class has bitten at least three times:
//!   - `Σ` (U+2211 / U+03A3) — summation, dropped from the sentence entirely;
//!   - `∆` (U+2206) — "Q = mc∆t" was read as "mc tê";
//!   - `⁻` (U+207B) — "10⁻³" was read as "mười lập phương", i.e. **off by six
//!     orders of magnitude** with no audible cue.
//!
//! # How the audit works
//!
//! `audit_unmapped` does not run the pipeline. It inspects the character
//! inventory of an input and returns those that would be dropped silently,
//! i.e. characters belonging to none of these groups:
//!
//!   1. letters / digits / whitespace — always survive;
//!   2. `READ_BY_MAP` — a reading exists in one of the lookup tables
//!      (`SYMBOLS_MAP`, `SUPERSCRIPTS_MAP`, `SUBSCRIPTS_MAP`,
//!      `CURRENCY_SYMBOL_MAP`);
//!   3. `READ_BY_PASS` — no table entry, but a dedicated pass turns the
//!      character into words (e.g. `°` via the temperature pass);
//!   4. `INTENTIONALLY_DROPPED` — punctuation and formatting whose removal is
//!      deliberate and recorded here.
//!
//! Because groups 3 and 4 must be declared explicitly, any unknown character
//! is reported by default. That is the point: silence must be opt-in.

use std::collections::HashSet;
use once_cell::sync::Lazy;

use crate::lang::vi::resources::{
    SYMBOLS_MAP, SUPERSCRIPTS_MAP, SUBSCRIPTS_MAP, CURRENCY_SYMBOL_MAP,
};

/// Characters with no lookup-table entry that a dedicated pass still converts
/// into spoken words. Each entry names the pass responsible, so the claim can
/// be re-checked when that pass changes.
static READ_BY_PASS: Lazy<HashSet<char>> = Lazy::new(|| {
    let mut s = HashSet::new();
    s.insert('°'); // expand_temperatures / unit pass -> "độ"
    s.insert('%'); // unit pass -> "phần trăm"
    s.insert('@'); // email + URL pass -> "a còng"
    s.insert('/'); // tech pass -> "gạch chéo"; fraction pass -> "trên"
    s.insert('\\'); // tech pass -> "gạch chéo"
    s.insert(':'); // tech pass -> "hai chấm"; ratio pass -> "trên"
    // Absolute value: "|x|" -> "giá trị tuyệt đối của x". A lone bar with no
    // closing partner, as in the conditional probability "P(A|B)", is still
    // dropped — a known gap, not a silent one.
    s.insert('|');
    s
});

/// Characters allowed to disappear, with the reason. This list is
/// documentation: adding an entry asserts "dropping this loses no spoken
/// information".
static INTENTIONALLY_DROPPED: Lazy<HashSet<char>> = Lazy::new(|| {
    let mut s = HashSet::new();
    // Sentence punctuation: kept as prosody or folded into comma/period by the
    // final stage, never spoken as a word.
    for c in ".,!?;".chars() { s.insert(c); }
    // Quotes, including curly and guillemet forms that sanitize_unicode folds
    // into the straight ASCII quote.
    for c in "'\"\u{2018}\u{2019}\u{201C}\u{201D}\u{201E}\u{00AB}\u{00BB}".chars() { s.insert(c); }
    // Brackets: the enclosed text survives; only the pair becomes a comma.
    for c in "()[]{}".chars() { s.insert(c); }
    // Hyphen / underscore: the final stage turns them into a word boundary.
    for c in "-_\u{2013}\u{2014}\u{2212}".chars() { s.insert(c); }
    // Invisible formatting characters, removed by sanitize_unicode up front.
    for c in "\u{200B}\u{200C}\u{200D}\u{2060}\u{FEFF}\u{034F}\u{180E}\u{00AD}".chars() { s.insert(c); }
    // Ellipsis forms, folded to a single period before any pass runs.
    for c in "\u{2024}\u{2025}\u{2026}".chars() { s.insert(c); }
    s
});

/// Characters in `text` that the pipeline would drop with no reading attached.
///
/// Returns them de-duplicated, in order of first appearance. An empty result
/// means the input is safe.
///
/// Intended use is testing: run it over corpora and over an inventory of
/// Unicode characters common in Vietnamese technical prose. A non-empty result
/// means a new character needs either a reading or an explicit entry in
/// `INTENTIONALLY_DROPPED`.
pub fn audit_unmapped(text: &str) -> Vec<char> {
    // Mirror the pipeline: `sanitize_unicode` runs first and folds look-alike
    // characters (`℃` -> `°C`, `‐` -> `-`) into forms the rules recognise, so
    // auditing the raw input would report characters that never reach a pass.
    let sanitized = crate::lang::vi::sanitize_unicode(text);
    let mut seen: HashSet<char> = HashSet::new();
    let mut out: Vec<char> = Vec::new();
    for c in sanitized.chars() {
        if seen.insert(c) && !is_covered(c) {
            out.push(c);
        }
    }
    out
}

fn is_covered(c: char) -> bool {
    c.is_alphanumeric()
        || c.is_whitespace()
        || c.is_control()
        // Combining marks modify a base character that does survive. The mark
        // itself is lost — "z̄" is read as "dét", not "dét gạch ngang" — but the
        // word remains intelligible, so this is a documented limitation rather
        // than a disappearance.
        || is_combining_mark(c)
        || SYMBOLS_MAP.contains_key(&c)
        || SUPERSCRIPTS_MAP.contains_key(&c)
        || SUBSCRIPTS_MAP.contains_key(&c)
        || CURRENCY_SYMBOL_MAP.contains_key(c.to_string().as_str())
        || READ_BY_PASS.contains(&c)
        || INTENTIONALLY_DROPPED.contains(&c)
}

/// Unicode combining marks (general categories Mn, Mc, Me).
fn is_combining_mark(c: char) -> bool {
    matches!(c as u32,
        0x0300..=0x036F   // combining diacritical marks
        | 0x1AB0..=0x1AFF // extended
        | 0x1DC0..=0x1DFF // supplement
        | 0x20D0..=0x20FF // for symbols
        | 0xFE20..=0xFE2F // half marks
    )
}
