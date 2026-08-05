//! Units of measure, prime marks and ratios: a quantity and the symbol that
//! names it, read together.
//!
//! # Why this exists
//!
//! Vietnamese has read `60 km/h` as "sáu mươi ki lô mét trên giờ" for a long
//! time. Thai and Indonesian had no unit table at all, so the abbreviation
//! reached the G2P stage as bare Latin letters and the `/` was voiced by the
//! symbol pass as the *punctuation* name:
//!
//! ```text
//! 60 km/h   ->  หกสิบ km ทับ h          /  enam puluh km garis miring jam
//! 50 m2     ->  ห้าสิบ m สอง             /  lima puluh m dua
//! ```
//!
//! Both are wrong in the same way: `/` between two units is "per", not
//! "slash", and a trailing 2 is an exponent, not a count.
//!
//! # What counts as a unit
//!
//! Only an abbreviation **immediately preceded by a digit** and present in the
//! language's table. Both conditions matter. Without the digit, `m` is a
//! letter and `ha` is an Indonesian interjection; without the table, every
//! letter run after a number would be claimed — which is exactly how licence
//! plates and model numbers get misread.
//!
//! # Ordering
//!
//! [`expand`] must run **before** the generic number pass, which would consume
//! the digit this module matches on, and before any pass that voices `°`, `/`,
//! `'` or `"` as punctuation.

use once_cell::sync::Lazy;
use regex::{Captures, Regex};

/// What one language calls the pieces of a measurement.
pub struct UnitWords {
    /// `/` between two units: "ต่อ", "per". Not the punctuation name — that
    /// reading ("ทับ", "garis miring") is what this module exists to prevent.
    pub per: &'static str,
    /// Build "square X" and "cubic X". These are functions rather than
    /// affixes because the languages disagree on position: Thai prefixes
    /// (ตารางเมตร) where Indonesian suffixes (meter persegi).
    pub square: fn(&str) -> String,
    pub cubic: fn(&str) -> String,
    /// Resolve one abbreviation to its spoken form, or `None` if this
    /// language does not claim it. Keeping the table in the language module
    /// lets each own its vocabulary.
    pub lookup: fn(&str) -> Option<&'static str>,
    /// `5'6"` — the imperial pair.
    pub feet: &'static str,
    pub inches: &'static str,
    /// `13° 45' 30"` — the sexagesimal pair, used for coordinates.
    pub arcminute: &'static str,
    pub arcsecond: &'static str,
    /// `3:1` — "ต่อ", "banding".
    pub ratio: &'static str,
}

/// A number, a unit abbreviation, an optional exponent, and optionally a
/// second unit after a slash: `50 m2`, `60 km/h`, `9.8 m/s²`.
///
/// Only the LAST digit of the quantity is captured, and it is re-emitted
/// verbatim — the digits are left for the number stage rather than read here,
/// so this module never has to know how a language says "nine point eight".
static RE_UNIT: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(\d)\s*([a-z]{1,4})([²³23])?(?:\s*/\s*([a-z]{1,4})([²³23])?)?\b").unwrap()
});

/// `13° 45'` and `13° 45' 30"`. Must run before the pass that reads `°`
/// alone, which would otherwise consume the degrees and strand the primes.
static RE_DEG_MIN_SEC: Lazy<Regex> = Lazy::new(|| {
    Regex::new("(\\d+)\\s*°\\s*(\\d+)\\s*['\u{2032}](?:\\s*(\\d+)\\s*[\"\u{2033}])?").unwrap()
});
/// `5'6"` — feet and inches, which only ever appear as a pair.
static RE_FEET_INCH: Lazy<Regex> = Lazy::new(|| {
    Regex::new("\\b(\\d+)\\s*['\u{2032}]\\s*(\\d+)\\s*[\"\u{2033}]").unwrap()
});
/// `3:1`. Safe only after the clock-time pass has taken what it wants; a
/// ratio and a time are written identically.
static RE_RATIO: Lazy<Regex> = Lazy::new(|| Regex::new(r"\b(\d{1,3})\s*:\s*(\d{1,3})\b").unwrap());

fn read_unit(name: &str, exp: Option<&str>, w: &UnitWords) -> Option<String> {
    let base = (w.lookup)(&name.to_lowercase())?;
    Some(match exp {
        Some("2") | Some("²") => (w.square)(base),
        Some("3") | Some("³") => (w.cubic)(base),
        _ => base.to_string(),
    })
}

/// Expand units, prime marks and ratios in `text`.
pub fn expand(text: &str, w: &UnitWords) -> String {
    // primes first: they carry a ° that the degree pass would otherwise claim
    let out = RE_DEG_MIN_SEC.replace_all(text, |c: &Captures| match c.get(3) {
        Some(sec) => format!(
            " {}° {} {} {} {} ",
            &c[1], &c[2], w.arcminute, sec.as_str(), w.arcsecond
        ),
        None => format!(" {}° {} {} ", &c[1], &c[2], w.arcminute),
    });
    let out = RE_FEET_INCH.replace_all(&out, |c: &Captures| {
        format!(" {} {} {} {} ", &c[1], w.feet, &c[2], w.inches)
    });
    let out = RE_RATIO.replace_all(&out, |c: &Captures| {
        format!(" {} {} {} ", &c[1], w.ratio, &c[2])
    });
    RE_UNIT
        .replace_all(&out, |c: &Captures| {
            let first = match read_unit(&c[2], c.get(3).map(|m| m.as_str()), w) {
                Some(s) => s,
                // Not a unit this language claims. Leave the whole match
                // alone: a model number or a plate must not be reshaped by a
                // pass that did not recognise it.
                None => return c[0].to_string(),
            };
            match c.get(4) {
                Some(second) => {
                    match read_unit(second.as_str(), c.get(5).map(|m| m.as_str()), w) {
                        Some(s) => format!("{} {} {} {} ", &c[1], first, w.per, s),
                        None => c[0].to_string(),
                    }
                }
                None => format!("{} {} ", &c[1], first),
            }
        })
        .into_owned()
}

/// Does a prime mark carrying a measurement survive [`expand`]?
///
/// `'` and `"` are ordinarily quotation marks, and dropping them is right.
/// Next to a digit they are minutes and feet, and dropping them loses the
/// measurement — the same split the hyphen needed. Deciding from the
/// character alone is what let `5'6"` read as "five six", so this asks the
/// pass itself rather than assuming either way.
pub fn unhandled_prime(text: &str, w: &UnitWords) -> Option<char> {
    let out = expand(text, w);
    let chars: Vec<char> = out.chars().collect();
    chars.iter().enumerate().find_map(|(i, c)| {
        let is_prime = matches!(c, '\'' | '"' | '\u{2032}' | '\u{2033}');
        let after_digit = i > 0 && chars[i - 1].is_ascii_digit();
        (is_prime && after_digit).then_some(*c)
    })
}

/// Characters this module reads, for the silent-deletion audits.
pub fn handled_chars() -> &'static str {
    "°'\"\u{2032}\u{2033}"
}
