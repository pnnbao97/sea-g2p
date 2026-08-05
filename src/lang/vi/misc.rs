//! Symbols, acronyms, Roman numerals, licence plates and letter names.
//!
//! This module backs several pipeline stages; `normalize_others` is stage 15,
//! the last chance for anything still unread.
//!
//! # Where silent deletion happens
//!
//! `normalize_others` ends with `RE_CLEAN_OTHERS`, which strips every character
//! outside its whitelist. That is the safety net keeping stray bytes out of the
//! TTS tokenizer — and also the exact point where an undeclared symbol vanishes
//! without a sound. `∆`, `⁻` and `Σ` were all lost here. Before adding a symbol
//! anywhere in the codebase, give it a reading or declare it in
//! [`crate::lang::vi::audit`]; `tests/test_invariants.py` enforces this.
//!
//! # Context-dependent readings
//!
//! Several rules here need a lead word rather than a pattern, because the same
//! characters mean different things:
//!
//!   - **Roman numerals** only expand after a cue word ("thế kỷ XXI", "chương
//!     IV"). Without one, "CD" and "MC" would become numbers. Single letters
//!     (I/V/X) demand the cue too, and L/C/D/M never expand alone.
//!   - **Weekday abbreviations** (T2–T7, CN) need a time cue ("sáng T2"), so
//!     "Model T2" and "ga T3" survive.
//!   - **Licence plates** run before the clock pass, since "51H" also matches
//!     an hour, and before the multiplication pass, since "59X1" contains an X.

use fancy_regex::{Regex as FRegex, Captures as FCaps};
use regex::{Regex, Captures};
use once_cell::sync::Lazy;
use crate::lang::vi::num2vi::{n2w, n2w_single};
use crate::core::abbrev::Reading;
use crate::lang::vi::resources::{
    VI_LETTER_NAMES, DOMAIN_SUFFIX_MAP,
    ROMAN_NUMERALS, ROMAN_KEYWORDS, ABBRS, SYMBOLS_MAP, VI_ABBREV, MEASUREMENT_KEY_VI,
    CURRENCY_KEY, COMBINED_EXCEPTIONS, SUPERSCRIPTS_MAP, SUBSCRIPTS_MAP, ENGLISH_AMPERSAND
};
use crate::lang::vi::technical::normalize_slashes;

const VI_UPPER: &str = "ĐĂÂÊÔƠƯ";

// ─ Patterns requiring look-arounds ───────────────────────────────────────
static RE_ROMAN_NUMBER: Lazy<FRegex> = Lazy::new(|| {
    // UPPERCASE ONLY: real Roman numerals are always capitalised ("Chương IV",
    // "Edward II"). Lowercase is rejected because ordinary Vietnamese syllables
    // ("di", "vi", "li", "cd") have the same shape, and accepting them turns
    // "lần di chuyển" into "lần 501 chuyển".
    FRegex::new(r"\b(?=[IVXLCDM]{2,})(?:M{0,4}(?:CM|CD|D?C{0,3})(?:XC|XL|L?X{0,3})(?:IX|IV|V?I{0,3}))(?<=[IVXLCDM])\b").unwrap()
});
// Roman numerals opening a line as a SECTION NUMBER: "I. VỀ ĐỀ NGHỊ…",
// "II. Về …". These read as numbers ("một", "hai"), not letters, and include the
// single-character case ("I") that RE_ROMAN_NUMBER misses by requiring two
// characters plus a cue word. Conditions: start of line, followed by ".".
//
// Groups: (1) indentation, (2) the numeral, (3) period and spacing, (4) the
// uppercase run of the heading that follows, matched by lookahead so it is not
// consumed. Group 4 separates a section number from an abbreviated personal name
// ("C. Mác", "V. Nguyễn"): a single character counts as a number only when the
// heading itself is uppercase, i.e. at least two capitals.
static RE_ROMAN_LIST_MARKER: Lazy<FRegex> = Lazy::new(|| {
    FRegex::new(r"(?m)^([ \t]*)([IVXLCDM]+)(\.[ \t]+)(?=(\p{Lu}+))").unwrap()
});
// Real section numbers rarely pass XX. Anything larger is almost certainly an
// abbreviated name, since C=100, L=50, D=500 and M=1000 are all common initials.
// Excluding them stops "C. Mác" from being read as "một trăm".
const ROMAN_MARKER_MAX: i32 = 30;
// I/V/X only: a lone L, C, D or M is nearly always an initial, not a numeral.
static RE_ROMAN_SINGLE: Lazy<FRegex> = Lazy::new(|| {
    FRegex::new(r"\b[IVX]\b(?!['’])").unwrap()
});
// Drop the period after a title abbreviation when a proper name follows
// ("TS. Nguyễn" -> "TS Nguyễn"), so the dot is not mistaken for a sentence
// boundary and does not introduce a false pause.
static RE_TITLE_DOT: Lazy<FRegex> = Lazy::new(|| {
    FRegex::new(r"\b(TS|GS|BS|ThS|PGS|KS|ĐH)\.\s+(?=\p{Lu})").unwrap()
});
// "Q.1"/"P.7" and "Q.Bình Thạnh"/"P.Bến Nghé" -> "quận"/"phường". The proper
// name form requires an uppercase letter followed by a lowercase one, which
// keeps "P.S." and other abbreviations out.
static RE_DISTRICT_DOT: Lazy<FRegex> = Lazy::new(|| {
    FRegex::new(r"\bQ\.\s*(?=\d|\p{Lu}\p{Ll})").unwrap()
});
static RE_WARD_DOT: Lazy<FRegex> = Lazy::new(|| {
    FRegex::new(r"\bP\.\s*(?=\d|\p{Lu}\p{Ll})").unwrap()
});
// The lookahead excludes English contractions ("I'm", "I'll", "I'd"): a letter,
// an apostrophe and another letter must stay together for G2P to find them in
// the English dictionary, rather than being read as "i" plus "'m".
static RE_STANDALONE_LETTER: Lazy<FRegex> = Lazy::new(|| {
    FRegex::new(r"(?<![\''])\b([a-zA-Z])(?!['’]\w)\b(\.?)").unwrap()
});
pub static RE_ACRONYM: Lazy<FRegex> = Lazy::new(|| {
    FRegex::new(&format!(r"\b(?=[A-Z{}a-z{}0-9]*[A-Z{}])(?:[A-Z{}][a-z{}]?\d*){{2,}}\b", VI_UPPER, VI_UPPER, VI_UPPER, VI_UPPER, "đăâêôơư")).unwrap()
});
static RE_VERSION: Lazy<FRegex> = Lazy::new(|| {
    FRegex::new(r"(?<![-\u2013\u2014])\b(\d+(?:\.\d+){2,})\b").unwrap()
});
static RE_PRIME: Lazy<FRegex> = Lazy::new(|| {
    // Count the primes: f' -> "phẩy", y'' -> "phẩy phẩy" (second derivative).
    FRegex::new(r"(\b[a-zA-Z0-9])(['\u2019]+)(?!\w)").unwrap()
});
// Absolute value |x|, |x+1| -> "giá trị tuyệt đối của …". The content must have
// no whitespace at its edges, which keeps the rule away from the pipes of a
// markdown table ("| cột |").
static RE_ABS: Lazy<FRegex> = Lazy::new(|| {
    FRegex::new(r"\|(\S(?:[^|]*\S)?)\|").unwrap()
});
static RE_PRIME_DIGIT: Lazy<FRegex> = Lazy::new(|| {
    FRegex::new(r"(?<=\d)(['\u2019]+|[\x22\u201D])").unwrap()
});

