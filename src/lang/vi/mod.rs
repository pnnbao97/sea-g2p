//! Vietnamese text normalization, run before grapheme-to-phoneme conversion.
//!
//! # Architecture: a staged pipeline
//!
//! `clean_vietnamese_text_ctx` applies a fixed sequence of stages (`stage_*`).
//! The order is **not** arbitrary: each stage assumes earlier ones have already
//! resolved a particular class of ambiguity. The table below is the contract
//! between stages — reordering without updating it reliably produces silent
//! misreadings.
//!
//! | # | Stage | Does | Why it sits here |
//! |---|-------|------|------------------|
//! | 0 | `detect_context` | Classify the sentence as Vietnamese or English | Every later stage reads these flags |
//! | 1 | `protect_spans` | Mask emails, URLs, paths, camelCase exceptions | Technical spans must be untouchable before any word/symbol splitting runs |
//! | 2 | `early_lexical` | Weekday abbreviations, `text2text`, English pre-normalization | Before any letter-digit pass, otherwise "T2" becomes "tê hai" |
//! | 3 | `superscripts` | `m²`→`m2`, `10⁻³`→"mũ trừ ba", `10²³` | Before `expand_symbols`, which would read each exponent character separately |
//! | 4 | `math` | Bare formulas: split variable clusters, factorials, binary minus | Before term splitting and the multiplication pass, so "mc²" / "4ac" survive intact |
//! | 5 | `split_terms` | Split camelCase | After math, which needs whole tokens |
//! | 6 | `codes` | Licence plates, identifiers, chemical coefficients | Before the multiplication pass ("59X1" contains X) and before clock times ("51H") |
//! | 7 | `arithmetic` | Powers, multiplication, context-dependent minus / range | After codes, so identifiers are not consumed |
//! | 8 | `abbreviations` | Abbreviations, address prefixes, money slang | Before dates, since some abbreviations embed digits |
//! | 9 | `percent_scores` | Percentage ranges, negative percentages, sport scores | Before dates: "3-1" must not become a date |
//! | 10 | `datetime` | Dates and clock times | After codes and scores, before the generic number passes |
//! | 11 | `phones` | Phone numbers, hotlines, grouped digit runs | Before generic numbers, which would read them as cardinals |
//! | 12 | `ranges_signs` | Numeric ranges, negative signs, dash → comma | After dates and phones have consumed the legitimate dashes |
//! | 13 | `units` | Measurement units, currencies, height and weight | Once every sign and numeric cluster is settled |
//! | 14 | `decimals` | Decimal marks and thousand separators | Last step of numeric processing |
//! | 15 | `residual` | Leftover symbols, stray numbers, "âm âm" collapse | Cleans up whatever no earlier stage claimed |
//! | 16 | `letters` | Single letters → Vietnamese letter names | Last, otherwise it would swallow letters the stages above still need |
//! | 17 | `finalize` | Unmask, restore tags, collapse whitespace | Must be last |
//!
//! # Invariant: nothing disappears in silence
//!
//! Any non-alphanumeric character reaching the end of the pipeline is deleted
//! by `RE_CLEAN_OTHERS` **without a trace** — the root cause of a whole family
//! of past defects (`∆` U+2206, `⁻` U+207B, `Σ`). The [`audit`] module lists
//! such characters for a given input so tests catch them before production
//! does. See [`audit::audit_unmapped`].

pub mod num2vi;
pub mod num2en;
pub mod resources;
pub mod vi_top_syllables;
pub mod vi_bigrams;
pub mod numerical;
pub mod datestime;
pub mod units;
pub mod technical;
pub mod misc;
pub mod audit;

use pyo3::prelude::*;
// fancy_regex only for patterns requiring look-arounds
use fancy_regex::{Regex as FRegex, Captures as FCaps};
// regex crate for simple patterns (Thompson NFA - much faster than fancy_regex backtracker)
use regex::{Regex, Captures};
use once_cell::sync::Lazy;
use unicode_normalization::UnicodeNormalization;
use crate::lang::vi::numerical::{normalize_number_vi, RE_MULTIPLY, expand_multiply_number};
use crate::lang::vi::datestime::{normalize_date, normalize_time};
use crate::lang::vi::units::{expand_units_and_currency, expand_compound_units, expand_scientific_notation, fix_english_style_numbers, expand_power_of_ten, expand_height_weight};
use crate::lang::vi::misc::{normalize_others, expand_standalone_letters, expand_weekday_abbr, RE_ACRONYMS_EXCEPTIONS, RE_ACRONYM};
use crate::lang::vi::technical::{normalize_technical, normalize_emails, RE_TECHNICAL, RE_EMAIL};
use crate::lang::vi::resources::{COMBINED_EXCEPTIONS, MEASUREMENT_KEY_VI};

// ── Tier 1: regex crate (Thompson NFA, much faster for simple patterns) ────
static RE_EXTRA_SPACES: Lazy<Regex> = Lazy::new(|| Regex::new(r"[ \t\xA0]+").unwrap());
static RE_EXTRA_COMMAS: Lazy<Regex> = Lazy::new(|| Regex::new(r",\s*,").unwrap());
// Single-character ellipses ․(U+2024) ‥(U+2025) …(U+2026) fold to "." so they
// travel the same path as "...", which RE_MULTI_DOT then reduces to one period.
static RE_ELLIPSIS: Lazy<Regex> = Lazy::new(|| Regex::new(r"[\u{2024}\u{2025}\u{2026}]").unwrap());
static RE_MULTI_DOT: Lazy<Regex> = Lazy::new(|| Regex::new(r"\.[\s.]*\.").unwrap());
// Collapse repeated or mixed terminators ("!!!", "???", "?!", "!?!?") to the
// first mark.
static RE_MULTI_BANG: Lazy<Regex> = Lazy::new(|| Regex::new(r"([!?])[!?\s]*[!?]").unwrap());
static RE_COMMA_BEFORE_PUNCT: Lazy<Regex> = Lazy::new(|| Regex::new(r",\s*([.!?;])").unwrap());
static RE_SPACE_BEFORE_PUNCT: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s+([,.!?;:])").unwrap());
// Rewritten to avoid lookahead: capture the following char so regex crate can handle it
static RE_MISSING_SPACE_AFTER_PUNCT: Lazy<Regex> = Lazy::new(|| Regex::new(r"([.,!?;:])([^\s\d<])").unwrap());
static RE_INTERNAL_EN_TAG: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)(?s)(__start_en__.*?__end_en__|<en>.*?</en>)").unwrap());
static RE_DOT_BETWEEN_DIGITS: Lazy<Regex> = Lazy::new(|| Regex::new(r"(\d+)\.(\d+)").unwrap());
// "âm âm năm" — the text already said "âm" and the number kept its minus sign.
static RE_DOUBLE_AM: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)\bâm\s+âm\s+(\S+)").unwrap());
static RE_ENTOKEN: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)ENTOKEN\d+").unwrap());
static RE_EN_TAG: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?si)<en>.*?</en>").unwrap());
// Author-declared formula region: <math>...</math>. Inside it every letter
// cluster except function names is split into individual letters so they are
// read by name ("4ac" -> "bốn a xê", "dx" -> "đê ích"). The rest — symbols,
// exponents, radicals — is left to the normal pipeline.
static RE_MATH_TAG: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?si)<math>(.*?)</math>").unwrap());
static RE_MATH_WORD: Lazy<Regex> = Lazy::new(|| Regex::new(r"[a-zA-Z]+").unwrap());
// Minus at the START of a formula ("-a + b") is a sign: "âm a …".
static RE_LEAD_NEG: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[-–—]\s*([a-zA-Z0-9])").unwrap());
// Inside <math>: factorials "n!", "5!", "(n+1)!" -> "… giai thừa".
static RE_MATH_FACTORIAL: Lazy<FRegex> = Lazy::new(|| FRegex::new(r"(?<=[0-9A-Za-z)])!").unwrap());
// Inside <math>: BINARY minus between two operands (c-d, a-b, 5-3) -> "trừ".
// Look-around means the operands are not consumed, so chains like "a-b-c" work.
// After "(" or "=" it deliberately fails to match, leaving the unary branch to
// read "âm".
static RE_MATH_BIN_MINUS: Lazy<FRegex> = Lazy::new(|| FRegex::new(r"(?<=[0-9A-Za-z)])\s*[-–—]\s*(?=[0-9A-Za-z(∫√])").unwrap());
// Natural logarithm. Lowercase only: "LN" in a run of capitals is an
// initialism, not a function.
static RE_LN: Lazy<Regex> = Lazy::new(|| Regex::new(r"\bln\b").unwrap());
static MATH_FUNCS: Lazy<std::collections::HashSet<&'static str>> = Lazy::new(|| {
    [
        "sin", "cos", "tan", "cot", "sec", "csc", "sinh", "cosh", "tanh", "coth",
        "arcsin", "arccos", "arctan", "asin", "acos", "atan", "log", "ln", "lg",
        "exp", "lim", "max", "min", "sup", "inf", "det", "dim", "deg", "gcd",
        "lcm", "mod", "arg", "sgn", "rank", "tr",
    ].into_iter().collect()
});

