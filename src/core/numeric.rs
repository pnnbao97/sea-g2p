//! Numeric and mathematical notation, shared by every language.
//!
//! The *patterns* are language-independent — a minus sign is a minus sign in
//! any script — while the words are not. This module holds one implementation
//! of each pattern and takes the words from a [`NumericWords`] table, so a
//! fix to a pattern reaches every language at once.
//!
//! # Why this exists
//!
//! Without it these notations were **deleted in silence**, the defect class
//! the audit modules were written to prevent:
//!
//!   - `-5 derajat` read as "five degrees" — the sign vanished and the
//!     temperature changed by ten;
//!   - `10^6` read as "ten six" — six orders of magnitude, with no audible
//!     cue that anything was lost;
//!   - `5 m²` read as "five m" — the area became a length.
//!
//! # Ordering
//!
//! Callers must run [`expand`] **before** their generic number pass, which
//! would otherwise consume the digits these patterns are built from, and
//! after any pass that claims digit-bearing spans of its own (dates, phone
//! numbers, currency).

use once_cell::sync::Lazy;
use regex::{Captures, Regex};

/// The words one language uses for the notations below.
pub struct NumericWords {
    /// Negative sign: "âm", "ลบ", "minus".
    pub minus: &'static str,
    /// Range: "đến", "ถึง", "sampai".
    pub to: &'static str,
    /// Exponent: "mũ", "ยกกำลัง", "pangkat".
    pub power: &'static str,
    /// Idiomatic square and cube, where the language has them; otherwise
    /// leave these equal to `power` + the digit.
    pub squared: &'static str,
    pub cubed: &'static str,
    /// Multiplication: "nhân", "คูณ", "kali".
    pub times: &'static str,
    /// Division / fraction: "trên", "ส่วน", "per".
    pub over: &'static str,
    /// Sports score separator: "-" read as "thắng"/"ต่อ"/"lawan".
    pub score: &'static str,
}

/// Digit -> word, supplied by the language's number module.
pub type DigitFn = fn(char) -> &'static str;
/// Digit string -> cardinal words.
pub type CardinalFn = fn(&str) -> String;