// ─ Simple patterns (regex crate — Thompson NFA, fast) ──────────────────
static RE_LETTER: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?i)(chữ|chữ cái|kí tự|ký tự)\s+(['"]?)([a-z])(['"]?)\b"#).unwrap()
});
static RE_ALPHANUMERIC: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b(\d+)([a-zA-Z])\b").unwrap()
});
// English acronyms and brands joined by "&" (R&D, R & D, AT&T, S&P).
static RE_AMPERSAND_ACRONYM: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\b([a-z]{1,4})\s*&\s*([a-z]{1,4})\b").unwrap()
});
// Clothing size labels REQUIRE "size" or "cỡ" in front (size M/L/XL, cỡ M).
// With that cue, S/M/L/XL are labels read as letters rather than units, which
// is what stops them becoming "triệu" or "lít".
static RE_SIZE_LABEL: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\b(size|cỡ)\s+((?:xxxl|xxl|xl|xs|[sml])(?:\s*/\s*(?:xxxl|xxl|xl|xs|[sml]))*)\b").unwrap()
});

pub fn expand_size_labels(text: &str) -> String {
    RE_SIZE_LABEL.replace_all(text, |caps: &Captures| {
        let prefix = caps.get(1).unwrap().as_str();
        let spelled: Vec<String> = caps.get(2).unwrap().as_str()
            .split('/')
            .map(|s: &str| {
                let letters = s.trim().to_lowercase().chars()
                    .map(|c: char| c.to_string())
                    .collect::<Vec<String>>()
                    .join(" ");
                format!("__start_en__{}__end_en__", letters)
            })
            .collect();
        format!("{} {}", prefix, spelled.join(" "))
    }).into_owned()
}
static RE_LETTER_DIGIT: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b([a-zA-Z])(\d+)\b").unwrap()
});
static RE_BRACKETS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"[\(\[\{]\s*(.*?)\s*[\)\]\}]").unwrap()
});
static RE_STRIP_BRACKETS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"[\[\]\(\)\{\}]").unwrap()
});
static RE_TEMP_C_NEG: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)-(\d+(?:[.,]\d+)?)\s*°\s*c\b").unwrap()
});
static RE_TEMP_F_NEG: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)-(\d+(?:[.,]\d+)?)\s*°\s*f\b").unwrap()
});
static RE_TEMP_C: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(\d+(?:[.,]\d+)?)\s*°\s*c\b").unwrap()
});
static RE_TEMP_F: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(\d+(?:[.,]\d+)?)\s*°\s*f\b").unwrap()
});
static RE_DEGREE: Lazy<Regex> = Lazy::new(|| Regex::new(r"°").unwrap());
static RE_STANDARD_COLON: Lazy<FRegex> = Lazy::new(|| {
    // The lookbehind prevents a partial match inside "1.5:1"; the lookahead
    // blocks a continuing or decimal number but still ALLOWS sentence-final
    // punctuation, so "2:1." remains the ratio "hai trên một".
    FRegex::new(r"(?<![.,\d])\b(\d+):(\d+(?:\.\d+)?)\b(?!\d)(?![.,]\d)").unwrap()
});
// Ratios of three or more parts separated by ":" (1:2:3). These are not times —
// normalize_time has already rejected them — and are read joined by "trên".
static RE_RATIO_MULTI: Lazy<FRegex> = Lazy::new(|| {
    // Blocks a continuing number (:\d, \d) and decimals (.\d, ,\d) while
    // ALLOWING sentence-final punctuation, so "1:2:3." stays one ratio instead
    // of being cut into "một trên hai, ba".
    FRegex::new(r"(?<![.,\d:])\d+(?::\d+){2,}(?![\d:])(?![.,]\d)").unwrap()
});
static RE_CLEAN_OTHERS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"[^a-zA-Z0-9\sàáảãạăắằẳẵặâấầẩẫậèéẻẽẹêếềểễệìíỉĩịòóỏõọôốồổỗộơớờởỡợùúủũụưứừửữựỳýỷỹỵđÀÁẢÃẠĂẮẰẲẴẶÂẤẦẨẪẬÈÉẺẼẸÊẾỀỂỄỆÌÍỈĨỊÒÓỎÕỌÔỐỒỔỖỘƠỚỜỞỠỢÙÚỦŨỤƯỨỪỬỮỰỲÝỶỸỴĐ.,!?_\'\'-]").unwrap()
});
static RE_CLEAN_QUOTES: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"[“”„]"#).unwrap()
});
static RE_CLEAN_QUOTES_EDGES: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(^|[\s.,!?;:])[\u2018\u2019']+|[\u2018\u2019']+($|[\s.,!?;:])").unwrap()
});
static RE_COLON_SEMICOLON: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"[:;]").unwrap()
});
static RE_UNIT_POWERS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b([a-zA-Z]+)\^([-+]?\d+)\b").unwrap()
});
pub static RE_ACRONYMS_EXCEPTIONS: Lazy<Regex> = Lazy::new(|| {
    let mut keys: Vec<String> = COMBINED_EXCEPTIONS.keys().map(|k: &String| k.to_string()).collect();
    keys.sort_by_key(|b: &String| std::cmp::Reverse(b.len()));
    let pattern = keys.iter().map(|k: &String| format!(r"\b{}\b", regex::escape(k))).collect::<Vec<String>>().join("|");
    Regex::new(&pattern).unwrap()
});
pub static DOMAIN_SUFFIXES_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\.(com|vn|net|org|edu|gov|io|biz|info)\b").unwrap()
});
// Sentence boundaries for the ALL-CAPS heuristic: terminators .!? OR a newline.
//
// Newlines matter because headings are a boundary of their own: an uppercase
// heading usually has no final period and is separated only by a line break
// ("…CÔNG TRẠNG\n\nHuân chương…"). Without splitting on \n the heading merges
// with the ordinary paragraph below it, the run stops looking all-caps, and
// toneless Vietnamese words (LAO, KHEN) get read as English (<en>l a o</en>).
// See issue #177.
static RE_ACRONYMS_SPLIT: Lazy<regex::Regex> = Lazy::new(|| {
    regex::Regex::new(r"([.!?]+(?:\s+|$)|\n+)").unwrap()
});