fn split_math_letters(content: &str) -> String {
    RE_MATH_WORD.replace_all(content, |caps: &Captures| {
        let w = caps.get(0).unwrap().as_str();
        if w.chars().count() == 1 || MATH_FUNCS.contains(w.to_lowercase().as_str()) {
            w.to_string()
        } else {
            w.chars().map(|c: char| c.to_string()).collect::<Vec<String>>().join(" ")
        }
    }).into_owned()
}

// ─ Bare formulas, written without <math> tags ─────────────────────────────
// Find runs of maths-shaped tokens in ordinary prose and apply the same
// treatment as a <math> region: split variable clusters ("mc²" -> "m c²",
// "4ac" -> "4 a c"), factorials, binary minus.
//
// The admission rules are deliberately strict, because prose and formulas share
// an alphabet:
//  - every token in the run consists only of digits, operators, brackets, Greek
//    letters or super/subscripts, and any ASCII letter cluster inside it is at
//    most three characters long or is a function name;
//  - a token of PURE LETTERS two or more characters long ("ma", "khi") is
//    admitted only when adjacent to a token containing an operator, so "F = ma"
//    takes "ma" while "khi x = 1" leaves "khi" alone;
//  - the run must contain at least one strong maths mark: = √ ∫ ± ≤ ≥ ≠ or a
//    super/subscript.
fn is_strong_math_char(c: char) -> bool {
    matches!(c, '=' | '√' | '∫' | '±' | '≤' | '≥' | '≠')
        || crate::lang::vi::resources::SUPERSCRIPTS_MAP.contains_key(&c)
        || crate::lang::vi::resources::SUBSCRIPTS_MAP.contains_key(&c)
}

// "/", √ and ∫ do NOT count as *contextual* operators. A fraction or radical
// inside a token ("Σ(1/2ⁿ)", "1/√3,") sitting next to a toneless Vietnamese word
// ("khi", "ta") would otherwise drag that word into the formula run. They remain
// valid maths evidence *within* a token.
fn has_operator_char(tok: &str) -> bool {
    tok.chars().any(|c| matches!(c, '=' | '+' | '-' | '–' | '—' | '±' | '*' | '×' | '÷'))
}

// Differentials dx/dy/dz/du/dv/dt always count as maths tokens, even next to a
// token with no operator ("∫sin x dx"), and are still split into letters when
// read ("đê ích").
static MATH_DIFFS: Lazy<std::collections::HashSet<&'static str>> = Lazy::new(|| {
    ["dx", "dy", "dz", "du", "dv", "dt"].into_iter().collect()
});

/// Returns `(passes the character test, is a multi-letter word needing context)`.
fn classify_math_token(tok: &str) -> (bool, bool) {
    if tok.is_empty() { return (false, false); }
    let allowed = |c: char| {
        c.is_ascii_alphanumeric()
            || "()[]{}+-–—*/=±≤≥≠≈×÷√∫|!'^.,;:°½¼¾⅓⅔∆∑".contains(c)
            || ('α'..='ω').contains(&c) || ('Α'..='Ω').contains(&c)
            || crate::lang::vi::resources::SUPERSCRIPTS_MAP.contains_key(&c)
            || crate::lang::vi::resources::SUBSCRIPTS_MAP.contains_key(&c)
    };
    if !tok.chars().all(allowed) { return (false, false); }
    let mut run = String::new();
    let mut runs: Vec<String> = Vec::new();
    for c in tok.chars() {
        if c.is_ascii_alphabetic() { run.push(c); }
        else if !run.is_empty() { runs.push(std::mem::take(&mut run)); }
    }
    if !run.is_empty() { runs.push(run); }
    for r in &runs {
        if r.chars().count() > 3 && !MATH_FUNCS.contains(r.to_lowercase().as_str()) {
            return (false, false);
        }
    }
    // Pure letters — no digit, operator or maths symbol anywhere in the token,
    // and trailing punctuation such as the comma in "thi," does not count as
    // evidence — several characters long, and neither a function name nor a
    // differential. Such tokens collide with real words, so they need an
    // operator beside them before being admitted.
    let has_math_evidence = tok.chars().any(|c| {
        c.is_ascii_digit()
            || "+-–—*/=±≤≥≠≈×÷√∫^'!½¼¾⅓⅔∆∑".contains(c)
            || ('α'..='ω').contains(&c) || ('Α'..='Ω').contains(&c)
            || crate::lang::vi::resources::SUPERSCRIPTS_MAP.contains_key(&c)
            || crate::lang::vi::resources::SUBSCRIPTS_MAP.contains_key(&c)
    });
    let pure_alpha_multi = runs.len() == 1
        && runs[0].chars().count() >= 2
        && !has_math_evidence
        && {
            let lower0 = runs[0].to_lowercase();
            !MATH_FUNCS.contains(lower0.as_str()) && !MATH_DIFFS.contains(lower0.as_str())
        };
    (true, pure_alpha_multi)
}

fn split_math_token(tok: &str, protect_units: bool) -> String {
    RE_MATH_WORD.replace_all(tok, |caps: &Captures| {
        let w = caps.get(0).unwrap().as_str();
        let lower = w.to_lowercase();
        if w.chars().count() == 1
            || MATH_FUNCS.contains(lower.as_str())
            || (protect_units && MEASUREMENT_KEY_VI.contains_key(lower.as_str()))
        {
            w.to_string()
        } else {
            w.chars().map(|c: char| c.to_string()).collect::<Vec<String>>().join(" ")
        }
    }).into_owned()
}

