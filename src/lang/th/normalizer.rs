//! Thai text normalization, run before segmentation and G2P.
//!
//! # Architecture: a staged pipeline
//!
//! [`normalize`] applies a fixed sequence of stages. As in the Vietnamese
//! module the order is a **contract**, not a preference: each stage assumes
//! earlier ones have already resolved a class of ambiguity, and reordering
//! silently changes readings.
//!
//! | # | Stage | Does | Why it sits here |
//! |---|-------|------|------------------|
//! | 1 | `spelling` | Fold Unicode quirks, Thai digits ๐-๙ -> ASCII | Everything downstream matches on canonical text and ASCII digits |
//! | 2 | `spans` | Emails and URLs, read whole | FIRST after folding: every later stage has a claim on the punctuation inside them |
//! | 3 | `abbreviations` | Table-driven expansion (รธน., ม.ค., ส.ส.) | Before dates and numbers: these contain periods and digits that later stages would claim |
//! | 4 | `datetime` | Dates (6/1/2560) and clock times | After abbreviations supplied the month names; before generic numbers |
//! | 5 | `phones` | Phone numbers, read figure by figure | Before units and numbers, which would read them as cardinals and drop the leading 0 |
//! | 6 | `units` | Currency, percentage, temperature | Before generic numbers, so the quantity and its unit are read together |
//! | 7 | `math` | Minus, ranges, powers, superscripts, fractions | Before the generic number pass, which would consume their digits |
//! | 8 | `numbers` | Decimals, thousands separators, cardinals | Once every specialised numeric form is consumed |
//! | 9 | `symbols` | Remaining mathematical / typographic symbols | Anything the passes above did not claim |
//! | 10 | `residual` | Strip leftovers, collapse whitespace | Must be last |
//!
//! `ๆ` (mai yamok) is deliberately NOT handled here. It repeats the preceding
//! **word**, and in a script without spaces "the preceding word" only exists
//! after segmentation — a text-level regex grabs the whole Thai run instead,
//! turning คนต่างๆ into "คน ต่าง คน ต่าง" rather than "คน ต่าง ต่าง".
//! [`super::Thai::phonemize`] applies it per token, where the boundary is known.
//!
//! # Invariant: nothing disappears in silence
//!
//! The final stage deletes characters it cannot read. That is the same defect
//! generator the Vietnamese pipeline documents: a symbol nobody declared
//! vanishes and the output still reads fluently, so the loss is inaudible.
//! [`audit_unmapped`] reports which characters of a given input would be
//! dropped, and a test asserts the list stays empty for the symbol inventory
//! we claim to support.

use once_cell::sync::Lazy;
use regex::{Captures, Regex};

use super::num2th::{digit_word, n2w, n2w_decimal, n2w_single};
use crate::core::numeric::{self, NumericWords};
use crate::core::roman::{self, RomanCues};
use crate::core::spans::{self, SpanWords};
use crate::core::units::{self, UnitWords};

/// Words that license a Roman numeral in Thai. Reign names dominate:
/// รัชกาลที่ ๙ is written with a Latin numeral as often as a Thai one.
const ROMAN: RomanCues = RomanCues {
    words: &["รัชกาลที่", "รัชกาล", "ที่", "เล่มที่", "บทที่", "ครั้งที่",
             "ศตวรรษที่", "ลำดับที่", "ฉบับที่", "ภาคที่", "สมัยที่"],
};

/// Thai words for the pieces of an email address or URL.
const SPANS: SpanWords = SpanWords {
    at: "แอท",
    dot: "จุด",
    slash: "ทับ",
    dash: "ขีด",
    underscore: "ขีดล่าง",
    colon: "ทวิภาค",
    spell: spell_latin,
};

/// Latin letters spelled with the names Thai speakers use: "https" reads
/// เอช-ที-ที-พี-เอส, matching how Vietnamese reads it "hát tê tê phê ét"
/// rather than leaving the scheme for the G2P stage to guess at.
fn spell_latin(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .filter_map(|c| TH_LATIN_LETTERS.get(&c).copied())
        .collect::<Vec<_>>()
        .join(" ")
}

