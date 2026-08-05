//! Indonesian text normalization.
//!
//! # Architecture: a staged pipeline
//!
//! Same contract as the Vietnamese and Thai modules — the order encodes what
//! each stage assumes the earlier ones have already resolved.
//!
//! | # | Stage | Does | Why it sits here |
//! |---|-------|------|------------------|
//! | 1 | `spans` | Emails and URLs, read whole | FIRST: every later stage has a claim on the punctuation inside them |
//! | 2 | `abbreviations` | yg, dgn, tdk, DPR, Rp | Before numbers and dates: the money and date forms embed digits these would claim |
//! | 3 | `datetime` | 17/8/1945, 14:30 | After abbreviations supplied month names; before generic numbers |
//! | 4 | `money` | Rp1.250.000 | Before numbers: Indonesian groups thousands with a PERIOD, so the generic pass would read it as a decimal |
//! | 5 | `math` | Minus, ranges, powers, superscripts, fractions | Before the generic number pass, which would consume their digits |
//! | 6 | `numbers` | decimals, thousands, cardinals | Once the specialised forms are consumed |
//! | 7 | `symbols` | %, °, mathematical signs | Anything left |
//! | 8 | `residual` | punctuation, whitespace | Must be last |
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
use unicode_normalization::UnicodeNormalization;

use super::num2id::{digit_word, n2w, n2w_decimal, n2w_single};
use crate::core::numeric::{self, NumericWords};
use crate::core::roman::{self, RomanCues};
use crate::core::spans::{self, SpanWords};
use crate::core::units::{self, UnitWords};

/// Indonesian words for units, prime marks and ratios. Indonesian suffixes
/// where Thai prefixes: meter persegi, not "persegi meter".
const UNITS: UnitWords = UnitWords {
    per: "per",
    square: |base| format!("{} persegi", base),
    cubic: |base| format!("{} kubik", base),
    lookup: |name| ID_UNITS.get(name).copied(),
    feet: "kaki",
    inches: "inci",
    arcminute: "menit",
    arcsecond: "detik",
    ratio: "banding",
};

/// Words that license a Roman numeral in Indonesian.
const ROMAN: RomanCues = RomanCues {
    words: &["perang dunia", "abad", "abad ke-", "bab", "jilid", "ke-",
             "kelas", "periode", "seri", "bagian", "pasal"],
};

/// Indonesian words for the pieces of an email address or URL.
const SPANS: SpanWords = SpanWords {
    at: "at",
    dot: "titik",
    slash: "garis miring",
    dash: "strip",
    underscore: "garis bawah",
    colon: "titik dua",
    spell: spell_latin,
};

/// Latin letters spelled with Indonesian letter names: "https" reads
/// ha-te-te-pe-es.
fn spell_latin(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .filter_map(|c| ID_LETTER_NAMES.get(&c).copied())
        .collect::<Vec<_>>()
        .join(" ")
}

/// Indonesian words for the shared numeric notations.
const NUMERIC: NumericWords = NumericWords {
    minus: "minus",
    to: "sampai",
    power: "pangkat",
    squared: "kuadrat",
    cubed: "pangkat tiga",
    times: "kali",
    over: "per",
    score: "lawan",
};
use super::resources::{ID_ABBREV, ID_LETTER_NAMES, ID_MONTHS, ID_SYMBOLS, ID_UNITS};
use crate::core::abbrev::Reading;

// ── Stage 0: fold Latin diacritics ──────────────────────────────────────────

/// Fold a diacritic onto its base letter: à -> a, ā -> a, ṁ -> m.
///
/// Indonesian is written in plain ASCII, so the residual stage drops
/// everything else — which quietly ate the letter instead of the accent.
/// `ārati` became `rati`, losing its first sound; `voilà` became `voil`.
/// Decomposing and dropping only the combining mark keeps the word.
///
/// This cannot rescue a letter from another script: the `г` in `pasaгan` is
/// Cyrillic ge, a homoglyph typo in the corpus, and does not decompose to
/// anything Latin. It is still dropped — but [`audit_unmapped`] now reports
/// it, which is the difference between a known gap and a silent one.
fn stage_fold(text: &str) -> String {
    text.nfd()
        .filter(|c| !matches!(c, '\u{0300}'..='\u{036F}'))
        .collect::<String>()
        .nfc()
        .collect()
}