fn expand_inline_math(text: &str) -> String {
    text.split('\n').map(|line| {
        let toks: Vec<&str> = line.split_whitespace().collect();
        if toks.len() < 2 { return line.to_string(); }
        let classes: Vec<(bool, bool)> = toks.iter().map(|t| classify_math_token(t)).collect();
        let mathy: Vec<bool> = (0..toks.len()).map(|i| {
            let (base, conditional) = classes[i];
            if !base { return false; }
            if !conditional { return true; }
            let prev_op = i > 0 && classes[i - 1].0 && has_operator_char(toks[i - 1]);
            let next_op = i + 1 < toks.len() && classes[i + 1].0 && has_operator_char(toks[i + 1]);
            prev_op || next_op
        }).collect();
        let mut out: Vec<String> = Vec::with_capacity(toks.len());
        let mut i = 0;
        while i < toks.len() {
            if !mathy[i] { out.push(toks[i].to_string()); i += 1; continue; }
            let mut j = i;
            while j < toks.len() && mathy[j] { j += 1; }
            let span = toks[i..j].join(" ");
            let strong = span.chars().any(is_strong_math_char);
            // A single-token run needs a strong mark OTHER than "=" ("½mv²",
            // "eˣ"). "id=1;" or "x=5" is too little evidence, and "id" must stay
            // intact for G2P to look up.
            let strong_non_eq = span.chars().any(|c| c != '=' && is_strong_math_char(c));
            if (j - i >= 2 && strong) || (j - i == 1 && strong_non_eq) {
                let mut parts: Vec<String> = Vec::with_capacity(j - i);
                for (k, t) in toks[i..j].iter().enumerate() {
                    // A letter cluster that is a MEASUREMENT UNIT following a
                    // number ("170 km²"), or alone in its token ("km²"), is left
                    // for the unit pass. After "=" the same letters are a product
                    // of variables instead ("= mg" -> "m g").
                    let prev_numeric = k == 0
                        || toks[i + k - 1].chars().any(|c| c.is_ascii_digit());
                    parts.push(split_math_token(t, prev_numeric));
                }
                let s = parts.join(" ");
                let s = RE_MATH_FACTORIAL.replace_all(&s, " giai thừa ").into_owned();
                let s = RE_MATH_BIN_MINUS.replace_all(&s, " trừ ").into_owned();
                out.push(s);
            } else {
                out.push(span);
            }
            i = j;
        }
        out.join(" ")
    }).collect::<Vec<_>>().join("\n")
}
// Context words must be written as real Vietnamese characters. Do NOT use byte
// escapes like `\xHH` inside a raw string `r"..."`: raw strings do not process
// escapes, so the engine reads `\xe1` as codepoint U+00E1 and "bằng" turns into
// mojibake that can never match.
static RE_CONTEXT_TRU: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)\b(b\u1eb1ng|t\u00ednh|k\u1ebft qu\u1ea3)\s+(\d+(?:[.,]\d+)?)\s*[-\u2013\u2014]\s*(\d+(?:[.,]\d+)?)\b").unwrap());
static RE_CONTEXT_TRU_POST: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)\b(\d+(?:[.,]\d+)?)\s*[-\u2013\u2014]\s*(\d+(?:[.,]\d+)?)\s+(b\u1eb1ng|t\u00ednh|k\u1ebft qu\u1ea3)\b").unwrap());
static RE_CONTEXT_DEN: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)\b(t\u1eeb|kho\u1ea3ng|trong)\s+(\d+(?:[.,]\d+)?)\s*[-\u2013\u2014]\s*(\d+(?:[.,]\d+)?)\b").unwrap());
static RE_EQ_MINUS: Lazy<Regex> = Lazy::new(|| Regex::new(r"([\d./]+)\s*[-\u2013\u2014]\s*([\d./]+)\s*=").unwrap());
static RE_EQ_NEG: Lazy<Regex> = Lazy::new(|| Regex::new(r"=\s*[-\u2013\u2014](\d+(?:[./]\d+)?)").unwrap());
static RE_PHONE_WITH_DASH: Lazy<Regex> = Lazy::new(|| Regex::new(r"\b(0\d{2,3})[\u2013\-\u2014](\d{3,4})[\u2013\-\u2014](\d{4})\b").unwrap());
static RE_POWER_OF_TEN_IMPLICIT: Lazy<Regex> = Lazy::new(|| Regex::new(r"\b10\^([-+]?\d+)\b").unwrap());
static RE_TO_SANG: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s*(?:->|=>)\s*").unwrap());
static RE_MULTI_COMMA: Lazy<Regex> = Lazy::new(|| Regex::new(r"\b(\d+(?:,\d+){2,})\b").unwrap());
static RE_NUMERIC_DASH_GROUPS: Lazy<Regex> = Lazy::new(|| Regex::new(r"\b\d+(?:[\u2013\-\u2014]\d+){2,}\b").unwrap());
static RE_PHONE_SPACE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\b0\d{2,3}(?:\s\d{3}){2}\b").unwrap());
// Percentage range "5-7%" -> "5 đến 7%". The percent sign is what proves this is
// a range rather than a fraction or a date, and the rule must precede
// normalize_date, which would otherwise claim it.
static RE_RANGE_PCT: Lazy<Regex> = Lazy::new(|| Regex::new(r"(\d+(?:[.,]\d+)?)\s*[-–—]\s*(\d+(?:[.,]\d+)?)\s*%").unwrap());
// Negative percentage: "-5%" -> "âm 5%", keeping the "%" for the unit pass to
// read as "phần trăm". The lookbehind (?<![\d.,]) confirms the minus is unary
// rather than a subtraction; genuine ranges like "10-5%" were already consumed
// by RE_RANGE_PCT above.
static RE_PCT_NEG: Lazy<FRegex> = Lazy::new(|| FRegex::new(r"(?<![\d.,])[-–—]\s?(\d+(?:[.,]\d+)?)\s*%").unwrap());
// Address abbreviations: "P.5" -> "phường 5", "Q.1" -> "quận 1",
// "Đ.3/2" -> "đường 3/2". A digit must follow, which rules out "P.S." and
// abbreviated personal names.
static RE_ADDR_ABBR: Lazy<FRegex> = Lazy::new(|| FRegex::new(r"\b([PQĐ])\.\s*(?=\d)").unwrap());
// Sport scores are read as two separate numbers ("2-1" -> "hai một"), not as a
// range ("đến") or a fraction ("trên"). Two signals identify them: a score
// keyword immediately before, or the numbers sitting between two capitalized
// proper nouns, i.e. team names.
static RE_SCORE_KW: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)\b(thắng|thua|hòa|hoà|tỉ số|tỷ số|chung cuộc|đánh bại|cầm hòa|cầm hoà)\s+(\d{1,2})\s*[-–—]\s*(\d{1,2})\b").unwrap());
static RE_SCORE_TEAMS: Lazy<Regex> = Lazy::new(|| Regex::new(r"(\p{Lu}\p{L}+(?:\s\p{Lu}\p{L}+)?)\s(\d{1,2})\s*[-–—]\s*(\d{1,2})\s(\p{Lu}\p{L}+)").unwrap());
// Structural or range prefixes before "number-number" rule out a score reading,
// leaving the range logic in charge.
const SCORE_EXCLUDE: [&str; 26] = [
    "điều", "khoản", "chương", "mục", "phần", "điểm", "tiết", "tập", "quyển",
    "hồi", "kỳ", "kì", "quý", "tháng", "ngày", "năm", "tuần", "từ", "khoảng",
    "trong", "bài", "câu", "trang", "dòng", "khóa", "khoá",
];

// Service numbers 1800/1900 are read figure by figure, never as a cardinal.
static RE_HOTLINE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\b(1800|1900)[\s.\-–—]?(\d{3,6})\b").unwrap());
// Landline numbers: an area code, bracketed or not, plus two groups of three or
// four digits separated by SPACES — read figure by figure. Examples:
// "(028) 3822 1234", "024 3822 1234", "+84 28 3822 1234". The lookbehind
// (?<![\d.,]) prevents matching inside a dot-separated amount (1.000.000.000).
static RE_LANDLINE: Lazy<FRegex> = Lazy::new(|| FRegex::new(r"(?<![\d.,])\(?(?:\+84\s?|0)\d{1,3}\)?\s\d{3,4}\s\d{3,4}(?!\d)").unwrap());

// Colloquial money: "500k" -> five hundred thousand, "1tr"/"15tr" -> millions,
// "1tr5" -> one and a half million. Restricted to lowercase k/tr and at most
// four digits, which avoids uppercase model suffixes ("i9-14900K", "RTX"), while
// the (?!\w) lookahead stops it eating "5kg", "5km" or "4trung".
static RE_MONEY_K: Lazy<FRegex> = Lazy::new(|| FRegex::new(r"\b(\d{1,4})k(?![\w])").unwrap());
static RE_MONEY_TR: Lazy<FRegex> = Lazy::new(|| FRegex::new(r"\b(\d{1,4})tr(\d)?(?![\w])").unwrap());

