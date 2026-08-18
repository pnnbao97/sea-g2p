//! Foreign Latin letters folded onto their ASCII base.
//!
//! # The problem
//!
//! `RE_CLEAN_OTHERS` (see [`crate::lang::vi::misc`]) keeps ASCII letters,
//! digits and the Vietnamese alphabet, and replaces **everything else with a
//! space**. A German, Nordic or Slavic name therefore does not merely lose its
//! accent — it is cut in half, and each half is then read on its own:
//!
//! ```text
//! "Müller nói vậy"  ->  "M ller nói vậy"  ->  "mờ ller nói vậy"
//! "Straße số 5"     ->  "Stra e số 5"     ->  "stra e số năm"
//! ```
//!
//! The leading `M` became a stray single letter, so the spell-out pass read it
//! as a letter name. The damage is worse than a dropped diacritic: it invents a
//! word boundary that was never in the text.
//!
//! # The fold
//!
//! `ä` and `ö` differ from `a` and `o` in ways Vietnamese phonology cannot
//! express anyway, so the base letter loses nothing a Vietnamese voice could
//! have said. Folding it keeps the word whole: `Müller` -> `Muller`, which the
//! syllable passes and the G2P read as one name.
//!
//! Two mechanisms, in order:
//!
//!  1. **Canonical decomposition.** NFD splits a precomposed letter into a base
//!     plus combining marks; dropping the marks leaves the base. This covers
//!     the whole of Latin-1 Supplement, Latin Extended-A and most of Extended-B
//!     without a table: `ä ö ü ñ ç ë å š ž č ř ę ő ğ` and the rest.
//!  2. **[`IRREDUCIBLE`]** — the letters NFD leaves alone because their mark is
//!     part of the glyph (a stroke, a ligature): `ß ø ł æ œ þ ð ħ ŧ ı`.
//!
//! # What is never folded
//!
//!  - **Vietnamese letters.** `ế` decomposes to `e` + two marks, so without an
//!    explicit guard this pass would flatten the language it serves. Every
//!    character of [`VI_LETTERS`] is exempt before decomposition is attempted.
//!  - **Non-Latin scripts.** Greek `α` and Cyrillic `а` survive NFD unchanged,
//!    and the result is not ASCII, so the fold declines them. Greek in
//!    particular already has readings in `SYMBOLS_MAP` (`α` -> "an pha") that
//!    a fold to `a` would destroy.
//!  - **`µ`, `Ω` and friends.** Same rule: NFD keeps them non-ASCII, so the
//!    unit tables keep their entries.

use std::collections::{HashMap, HashSet};

use once_cell::sync::Lazy;
use unicode_normalization::UnicodeNormalization;

use crate::lang::vi::resources::VI_LETTERS;

static VI_LETTER_SET: Lazy<HashSet<char>> = Lazy::new(|| VI_LETTERS.chars().collect());

/// Letters whose Latin base no decomposition recovers, because the difference
/// is a stroke through the glyph or a ligature rather than a combining mark.
///
/// The values follow the spelling convention of the source language where one
/// exists (`ß` -> "ss", `æ` -> "ae"), otherwise the plain base letter.
static IRREDUCIBLE: Lazy<HashMap<char, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();
    // German sharp s. "Straße" -> "Strasse", the accepted ASCII spelling.
    m.insert('ß', "ss"); m.insert('ẞ', "SS");
    // Nordic o with stroke: Ø / ø.
    m.insert('ø', "o"); m.insert('Ø', "O");
    m.insert('ǿ', "o"); m.insert('Ǿ', "O");
    // Ligatures.
    m.insert('æ', "ae"); m.insert('Æ', "AE");
    m.insert('œ', "oe"); m.insert('Œ', "OE");
    // Polish l with stroke: "Łódź" -> "Lodz".
    m.insert('ł', "l"); m.insert('Ł', "L");
    // Icelandic thorn and eth. Note that Croatian "đ" is NOT here: it is a
    // Vietnamese letter, already pronounceable, and VI_LETTERS keeps it.
    m.insert('þ', "th"); m.insert('Þ', "TH");
    m.insert('ð', "d"); m.insert('Ð', "D");
    // Maltese and Sami strokes.
    m.insert('ħ', "h"); m.insert('Ħ', "H");
    m.insert('ŧ', "t"); m.insert('Ŧ', "T");
    // Turkish dotless i. Its capital "I" is plain ASCII already; the dotted
    // capital "İ" decomposes to I + dot above and needs no entry.
    m.insert('ı', "i");
    // Kra and eng, from older Greenlandic and from Sami orthography.
    m.insert('ĸ', "k");
    m.insert('ŋ', "ng"); m.insert('Ŋ', "NG");
    // Ligatures and typographic leftovers that would otherwise be the only
    // holes in Latin-1 Supplement and Latin Extended-A: Dutch IJ, Catalan
    // l·l, the deprecated 'n, and the long s.
    m.insert('ĳ', "ij"); m.insert('Ĳ', "IJ");
    m.insert('ŀ', "l"); m.insert('Ŀ', "L");
    m.insert('ŉ', "n");
    m.insert('ſ', "s");
    m
});

/// The ASCII spelling of `c`, or `None` if `c` must be left exactly as it is.
///
/// `None` covers ASCII itself, non-letters, Vietnamese letters and every
/// non-Latin script — see the module documentation for why each is excluded.
fn fold_letter(c: char) -> Option<String> {
    if c.is_ascii() || !c.is_alphabetic() || VI_LETTER_SET.contains(&c) {
        return None;
    }
    if let Some(base) = IRREDUCIBLE.get(&c) {
        return Some((*base).to_string());
    }
    let base: String = c.nfd().filter(|d: &char| !is_combining_mark(*d)).collect();
    // An empty result would mean the character *is* a mark; a non-ASCII one
    // means a script the fold has no business touching.
    if !base.is_empty() && base.chars().all(|d: char| d.is_ascii_alphabetic()) {
        return Some(base);
    }
    None
}

/// Combining marks, by the same ranges [`crate::lang::vi::audit`] uses.
fn is_combining_mark(c: char) -> bool {
    matches!(c as u32,
        0x0300..=0x036F   // combining diacritical marks
        | 0x1AB0..=0x1AFF // extended
        | 0x1DC0..=0x1DFF // supplement
        | 0x20D0..=0x20FF // for symbols
        | 0xFE20..=0xFE2F // half marks
    )
}

/// Rewrite every foreign Latin letter in `text` as its ASCII base.
///
/// Expects NFC input: a decomposed `u` + U+0308 is two characters, neither of
/// which the fold touches, and the mark would then be dropped later anyway.
/// The pipeline composes before calling this.
pub fn fold_foreign_letters(text: &str) -> String {
    if text.is_ascii() {
        return text.to_string();
    }
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match fold_letter(c) {
            Some(base) => out.push_str(&base),
            None => out.push(c),
        }
    }
    out
}