/// Vietnamese vowels in lowercase, including every tone mark.
const VI_VOWELS: &str = "aáàảãạăắằẳẵặâấầẩẫậeéèẻẽẹêếềểễệiíìỉĩịoóòỏõọôốồổỗộơớờởỡợuúùủũụưứừửữựyýỳỷỹỵ";

fn is_vi_vowel(c: char) -> bool {
    VI_VOWELS.contains(c)
}

/// Is `s` a single valid Vietnamese syllable — optional onset, vowel nucleus,
/// optional coda?
///
/// Used to tell an uppercase Vietnamese WORD ("CHƯƠNG", "ĐƯỜNG", "PHƯỜNG") from
/// an acronym or formula made of letters: "ĐKVĐ" has no valid nucleus, so it is
/// not a syllable and stays spelled out.
/// Biased towards STRICT: return false when unsure. The consequence is spelling
/// the letters out, which is safer than mistakenly reading an acronym as a word.
fn is_vietnamese_syllable(s: &str) -> bool {
    let lower = s.to_lowercase();
    let chars: Vec<char> = lower.chars().collect();
    let n = chars.len();
    if n == 0 || chars.iter().any(|c: &char| !c.is_alphabetic()) {
        return false;
    }
    // Onset: try the longest first, and accept it only if a vowel follows.
    const ONSETS: [&str; 28] = [
        "ngh", "ng", "nh", "ch", "gh", "gi", "kh", "ph", "th", "tr", "qu",
        "b", "c", "d", "đ", "g", "h", "k", "l", "m", "n", "p", "q", "r", "s", "t", "v", "x",
    ];
    let mut i = 0usize;
    for cand in ONSETS.iter() {
        let cl = cand.chars().count();
        if i + cl < n {
            let prefix: String = chars[i..i + cl].iter().collect();
            if prefix == *cand && is_vi_vowel(chars[i + cl]) {
                i += cl;
                break;
            }
        }
    }
    // Nucleus: a run of consecutive vowels, at least one required.
    let v_start = i;
    while i < n && is_vi_vowel(chars[i]) {
        i += 1;
    }
    if i == v_start {
        return false;
    }
    // Coda: whatever remains must be empty or a valid final consonant cluster.
    let coda: String = chars[i..].iter().collect();
    matches!(
        coda.as_str(),
        "" | "c" | "ch" | "m" | "n" | "ng" | "nh" | "p" | "t"
    )
}