// ── Tier 2: fancy_regex (REQUIRED for look-around assertions) ────────────────
// RE_COMBINED_TECH_EMAIL removed — two separate passes are faster (mirrors Python)
// The trailing lookahead `(?![a-zA-Z])` catches a second number glued to a
// letter ("5 - 2i", "1 - 2sin²x"). That is subtraction inside a formula, not a
// range, so the match is left to RE_MATH_MINUS_COEF.
static RE_RANGE: Lazy<FRegex> = Lazy::new(|| FRegex::new(r"(?<!\d)(?<!\d[,.])(?<![a-zA-Z])(\d{1,15}(?:[,.]\d{1,15})?)(\s*)[\u2013\-\u2014](\s*)(\d{1,15}(?:[,.]\d{1,15})?)(?!\d)(?![.,]\d)(?![a-zA-Z])").unwrap());
static RE_DASH_TO_COMMA: Lazy<FRegex> = Lazy::new(|| FRegex::new(r"(?<=\s)[\u2013\-\u2014](?=\s)").unwrap());
// Minus inside a FORMULA: " - " becomes " trừ " when the left side ends in an
// exponent (b² - 4ac) or the right side is a number-letter coefficient
// (2x - 3y). Deliberately conservative, so a dash in ordinary prose
// ("Hà Nội - thủ đô") still becomes a comma.
static RE_MATH_MINUS_SUP: Lazy<FRegex> = Lazy::new(|| FRegex::new(r"(?<=[\u00b2\u00b3\u2074\u2075\u2076\u2077\u2078\u2079\u207f\u2071])\s*[-\u2013\u2014]\s*").unwrap());
static RE_MATH_MINUS_COEF: Lazy<FRegex> = Lazy::new(|| FRegex::new(r"\s[-\u2013\u2014]\s(?=\d+[a-zA-Z])").unwrap());
static RE_MATH_MINUS_COEFL: Lazy<FRegex> = Lazy::new(|| FRegex::new(r"(?<=\d[a-zA-Z])\s[-\u2013\u2014]\s(?=\d)").unwrap());
// UNARY minus before a variable (-b, =-x, (-y) -> "âm b"). It matches only after
// an operator, bracket or equals sign — never after an operand — so "a - b"
// stays subtraction. Function names are included ("-sin x" -> "âm sin x"), and
// the single-letter alternative needs a word boundary so "-sin" is not torn into
// "âm s" plus "in".
static RE_NEG_VAR1: Lazy<FRegex> = Lazy::new(|| FRegex::new(r"(?<=[(\[{=\u00b1+*/\u00d7\u00f7])[-\u2013\u2014]((?:sin|cos|tan|cot|log|ln|lim)\b|[a-zA-Z]\b)").unwrap());
static RE_NEG_VAR2: Lazy<FRegex> = Lazy::new(|| FRegex::new(r"(?<=[(\[{=\u00b1+*/\u00d7\u00f7]\s)[-\u2013\u2014]((?:sin|cos|tan|cot|log|ln|lim)\b|[a-zA-Z]\b)").unwrap());
// "-sin x" mid-sentence ("là -sin x"): a minus after whitespace and directly
// before a trigonometric function name reads as "âm sin".
static RE_NEG_FUNC: Lazy<FRegex> = Lazy::new(|| FRegex::new(r"(?<=\s)[-\u2013\u2014](?=(?:sin|cos|tan|cot|log|ln|lim)\b)").unwrap());
// Factorials outside <math>: "5! = 120", "n!/(", "O(n!)". The "!" must follow a
// single complete alphanumeric token and precede a maths character, which keeps
// exclamations out — "tuyệt!" has letters immediately before, so \b fails.
static RE_FACTORIAL_INLINE: Lazy<FRegex> = Lazy::new(|| FRegex::new(r"(?<=\b[0-9a-zA-Z])!(?=\s*[)=/,(+\u00d7*-])").unwrap());
// A run of consecutive Unicode superscripts ("10²³") is ONE exponent
// ("mười mũ hai mươi ba"), not the sequence "bình phương lập phương".
static RE_SUPERSCRIPT_RUN: Lazy<Regex> = Lazy::new(|| Regex::new(r"[\u2070\u00b9\u00b2\u00b3\u2074\u2075\u2076\u2077\u2078\u2079]{2,}").unwrap());
// SIGNED exponents ("10⁻³", "2⁻¹", "10⁺⁶"). The sign ⁻ (U+207B) used to be
// swallowed, so "10⁻³" was read as "mười lập phương" — six orders of magnitude
// off. Must run BEFORE RE_SUPERSCRIPT_RUN, which would consume the digits alone.
static RE_SUPERSCRIPT_SIGNED: Lazy<Regex> = Lazy::new(|| Regex::new(r"([\u207b\u207a])([\u2070\u00b9\u00b2\u00b3\u2074\u2075\u2076\u2077\u2078\u2079]+)").unwrap());

fn sup_digits(s: &str) -> String {
    s.chars().map(|c| match c {
        '\u{2070}' => '0', '\u{00b9}' => '1', '\u{00b2}' => '2', '\u{00b3}' => '3', '\u{2074}' => '4',
        '\u{2075}' => '5', '\u{2076}' => '6', '\u{2077}' => '7', '\u{2078}' => '8', _ => '9',
    }).collect()
}
// Area and volume units written with Unicode superscripts ("68 m²", "170 km³")
// fold to their ASCII forms ("m2", "km3") so they match the unit table and read
// "mét vuông" / "mét khối" instead of "mét bình phương". Applied only to real
// unit letters, so formula variables ("r²", "c²") keep "bình phương".
static RE_SUP_UNIT: Lazy<Regex> = Lazy::new(|| Regex::new(r"\b(km|cm|mm|m)\u00b2").unwrap());
static RE_SUP_UNIT3: Lazy<Regex> = Lazy::new(|| Regex::new(r"\b(km|cm|mm|m)\u00b3").unwrap());
// A coefficient BEFORE a chemical formula ("6CO2", "2HCl", "2H2O") is detached
// so the letters follow the acronym branch ("xê ô hai"). Strict conditions:
//  - the coefficient is exactly ONE digit, which avoids "11T14" in ISO
//    timestamps and "14H30";
//  - the letter cluster must contain a LOWERCASE letter (Cl, Na) or a DIGIT
//    (H2, O2, CO2), ruling out "2FA", "3D" and "1TB", which are plain uppercase;
//  - it must not be a clock form "4H30", i.e. H followed by exactly two final
//    digits.
static RE_CHEM_DIGIT_PREFIX: Lazy<FRegex> = Lazy::new(|| {
    FRegex::new(r"\b(\d)(?!H\d{2}\b)(?=[A-Z][a-z]|[A-Z][A-Z]?\d|[A-Z][A-Z][a-z])").unwrap()
});
// Spaced minus between two single lowercase variables ("x - a", "u - v").
static RE_MINUS_SINGLE_VARS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b([a-z])\s+[-–—]\s+([a-z])\b").unwrap()
});
static RE_FLOAT_WITH_COMMA: Lazy<FRegex> = Lazy::new(|| FRegex::new(r"(?<![\d.])(\d+(?:\.\d{3})*),(\d+)(%)?").unwrap());
static RE_STRIP_DOT_SEP: Lazy<FRegex> = Lazy::new(|| FRegex::new(r"(?<![\d.])\d+(?:\.\d{3})+(?![\d.])").unwrap());
static RE_LONG_NUM: Lazy<FRegex> = Lazy::new(|| FRegex::new(r"(?<!\d)(?<!\d[,.])([-–—]?)(\d{7,})(?!\d)(?![.,]\d)").unwrap());
static RE_CAMEL_CASE: Lazy<FRegex> = Lazy::new(|| FRegex::new(r"(?<=[a-z])(?=[A-Z])|(?<=[A-Z])(?=[A-Z][a-z])").unwrap());
static RE_POTENTIAL_CONCAT: Lazy<Regex> = Lazy::new(|| Regex::new(r"\b[a-zA-Z]{3,}\b").unwrap());

fn cleanup_whitespace(text: &str) -> String {
    let mut res = RE_MULTI_DOT.replace_all(text, ".").into_owned();
    res = RE_MULTI_BANG.replace_all(&res, "$1").into_owned();
    res = RE_EXTRA_SPACES.replace_all(&res, " ").into_owned();
    res = RE_EXTRA_COMMAS.replace_all(&res, ",").into_owned();
    res = RE_COMMA_BEFORE_PUNCT.replace_all(&res, "$1").into_owned();
    res = RE_SPACE_BEFORE_PUNCT.replace_all(&res, "$1").into_owned();
    // Pattern now captures the char after punct; replace with "$1 $2" (insert space)
    res = RE_MISSING_SPACE_AFTER_PUNCT.replace_all(&res, "$1 $2").into_owned();
    res.trim().trim_matches(',').to_string()
}