/// Thai words for the shared numeric notations.
const NUMERIC: NumericWords = NumericWords {
    minus: "ลบ",
    to: "ถึง",
    power: "ยกกำลัง",
    squared: "กำลังสอง",
    cubed: "กำลังสาม",
    times: "คูณ",
    over: "ส่วน",
    score: "ต่อ",
};
/// Thai words for units, prime marks and ratios. Thai builds "square metre"
/// and "cubic metre" by prefixing, where Indonesian suffixes.
const UNITS: UnitWords = UnitWords {
    per: "ต่อ",
    square: |base| format!("ตาราง{}", base),
    cubic: |base| format!("ลูกบาศก์{}", base),
    lookup: |name| TH_LATIN_UNITS.get(name).copied(),
    feet: "ฟุต",
    inches: "นิ้ว",
    arcminute: "ลิปดา",
    arcsecond: "ฟิลิปดา",
    ratio: "ต่อ",
};
use super::resources::{
    thai_digit_to_ascii, TH_ABBREV, TH_LATIN_LETTERS, TH_LATIN_UNITS, TH_SYMBOLS, TH_UNITS,
};
use crate::core::abbrev::Reading;
use super::segment::normalize_spelling;

// ── Stage 1: spelling ───────────────────────────────────────────────────────

/// Fold spelling quirks and convert Thai digits to ASCII.
///
/// Thai digits are a *numeral system*, not decoration: ๒๕๖๐ is 2560. Mapping
/// them here means every later numeric pattern only has to know ASCII.
fn stage_spelling(text: &str) -> String {
    let folded = normalize_spelling(text);
    folded
        .chars()
        .map(|c| thai_digit_to_ascii(c).unwrap_or(c))
        .collect()
}

// ── Stage 1b: protected spans ───────────────────────────────────────────────

/// Email addresses and URLs, read before anything else can voice the
/// punctuation inside them. Without this `https://www.google.com` came out as
/// "https, ทับ ทับ www.google.com" — separators spoken, domain unread.
fn stage_spans(text: &str) -> String {
    spans::expand(text, &SPANS)
}

// ── Stage 1c: weekdays ──────────────────────────────────────────────────────

/// Weekday abbreviations, licensed by the วัน that precedes them.
///
/// Every one of these letters abbreviates something else as well — ศ. is
/// ศาสตราจารย์ (professor), ส. is สมาชิก, อ. is อาจารย์ or อำเภอ, จ. is
/// จังหวัด — and the abbreviation table gets there first, so `วันศ.` was
/// being read as "วัน ศาสตราจารย์". Requiring the วัน cue is what makes the
/// weekday reading safe, the same policy Vietnamese uses for T2–T7.
///
/// Longest alternatives first: อา and พฤ must win over อ and พ.
static RE_WEEKDAY: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"วัน\s*(อา|พฤ|จ|อ|พ|ศ|ส)\.").unwrap());

fn stage_weekdays(text: &str) -> String {
    RE_WEEKDAY
        .replace_all(text, |c: &Captures| {
            let day = match &c[1] {
                "จ" => "จันทร์",
                "อ" => "อังคาร",
                "พ" => "พุธ",
                "พฤ" => "พฤหัสบดี",
                "ศ" => "ศุกร์",
                "ส" => "เสาร์",
                _ => "อาทิตย์",
            };
            format!("วัน{}", day)
        })
        .into_owned()
}

// ── Stage 2: abbreviations ──────────────────────────────────────────────────

static RE_ABBREV: Lazy<Regex> = Lazy::new(|| {
    let mut keys: Vec<&str> = TH_ABBREV.replacement_keys().collect();
    // longest first: ตร.กม. must win over ตร. and กม.
    keys.sort_by_key(|k| std::cmp::Reverse(k.len()));
    Regex::new(&keys.iter().map(|k| regex::escape(k)).collect::<Vec<_>>().join("|")).unwrap()
});