/// True when the LAST word of `preceding` — the text immediately before the
/// numeral — is in `ROMAN_KEYWORDS`. Punctuation attached to the word is ignored.
fn has_roman_context(preceding: &str) -> bool {
    let last = preceding
        .split(|c: char| c.is_whitespace())
        .rev()
        .find(|w: &&str| !w.is_empty())
        .unwrap_or("")
        .trim_matches(|c: char| !c.is_alphanumeric())
        .to_lowercase();
    !last.is_empty() && ROMAN_KEYWORDS.contains(last.as_str())
}

/// Convert a Roman numeral to its integer value; 0 if empty or malformed.
// ─ Licence plates and identifiers ────────────────────────────────────────
// Vietnamese plates: "51H-123.45", "30K-567.89", "51K1-123.45". Must run BEFORE
// the clock pass, since "51H" matches the "<number>h" pattern and would be read
// as "năm mươi mốt giờ".
static RE_PLATE: Lazy<Regex> = Lazy::new(|| {
    // Tail: "123.45" for cars, or a bare run of two to five digits — "12345"
    // and "1234" for motorbikes, "234" for lot or room codes like "51M-234".
    // All are read figure by figure.
    Regex::new(r"\b(\d{2})([A-Z]{1,2}\d?)\s*[-–]\s*(\d{3}\.\d{2}|\d{2,5})\b").unwrap()
});
// Truncated plates — province code and series only, with no digit run: "biển số
// 51H", "BKS 30K". A cue word is mandatory because a bare "51h" is a legitimate
// duration ("làm 51h mỗi tuần").
static RE_PLATE_LEAD: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b([Bb]iển(?:\s+số|\s+kiểm\s+soát)?|BKS)\s+(\d{2})([A-Z]{1,2}\d?)\b").unwrap()
});
// Letter-digit codes such as "ABC-1234" or "XYZ-9876": the digits are read one
// by one, as codes are. At least three digits are required so that "COVID-19",
// "U-17" and "F-16" keep their more natural cardinal reading.
static RE_CODE_DIGITS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b([A-Z]{2,6})-(\d{3,6})\b").unwrap()
});
// "#45021" (order or ticket number): drop the "#" rather than saying "thăng",
// and read the digits individually.
static RE_HASH_ID: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"#(\d{3,8})\b").unwrap()
});
// "tổng đài 1900", "tổng đài 1800.6601": service numbers are read figure by
// figure, never as cardinals. Dot-separated groups are accepted too, since
// n2w_single filters out non-digits itself.
static RE_HOTLINE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\b(tổng đài|hotline|đầu số)\s+(\d{3,8}(?:\.\d{2,6})*)\b").unwrap()
});