fn expand_scores(text: &str) -> String {
    use crate::lang::vi::num2vi::n2w;
    let res = RE_SCORE_KW.replace_all(text, |caps: &Captures| {
        format!("{} {} {}",
            caps.get(1).unwrap().as_str(),
            n2w(caps.get(2).unwrap().as_str()),
            n2w(caps.get(3).unwrap().as_str()))
    }).into_owned();
    RE_SCORE_TEAMS.replace_all(&res, |caps: &Captures| {
        let team1 = caps.get(1).unwrap().as_str();
        let last = team1.rsplit(' ').next().unwrap_or(team1).to_lowercase();
        if SCORE_EXCLUDE.contains(&last.as_str()) {
            return caps.get(0).unwrap().as_str().to_string();
        }
        format!("{} {} {} {}", team1,
            n2w(caps.get(2).unwrap().as_str()),
            n2w(caps.get(3).unwrap().as_str()),
            caps.get(4).unwrap().as_str())
    }).into_owned()
}

fn expand_money_slang(text: &str) -> String {
    let res = RE_MONEY_TR.replace_all(text, |caps: &FCaps| {
        let x = crate::lang::vi::num2vi::n2w(caps.get(1).unwrap().as_str());
        match caps.get(2) {
            Some(y) => format!(" {} triệu {} trăm nghìn ", x, crate::lang::vi::num2vi::n2w(y.as_str())),
            None => format!(" {} triệu ", x),
        }
    }).into_owned();
    RE_MONEY_K.replace_all(&res, |caps: &FCaps| {
        format!(" {} nghìn ", crate::lang::vi::num2vi::n2w(caps.get(1).unwrap().as_str()))
    }).into_owned()
}

fn split_concatenated_terms(text: &str) -> String {
    let re_potential = &*RE_POTENTIAL_CONCAT;
    let re_camel = &*RE_CAMEL_CASE;
    let re_acronym = &*RE_ACRONYM;

    re_potential.replace_all(text, |caps: &Captures| {
        let word = caps.get(0).unwrap().as_str();
        // Keep known camelCase units (kWh, mAh, mWh) whole so the unit pass can
        // match them. Splitting "kWh" into "k Wh" would be read "ca wh".
        if re_acronym.is_match(word).unwrap_or(false)
            || MEASUREMENT_KEY_VI.contains_key(word.to_lowercase().as_str())
        {
            word.to_string()
        } else {
            re_camel.replace_all(word, " ").into_owned()
        }
    }).into_owned()
}

/// Fold the invisible and look-alike Unicode that arrives with text pasted from
/// Word, the web or a PDF into the plain forms the rest of the pipeline expects.
///
/// Without this step those characters reach the TTS tokenizer, become
/// out-of-vocabulary tokens, and make the model emit noise before it reads the
/// rest of the sentence (issue #177).
///
/// The mapping, and why each group is treated the way it is:
///
///  - **Zero-width characters that mark a word boundary** become a space, so
///    two syllables do not fuse into one OOV token: ZWSP (a space by origin),
///    ZWNJ ("non-joiner", i.e. keep apart) and the soft hyphen, which editors
///    place exactly at Vietnamese syllable boundaries. Extra spaces collapse
///    later anyway.
///  - **Zero-width joiners and markers** are removed outright, since fusing is
///    the correct result: ZWJ, word joiner, BOM/ZWNBSP, combining grapheme
///    joiner, Mongolian vowel separator.
///  - **Look-alike punctuation** is folded to its ASCII twin so the pattern
///    rules match: curly quotes to `'` (so "I’m" hits the English dictionary),
///    typographic hyphens to `-` (otherwise "text‐to‐speech" loses its word
///    boundaries entirely), angle brackets to parentheses.
///  - **Precomposed unit signs** expand to their two-character forms so the
///    temperature pass sees them: `℃` to `°C`, `℉` to `°F`.
///  - **Every other Unicode space** (NBSP, ogham, en/em space, narrow and
///    medium NBSP, ideographic space, line and paragraph separator, NEL,
///    vertical tab, form feed) becomes an ASCII space. `\n`, `\r` and `\t`
///    survive because later passes rely on them.
///  - **Remaining C0/C1 control characters** (NUL, BEL, ESC…) are dropped.
///
/// Characters folded here need no entry in the audit tables: by the time the
/// pipeline runs they no longer exist.
pub(crate) fn sanitize_unicode(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '\u{2018}' | '\u{2019}' => out.push('\''),
            // U+2010 HYPHEN and U+2011 NON-BREAKING HYPHEN look like ASCII '-'
            // but matched no rule, so they were deleted and the words around
            // them ran together.
            '\u{2010}' | '\u{2011}' => out.push('-'),
            '\u{27E8}' => out.push('('),
            '\u{27E9}' => out.push(')'),
            '\u{2103}' => out.push_str("°C"),
            '\u{2109}' => out.push_str("°F"),
            '\u{200B}' | '\u{200C}' | '\u{00AD}' => out.push(' '),
            '\u{200D}' | '\u{2060}' | '\u{FEFF}' | '\u{034F}' | '\u{180E}' => {}
            '\n' | '\r' | '\t' => out.push(c),
            _ if c.is_whitespace() => out.push(' '),
            _ if c.is_control() => {}
            _ => out.push(c),
        }
    }
    out
}

pub fn clean_vietnamese_text(text: &str) -> String {
    clean_vietnamese_text_ctx(text, false)
}

// Word-like tokens (letters only, at least two characters), used to decide
// whether a sentence is English.
static RE_WORDISH: Lazy<Regex> = Lazy::new(|| Regex::new(r"\b[a-zA-Z]{2,}\b").unwrap());

// Exceptions written in camelCase ("arXiv") or containing "&" ("GD&ĐT") must be
// masked BEFORE any word- or symbol-splitting pass, which would otherwise break
// them apart or turn the "&" into "và".
static RE_EARLY_EXCEPTIONS: Lazy<Option<Regex>> = Lazy::new(|| {
    let keys: Vec<String> = COMBINED_EXCEPTIONS.keys()
        .filter(|k: &&String| {
            k.contains('&')
                || k.as_bytes().windows(2).any(|w: &[u8]| {
                    (w[0] as char).is_ascii_lowercase() && (w[1] as char).is_ascii_uppercase()
                })
        })
        .map(|k: &String| regex::escape(k))
        .collect();
    if keys.is_empty() { return None; }
    Some(Regex::new(&format!(r"\b(?:{})\b", keys.join("|"))).unwrap())
});

/// Language flags derived once per input and read by every later stage.
///
/// Both are computed from the raw input, never from partially normalized text,
/// so a stage cannot accidentally change how a later stage classifies the
/// sentence.
#[derive(Clone, Copy)]
struct Ctx {
    /// The sentence contains diacritic Vietnamese letters. Paths, URLs and
    /// emails are then read the Vietnamese way ("gạch chéo", "gạch nối", and
    /// toneless syllable splitting: "thongbao" -> "thong bao").
    vi_ctx: bool,
    /// The sentence is treated as pure English: numbers and symbols are read
    /// in English ("3" -> "three", "." -> "dot").
    en_ctx: bool,
}

/// Stage 0 — classify the sentence.
///
/// English mode requires no Vietnamese diacritics **and** either three
/// word-like tokens containing lowercase letters, or two tokens that are
/// genuine dictionary English words. The two-word relaxation exists for inputs
/// such as "print 3D technology", where the digit-bearing token does not count.
/// Capitalized proper nouns never count, which keeps fragments like
/// "Arsenal 3-0 Chelsea" — and bare snippets such as "50km" or "3 x 4" — on the
/// Vietnamese path. `force_vi` (used for `<math>` content) disables English
/// mode outright.
fn stage_detect_context(text: &str, force_vi: bool) -> Ctx {
    let vi_ctx = text.chars().any(|c: char| c.is_alphabetic() && !c.is_ascii());

    let lowercase_wordish = RE_WORDISH.find_iter(text)
        .filter(|m: &regex::Match| m.as_str().chars().any(|c: char| c.is_ascii_lowercase()))
        .take(3).count();
    let dictionary_words = RE_WORDISH.find_iter(text)
        .filter(|m: &regex::Match| m.as_str().chars().all(|c: char| c.is_ascii_lowercase()))
        .filter(|m: &regex::Match| crate::lang::en::top_words::EN_TOP_WORDS.contains(m.as_str()))
        .take(2).count();

    let en_ctx = !force_vi
        && !vi_ctx
        && (lowercase_wordish >= 3 || dictionary_words >= 2);

    Ctx { vi_ctx, en_ctx }
}