/// Expand table entries. Runs before the date and number stages because Thai
/// abbreviations embed the very characters those stages match on: periods
/// (ม.ค.) and, through units, digits.
fn stage_abbreviations(text: &str) -> String {
    RE_ABBREV
        .replace_all(text, |c: &Captures| {
            let key = &c[0];
            match TH_ABBREV.get(key) {
                Some(Reading::Expand(v)) | Some(Reading::Fixed(v)) => format!(" {} ", v),
                _ => key.to_string(),
            }
        })
        .into_owned()
}

// ── Stage 3: datetime ───────────────────────────────────────────────────────

static RE_DATE_SLASH: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b(\d{1,2})/(\d{1,2})/(\d{4})\b").unwrap()
});
/// `14:30` — a colon is unambiguous, so no cue is needed.
static RE_TIME_COLON: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b(\d{1,2}):(\d{2})\s*(?:นาฬิกา|น\.)?").unwrap()
});
/// `14.30 น.` — the dotted form REQUIRES the นาฬิกา / น. cue. A period
/// between digits is a decimal point far more often than a clock separator:
/// without the cue "3.14 เมตร" was read as "3 นาฬิกา 14 นาที".
static RE_TIME_DOT: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b(\d{1,2})\.(\d{2})\s*(?:นาฬิกา|น\.)").unwrap()
});

const MONTH_BY_NUM: [&str; 12] = [
    "มกราคม", "กุมภาพันธ์", "มีนาคม", "เมษายน", "พฤษภาคม", "มิถุนายน",
    "กรกฎาคม", "สิงหาคม", "กันยายน", "ตุลาคม", "พฤศจิกายน", "ธันวาคม",
];

/// Dates and clock times.
///
/// A Thai date is read day-month-year with the year as a plain cardinal, and
/// Buddhist-era years (2560) are ordinary numbers — no conversion, since the
/// text says what it says.
fn stage_datetime(text: &str) -> String {
    let out = RE_DATE_SLASH.replace_all(text, |c: &Captures| {
        let d: u32 = c[1].parse().unwrap_or(0);
        let m: usize = c[2].parse().unwrap_or(0);
        if d == 0 || d > 31 || m == 0 || m > 12 {
            return c[0].to_string();
        }
        // The era marker is written out, not abbreviated: this stage runs
        // AFTER `stage_abbreviations`, so a "พ.ศ." emitted here would never
        // be expanded and would read as the letters พอ-สอ with two periods.
        format!(
            " วันที่ {} {} พุทธศักราช {} ",
            n2w(&c[1]),
            MONTH_BY_NUM[m - 1],
            n2w(&c[3])
        )
    });
    let out = RE_TIME_COLON.replace_all(&out, read_clock).into_owned();
    RE_TIME_DOT.replace_all(&out, read_clock).into_owned()
}

fn read_clock(c: &Captures) -> String {
    let h: u32 = c[1].parse().unwrap_or(99);
    let mi: u32 = c[2].parse().unwrap_or(99);
    if h > 23 || mi > 59 {
        return c[0].to_string();
    }
    if mi == 0 {
        format!(" {} นาฬิกา ", n2w(&c[1]))
    } else {
        format!(" {} นาฬิกา {} นาที ", n2w(&c[1]), n2w(&c[2]))
    }
}

// ── Stage 4: phones ─────────────────────────────────────────────────────────

/// Thai mobile and landline numbers, written 08x-xxx-xxxx or as one run.
static RE_PHONE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b0\d{1,2}[- ]?\d{3}[- ]?\d{3,4}\b").unwrap()
});

/// Phone numbers are identifiers, not quantities: every figure is spoken
/// separately, and the leading zero must survive — read as a cardinal "081"
/// becomes "eighty-one" and the 0 disappears entirely.
fn stage_phones(text: &str) -> String {
    RE_PHONE
        .replace_all(text, |c: &Captures| format!(" {} ", n2w_single(&c[0])))
        .into_owned()
}

// ── Stage 5: units ──────────────────────────────────────────────────────────