fn spell_plate_serie(serie: &str) -> String {
    serie.chars()
        .map(|c| {
            if c.is_ascii_digit() {
                n2w_single(&c.to_string())
            } else {
                let cl = c.to_lowercase().to_string();
                VI_LETTER_NAMES.get(cl.as_str()).map(|s| s.to_string()).unwrap_or(cl)
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn spell_plate_tail(tail: &str) -> String {
    tail.chars()
        .map(|c| if c == '.' { "chấm".to_string() } else { n2w_single(&c.to_string()) })
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn expand_codes_and_plates(text: &str) -> String {
    let mut res = RE_PLATE.replace_all(text, |caps: &Captures| {
        format!("{} {} {}",
            n2w(caps.get(1).unwrap().as_str()),
            spell_plate_serie(caps.get(2).unwrap().as_str()),
            spell_plate_tail(caps.get(3).unwrap().as_str()))
    }).into_owned();
    res = RE_PLATE_LEAD.replace_all(&res, |caps: &Captures| {
        format!("{} {} {}",
            caps.get(1).unwrap().as_str(),
            n2w(caps.get(2).unwrap().as_str()),
            spell_plate_serie(caps.get(3).unwrap().as_str()))
    }).into_owned();
    res = RE_CODE_DIGITS.replace_all(&res, |caps: &Captures| {
        format!("{} {}", caps.get(1).unwrap().as_str(), n2w_single(caps.get(2).unwrap().as_str()))
    }).into_owned();
    res = RE_HASH_ID.replace_all(&res, |caps: &Captures| {
        n2w_single(caps.get(1).unwrap().as_str())
    }).into_owned();
    res = RE_HOTLINE.replace_all(&res, |caps: &Captures| {
        format!("{} {}", caps.get(1).unwrap().as_str(), n2w_single(caps.get(2).unwrap().as_str()))
    }).into_owned();
    res
}

pub fn roman_to_int(match_str: &str) -> i32 {
    let num = match_str.to_uppercase();
    let chars: Vec<char> = num.chars().collect();
    let mut result = 0;
    for i in 0..chars.len() {
        let val = *ROMAN_NUMERALS.get(&chars[i]).unwrap_or(&0);
        if i + 1 < chars.len() && val < *ROMAN_NUMERALS.get(&chars[i+1]).unwrap_or(&0) {
            result -= val;
        } else {
            result += val;
        }
    }
    result
}

pub fn expand_roman(match_str: &str) -> String {
    if match_str.is_empty() {
        return String::new();
    }
    let result = roman_to_int(match_str);
    if result == 0 {
        return match_str.to_string();
    }
    format!(" {} ", n2w(&result.to_string()))
}

pub fn expand_unit_powers(text: &str) -> String {
    RE_UNIT_POWERS.replace_all(text, |caps: &Captures| {
        let base = caps.get(1).unwrap().as_str();
        let power = caps.get(2).unwrap().as_str();
        let power_norm = if power.starts_with('-') {
            format!("trừ {}", n2w(&power[1..]))
        } else {
            n2w(&power.replace('+', ""))
        };
        let base_lower = base.to_lowercase();
        let full_base = MEASUREMENT_KEY_VI.get(base_lower.as_str())
            .or(CURRENCY_KEY.get(base_lower.as_str()))
            .copied()
            .unwrap_or(base);
        format!(" {} mũ {} ", full_base, power_norm)
    }).to_string()
}

pub fn expand_letter(text: &str) -> String {
    RE_LETTER.replace_all(text, |caps: &Captures| {
        let prefix = caps.get(1).unwrap().as_str();
        let char = caps.get(3).unwrap().as_str();
        if let Some(name) = VI_LETTER_NAMES.get(char.to_lowercase().as_str()) {
            format!("{} {} ", prefix, name)
        } else {
            caps.get(0).unwrap().as_str().to_string()
        }
    }).to_string()
}

pub fn expand_abbreviations(text: &str) -> String {
    let mut result = text.to_string();
    for (k, v) in ABBRS.iter() {
        result = result.replace(k, v);
    }
    result
}

pub fn expand_standalone_letters(text: &str) -> String {
    let end_pos = text.trim_end().len();
    RE_STANDALONE_LETTER.replace_all(text, |caps: &FCaps| {
        let char_raw = caps.get(1).unwrap().as_str();
        let char_lower = char_raw.to_lowercase();
        let dot = caps.get(2).unwrap().as_str();
        if let Some(name) = VI_LETTER_NAMES.get(char_lower.as_str()) {
            // A period after an uppercase letter mid-sentence abbreviates a
            // name ("R. Nguyễn") and is dropped; at the END of the string it
            // terminates the sentence ("… = 2R.") and is kept.
            let at_end = caps.get(0).unwrap().end() >= end_pos;
            if char_raw.chars().next().unwrap().is_uppercase() && dot == "." && !at_end {
                format!(" {} ", name)
            } else {
                format!(" {}{} ", name, dot)
            }
        } else {
            caps.get(0).unwrap().as_str().to_string()
        }
    }).to_string()
}

// T2..T7 and CN are weekdays ONLY with a time cue in front ("sáng T2", "từ T2
// đến T6", "nghỉ T7"). Without one, "Model T2" and "tòa T3" stay as written.
static RE_WEEKDAY_LEAD: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\b(sáng|trưa|chiều|tối|đêm|hôm|ngày|từ|đến|tới|vào|mỗi|hằng|nghỉ)\s+(?:T([2-7])|CN)\b").unwrap()
});
// Chaining: in "thứ hai, T4 và CN" the later T/CN follow an already converted
// "thứ X" and inherit the weekday reading.
static RE_WEEKDAY_CHAIN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(thứ (?:hai|ba|tư|năm|sáu|bảy))(\s*(?:,|và|-|–|đến|tới)\s*)(?:T([2-7])|CN)\b").unwrap()
});

fn weekday_name(d: &str) -> &'static str {
    match d {
        "2" => "thứ hai", "3" => "thứ ba", "4" => "thứ tư",
        "5" => "thứ năm", "6" => "thứ sáu", "7" => "thứ bảy",
        _ => "chủ nhật",
    }
}

pub fn expand_weekday_abbr(text: &str) -> String {
    if !text.contains('T') && !text.contains("CN") { return text.to_string(); }
    let mut res = RE_WEEKDAY_LEAD.replace_all(text, |caps: &Captures| {
        let lead = caps.get(1).unwrap().as_str();
        let day = caps.get(2).map(|m| m.as_str()).unwrap_or("cn");
        format!("{} {}", lead, weekday_name(day))
    }).into_owned();
    // Lists can be long ("từ T2, T4 và CN"), so iterate to a fixed point.
    for _ in 0..6 {
        let next = RE_WEEKDAY_CHAIN.replace_all(&res, |caps: &Captures| {
            let day = caps.get(3).map(|m| m.as_str()).unwrap_or("cn");
            format!("{}{}{}", caps.get(1).unwrap().as_str(), caps.get(2).unwrap().as_str(), weekday_name(day))
        }).into_owned();
        if next == res { break; }
        res = next;
    }
    res
}