/// Stage 2 — lexical rewrites that must precede every letter-digit pass.
///
/// Weekday abbreviations first: once a generic pass sees "T2" it reads it as
/// "tê hai" and the weekday is gone. English pre-normalization runs last here
/// because it consumes all digits, turning the Vietnamese numeric passes into
/// no-ops for English sentences.
fn stage_early_lexical(text: &str, ctx: Ctx) -> String {
    let mut out = expand_weekday_abbr(text);
    // "text2text" / "sale4u": a 2 or 4 wedged between lowercase letters reads
    // as "two" / "four" regardless of sentence language.
    out = crate::lang::vi::num2en::expand_sandwich_digits(&out);
    if ctx.en_ctx {
        out = crate::lang::vi::num2en::english_prenormalize(&out);
    }
    out
}

/// Stage 3 — Unicode superscripts.
///
/// Runs before `expand_symbols`, which would otherwise read each exponent
/// character on its own ("10²³" as "mười bình phương lập phương"). Order within
/// the stage matters too: unit forms (`m²`) fold to ASCII first, then signed
/// exponents (`10⁻³`), then bare runs (`10²³`) — a signed run must not be
/// consumed by the unsigned rule, which would drop the minus.
fn stage_superscripts(text: &str) -> String {
    let mut out = RE_SUP_UNIT.replace_all(text, "${1}2").into_owned();
    out = RE_SUP_UNIT3.replace_all(&out, "${1}3").into_owned();

    out = RE_SUPERSCRIPT_SIGNED.replace_all(&out, |caps: &Captures| {
        let sign = if caps.get(1).unwrap().as_str() == "\u{207b}" { "trừ " } else { "" };
        let digits = sup_digits(caps.get(2).unwrap().as_str());
        format!(" mũ {}{} ", sign, crate::lang::vi::num2vi::n2w(&digits))
    }).into_owned();

    RE_SUPERSCRIPT_RUN.replace_all(&out, |caps: &Captures| {
        let digits = sup_digits(caps.get(0).unwrap().as_str());
        format!(" mũ {} ", crate::lang::vi::num2vi::n2w(&digits))
    }).into_owned()
}

/// Stage 4 — bare formulas, i.e. maths written without `<math>` tags.
///
/// Must precede term splitting, the multiplication pass and the range pass so
/// that variable clusters ("mc²", "4ac"), factorials and binary minus signs are
/// still whole tokens. Skipped for English sentences, whose own pre-pass has
/// already rewritten the arithmetic.
fn stage_math(text: &str, ctx: Ctx) -> String {
    if ctx.en_ctx {
        return text.to_string();
    }
    let mut out = expand_inline_math(text);
    out = RE_FACTORIAL_INLINE.replace_all(&out, " giai thừa ").into_owned();
    out = RE_NEG_FUNC.replace_all(&out, " âm ").into_owned();
    // "x - a", "u - v": minus between two single lowercase variables. Ordinary
    // hyphenated words are safe because both sides must be a lone letter.
    RE_MINUS_SINGLE_VARS.replace_all(&out, "$1 trừ $2").into_owned()
}

/// Stage 6 — licence plates, identifiers and chemical coefficients.
///
/// Must precede the multiplication pass ("59X1" contains an X) and the clock
/// pass ("51H" matches the hour pattern). English sentences are left alone for
/// their own branch to handle.
fn stage_codes(text: &str, ctx: Ctx) -> String {
    if ctx.en_ctx {
        return text.to_string();
    }
    let out = crate::lang::vi::misc::expand_codes_and_plates(text);
    // Detach the coefficient in front of a chemical formula ("6CO2" -> "6 CO2")
    // so the letters take the same acronym path they would when standing alone.
    RE_CHEM_DIGIT_PREFIX.replace_all(&out, "$1 ").into_owned()
}

/// Stage 7 — arithmetic written inline.
///
/// Powers first, then multiplication, then the context-sensitive readings of a
/// dash: "trừ" when flanked by an arithmetic cue, "đến" when it spans a range.
/// Both dash rules need the operands intact, so they must follow the code stage
/// and precede the generic range pass.
fn stage_arithmetic(text: &str) -> String {
    let mut out = expand_power_of_ten(text);
    out = RE_MULTIPLY.replace_all(&out, |caps: &FCaps| {
        expand_multiply_number(caps.get(0).unwrap().as_str())
    }).to_string();

    out = RE_CONTEXT_TRU.replace_all(&out, " $1 $2 trừ $3 ").into_owned();
    out = RE_CONTEXT_TRU_POST.replace_all(&out, " $1 trừ $2 $3 ").into_owned();
    out = RE_CONTEXT_DEN.replace_all(&out, " $1 $2 đến $3 ").into_owned();

    out = RE_EQ_MINUS.replace_all(&out, |caps: &Captures| {
        format!("{} trừ {} =", caps.get(1).unwrap().as_str(), caps.get(2).unwrap().as_str())
    }).into_owned();

    RE_EQ_NEG.replace_all(&out, |caps: &Captures| {
        format!("= âm {}", caps.get(1).unwrap().as_str())
    }).into_owned()
}

/// Stage 8 — abbreviations, address prefixes and colloquial money.
///
/// Runs before the date stage because some abbreviations embed digits that the
/// date patterns would otherwise claim.
fn stage_abbreviations(text: &str) -> String {
    let mut out = crate::lang::vi::misc::expand_abbreviations(text);
    out = RE_ADDR_ABBR.replace_all(&out, |caps: &FCaps| {
        match caps.get(1).unwrap().as_str() {
            "P" => " phường ",
            "Q" => " quận ",
            _ => " đường ",
        }.to_string()
    }).into_owned();
    out = expand_money_slang(&out);
    expand_scientific_notation(&out)
}

/// Stage 9 — percentages and sport scores.
///
/// Must precede the date stage: "3-1" is a score, not the third of January.
/// The percent sign is what disambiguates a range from a fraction, so the range
/// rule keys on it explicitly.
fn stage_percent_scores(text: &str) -> String {
    let mut out = RE_RANGE_PCT.replace_all(text, "$1 đến $2%").into_owned();
    out = RE_PCT_NEG.replace_all(&out, " âm $1% ").into_owned();
    expand_scores(&out)
}

/// Stage 11 — digit runs that are identifiers rather than quantities.
///
/// Phone numbers, hotlines and dash-separated groups are read figure by figure.
/// Must run before the generic number passes, which would otherwise say
/// "one million nine hundred thousand" for a service number.
fn stage_phones(text: &str) -> String {
    use crate::lang::vi::num2vi::n2w_single;

    let mut out = RE_NUMERIC_DASH_GROUPS.replace_all(text, |caps: &Captures| {
        caps.get(0).unwrap().as_str()
            .split(&['-', '\u{2013}', '\u{2014}'][..])
            .map(n2w_single)
            .collect::<Vec<String>>()
            .join(", ")
    }).into_owned();

    out = RE_LANDLINE.replace_all(&out, |caps: &FCaps| {
        let matched = caps.get(0).unwrap().as_str();
        let groups: Vec<String> = matched
            .split(|c: char| !c.is_ascii_digit())
            .filter(|s: &&str| !s.is_empty())
            .map(n2w_single)
            .collect();
        let prefix = if matched.contains('+') { "cộng " } else { "" };
        format!(" {}{} ", prefix, groups.join(", "))
    }).into_owned();

    out = RE_PHONE_SPACE.replace_all(&out, |caps: &Captures| {
        caps.get(0).unwrap().as_str()
            .split_whitespace()
            .map(n2w_single)
            .collect::<Vec<String>>()
            .join(", ")
    }).into_owned();

    out = RE_PHONE_WITH_DASH.replace_all(&out, |caps: &Captures| {
        format!(" {}, {}, {} ",
            n2w_single(caps.get(1).unwrap().as_str()),
            n2w_single(caps.get(2).unwrap().as_str()),
            n2w_single(caps.get(3).unwrap().as_str()))
    }).into_owned();

    RE_HOTLINE.replace_all(&out, |caps: &Captures| {
        format!(" {} {} ",
            n2w_single(caps.get(1).unwrap().as_str()),
            n2w_single(caps.get(2).unwrap().as_str()))
    }).into_owned()
}