static RE_CURRENCY_PREFIX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"([฿$€£¥₩])\s*([\d,]+(?:\.\d+)?)").unwrap()
});
static RE_PERCENT: Lazy<Regex> = Lazy::new(|| Regex::new(r"([\d,]+(?:\.\d+)?)\s*%").unwrap());
static RE_DEGREE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"([\d,]+(?:\.\d+)?)\s*°\s*([CF])?").unwrap()
});

/// Currency, percentage and temperature, all of which put the unit **after**
/// the quantity when spoken even when written before it (฿500 -> ห้าร้อยบาท).
fn stage_units(text: &str) -> String {
    // Measurements first: the degree pass below would otherwise consume the °
    // of "13° 45'" and leave the arcminute stranded.
    let text = units::expand(text, &UNITS);
    let out = RE_CURRENCY_PREFIX.replace_all(&text, |c: &Captures| {
        let unit = TH_UNITS.get(&c[1]).copied().unwrap_or("");
        format!(" {} {} ", read_number(&c[2]), unit)
    });
    let out = RE_PERCENT.replace_all(&out, |c: &Captures| {
        format!(" {} เปอร์เซ็นต์ ", read_number(&c[1]))
    });
    RE_DEGREE
        .replace_all(&out, |c: &Captures| {
            let scale = match c.get(2).map(|m| m.as_str()) {
                Some("C") => "องศาเซลเซียส",
                Some("F") => "องศาฟาเรนไฮต์",
                _ => "องศา",
            };
            format!(" {} {} ", read_number(&c[1]), scale)
        })
        .into_owned()
}

/// Words that mark what follows as an identifier rather than a quantity.
const PLATE_CUES: &[&str] = &["ทะเบียน", "เลขที่", "รหัส", "หมายเลข", "เที่ยวบิน"];

/// Licence plates and codes, read figure by figure. Gated on a cue because
/// letters-then-digits is far too common a shape to claim on sight.
fn stage_identifiers(text: &str) -> String {
    spans::expand_identifiers(text, &SPANS, n2w_single, PLATE_CUES)
}

// ── Stage 5a: Roman numerals ────────────────────────────────────────────────

/// Only after a cue word: without one, "CD" and "MC" are ordinary letters.
/// Thai needs this more than Vietnamese does, since reign names are written
/// this way constantly (รัชกาลที่ IX).
fn stage_roman(text: &str) -> String {
    roman::expand(text, &ROMAN, n2w)
}

// ── Stage 5b: mathematical notation ─────────────────────────────────────────

/// Runs before the generic number pass, which would otherwise eat the digits
/// these patterns are built from. See [`crate::core::numeric`] for why each
/// of these was a silent deletion before.
fn stage_math(text: &str) -> String {
    numeric::expand(text, &NUMERIC, digit_word, n2w)
}

// ── Stage 6: numbers ────────────────────────────────────────────────────────

/// A comma only counts as a thousands separator when three digits follow it.
/// The looser `\d[\d,]*` swallowed the comma that ends a clause, so
/// "ISBN 3211812164, ISBN ..." looked like grouped digits and was read as the
/// cardinal three billion two hundred eleven million … instead of figure by
/// figure.
static RE_NUMBER: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\d+(?:,\d{3})*(?:\.\d+)?").unwrap()
});

/// Read one written number, honouring thousands separators and a decimal
/// point. Long digit runs with no separators (phone numbers, IDs) are read
/// figure by figure, the same policy the Vietnamese pipeline uses.
fn read_number(s: &str) -> String {
    let cleaned: String = s.chars().filter(|c| *c != ',').collect();
    match cleaned.split_once('.') {
        Some((int, frac)) => n2w_decimal(int, frac),
        None => {
            if cleaned.len() > 1 && cleaned.starts_with('0') {
                // a written leading zero marks an identifier, never a quantity
                n2w_single(&cleaned)
            } else if cleaned.len() > 6 && !s.contains(',') {
                n2w_single(&cleaned)
            } else {
                n2w(&cleaned)
            }
        }
    }
}

fn stage_numbers(text: &str) -> String {
    RE_NUMBER
        .replace_all(text, |c: &Captures| format!(" {} ", read_number(&c[0])))
        .into_owned()
}

// ── Stage 6b: lone Latin letters ────────────────────────────────────────────