pub fn normalize_acronyms(text: &str) -> String {
    let mut result = Vec::new();
    let re_split = &*RE_ACRONYMS_SPLIT;

    let mut last = 0;
    let mut final_parts = Vec::new();
    for mat in re_split.find_iter(text) {
        final_parts.push(&text[last..mat.start()]);
        final_parts.push(mat.as_str());
        last = mat.end();
    }
    final_parts.push(&text[last..]);

    for i in (0..final_parts.len()).step_by(2) {
        let s = final_parts[i];
        let sep = if i + 1 < final_parts.len() { final_parts[i+1] } else { "" };
        if s.is_empty() {
            result.push(sep.to_string());
            continue;
        }

        let words: Vec<&str> = s.split_whitespace().collect();
        // Decide whether the span is all-caps — a heading or shouted sentence,
        // to be read as Vietnamese prose:
        //   - purely numeric or punctuation tokens ("4", "06") do NOT count,
        //     otherwise "CHƯƠNG 4" would fail the test and its Vietnamese word
        //     would be spelled out letter by letter;
        //   - tokens mixing letters and digits (CO2, H2O, B2B) are formulas or
        //     codes rather than prose, and go to the acronym branch instead
        //     ("CO2" -> "xê ô hai").
        let letter_tokens: Vec<&&str> = words.iter()
            .filter(|w: &&&str| w.chars().any(|c: char| c.is_alphabetic()))
            .collect();
        let is_all_caps = !letter_tokens.is_empty()
            && letter_tokens.iter().all(|w: &&&str| {
                !w.chars().any(|c: char| c.is_numeric())
                    && w.chars().filter(|c: &char| c.is_alphabetic()).all(|c: char| c.is_uppercase())
            });

        let mut processed_s = s.to_string();
        if !is_all_caps {
            processed_s = RE_ACRONYM.replace_all(&processed_s, |caps: &FCaps| {
                let word = caps.get(0).unwrap().as_str();
                if word.chars().all(|c: char| c.is_ascii_digit()) { return word.to_string(); }
                // Table first: an entry's reading mode always wins over any
                // heuristic below. Expand/Fixed normally fire in an earlier
                // stage; matching them here too keeps the table authoritative
                // regardless of stage order.
                match VI_ABBREV.get(word) {
                    Some(Reading::WordEn) => {
                        return format!("__start_en__{}__end_en__", word.to_lowercase());
                    }
                    Some(Reading::LettersNative) => {
                        let parts: Vec<String> = word.chars()
                            .filter_map(|c: char| {
                                let cl = c.to_lowercase().to_string();
                                VI_LETTER_NAMES.get(cl.as_str()).map(|n: &&str| n.to_string())
                            })
                            .collect();
                        return parts.join(" ");
                    }
                    Some(Reading::LettersEn) => {
                        let spaced = word.chars()
                            .map(|c: char| c.to_lowercase().to_string())
                            .collect::<Vec<String>>().join(" ");
                        return format!("__start_en__{}__end_en__", spaced);
                    }
                    // Expand/Fixed are NOT handled here: they belong to the
                    // exceptions stage, which is gated on sentence language
                    // ("The VN team" spells v-n; "đội VN" expands việt nam).
                    Some(Reading::Expand(_)) | Some(Reading::Fixed(_)) | None => {}
                }

                let has_vi_letter = word.chars().any(|c: char| !c.is_ascii() && c.is_alphabetic());
                let is_mixed_case = word.chars().any(|c: char| c.is_lowercase()) && word.chars().any(|c: char| c.is_uppercase());
                let has_subscript = word.chars().any(|c: char| c >= '₀' && c <= '₉');

                // An all-caps Vietnamese token that forms ONE valid syllable is
                // a word, not an acronym or formula, so it is left whole:
                // "CHƯƠNG" -> "chương" inside lowercase prose, while "ĐKVĐ" is
                // still spelled out.
                if has_vi_letter && is_vietnamese_syllable(word) {
                    return word.to_lowercase();
                }

                // ASCII all-caps token, table-first policy: the abbreviation
                // table decides known acronyms (checked above); for the rest
                // the DICTIONARY is the arbiter of wordhood — an all-caps
                // token whose lowercase form is both a valid Vietnamese
                // syllable AND a dictionary word is a shouted word, not an
                // acronym ("tôi HO một cái" -> "ho"). Both conditions are
                // required: the dictionary alone still contains English
                // entries ("it", "us") that the syllable test rejects.
                if !has_vi_letter && !is_mixed_case
                    && !word.chars().any(|c: char| c.is_numeric())
                    && is_vietnamese_syllable(word)
                    && crate::lang::vi::technical::dict_has_vi(&word.to_lowercase()) {
                    return word.to_lowercase();
                }

                if has_vi_letter || is_mixed_case || has_subscript {
                    let mut parts = Vec::new();
                    for c in word.chars() {
                        let cl = c.to_lowercase().to_string();
                        if c.is_ascii_digit() { parts.push(n2w_single(&c.to_string())); }
                        else if let Some(name) = VI_LETTER_NAMES.get(cl.as_str()) { parts.push(name.to_string()); }
                        else if let Some(sub_name) = SUBSCRIPTS_MAP.get(&c) { parts.push(sub_name.trim().to_string()); }
                        else if c.is_alphabetic() { parts.push(cl); }
                    }
                    return parts.join(" ");
                }

                if word.chars().any(|c: char| c.is_ascii_digit() || (c >= '₀' && c <= '₉')) {
                    let res: Vec<String> = word.chars().map(|c: char| {
                        if c.is_ascii_digit() { n2w_single(&c.to_string()) }
                        else if let Some(sub_name) = SUBSCRIPTS_MAP.get(&c) { sub_name.trim().to_string() }
                        else { VI_LETTER_NAMES.get(c.to_lowercase().to_string().as_str()).cloned().unwrap_or(c.to_string().as_str()).to_string() }
                    }).collect();
                    return res.join(" ");
                }

                let spaced_word = word.chars().filter(|c: &char| c.is_alphanumeric()).map(|c: char| c.to_lowercase().to_string()).collect::<Vec<String>>().join(" ");
                if !spaced_word.is_empty() { format!("__start_en__{}__end_en__", spaced_word) } else { word.to_string() }
            }).to_string();
        }
        result.push(processed_s + sep);
    }
    result.join("")
}