/// Stage 12 — numeric ranges and the remaining signs.
///
/// By this point dates, scores and phone numbers have consumed every dash that
/// means something else, so a dash between two numbers can safely be read as a
/// range. Digit-count similarity is the guard: "700-900" is a range, but
/// "5-1000" is more likely two separate figures.
fn stage_ranges_signs(text: &str) -> String {
    let mut out = RE_POWER_OF_TEN_IMPLICIT.replace_all(text, |caps: &Captures| {
        let exp = caps.get(1).unwrap().as_str();
        if let Some(rest) = exp.strip_prefix('-') {
            format!("mười mũ trừ {}", crate::lang::vi::num2vi::n2w(rest))
        } else {
            format!("mười mũ {}", crate::lang::vi::num2vi::n2w(&exp.replace('+', "")))
        }
    }).into_owned();

    out = RE_RANGE.replace_all(&out, |caps: &FCaps| {
        let n1_raw = caps.get(1).unwrap().as_str();
        let space_before = caps.get(2).unwrap().as_str();
        let space_after = caps.get(3).unwrap().as_str();
        let n2_raw = caps.get(4).unwrap().as_str();
        // Asymmetric spacing ("5 -3") reads as a sign, not a range.
        if !space_before.is_empty() && space_after.is_empty() {
            return caps.get(0).unwrap().as_str().to_string();
        }
        let digits = |s: &str| s.replace(',', "").replace('.', "").len() as i32;
        if (digits(n1_raw) - digits(n2_raw)).abs() <= 1 {
            format!(" {} đến {} ", n1_raw, n2_raw)
        } else {
            format!(" {} {} ", n1_raw, n2_raw)
        }
    }).to_string();

    out = RE_NEG_VAR1.replace_all(&out, " âm $1 ").into_owned();
    out = RE_NEG_VAR2.replace_all(&out, " âm $1 ").into_owned();
    out = RE_MATH_MINUS_SUP.replace_all(&out, " trừ ").into_owned();
    out = RE_MATH_MINUS_COEF.replace_all(&out, " trừ ").into_owned();
    out = RE_MATH_MINUS_COEFL.replace_all(&out, " trừ ").into_owned();
    out = RE_DASH_TO_COMMA.replace_all(&out, ",").into_owned();
    RE_TO_SANG.replace_all(&out, " sang ").into_owned()
}

/// Stage 13 — measurement units and currencies.
///
/// Runs once every sign and numeric cluster is settled, because unit rules key
/// on the number immediately to their left.
fn stage_units(text: &str) -> String {
    let mut out = expand_scientific_notation(text);
    out = expand_height_weight(&out);
    out = crate::lang::vi::misc::expand_size_labels(&out);
    out = expand_compound_units(&out);
    out = expand_units_and_currency(&out);
    // Long digit runs left over (account numbers, IDs) are read figure by figure.
    out = RE_LONG_NUM.replace_all(&out, |caps: &FCaps| {
        let sign = if caps.get(1).unwrap().as_str().is_empty() { "" } else { "âm " };
        format!(" {}{} ", sign,
            crate::lang::vi::num2vi::n2w_single(caps.get(2).unwrap().as_str()))
    }).to_string();
    fix_english_style_numbers(&out)
}

/// Stage 14 — decimal marks and thousand separators, the last numeric step.
fn stage_decimals(text: &str) -> String {
    let mut out = RE_MULTI_COMMA.replace_all(text, |caps: &Captures| {
        caps.get(1).unwrap().as_str()
            .split(',')
            .map(crate::lang::vi::num2vi::n2w_decimal)
            .collect::<Vec<String>>()
            .join(" phẩy ")
    }).into_owned();

    out = RE_FLOAT_WITH_COMMA.replace_all(&out, |caps: &FCaps| {
        let int_part = crate::lang::vi::num2vi::n2w(
            &caps.get(1).unwrap().as_str().replace('.', ""));
        let dec_part = caps.get(2).unwrap().as_str().trim_end_matches('0');
        let mut res = if dec_part.is_empty() {
            int_part
        } else {
            format!("{} phẩy {}", int_part, crate::lang::vi::num2vi::n2w_decimal(dec_part))
        };
        if caps.get(3).is_some() {
            res.push_str(" phần trăm");
        }
        format!(" {} ", res)
    }).to_string();

    RE_STRIP_DOT_SEP.replace_all(&out, |caps: &FCaps| {
        caps.get(0).unwrap().as_str().replace('.', "")
    }).to_string()
}

/// Stage 15 — whatever no earlier stage claimed.
///
/// `normalize_others` reads the remaining symbols and is also where
/// `RE_CLEAN_OTHERS` deletes anything still unrecognised — see the module-level
/// note on silent deletion and [`audit`].
fn stage_residual(text: &str, ctx: Ctx) -> String {
    let mut out = normalize_others(text, ctx.en_ctx);
    out = normalize_number_vi(&out);

    // The source already spelled "âm" while the number kept its minus sign
    // ("nhiệt độ âm -5 độ"), so a numeric pass added a second "âm". Collapse
    // the pair, but only when a number follows: "giá trị âm âm là dương" is a
    // genuine double negative and must survive.
    RE_DOUBLE_AM.replace_all(&out, |caps: &Captures| {
        let next = caps.get(1).unwrap().as_str();
        if crate::lang::vi::datestime::NUM_WORDS.contains(next.to_lowercase().as_str()) {
            format!("âm {}", next)
        } else {
            caps.get(0).unwrap().as_str().to_string()
        }
    }).into_owned()
}