// ── Stage 0: protected spans ────────────────────────────────────────────────

/// Email addresses and URLs, read before anything else can voice the
/// punctuation inside them.
fn stage_spans(text: &str) -> String {
    spans::expand(text, &SPANS)
}

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

// ── Stage 1b: weekdays ──────────────────────────────────────────────────────

/// Weekday abbreviations, licensed by the `hari` in front of them.
///
/// Every one of these is also an ordinary word or a name — Sen, Sel, Min,
/// Sab — so claiming them on sight would misread far more than it fixed. The
/// cue is what makes the reading safe, and it comes after the abbreviation
/// stage so that `hr Sen` has already become `hari Sen`.
///
/// The word boundary is what keeps the full spellings out: `hari Senin` does
/// not match `sen`.
static RE_WEEKDAY: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\b(hari)\s+(sen|sel|rab|kam|jum|sab|min)\b\.?").unwrap()
});

fn stage_weekdays(text: &str) -> String {
    RE_WEEKDAY
        .replace_all(text, |c: &Captures| {
            let day = match c[2].to_lowercase().as_str() {
                "sen" => "Senin",
                "sel" => "Selasa",
                "rab" => "Rabu",
                "kam" => "Kamis",
                "jum" => "Jumat",
                "sab" => "Sabtu",
                _ => "Minggu",
            };
            format!("{} {}", &c[1], day)
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

/// Words that mark what follows as an identifier rather than a quantity.
const PLATE_CUES: &[&str] = &["plat", "nomor polisi", "nopol", "nomor", "kode", "penerbangan"];

/// Licence plates and codes, read figure by figure, gated on a cue.
fn stage_identifiers(text: &str) -> String {
    spans::expand_identifiers(text, &SPANS, n2w_single, PLATE_CUES)
}

// ── Stage 3c: units ─────────────────────────────────────────────────────────

/// Units, prime marks and ratios. After the clock-time pass, which has the
/// prior claim on `14:30`; before the number pass, which would consume the
/// digit this stage matches on.
fn stage_units(text: &str) -> String {
    units::expand(text, &UNITS)
}

// ── Stage 3d: ordinals ──────────────────────────────────────────────────────

/// `ke-3`, and a `ke-` left stranded by the Roman pass ("abad ke- dua puluh").
static RE_ORDINAL: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)\bke-\s*(\d+)?").unwrap());

/// Indonesian writes every ordinal with a hyphen — ke-3, abad ke-20 — and
/// speaks it as one word: ketiga. Left alone the hyphen reached the G2P stage
/// as a literal character, giving "ke- tiga".
fn stage_ordinals(text: &str) -> String {
    RE_ORDINAL
        .replace_all(text, |c: &Captures| match c.get(1) {
            Some(n) => format!(" ke{} ", n2w(n.as_str())),
            None => " ke ".to_string(),
        })
        .into_owned()
}

// ── Stage 2b: Roman numerals ────────────────────────────────────────────────

/// Only after a cue word: without one, "CD" and "MC" are ordinary letters.
fn stage_roman(text: &str) -> String {
    roman::expand(text, &ROMAN, n2w)
}

// ── Stage 3b: mathematical notation ─────────────────────────────────────────

/// Runs before the generic number pass, which would otherwise eat the digits
/// these patterns are built from. Before this stage "10^6" read as "sepuluh
/// enam" — six orders of magnitude lost with no audible cue.
fn stage_math(text: &str) -> String {
    numeric::expand(text, &NUMERIC, digit_word, n2w)
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
/// An apostrophe between letters is older orthography — do'a, Jum'at — whose
/// modern spelling simply closes up. Joining is therefore the right reading;
/// replacing it with a space, as the general drop below would, splits one
/// word into two.
static RE_APOSTROPHE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)([a-z])['’]([a-z])").unwrap());
static RE_DROP: Lazy<Regex> = Lazy::new(|| Regex::new(r"[^A-Za-z0-9\s,.!?-]").unwrap());
static RE_SPACES: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s+").unwrap());

/// Keep the hyphen that means something and drop the one that does not.
///
/// Between letters it is reduplication — orang-orang, anak-anak — and the
/// word is wrong without it. Anywhere else it is not a word at all, and
/// leaving it in place is how `COVID-19` reached the G2P stage as
/// "COVID- sembilan belas" with a literal dash in the middle.
fn strip_non_word_hyphens(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    chars
        .iter()
        .enumerate()
        .map(|(i, c)| {
            if !matches!(c, '-' | '–' | '—') {
                return *c;
            }
            let joins_words = i > 0
                && chars[i - 1].is_alphabetic()
                && chars.get(i + 1).is_some_and(|n| n.is_alphabetic());
            if joins_words {
                *c
            } else {
                ' '
            }
        })
        .collect()
}

fn stage_residual(text: &str) -> String {
    let out = RE_PAUSE.replace_all(text, ",");
    let out = RE_ELLIPSIS.replace_all(&out, ".");
    let out = RE_APOSTROPHE.replace_all(&out, "$1$2");
    let out = strip_non_word_hyphens(&out);
    let out = RE_DROP.replace_all(&out, " ");
    RE_SPACES.replace_all(&out, " ").trim().to_string()
}

/// Normalize Indonesian text into a form the G2P stage can read.
pub fn normalize(text: &str) -> String {
    let mut s = stage_fold(text);
    s = stage_spans(&s);
    s = stage_abbreviations(&s);
    // after abbreviations, so "hr Sen" has become "hari Sen" and the cue is there
    s = stage_weekdays(&s);
    s = stage_datetime(&s);
    s = stage_identifiers(&s);
    s = stage_roman(&s);
    // after the Roman pass, so "abad ke-XX" has become "abad ke- dua puluh"
    // and this stage can clear the stranded hyphen along with "ke-3"
    s = stage_ordinals(&s);
    s = stage_money(&s);
    s = stage_units(&s);
    s = stage_math(&s);
    s = stage_numbers(&s);
    s = stage_symbols(&s);
    stage_residual(&s)
}

/// Punctuation and formatting whose removal changes nothing audible.
///
/// A hyphen between letters is kept by the pipeline anyway (Indonesian
/// reduplicates with one: orang-orang), and a hyphen next to a DIGIT is a
/// minus sign, a range or a score — see [`unhandled_numeric_hyphen`]. Neither
/// case is a silent drop, which is why `-` is absent from this list.
const INTENTIONALLY_DROPPED: &str = "\"“”‘’()[]{}«»_|\\*#…:;\u{200B}\u{FEFF}";

/// Does a hyphen carrying numeric meaning survive [`stage_math`]?
///
/// A hyphen next to a digit is a minus sign, a range or a score, so dropping
/// it changes what the sentence says. Rather than judge from the character
/// alone — which either hides real losses or cries wolf on every minus the
/// pipeline already reads — this runs the numeric pass and asks whether one
/// is still there afterwards, so the check stays correct as that pass grows.
fn unhandled_numeric_hyphen(text: &str) -> bool {
    let text = stage_math(text);
    let chars: Vec<char> = text.chars().collect();
    chars.iter().enumerate().any(|(i, c)| {
        matches!(c, '-' | '\u{2013}' | '\u{2014}')
            && (chars.get(i + 1).is_some_and(char::is_ascii_digit)
                || (i > 0 && chars[i - 1].is_ascii_digit()))
    })
}

/// Characters of `text` that would be deleted without becoming a word.
pub fn audit_unmapped(text: &str) -> Vec<char> {
    let mut out = Vec::new();
    if unhandled_numeric_hyphen(text) {
        out.push('-');
    }
    // A prime after a digit is a measurement, not a quotation mark.
    if let Some(c) = units::unhandled_prime(text, &UNITS) {
        out.push(c);
    }
    for c in text.chars() {
        if crate::core::numeric::handled_chars().contains(c)
            || crate::core::spans::handled_chars().contains(c)
            || crate::core::units::handled_chars().contains(c)
            // must match stage_residual's RE_DROP, which keeps ASCII only.
            // `is_alphanumeric` accepts ā à ṁ and the Cyrillic г, so the
            // audit passed words the pipeline was busy deleting.
            || c.is_ascii_alphanumeric()
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