// A minus sign is only a minus when a digit follows AND nothing that could be
// the left operand precedes it: "-5" is negative five, "3-1" is a score and
// "10-20" a range, both handled below.
static RE_MINUS: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?:^|[\s(\[])-(\d)").unwrap());
static RE_RANGE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\b(\d+)\s*[-–—]\s*(\d+)\b").unwrap());
/// Subtraction between non-numeric operands: `b² - 4ac`. Spaces on BOTH
/// sides are required, which separates it from a compound hyphen — Indonesian
/// reduplicates with one, orang-orang — and from the unary sign above.
/// Without this the operator simply vanished.
static RE_SUBTRACT: Lazy<Regex> = Lazy::new(|| Regex::new(r"(\S)\s+[-–—]\s+(\S)").unwrap());
/// An equals sign anywhere means the dashes in the line are arithmetic, not
/// ranges: "5 - 3 = 2" is a subtraction, "10-20 ปี" is a span of years. Both
/// are digit-dash-digit and nothing inside the pattern tells them apart.
static RE_HAS_EQUALS: Lazy<Regex> = Lazy::new(|| Regex::new(r"=").unwrap());
static RE_POWER: Lazy<Regex> = Lazy::new(|| Regex::new(r"\b(\d+)\s*\^\s*(-?\d+)").unwrap());
/// E-notation: `1.5e10`, `6,02e23`, `1e-9`. The decimal mark is either a
/// period or a comma because Indonesian writes 1,5 where Thai writes 1.5.
/// Left alone the exponent read as a bare cardinal — "one point five e ten" —
/// which is off by ten orders of magnitude with nothing audible to signal it.
static RE_SCI: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\b(\d+(?:[.,]\d+)?)[eE]([-+]?\d+)\b").unwrap());
static RE_TIMES: Lazy<Regex> = Lazy::new(|| Regex::new(r"\b(\d+)\s*[x×*]\s*(\d+)\b").unwrap());
static RE_FRACTION: Lazy<Regex> = Lazy::new(|| Regex::new(r"\b(\d+)\s*/\s*(\d+)\b").unwrap());

/// Superscript digits, indexed by value. These are NOT one contiguous block
/// and NOT one byte width: ⁰ and ⁴-⁹ are U+2070/U+2074-2079 at three bytes,
/// while ¹²³ are the Latin-1 U+00B9/U+00B2/U+00B3 at two. Indexing this by
/// byte offset shifted every exponent by one and read m² as "to the power
/// one".
const SUPERSCRIPTS: [char; 10] = ['⁰', '¹', '²', '³', '⁴', '⁵', '⁶', '⁷', '⁸', '⁹'];
const SUBSCRIPTS: [char; 10] = ['₀', '₁', '₂', '₃', '₄', '₅', '₆', '₇', '₈', '₉'];

fn script_value(table: &[char; 10], c: char) -> Option<char> {
    table
        .iter()
        .position(|x| *x == c)
        .map(|v| char::from_digit(v as u32, 10).expect("0-9"))
}

/// Expand every numeric notation in `text`.
///
/// `range_is_score` decides how `3-1` reads: as a range ("three to one") or
/// as a score. Both are digit-dash-digit and only context tells them apart,
/// so the caller picks the default its register needs; a range between large
/// numbers is always read as a range regardless.
pub fn expand(
    text: &str,
    w: &NumericWords,
    digit: DigitFn,
    cardinal: CardinalFn,
) -> String {
    // superscripts and subscripts first: they are single characters that the
    // digit patterns below cannot see
    let mut out = expand_scripts(text, w, digit);

    // e-notation before the exponent patterns below, which would read its
    // digits as an ordinary product. The mantissa is left as digits for the
    // number stage; only the operator and the exponent become words here.
    out = RE_SCI
        .replace_all(&out, |c: &Captures| {
            let exp = &c[2];
            let exp = exp.strip_prefix('+').unwrap_or(exp);
            match exp.strip_prefix('-') {
                Some(rest) => format!(
                    " {} {} {} {} {} {} ",
                    &c[1], w.times, cardinal("10"), w.power, w.minus, cardinal(rest)
                ),
                None => format!(
                    " {} {} {} {} {} ",
                    &c[1], w.times, cardinal("10"), w.power, cardinal(exp)
                ),
            }
        })
        .into_owned();

    out = RE_POWER
        .replace_all(&out, |c: &Captures| {
            let exp = &c[2];
            match exp.strip_prefix('-') {
                Some(rest) => format!(" {} {} {} {} ", cardinal(&c[1]), w.power, w.minus, cardinal(rest)),
                None => format!(" {} {} {} ", cardinal(&c[1]), w.power, cardinal(exp)),
            }
        })
        .into_owned();

    out = RE_TIMES
        .replace_all(&out, |c: &Captures| {
            format!(" {} {} {} ", cardinal(&c[1]), w.times, cardinal(&c[2]))
        })
        .into_owned();

    out = RE_FRACTION
        .replace_all(&out, |c: &Captures| {
            format!(" {} {} {} ", cardinal(&c[1]), w.over, cardinal(&c[2]))
        })
        .into_owned();

    // A dash between two numbers is a range in prose and a subtraction in an
    // equation; nothing inside the pattern distinguishes them, so the
    // presence of an equals sign decides.
    let joiner = if RE_HAS_EQUALS.is_match(&out) { w.minus } else { w.to };
    out = RE_RANGE
        .replace_all(&out, |c: &Captures| {
            format!(" {} {} {} ", cardinal(&c[1]), joiner, cardinal(&c[2]))
        })
        .into_owned();

    out = RE_MINUS
        .replace_all(&out, |c: &Captures| format!(" {} {}", w.minus, &c[1]))
        .into_owned();

    RE_SUBTRACT
        .replace_all(&out, |c: &Captures| format!("{} {} {}", &c[1], w.minus, &c[2]))
        .into_owned()
}

/// `m²` -> "m squared", `10⁻³` -> "ten to the minus three", `H₂O` -> "H two O".
fn expand_scripts(text: &str, w: &NumericWords, digit: DigitFn) -> String {
    let mut out = String::with_capacity(text.len());
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == '⁻' || SUPERSCRIPTS.contains(&c) {
            let negative = c == '⁻';
            if negative {
                i += 1;
            }
            let mut digits = String::new();
            while i < chars.len() {
                match script_value(&SUPERSCRIPTS, chars[i]) {
                    Some(d) => {
                        digits.push(d);
                        i += 1;
                    }
                    None => break,
                }
            }
            if digits.is_empty() {
                out.push(' ');
                continue;
            }
            out.push(' ');
            if negative {
                out.push_str(&format!("{} {} ", w.power, w.minus));
                for d in digits.chars() {
                    out.push_str(digit(d));
                    out.push(' ');
                }
            } else if digits == "2" {
                out.push_str(w.squared);
                out.push(' ');
            } else if digits == "3" {
                out.push_str(w.cubed);
                out.push(' ');
            } else {
                out.push_str(w.power);
                out.push(' ');
                for d in digits.chars() {
                    out.push_str(digit(d));
                    out.push(' ');
                }
            }
            continue;
        }
        if let Some(d) = script_value(&SUBSCRIPTS, c) {
            out.push(' ');
            out.push_str(digit(d));
            out.push(' ');
            i += 1;
            continue;
        }
        out.push(c);
        i += 1;
    }
    out
}

/// Characters this module reads. An audit that does not know about these
/// reports them as unmapped; one that treats them as "intentionally dropped"
/// hides exactly the deletions this module exists to prevent.
pub fn handled_chars() -> &'static str {
    "⁰¹²³⁴⁵⁶⁷⁸⁹⁻₀₁₂₃₄₅₆₇₈₉^×*/"
}