pub fn clean_vietnamese_text_ctx(text: &str, force_vi: bool) -> String {
    let mut mask_map: Vec<(String, String)> = Vec::new();
    let mut current_text = text.to_string();

    let ctx = stage_detect_context(text, force_vi);
    let Ctx { vi_ctx, en_ctx } = ctx;

    let protect = |original: String, map: &mut Vec<(String, String)>| -> String {
        let idx = map.len();
        let mask = format!("mask{:0>4}mask", idx).chars().map(|c: char| {
            if c.is_ascii_digit() {
                ((c as u8 - b'0') + b'a') as char
            } else {
                c
            }
        }).collect::<String>();
        map.push((mask.clone(), original));
        mask
    };

    // ── Stage 1: protect_spans ────────────────────────────────────────────
    // Normalize each technical span to its final wording immediately, then
    // replace it with an opaque mask. Everything downstream sees a plain word,
    // so no later pass can split or rewrite a URL, path or email.

    // Placeholders left by the caller for pre-tagged English spans.
    current_text = RE_ENTOKEN.replace_all(&current_text, |caps: &Captures| {
        let orig = caps.get(0).unwrap().as_str();
        protect(orig.to_lowercase(), &mut mask_map)
    }).into_owned();

    // Emails before URLs: the email pattern is the more specific of the two.
    let temp_email = current_text.clone();
    current_text = RE_EMAIL.replace_all(&temp_email, |caps: &FCaps| {
        let orig = caps.get(0).unwrap().as_str();
        let val = normalize_emails(orig, vi_ctx, en_ctx);
        protect(val, &mut mask_map)
    }).to_string();

    let temp_tech = current_text.clone();
    current_text = RE_TECHNICAL.replace_all(&temp_tech, |caps: &FCaps| {
        let orig = caps.get(0).unwrap().as_str();
        // Hyphenated English phrases ("text-to-speech", "state-of-the-art",
        // "plug-and-play") match the technical pattern but are not identifiers:
        // ASCII letters plus hyphens, with at least one lowercase letter. Turn
        // the hyphens into spaces so G2P reads them as English words instead of
        // announcing "gạch ngang" or spelling "to" out letter by letter.
        if orig.contains('-')
            && orig.chars().all(|c: char| c.is_ascii_alphabetic() || c == '-')
            && orig.chars().any(|c: char| c.is_ascii_lowercase())
        {
            return orig.replace('-', " ");
        }
        let val = if !en_ctx && RE_ACRONYMS_EXCEPTIONS.is_match(orig) {
            COMBINED_EXCEPTIONS.get(orig).cloned().unwrap_or(orig.to_string())
        } else {
            normalize_technical(orig, vi_ctx, en_ctx)
        };
        protect(val, &mut mask_map)
    }).to_string();

    // camelCase exceptions ("arXiv") are applied and masked early so the camel
    // splitter cannot break them into "ar Xiv" — "xiv" would then hit a Roman
    // numeral entry. Other exceptions (TS., GS., B2B) stay in `normalize_others`
    // on purpose: masking them here would defeat the "TS. Nguyễn" title rule.
    if let Some(re) = RE_EARLY_EXCEPTIONS.as_ref() {
        let temp_exc = current_text.clone();
        current_text = re.replace_all(&temp_exc, |caps: &Captures| {
            let orig = caps.get(0).unwrap().as_str();
            let val = COMBINED_EXCEPTIONS.get(orig).cloned().unwrap_or(orig.to_string());
            protect(val, &mut mask_map)
        }).to_string();
    }

    current_text = stage_early_lexical(&current_text, ctx);
    current_text = stage_superscripts(&current_text);
    current_text = stage_math(&current_text, ctx);
    current_text = split_concatenated_terms(&current_text);
    current_text = stage_codes(&current_text, ctx);

    current_text = stage_arithmetic(&current_text);
    current_text = stage_abbreviations(&current_text);
    current_text = stage_percent_scores(&current_text);

    // ── Stage 10: datetime ────────────────────────────────────────────────
    current_text = normalize_date(&current_text);
    current_text = normalize_time(&current_text);

    current_text = stage_phones(&current_text);
    current_text = stage_ranges_signs(&current_text);
    current_text = stage_units(&current_text);
    current_text = stage_decimals(&current_text);
    current_text = stage_residual(&current_text, ctx);

    // ── Stage 16: letters ─────────────────────────────────────────────────
    let temp_text3 = current_text.clone();
    current_text = RE_INTERNAL_EN_TAG.replace_all(&temp_text3, |caps: &Captures| {
        protect(caps.get(0).unwrap().as_str().to_string(), &mut mask_map)
    }).into_owned();

    // English sentences keep bare letters as-is ("plan B" stays "b") so G2P can
    // read them with an English voice; Vietnamese ones get letter names.
    if !en_ctx {
        current_text = expand_standalone_letters(&current_text);
    }

    if current_text.contains('.') {
        current_text = RE_DOT_BETWEEN_DIGITS.replace_all(&current_text, |caps: &Captures| {
            format!("{} chấm {}", caps.get(1).unwrap().as_str(), caps.get(2).unwrap().as_str())
        }).into_owned();
    }

    // ── Stage 17: finalize ────────────────────────────────────────────────
    // Restore the protected spans, convert internal markers back to tags, and
    // normalize spacing. Nothing may rewrite text after this point.
    for (mask, original) in mask_map {
        current_text = current_text.replace(&mask, &original);
        current_text = current_text.replace(&mask.to_lowercase(), &original);
    }

    current_text = current_text.replace("__start_en__", "<en>").replace("__end_en__", "</en>");
    current_text = current_text.replace('_', " ").replace('-', " ");
    current_text = cleanup_whitespace(&current_text);
    current_text.to_lowercase()
}

#[pyclass]
pub struct Normalizer {
    #[pyo3(get)]
    pub lang: String,
}

#[pymethods]
impl Normalizer {
    #[new]
    #[pyo3(signature = (lang="vi", dict_path=None))]
    pub fn new(lang: &str, dict_path: Option<&str>) -> Self {
        // Load the phoneme dictionary, used to look words up when reading
        // paths, URLs and emails the Vietnamese way. A failed load is ignored:
        // the normalizer still runs off its built-in whitelist.
        if let Some(p) = dict_path {
            crate::lang::vi::technical::init_norm_dict(p);
        }
        Normalizer { lang: lang.to_string() }
    }

    /// Characters in `text` that normalization would drop without producing any
    /// spoken word.
    ///
    /// Returns an empty list when the input is fully covered. A non-empty
    /// result means those characters need either a reading or an explicit
    /// entry in the audit module's `INTENTIONALLY_DROPPED` set — see
    /// [`crate::lang::vi::audit`] for why this matters.
    pub fn audit(&self, text: &str) -> Vec<String> {
        crate::lang::vi::audit::audit_unmapped(text)
            .into_iter()
            .map(|c| c.to_string())
            .collect()
    }

    #[pyo3(signature = (text, punc_norm=false))]
    pub fn normalize(&self, text: &str, punc_norm: bool) -> String {
        if text.is_empty() { return String::new(); }

        let nfc_text: String = sanitize_unicode(text).nfc().collect();
        // Fold ellipses (… ‥ ․) to "." right away so they follow the same path
        // as "...". Any later and an earlier pass would have swallowed "…" as a
        // separator.
        let mut current_text = RE_ELLIPSIS.replace_all(&nfc_text, ".").into_owned();

        // "ln" is the natural logarithm, but nothing downstream knows that: it
        // is a vowel-less pair of Latin letters, so the G2P stage spelled it as
        // English initials, ˌɛlˈɛn. Rewriting it to "log" — which is how a
        // Vietnamese speaker reads it aloud anyway — is done here, before the
        // <math> pass, so it applies inside a formula and in running prose
        // alike ("tính ln của x").
        current_text = RE_LN.replace_all(&current_text, "log").into_owned();

        // <math> regions: split variable clusters into single letters, drop the
        // tags, and let the normal pipeline read the letters and symbols. Done
        // before <en> extraction. Formulas are always read as Vietnamese
        // (force_vi) even when they contain no diacritics.
        let had_math = current_text.to_lowercase().contains("<math>");
        if had_math {
            current_text = RE_MATH_TAG.replace_all(&current_text, |caps: &Captures| {
                let inner = split_math_letters(caps.get(1).unwrap().as_str());
                let inner = RE_MATH_FACTORIAL.replace_all(&inner, " giai thừa ").into_owned();
                let inner = RE_MATH_BIN_MINUS.replace_all(&inner, " trừ ").into_owned();
                let inner = RE_LEAD_NEG.replace(inner.trim_start(), "âm $1");
                format!(" {} ", inner)
            }).into_owned();
        }

        let mut en_contents = Vec::new();
        let placeholder_pattern = "ENTOKEN{}";

        let temp_text = current_text.clone();
        current_text = RE_EN_TAG.replace_all(&temp_text, |caps: &Captures| {
            en_contents.push(caps.get(0).unwrap().as_str().to_string());
            placeholder_pattern.replace("{}", &en_contents.len().saturating_sub(1).to_string())
        }).into_owned();

        current_text = clean_vietnamese_text_ctx(&current_text, had_math);

        current_text = RE_EXTRA_SPACES.replace_all(&current_text, " ").trim().to_string();

        if !en_contents.is_empty() {
            for (idx, content) in en_contents.iter().enumerate() {
                let placeholder = placeholder_pattern.replace("{}", &idx.to_string()).to_lowercase();
                current_text = current_text.replace(&placeholder, content);
            }
        }

        let result = RE_EXTRA_SPACES.replace_all(&current_text, " ").trim().to_string();

        if punc_norm {
            crate::punc::apply_punc_norm(&result)
        } else {
            result
        }
    }

    #[pyo3(signature = (texts, punc_norm=false))]
    pub fn normalize_batch(&self, py: Python<'_>, texts: Vec<String>, punc_norm: bool) -> PyResult<Vec<String>> {
        py.allow_threads(|| {
            use rayon::prelude::*;
            Ok(texts.into_par_iter().map(|t| self.normalize(&t, punc_norm)).collect())
        })
    }
}