/// A Latin letter standing on its own is a letter NAME, not a word.
///
/// Left alone it reached the English engine, which put English phonemes in
/// the middle of a Thai sentence: `วิตามิน C` ended in `sˈiː` — English
/// stress, no tone, a token outside the Thai inventory a Thai voice was
/// trained on — and `A4` came out as the bare vowel `ɐ`. These are the same
/// letter names the URL pass already speaks.
///
/// Only a letter that stands alone qualifies. Inside an English phrase it
/// belongs to that phrase: the A of "Grade A student" is read by the English
/// engine together with its neighbours, which is the code-switching this
/// pipeline exists to support.
fn stage_letters(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    for (i, c) in chars.iter().enumerate() {
        let lone = c.is_ascii_alphabetic()
            && !(i > 0 && chars[i - 1].is_ascii_alphabetic())
            && !chars.get(i + 1).is_some_and(char::is_ascii_alphabetic);
        match TH_LATIN_LETTERS
            .get(&c.to_ascii_lowercase())
            .filter(|_| lone && !english_word_beside(&chars, i))
        {
            Some(name) => {
                out.push(' ');
                out.push_str(name);
                out.push(' ');
            }
            None => out.push(*c),
        }
    }
    out
}

/// Is the nearest word on either side an English one?
fn english_word_beside(chars: &[char], i: usize) -> bool {
    let run = |start: isize, step: isize| {
        let mut j = start;
        let at = |j: isize| (j >= 0 && (j as usize) < chars.len()).then(|| chars[j as usize]);
        // one run of spaces separates a neighbouring word from this letter
        while at(j) == Some(' ') {
            j += step;
        }
        let mut n = 0;
        while at(j).is_some_and(|c| c.is_ascii_alphabetic()) {
            n += 1;
            j += step;
        }
        n
    };
    run(i as isize - 1, -1) >= 2 || run(i as isize + 1, 1) >= 2
}

// ── Stage 7: symbols ────────────────────────────────────────────────────────

/// Runs of two or more `=` `-` `*` `_` `#` are markup or a decorative rule,
/// never an operator: wiki headings such as `== อ้างอิง ==` were being read
/// as "equals equals References equals equals".
static RE_MARKUP_RUN: Lazy<Regex> = Lazy::new(|| Regex::new(r"[=\-*_#]{2,}").unwrap());

fn stage_symbols(text: &str) -> String {
    let text = RE_MARKUP_RUN.replace_all(text, " ");
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match TH_SYMBOLS.get(&c) {
            Some(w) => out.push_str(w),
            None => out.push(c),
        }
    }
    out
}

// ── Stage 8: residual ───────────────────────────────────────────────────────

static RE_SPACES: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s+").unwrap());
/// Marks that are a pause rather than a word. Mapped to the same forms the
/// Vietnamese pipeline uses, so both languages hand the same punctuation
/// vocabulary to a downstream TTS: `;` and `:` become a comma, and every
/// ellipsis becomes a single period. Dropping them, as this stage used to,
/// erases the prosodic boundary the writer put there.
static RE_PAUSE: Lazy<Regex> = Lazy::new(|| Regex::new(r"[;:]").unwrap());
static RE_ELLIPSIS: Lazy<Regex> = Lazy::new(|| Regex::new(r"[…‥․]+|\.{2,}").unwrap());
static RE_DROP: Lazy<Regex> = Lazy::new(|| {
    // everything not Thai, Latin, digits, whitespace or kept punctuation
    Regex::new(r"[^\u{0E01}-\u{0E4E}A-Za-z0-9\s,.!?]").unwrap()
});

fn stage_residual(text: &str) -> String {
    let out = RE_PAUSE.replace_all(text, ",");
    let out = RE_ELLIPSIS.replace_all(&out, ".");
    let out = RE_DROP.replace_all(&out, " ");
    RE_SPACES.replace_all(&out, " ").trim().to_string()
}