pub fn expand_alphanumeric(text: &str) -> String {
    RE_ALPHANUMERIC.replace_all(text, |caps: &Captures| {
        let num = caps.get(1).unwrap().as_str();
        let char = caps.get(2).unwrap().as_str().to_lowercase();
        if let Some(name) = VI_LETTER_NAMES.get(char.as_str()) {
            let mut pronunciation = name.to_string();
            if char == "d" && (text.to_lowercase().contains("quốc lộ") || text.to_lowercase().contains("ql")) {
                pronunciation = "đê".to_string();
            }
            format!("{} {}", num, pronunciation)
        } else {
            caps.get(0).unwrap().as_str().to_string()
        }
    }).into_owned()
}

/// "R&D" and "R & D" become "<en>r and d</en>" for known English acronyms.
/// Anything not on the list ("A & B") is left alone, so its "&" still reads
/// "và".
pub fn expand_english_ampersand(text: &str) -> String {
    RE_AMPERSAND_ACRONYM.replace_all(text, |caps: &Captures| {
        let l = caps.get(1).unwrap().as_str();
        let r = caps.get(2).unwrap().as_str();
        let key = format!("{}{}", l.to_uppercase(), r.to_uppercase());
        if ENGLISH_AMPERSAND.contains(key.as_str()) {
            let spell = |s: &str| s.chars()
                .map(|c: char| c.to_lowercase().to_string())
                .collect::<Vec<String>>()
                .join(" ");
            format!(" __start_en__{} and {}__end_en__ ", spell(l), spell(r))
        } else {
            caps.get(0).unwrap().as_str().to_string()
        }
    }).into_owned()
}

pub fn expand_symbols(text: &str) -> String {
    let res = text.replace("<>", " khác ");
    let mut result = String::with_capacity(res.len());
    for c in res.chars() {
        if let Some(v) = SYMBOLS_MAP.get(&c) {
            result.push_str(v);
        } else if let Some(v) = SUPERSCRIPTS_MAP.get(&c) {
            result.push_str(v);
        } else if let Some(v) = SUBSCRIPTS_MAP.get(&c) {
            result.push_str(v);
        } else {
            result.push(c);
        }
    }
    result
}

pub fn expand_prime(text: &str) -> String {
    let res = RE_PRIME.replace_all(text, |caps: &FCaps| {
        let val = caps.get(1).unwrap().as_str().to_lowercase();
        let name = if val.chars().next().unwrap().is_ascii_digit() {
            n2w_single(&val)
        } else {
            VI_LETTER_NAMES.get(val.as_str()).cloned().unwrap_or(&val).to_string()
        };
        let count = caps.get(2).unwrap().as_str().chars().count();
        let phay = vec!["phẩy"; count].join(" ");
        format!("{} {}", name, phay)
    }).to_string();

    RE_PRIME_DIGIT.replace_all(&res, |caps: &FCaps| {
        let q = caps.get(1).unwrap().as_str();
        if q == "\"" || q == "\u{201D}" || q.len() > 1 {
            " phẩy phẩy ".to_string()
        } else {
            " phẩy ".to_string()
        }
    }).to_string()
}

pub fn expand_temperatures(text: &str) -> String {
    let mut res = RE_TEMP_C_NEG.replace_all(text, "âm $1 độ xê").into_owned();
    res = RE_TEMP_F_NEG.replace_all(&res, "âm $1 độ ép").into_owned();
    res = RE_TEMP_C.replace_all(&res, "$1 độ xê").into_owned();
    res = RE_TEMP_F.replace_all(&res, "$1 độ ép").into_owned();
    RE_DEGREE.replace_all(&res, " độ ").into_owned()
}