/// Normalize Thai text into a form the segmenter and G2P can read.
pub fn normalize(text: &str) -> String {
    let mut s = stage_spelling(text);
    s = stage_spans(&s);
    // before the abbreviation table, which claims ศ. อ. ส. for other readings
    s = stage_weekdays(&s);
    s = stage_abbreviations(&s);
    s = stage_datetime(&s);
    s = stage_phones(&s);
    s = stage_identifiers(&s);
    s = stage_roman(&s);
    s = stage_units(&s);
    s = stage_math(&s);
    s = stage_numbers(&s);
    // after numbers, so "A4" has already become "A" + สี่ and the letter stands alone
    s = stage_letters(&s);
    s = stage_symbols(&s);
    stage_residual(&s)
}

// ── Silent-deletion audit ───────────────────────────────────────────────────

/// Characters intentionally removed: punctuation and formatting whose absence
/// changes nothing a listener could hear.
///
/// Hyphens are NOT on this list. A hyphen next to a digit is a minus sign, a
/// range or a score, and blanket-dropping the character is precisely what let
/// "-5 องศา" lose its sign with the audit reporting nothing — see
/// [`unhandled_numeric_hyphen`].
const INTENTIONALLY_DROPPED: &str = "\"'“”‘’()[]{}«»_|\\*#…:;\u{200B}\u{FEFF}";

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

/// Report characters of `text` that would reach [`stage_residual`] and be
/// deleted **without becoming any word**.
///
/// Like the Vietnamese [`crate::lang::vi::audit`], this does not run the
/// pipeline; it inspects the character inventory so a missing table entry is
/// caught by a test rather than by a listener noticing a hole in a sentence.
/// The Thai block as [`stage_residual`] defines it — the same range its
/// keep-list uses, so the audit and the stage cannot drift apart.
fn is_thai(c: char) -> bool {
    ('\u{0E01}'..='\u{0E4E}').contains(&c)
}

pub fn audit_unmapped(text: &str) -> Vec<char> {
    let mut out = Vec::new();
    if unhandled_numeric_hyphen(text) {
        out.push('-');
    }
    // A prime after a digit is a measurement, not a quotation mark, so the
    // blanket entry in INTENTIONALLY_DROPPED below must not cover it.
    if let Some(c) = units::unhandled_prime(text, &UNITS) {
        out.push(c);
    }
    for c in text.chars() {
        // Hyphens are settled by digit_adjacent_hyphen above: a numeric one
        // is already reported, an ordinary one is genuinely droppable.
        if matches!(c, '-' | '\u{2013}' | '\u{2014}') {
            continue;
        }
        // Every character in the Thai block survives stage_residual, so none
        // of them can be a silent deletion. `is_alphanumeric` is not enough to
        // say so: the tone marks and thanthakhat are combining marks (Unicode
        // Mn), not letters, so the audit reported ่ ้ ๊ ๋ ็ ์ ๎ as dropped —
        // on text that keeps them. Thai marks tone on nearly every word, so
        // that fired on essentially every real sentence and buried the
        // findings the audit exists to surface.
        if is_thai(c)
            || crate::core::numeric::handled_chars().contains(c)
            || crate::core::spans::handled_chars().contains(c)
            || crate::core::units::handled_chars().contains(c)
            // ASCII only, matching stage_residual's keep-list. `is_alphanumeric`
            // accepts ā ṁ Ω and CJK — every one of which the stage deletes —
            // so the audit passed text the pipeline was busy dropping. Fixing
            // the combining-mark false positives above without this left the
            // guard blind in the direction that actually hides losses.
            || c.is_ascii_alphanumeric()
            || c.is_whitespace()
            || matches!(c, ',' | '.' | '!' | '?')
        {
            continue;
        }
        if TH_SYMBOLS.contains_key(&c) {
            continue;
        }
        // single-character unit and currency keys
        if TH_UNITS.contains_key(c.to_string().as_str()) {
            continue;
        }
        if c == 'ๆ' || c == 'ฯ' || thai_digit_to_ascii(c).is_some() {
            continue;
        }
        if INTENTIONALLY_DROPPED.contains(c) {
            continue;
        }
        if !out.contains(&c) {
            out.push(c);
        }
    }
    out
}