pub fn normalize_others(text: &str, en_ctx: bool) -> String {
    let text = RE_TITLE_DOT.replace_all(text, "$1 ").into_owned();
    let text = if en_ctx {
        text
    } else {
        let t = RE_DISTRICT_DOT.replace_all(&text, "quận ").into_owned();
        RE_WARD_DOT.replace_all(&t, "phường ").into_owned()
    };
    let text = RE_ABS.replace_all(&text, " giá trị tuyệt đối của $1 ").into_owned();
    // Pure-English sentences do NOT get Vietnamese expansions: "VN" stays as
    // it is, so the acronym pass spells English letters instead of producing
    // "việt nam".
    let mut res = if en_ctx {
        text
    } else {
        RE_ACRONYMS_EXCEPTIONS.replace_all(&text, |caps: &Captures| {
            COMBINED_EXCEPTIONS.get(caps.get(0).unwrap().as_str()).cloned().unwrap_or(caps.get(0).unwrap().as_str().to_string())
        }).into_owned()
    };

    res = normalize_slashes(&res);
    res = DOMAIN_SUFFIXES_RE.replace_all(&res, |caps: &Captures| {
        let suffix = DOMAIN_SUFFIX_MAP.get(caps.get(1).unwrap().as_str().to_lowercase().as_str()).copied().unwrap_or("");
        format!(" chấm {} ", if suffix.is_empty() { caps.get(1).unwrap().as_str() } else { suffix })
    }).into_owned();

    // Roman section numbers at the start of a line ("I. VỀ …", "II. Về …") read
    // as numbers. The "." is kept, since it provides the heading's pause; only
    // the numeral itself is replaced.
    res = RE_ROMAN_LIST_MARKER.replace_all(&res, |caps: &FCaps| {
        let lead = caps.get(1).unwrap().as_str();
        let roman = caps.get(2).unwrap().as_str();
        let tail = caps.get(3).unwrap().as_str();
        let head_upper = caps.get(4).map(|m| m.as_str()).unwrap_or("");
        let value = roman_to_int(roman);
        let single = roman.chars().count() == 1;
        // Reject implausibly large values, which are really initials
        // (C/L/D/M), and single characters whose heading is not fully
        // uppercase — those are abbreviated names ("V. Nguyễn", "I. Trần").
        if value <= 0 || value > ROMAN_MARKER_MAX
            || (single && head_upper.chars().count() < 2)
        {
            return caps.get(0).unwrap().as_str().to_string();
        }
        format!("{}{}{}", lead, n2w(&value.to_string()), tail)
    }).to_string();

    // Expand Roman numerals only with a cue word immediately before (thế kỷ,
    // chương, phần, đời, vua). Otherwise leave the run for the acronym branch,
    // which reads "CD", "MC" and "XL" as English letters.
    let roman_src = res.clone();
    res = RE_ROMAN_NUMBER.replace_all(&roman_src, |caps: &FCaps| {
        let m = caps.get(0).unwrap();
        if has_roman_context(&roman_src[..m.start()]) {
            expand_roman(m.as_str())
        } else {
            m.as_str().to_string()
        }
    }).to_string();

    // Single-character Roman numerals (I/V/X). RE_ROMAN_NUMBER requires two
    // characters, so "quý I", "chương V" and "khóa X" slip through it. One
    // character is far too easy to confuse with an ordinary letter, so a cue
    // word immediately before is mandatory here.
    let roman1_src = res.clone();
    res = RE_ROMAN_SINGLE.replace_all(&roman1_src, |caps: &FCaps| {
        let m = caps.get(0).unwrap();
        if has_roman_context(&roman1_src[..m.start()]) {
            n2w(&roman_to_int(m.as_str()).to_string())
        } else {
            m.as_str().to_string()
        }
    }).to_string();

    res = expand_letter(&res);
    res = expand_alphanumeric(&res);
    res = RE_LETTER_DIGIT.replace_all(&res, |caps: &Captures| {
        let l = caps.get(1).unwrap().as_str().to_lowercase();
        let d = caps.get(2).unwrap().as_str();
        if let Some(name) = VI_LETTER_NAMES.get(l.as_str()) {
            format!("{} {}", name, n2w(d))
        } else {
            caps.get(0).unwrap().as_str().to_string()
        }
    }).into_owned();
    res = expand_prime(&res);
    res = expand_unit_powers(&res);
    res = RE_CLEAN_QUOTES.replace_all(&res, "").into_owned();
    res = RE_CLEAN_QUOTES_EDGES.replace_all(&res, "$1 $2").into_owned();
    res = expand_english_ampersand(&res);
    res = expand_symbols(&res);
    res = RE_BRACKETS.replace_all(&res, ", $1, ").into_owned();
    res = RE_STRIP_BRACKETS.replace_all(&res, " ").into_owned();
    res = expand_temperatures(&res);
    res = normalize_acronyms(&res);

    res = RE_VERSION.replace_all(&res, |caps: &FCaps| {
        let parts: Vec<String> = caps.get(1).unwrap().as_str().split('.').map(|s: &str| {
            s.chars().map(|c: char| n2w_single(&c.to_string())).collect::<Vec<String>>().join(" ")
        }).collect();
        parts.join(" chấm ")
    }).to_string();

    // Multi-part ratios "1:2:3" -> "một trên hai trên ba", before the two-part
    // ratio rule.
    res = RE_RATIO_MULTI.replace_all(&res, |caps: &FCaps| {
        let parts: Vec<String> = caps.get(0).unwrap().as_str()
            .split(':')
            .map(|p: &str| n2w(p))
            .collect();
        format!(" {} ", parts.join(" trên "))
    }).to_string();

    // Handle numeric ratios/versions like 2:1 or 9001:2015
    res = RE_STANDARD_COLON.replace_all(&res, |caps: &FCaps| {
        let n1 = caps.get(1).unwrap().as_str();
        let n2 = caps.get(2).unwrap().as_str();
        let n1_val = n1.parse::<u64>().unwrap_or(0);

        // Heuristic: Use "trên" ONLY for pure integer-integer ratios where n1 is small.
        // Use a comma for EVERYTHING else (years, map scales 1:50.000, odds 1:2.5, etc.)
        if n1_val < 1000 && !n2.contains('.') {
            format!(" {} trên {} ", n1, n2)
        } else {
            format!("{}, {}", n1, n2)
        }
    }).to_string();

    res = RE_COLON_SEMICOLON.replace_all(&res, ", ").into_owned();
    RE_CLEAN_OTHERS.replace_all(&res, " ").into_owned()
}
