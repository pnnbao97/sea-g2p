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

use pyo3::prelude::*;
// fancy_regex only for patterns requiring look-arounds
use fancy_regex::{Regex as FRegex, Captures as FCaps};
// regex crate for simple patterns (Thompson NFA - much faster than fancy_regex backtracker)
use regex::{Regex, Captures};
use once_cell::sync::Lazy;
use unicode_normalization::UnicodeNormalization;
use crate::vi_normalizer::numerical::{normalize_number_vi, RE_MULTIPLY, expand_multiply_number};
use crate::vi_normalizer::datestime::{normalize_date, normalize_time};
use crate::vi_normalizer::units::{expand_units_and_currency, expand_compound_units, expand_scientific_notation, fix_english_style_numbers, expand_power_of_ten, expand_height_weight};
use crate::vi_normalizer::misc::{normalize_others, expand_standalone_letters, expand_weekday_abbr, RE_ACRONYMS_EXCEPTIONS, RE_ACRONYM};
use crate::vi_normalizer::technical::{normalize_technical, normalize_emails, RE_TECHNICAL, RE_EMAIL};
use crate::vi_normalizer::resources::{COMBINED_EXCEPTIONS, MEASUREMENT_KEY_VI};

// ── Tier 1: regex crate (Thompson NFA, much faster for simple patterns) ────
static RE_EXTRA_SPACES: Lazy<Regex> = Lazy::new(|| Regex::new(r"[ \t\xA0]+").unwrap());
static RE_EXTRA_COMMAS: Lazy<Regex> = Lazy::new(|| Regex::new(r",\s*,").unwrap());
// Ellipsis một-ký-tự: ․(U+2024) ‥(U+2025) …(U+2026) -> "." để đi chung đường
// với "..." (RE_MULTI_DOT gộp tiếp về một dấu chấm).
static RE_ELLIPSIS: Lazy<Regex> = Lazy::new(|| Regex::new(r"[\u{2024}\u{2025}\u{2026}]").unwrap());
static RE_MULTI_DOT: Lazy<Regex> = Lazy::new(|| Regex::new(r"\.[\s.]*\.").unwrap());
// Gộp chuỗi dấu kết câu lặp/hỗn hợp ("!!!", "???", "?!", "!?!?") về một dấu đầu tiên.
static RE_MULTI_BANG: Lazy<Regex> = Lazy::new(|| Regex::new(r"([!?])[!?\s]*[!?]").unwrap());
static RE_COMMA_BEFORE_PUNCT: Lazy<Regex> = Lazy::new(|| Regex::new(r",\s*([.!?;])").unwrap());
static RE_SPACE_BEFORE_PUNCT: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s+([,.!?;:])").unwrap());
// Rewritten to avoid lookahead: capture the following char so regex crate can handle it
static RE_MISSING_SPACE_AFTER_PUNCT: Lazy<Regex> = Lazy::new(|| Regex::new(r"([.,!?;:])([^\s\d<])").unwrap());
static RE_INTERNAL_EN_TAG: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)(?s)(__start_en__.*?__end_en__|<en>.*?</en>)").unwrap());
static RE_DOT_BETWEEN_DIGITS: Lazy<Regex> = Lazy::new(|| Regex::new(r"(\d+)\.(\d+)").unwrap());
// "âm âm năm" (văn bản có sẵn "âm" + số mang dấu trừ) -> "âm năm".
static RE_DOUBLE_AM: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)\bâm\s+âm\s+(\S+)").unwrap());
static RE_ENTOKEN: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)ENTOKEN\d+").unwrap());
static RE_EN_TAG: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?si)<en>.*?</en>").unwrap());
// Vùng công thức toán do người dùng khai báo: <math>...</math>. Bên trong, mọi cụm
// chữ cái (trừ tên hàm) được tách thành chữ rời để đọc tên chữ (4ac -> bốn a xê,
// dx -> đê ích). Phần còn lại để pipeline thường xử lý (ký hiệu, số mũ, căn...).
static RE_MATH_TAG: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?si)<math>(.*?)</math>").unwrap());
static RE_MATH_WORD: Lazy<Regex> = Lazy::new(|| Regex::new(r"[a-zA-Z]+").unwrap());
// Dấu trừ đứng ĐẦU công thức (-a + b) -> "âm a ...".
static RE_LEAD_NEG: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[-–—]\s*([a-zA-Z0-9])").unwrap());
// Trong <math>: giai thừa "n!"/"5!"/"(n+1)!" -> "... giai thừa".
static RE_MATH_FACTORIAL: Lazy<FRegex> = Lazy::new(|| FRegex::new(r"(?<=[0-9A-Za-z)])!").unwrap());
// Trong <math>: dấu trừ NHỊ PHÂN giữa hai toán hạng (c-d, a-b, 5-3) -> "trừ".
// Lookbehind/lookahead nên không nuốt toán hạng -> xử lý được chuỗi "a-b-c".
// Đứng sau "(" / "=" thì KHÔNG khớp -> để nhánh đơn nguyên đọc "âm".
static RE_MATH_BIN_MINUS: Lazy<FRegex> = Lazy::new(|| FRegex::new(r"(?<=[0-9A-Za-z)])\s*[-–—]\s*(?=[0-9A-Za-z(∫√])").unwrap());
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

// ─ Công thức "trần" (không bọc <math>) ────────────────────────────────────
// Nhận diện CỤM token dạng toán trong câu thường rồi áp cùng bộ xử lý như
// vùng <math>: tách cụm biến ("mc²" -> "m c²", "4ac" -> "4 a c"), giai thừa,
// trừ nhị phân. Điều kiện chặt để không đụng văn xuôi:
//  - mọi token trong cụm chỉ gồm chữ số/toán tử/ngoặc/Hy Lạp/mũ-chỉ số,
//    cụm chữ ASCII bên trong dài ≤3 hoặc là tên hàm;
//  - token THUẦN CHỮ ≥2 ký tự ("ma", "khi") chỉ được nhận khi đứng cạnh
//    token chứa toán tử ("F = ma" nhận "ma"; "khi x = 1" loại "khi");
//  - cụm phải chứa ít nhất MỘT dấu toán mạnh: = √ ∫ ± ≤ ≥ ≠ hoặc mũ/chỉ số.
fn is_strong_math_char(c: char) -> bool {
    matches!(c, '=' | '√' | '∫' | '±' | '≤' | '≥' | '≠')
        || crate::vi_normalizer::resources::SUPERSCRIPTS_MAP.contains_key(&c)
        || crate::vi_normalizer::resources::SUBSCRIPTS_MAP.contains_key(&c)
}

// KHÔNG tính "/" và √ ∫ là toán tử NGỮ CẢNH: phân số/căn nằm trong token
// ("Σ(1/2ⁿ)", "1/√3,") đứng cạnh từ Việt không dấu ("khi", "ta") sẽ kéo nhầm
// từ đó vào cụm công thức. (√ ∫ vẫn là bằng chứng toán BÊN TRONG token.)
fn has_operator_char(tok: &str) -> bool {
    tok.chars().any(|c| matches!(c, '=' | '+' | '-' | '–' | '—' | '±' | '*' | '×' | '÷'))
}

// Vi phân dx/dy/du/dv/dt: luôn coi là token toán (kể cả khi đứng cạnh token
// không có toán tử: "∫sin x dx") nhưng vẫn TÁCH chữ khi đọc ("đê ích").
static MATH_DIFFS: Lazy<std::collections::HashSet<&'static str>> = Lazy::new(|| {
    ["dx", "dy", "dz", "du", "dv", "dt"].into_iter().collect()
});

/// (đạt chuẩn ký tự toán, là token thuần chữ ≥2 ký tự cần xét ngữ cảnh)
fn classify_math_token(tok: &str) -> (bool, bool) {
    if tok.is_empty() { return (false, false); }
    let allowed = |c: char| {
        c.is_ascii_alphanumeric()
            || "()[]{}+-–—*/=±≤≥≠≈×÷√∫|!'^.,;:°½¼¾⅓⅔∆∑".contains(c)
            || ('α'..='ω').contains(&c) || ('Α'..='Ω').contains(&c)
            || crate::vi_normalizer::resources::SUPERSCRIPTS_MAP.contains_key(&c)
            || crate::vi_normalizer::resources::SUBSCRIPTS_MAP.contains_key(&c)
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
    // Thuần chữ (không có CHỮ SỐ/toán tử/ký hiệu toán trong token — dấu câu
    // đuôi như "thi," không được tính là bằng chứng), nhiều ký tự, không phải
    // tên hàm/vi phân -> dễ trùng từ thật, cần toán tử đứng cạnh mới nhận.
    let has_math_evidence = tok.chars().any(|c| {
        c.is_ascii_digit()
            || "+-–—*/=±≤≥≠≈×÷√∫^'!½¼¾⅓⅔∆∑".contains(c)
            || ('α'..='ω').contains(&c) || ('Α'..='Ω').contains(&c)
            || crate::vi_normalizer::resources::SUPERSCRIPTS_MAP.contains_key(&c)
            || crate::vi_normalizer::resources::SUBSCRIPTS_MAP.contains_key(&c)
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
            // Cụm 1 token chỉ nhận khi có dấu mạnh KHÁC "=" ("½mv²", "eˣ"):
            // "id=1;"/"x=5" quá ít bằng chứng, "id" phải giữ nguyên cho G2P.
            let strong_non_eq = span.chars().any(|c| c != '=' && is_strong_math_char(c));
            if (j - i >= 2 && strong) || (j - i == 1 && strong_non_eq) {
                let mut parts: Vec<String> = Vec::with_capacity(j - i);
                for (k, t) in toks[i..j].iter().enumerate() {
                    // Cụm chữ là ĐƠN VỊ ĐO đứng sau con số ("170 km²") hoặc
                    // trong token đơn lẻ ("km²") -> giữ nguyên cho pass đơn vị;
                    // sau "=" thì là tích biến ("= mg" -> "m g").
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
// L\u01b0u \u00fd: c\u00e1c t\u1eeb ng\u1eef c\u1ea3nh ph\u1ea3i l\u00e0 k\u00fd t\u1ef1 Vi\u1ec7t th\u1eadt. KH\u00d4NG d\u00f9ng byte-escape
// `\xHH` trong raw string `r"..."` \u2014 raw string kh\u00f4ng x\u1eed l\u00fd escape n\u00ean engine
// hi\u1ec3u `\xe1` l\u00e0 codepoint U+00E1, bi\u1ebfn "b\u1eb1ng" th\u00e0nh mojibake kh\u00f4ng bao gi\u1edd kh\u1edbp.
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
// Khoảng phần trăm "5-7%" -> "5 đến 7%" (dấu % xác nhận đây là KHOẢNG, không phải
// phân số/ngày-tháng). Chạy trước normalize_date để không bị đọc thành "trên".
static RE_RANGE_PCT: Lazy<Regex> = Lazy::new(|| Regex::new(r"(\d+(?:[.,]\d+)?)\s*[-–—]\s*(\d+(?:[.,]\d+)?)\s*%").unwrap());
// Phần trăm ÂM: "-5%" -> "âm 5%" (giữ % để pass đơn vị đọc "phần trăm"). Lookbehind
// (?<![\d.,]) đảm bảo dấu trừ là đơn nguyên, không phải hiệu (khoảng "10-5%" đã được
// RE_RANGE_PCT xử lý trước đó).
static RE_PCT_NEG: Lazy<FRegex> = Lazy::new(|| FRegex::new(r"(?<![\d.,])[-–—]\s?(\d+(?:[.,]\d+)?)\s*%").unwrap());
// Viết tắt địa chỉ: "P.5"->"phường 5", "Q.1"->"quận 1", "Đ.3/2"->"đường 3/2".
// Yêu cầu ngay sau là CHỮ SỐ -> tránh nhầm "P.S." hay tên viết tắt.
static RE_ADDR_ABBR: Lazy<FRegex> = Lazy::new(|| FRegex::new(r"\b([PQĐ])\.\s*(?=\d)").unwrap());
// Tỉ số thể thao: đọc thành hai số RỜI ("2-1" -> "hai một"), không phải khoảng
// ("đến") hay phân số ("trên"). Nhận diện qua: (a) từ khóa tỉ số đứng NGAY trước,
// (b) cụm số nằm giữa hai TÊN RIÊNG viết hoa (đội bóng).
static RE_SCORE_KW: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)\b(thắng|thua|hòa|hoà|tỉ số|tỷ số|chung cuộc|đánh bại|cầm hòa|cầm hoà)\s+(\d{1,2})\s*[-–—]\s*(\d{1,2})\b").unwrap());
static RE_SCORE_TEAMS: Lazy<Regex> = Lazy::new(|| Regex::new(r"(\p{Lu}\p{L}+(?:\s\p{Lu}\p{L}+)?)\s(\d{1,2})\s*[-–—]\s*(\d{1,2})\s(\p{Lu}\p{L}+)").unwrap());
// Tiền tố cấu trúc/khoảng đứng trước "số-số" -> KHÔNG phải tỉ số (giữ logic khoảng).
const SCORE_EXCLUDE: [&str; 26] = [
    "điều", "khoản", "chương", "mục", "phần", "điểm", "tiết", "tập", "quyển",
    "hồi", "kỳ", "kì", "quý", "tháng", "ngày", "năm", "tuần", "từ", "khoảng",
    "trong", "bài", "câu", "trang", "dòng", "khóa", "khoá",
];

// Tổng đài 1800/1900: đọc rời từng chữ số thay vì gộp thành số lớn.
static RE_HOTLINE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\b(1800|1900)[\s.\-–—]?(\d{3,6})\b").unwrap());
// Số điện thoại bàn: mã vùng (có/không ngoặc) + 2 nhóm 3-4 chữ số ngăn bởi KHOẢNG
// TRẮNG -> đọc rời từng số. vd "(028) 3822 1234", "024 3822 1234", "+84 28 3822 1234".
// Lookbehind (?<![\d.,]) chặn khớp giữa số phân tách bằng dấu chấm (1.000.000.000).
static RE_LANDLINE: Lazy<FRegex> = Lazy::new(|| FRegex::new(r"(?<![\d.,])\(?(?:\+84\s?|0)\d{1,3}\)?\s\d{3,4}\s\d{3,4}(?!\d)").unwrap());

// Tiền lóng: "500k" -> năm trăm nghìn, "1tr"/"15tr" -> triệu, "1tr5" -> một triệu
// năm trăm nghìn. CHỈ chữ thường k/tr và ≤4 chữ số -> tránh nhầm hậu tố model
// viết HOA ("i9-14900K", "RTX") và tránh nuốt "5kg"/"5km"/"4trung" (lookahead (?!\w)).
static RE_MONEY_K: Lazy<FRegex> = Lazy::new(|| FRegex::new(r"\b(\d{1,4})k(?![\w])").unwrap());
static RE_MONEY_TR: Lazy<FRegex> = Lazy::new(|| FRegex::new(r"\b(\d{1,4})tr(\d)?(?![\w])").unwrap());

// ── Tier 2: fancy_regex (REQUIRED for look-around assertions) ────────────────
// RE_COMBINED_TECH_EMAIL removed — two separate passes are faster (mirrors Python)
// Lookahead cu\u1ed1i `(?![a-zA-Z])`: s\u1ed1 th\u1ee9 hai D\u00cdNH ch\u1eef ("5 - 2i", "1 - 2sin\u00b2x")
// l\u00e0 ph\u00e9p TR\u1eea trong c\u00f4ng th\u1ee9c, kh\u00f4ng ph\u1ea3i kho\u1ea3ng -> nh\u01b0\u1eddng cho RE_MATH_MINUS_COEF.
static RE_RANGE: Lazy<FRegex> = Lazy::new(|| FRegex::new(r"(?<!\d)(?<!\d[,.])(?<![a-zA-Z])(\d{1,15}(?:[,.]\d{1,15})?)(\s*)[\u2013\-\u2014](\s*)(\d{1,15}(?:[,.]\d{1,15})?)(?!\d)(?![.,]\d)(?![a-zA-Z])").unwrap());
static RE_DASH_TO_COMMA: Lazy<FRegex> = Lazy::new(|| FRegex::new(r"(?<=\s)[\u2013\-\u2014](?=\s)").unwrap());
// D\u1ea5u tr\u1eeb trong C\u00d4NG TH\u1ee8C: " - " -> " tr\u1eeb " khi v\u1ebf tr\u00e1i k\u1ebft th\u00fac b\u1eb1ng s\u1ed1 m\u0169
// (b\u00b2 - 4ac) ho\u1eb7c v\u1ebf ph\u1ea3i l\u00e0 h\u1ec7 s\u1ed1 d\u00ednh s\u1ed1+ch\u1eef (2x - 3y). B\u1ea3o th\u1ee7 \u0111\u1ec3 kh\u00f4ng \u0111\u1ee5ng
// g\u1ea1ch ngang trong v\u0103n xu\u00f4i ("H\u00e0 N\u1ed9i - th\u1ee7 \u0111\u00f4" gi\u1eef nguy\u00ean th\u00e0nh d\u1ea5u ph\u1ea9y).
static RE_MATH_MINUS_SUP: Lazy<FRegex> = Lazy::new(|| FRegex::new(r"(?<=[\u00b2\u00b3\u2074\u2075\u2076\u2077\u2078\u2079\u207f\u2071])\s*[-\u2013\u2014]\s*").unwrap());
static RE_MATH_MINUS_COEF: Lazy<FRegex> = Lazy::new(|| FRegex::new(r"\s[-\u2013\u2014]\s(?=\d+[a-zA-Z])").unwrap());
static RE_MATH_MINUS_COEFL: Lazy<FRegex> = Lazy::new(|| FRegex::new(r"(?<=\d[a-zA-Z])\s[-\u2013\u2014]\s(?=\d)").unwrap());
// D\u1ea5u tr\u1eeb \u0110\u01a0N NGUY\u00caN tr\u01b0\u1edbc bi\u1ebfn ch\u1eef (-b, =-x, (-y) -> "\u00e2m b"). Ch\u1ec9 kh\u1edbp khi \u0111\u1ee9ng
// sau to\u00e1n t\u1eed/ngo\u1eb7c/b\u1eb1ng (kh\u00f4ng ph\u1ea3i sau to\u00e1n h\u1ea1ng) -> kh\u00f4ng \u0111\u1ee5ng "a - b" (tr\u1eeb).
// Nh\u1eadn c\u1ea3 T\u00caN H\u00c0M ("-sin x" -> "\u00e2m sin x"); ch\u1eef \u0111\u01a1n ph\u1ea3i tr\u1ecdn v\u1eb9n (\b) \u0111\u1ec3
// "-sin" kh\u00f4ng b\u1ecb x\u00e9 th\u00e0nh "\u00e2m s" + "in".
static RE_NEG_VAR1: Lazy<FRegex> = Lazy::new(|| FRegex::new(r"(?<=[(\[{=\u00b1+*/\u00d7\u00f7])[-\u2013\u2014]((?:sin|cos|tan|cot|log|ln|lim)\b|[a-zA-Z]\b)").unwrap());
static RE_NEG_VAR2: Lazy<FRegex> = Lazy::new(|| FRegex::new(r"(?<=[(\[{=\u00b1+*/\u00d7\u00f7]\s)[-\u2013\u2014]((?:sin|cos|tan|cot|log|ln|lim)\b|[a-zA-Z]\b)").unwrap());
// "-sin x" gi\u1eefa c\u00e2u v\u0103n ("l\u00e0 -sin x"): d\u1ea5u tr\u1eeb \u0111\u1ee9ng sau kho\u1ea3ng tr\u1eafng, ngay
// tr\u01b0\u1edbc t\u00ean h\u00e0m l\u01b0\u1ee3ng gi\u00e1c -> "\u00e2m sin".
static RE_NEG_FUNC: Lazy<FRegex> = Lazy::new(|| FRegex::new(r"(?<=\s)[-\u2013\u2014](?=(?:sin|cos|tan|cot|log|ln|lim)\b)").unwrap());
// Giai th\u1eeba ngo\u00e0i <math>: "5! = 120", "n!/(", "O(n!)". Ch\u1ec9 khi "!" \u0111\u1ee9ng sau
// M\u1ed8T k\u00fd t\u1ef1 alnum tr\u1ecdn v\u1eb9n (token 1 k\u00fd t\u1ef1) v\u00e0 tr\u01b0\u1edbc k\u00fd t\u1ef1 to\u00e1n -> kh\u00f4ng \u0111\u1ee5ng
// c\u00e2u c\u1ea3m th\u00e1n ("tuy\u1ec7t!" c\u00f3 ch\u1eef c\u00e1i li\u1ec1n tr\u01b0\u1edbc n\u00ean \b kh\u00f4ng kh\u1edbp).
static RE_FACTORIAL_INLINE: Lazy<FRegex> = Lazy::new(|| FRegex::new(r"(?<=\b[0-9a-zA-Z])!(?=\s*[)=/,(+\u00d7*-])").unwrap());
// C\u1ee5m m\u0169 Unicode li\u00ean ti\u1ebfp ("10\u00b2\u00b3") l\u00e0 M\u1ed8T s\u1ed1 m\u0169 ("m\u01b0\u1eddi m\u0169 hai m\u01b0\u01a1i ba"),
// kh\u00f4ng ph\u1ea3i chu\u1ed7i "b\u00ecnh ph\u01b0\u01a1ng l\u1eadp ph\u01b0\u01a1ng".
static RE_SUPERSCRIPT_RUN: Lazy<Regex> = Lazy::new(|| Regex::new(r"[\u2070\u00b9\u00b2\u00b3\u2074\u2075\u2076\u2077\u2078\u2079]{2,}").unwrap());
// S\u1ed1 m\u0169 c\u00f3 D\u1ea4U ("10\u207b\u00b3", "2\u207b\u00b9", "10\u207a\u2076"): d\u1ea5u \u207b (U+207B) tr\u01b0\u1edbc \u0111\u00e2y b\u1ecb nu\u1ed1t n\u00ean
// "10\u207b\u00b3" \u0111\u1ecdc th\u00e0nh "m\u01b0\u1eddi l\u1eadp ph\u01b0\u01a1ng". Ph\u1ea3i ch\u1ea1y TR\u01af\u1edaC RE_SUPERSCRIPT_RUN.
static RE_SUPERSCRIPT_SIGNED: Lazy<Regex> = Lazy::new(|| Regex::new(r"([\u207b\u207a])([\u2070\u00b9\u00b2\u00b3\u2074\u2075\u2076\u2077\u2078\u2079]+)").unwrap());

fn sup_digits(s: &str) -> String {
    s.chars().map(|c| match c {
        '\u{2070}' => '0', '\u{00b9}' => '1', '\u{00b2}' => '2', '\u{00b3}' => '3', '\u{2074}' => '4',
        '\u{2075}' => '5', '\u{2076}' => '6', '\u{2077}' => '7', '\u{2078}' => '8', _ => '9',
    }).collect()
}
// \u0110\u01a1n v\u1ecb di\u1ec7n t\u00edch/th\u1ec3 t\u00edch vi\u1ebft m\u0169 Unicode ("68 m\u00b2", "170 km\u00b3") quy v\u1ec1 d\u1ea1ng
// ASCII ("m2", "km3") \u0111\u1ec3 kh\u1edbp b\u1ea3ng \u0111\u01a1n v\u1ecb -> "m\u00e9t vu\u00f4ng"/"m\u00e9t kh\u1ed1i" thay v\u00ec
// "m\u00e9t b\u00ecnh ph\u01b0\u01a1ng". Ch\u1ec9 \u00e1p cho CH\u1eee \u0110\u01a0N V\u1eca th\u1eadt, kh\u00f4ng \u0111\u1ee5ng bi\u1ebfn c\u00f4ng th\u1ee9c
// ("r\u00b2", "c\u00b2" v\u1eabn \u0111\u1ecdc "b\u00ecnh ph\u01b0\u01a1ng").
static RE_SUP_UNIT: Lazy<Regex> = Lazy::new(|| Regex::new(r"\b(km|cm|mm|m)\u00b2").unwrap());
static RE_SUP_UNIT3: Lazy<Regex> = Lazy::new(|| Regex::new(r"\b(km|cm|mm|m)\u00b3").unwrap());
// H\u1ec7 s\u1ed1 \u0111\u1ee9ng TR\u01af\u1edaC c\u00f4ng th\u1ee9c h\u00f3a h\u1ecdc ("6CO2", "2HCl", "2H2O"): t\u00e1ch r\u1eddi s\u1ed1
// \u0111\u1ec3 ph\u1ea7n ch\u1eef \u0111i theo nh\u00e1nh acronym ("x\u00ea \u00f4 hai"...). \u0110i\u1ec1u ki\u1ec7n ch\u1eb7t:
//  - h\u1ec7 s\u1ed1 \u0111\u00fang M\u1ed8T ch\u1eef s\u1ed1 (kh\u00f4ng \u0111\u1ee5ng "11T14" trong ISO, "14H30");
//  - c\u1ee5m ch\u1eef sau \u0111\u00f3 ph\u1ea3i ch\u1ee9a ch\u1eef TH\u01af\u1edcNG (Cl, Na) ho\u1eb7c CH\u1eee S\u1ed0 (H2, O2, CO2)
//    -> lo\u1ea1i "2FA", "3D", "1TB" (to\u00e0n ch\u1eef HOA tr\u01a1n);
//  - kh\u00f4ng ph\u1ea3i d\u1ea1ng gi\u1edd "4H30" (H + \u0111\u00fang hai ch\u1eef s\u1ed1 cu\u1ed1i t\u1eeb).
static RE_CHEM_DIGIT_PREFIX: Lazy<FRegex> = Lazy::new(|| {
    FRegex::new(r"\b(\d)(?!H\d{2}\b)(?=[A-Z][a-z]|[A-Z][A-Z]?\d|[A-Z][A-Z][a-z])").unwrap()
});
// Trừ giữa hai biến chữ thường ĐƠN LẺ có khoảng trắng ("x - a", "u - v").
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
    use crate::vi_normalizer::num2vi::n2w;
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
        let x = crate::vi_normalizer::num2vi::n2w(caps.get(1).unwrap().as_str());
        match caps.get(2) {
            Some(y) => format!(" {} triệu {} trăm nghìn ", x, crate::vi_normalizer::num2vi::n2w(y.as_str())),
            None => format!(" {} triệu ", x),
        }
    }).into_owned();
    RE_MONEY_K.replace_all(&res, |caps: &FCaps| {
        format!(" {} nghìn ", crate::vi_normalizer::num2vi::n2w(caps.get(1).unwrap().as_str()))
    }).into_owned()
}

fn split_concatenated_terms(text: &str) -> String {
    let re_potential = &*RE_POTENTIAL_CONCAT;
    let re_camel = &*RE_CAMEL_CASE;
    let re_acronym = &*RE_ACRONYM;

    re_potential.replace_all(text, |caps: &Captures| {
        let word = caps.get(0).unwrap().as_str();
        // Giữ nguyên đơn vị viết camelCase đã biết (kWh, mAh, mWh...) để pass đơn vị
        // bắt được; nếu để splitter cắt "kWh" -> "k Wh" sẽ đọc sai ("ca wh").
        if re_acronym.is_match(word).unwrap_or(false)
            || MEASUREMENT_KEY_VI.contains_key(word.to_lowercase().as_str())
        {
            word.to_string()
        } else {
            re_camel.replace_all(word, " ").into_owned()
        }
    }).into_owned()
}

/// Dọn ký tự Unicode "ẩn" thường lẫn trong văn bản dán từ Word/web/PDF trước khi
/// chuẩn hóa. Nếu không dọn, chúng lọt tới tokenizer của TTS -> token OOV -> mô hình
/// sinh ra "tiếng lạ" trước khi đọc phần còn lại (issue #177).
///  - Zero-width mang tính RANH GIỚI TỪ -> đổi thành dấu cách để không dính 2 âm tiết
///    thành token OOV (và tránh văn bản kiểu "thànhphố"): ZWSP (vốn là "space"),
///    ZWNJ ("non-joiner" = tách rời), soft hyphen (điểm ngắt của trình soạn thảo,
///    thường rơi đúng ranh giới âm tiết tiếng Việt). Dấu cách thừa sẽ được gộp lại sau.
///  - Zero-width mang tính NỐI/đánh dấu -> bỏ hẳn (gộp lại là đúng): ZWJ ("joiner"),
///    word joiner, BOM/ZWNBSP, combining grapheme joiner, Mongolian vowel separator.
///  - Đổi MỌI khoảng trắng Unicode lạ (NBSP, ogham, en/em space, narrow/medium NBSP,
///    ideographic space, line/paragraph separator, NEL, vertical tab, form feed) về
///    dấu cách ASCII thường. Giữ nguyên '\n' '\r' '\t' vì các pass sau còn dựa vào.
///  - Bỏ ký tự điều khiển C0/C1 còn lại (NUL, BELL, ESC...).
fn sanitize_unicode(text: &str) -> String {
    text.chars().filter_map(|c: char| match c {
        // Nháy cong (smart quotes bàn phím/Word) -> nháy thẳng ASCII để contraction
        // tiếng Anh ("I’m" -> "I'm") khớp dict và các rule apostrophe phía sau.
        '\u{2018}' | '\u{2019}' => Some('\''),
        '\u{200B}' | '\u{200C}' | '\u{00AD}' => Some(' '),
        '\u{200D}' | '\u{2060}' | '\u{FEFF}' | '\u{034F}' | '\u{180E}' => None,
        '\n' | '\r' | '\t' => Some(c),
        _ if c.is_whitespace() => Some(' '),
        _ if c.is_control() => None,
        _ => Some(c),
    }).collect()
}

pub fn clean_vietnamese_text(text: &str) -> String {
    clean_vietnamese_text_ctx(text, false)
}

// Token "trông như từ" (thuần chữ cái, >=2 ký tự) — dùng nhận diện câu tiếng Anh.
static RE_WORDISH: Lazy<Regex> = Lazy::new(|| Regex::new(r"\b[a-zA-Z]{2,}\b").unwrap());

// Exception dạng camelCase ("arXiv") hoặc chứa "&" ("GD&ĐT") — phải mask
// TRƯỚC các pass tách từ/ký hiệu kẻo bị xé đôi hoặc "&" bị đổi thành "và".
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

pub fn clean_vietnamese_text_ctx(text: &str, force_vi: bool) -> String {
    let mut mask_map: Vec<(String, String)> = Vec::new();
    let mut current_text = text.to_string();

    // Câu chứa chữ tiếng Việt có dấu -> path/URL/email đọc kiểu Việt
    // ("gạch chéo", "gạch nối", tách âm tiết không dấu "thongbao" -> "thong bao").
    let vi_ctx = text.chars().any(|c: char| c.is_alphabetic() && !c.is_ascii());
    // Câu KHÔNG có chữ Việt có dấu VÀ có ít nhất 3 từ tiếng Anh thực (có chữ
    // thường, loại acronym USD/VND) -> coi là câu tiếng Anh: số/ký hiệu đọc
    // kiểu Anh ("3" -> "three", "." -> "dot"). Ngưỡng này giữ các mẩu trơ
    // ("50km", "3 x 4", "Arsenal 3-0 Chelsea", công thức toán) ở lại đường
    // tiếng Việt. force_vi (nội dung <math>) tắt hẳn chế độ Anh.
    // Nới ngưỡng: chỉ cần 2 từ nếu CẢ HAI thuần chữ thường VÀ là từ tiếng Anh
    // thật trong wordlist ("print 3D technology" chỉ có 2 từ đếm được vì "3D"
    // lẫn chữ số). Tên riêng viết Hoa không được tính nên "Arsenal 3-0 Chelsea"
    // vẫn ở lại đường tiếng Việt.
    let en_dict_words = RE_WORDISH.find_iter(text)
        .filter(|m: &regex::Match| m.as_str().chars().all(|c: char| c.is_ascii_lowercase()))
        .filter(|m: &regex::Match| crate::g2p::en_top_words::EN_TOP_WORDS.contains(m.as_str()))
        .take(2).count();
    let en_ctx = !force_vi && !vi_ctx
        && (RE_WORDISH.find_iter(text)
            .filter(|m: &regex::Match| m.as_str().chars().any(|c: char| c.is_ascii_lowercase()))
            .take(3).count() >= 3
            || en_dict_words >= 2);

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

    // Protect ENTOKEN placeholders (if any)
    current_text = RE_ENTOKEN.replace_all(&current_text, |caps: &Captures| {
        let orig = caps.get(0).unwrap().as_str();
        protect(orig.to_lowercase(), &mut mask_map)
    }).into_owned();

    // Protect emails first (simple pattern, matches @)
    let temp_email = current_text.clone();
    current_text = RE_EMAIL.replace_all(&temp_email, |caps: &FCaps| {
        let orig = caps.get(0).unwrap().as_str();
        let val = normalize_emails(orig, vi_ctx, en_ctx);
        protect(val, &mut mask_map)
    }).to_string();

    // Protect technical strings (URLs, paths, etc.) separately
    let temp_tech = current_text.clone();
    current_text = RE_TECHNICAL.replace_all(&temp_tech, |caps: &FCaps| {
        let orig = caps.get(0).unwrap().as_str();
        // Cụm từ tiếng Anh nối bằng gạch ngang (text-to-speech, state-of-the-art,
        // plug-and-play): chỉ gồm chữ cái ASCII + '-' và có ít nhất một chữ thường
        // -> KHÔNG phải định danh kỹ thuật. Tách thành các từ (gạch -> space) để
        // G2P đọc như tiếng Anh, không đọc "gạch ngang" hay spell "to" -> "t o".
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

    // Áp SỚM và mask các exception dạng camelCase ("arXiv" -> "arxiv") để camel
    // splitter không kịp xé thành "ar Xiv" ("xiv" dính entry số La Mã trong
    // dict). Các exception khác (TS., GS., B2B...) vẫn xử ở normalize_others
    // như cũ — mask sớm chúng sẽ phá rule chức danh "TS. Nguyễn".
    if let Some(re) = RE_EARLY_EXCEPTIONS.as_ref() {
        let temp_exc = current_text.clone();
        current_text = re.replace_all(&temp_exc, |caps: &Captures| {
            let orig = caps.get(0).unwrap().as_str();
            let val = COMBINED_EXCEPTIONS.get(orig).cloned().unwrap_or(orig.to_string());
            protect(val, &mut mask_map)
        }).to_string();
    }

    // T2..T7/CN có từ dẫn thời gian -> thứ trong tuần (trước khi các pass
    // chữ-số kịp đọc "T2" thành "tê hai").
    current_text = expand_weekday_abbr(&current_text);

    // "text2text"/"sale4u": số 2/4 kẹp giữa chữ thường đọc "two"/"four" (mọi ngữ cảnh).
    current_text = crate::vi_normalizer::num2en::expand_sandwich_digits(&current_text);

    // Câu thuần Anh: đổi số/giờ/%/đơn vị/ký hiệu thành chữ Anh TRƯỚC các pass
    // tiếng Việt — hết chữ số nên các pass sau tự thành no-op.
    if en_ctx {
        current_text = crate::vi_normalizer::num2en::english_prenormalize(&current_text);
    }

    // Đơn vị mũ Unicode -> ASCII trước mọi pass toán/đơn vị.
    current_text = RE_SUP_UNIT.replace_all(&current_text, "${1}2").into_owned();
    current_text = RE_SUP_UNIT3.replace_all(&current_text, "${1}3").into_owned();

    // Số mũ mang dấu ("10⁻³" -> "mười mũ trừ ba") — trước cụm mũ không dấu.
    current_text = RE_SUPERSCRIPT_SIGNED.replace_all(&current_text, |caps: &Captures| {
        let sign = if caps.get(1).unwrap().as_str() == "⁻" { "trừ " } else { "" };
        let digits = sup_digits(caps.get(2).unwrap().as_str());
        format!(" mũ {}{} ", sign, crate::vi_normalizer::num2vi::n2w(&digits))
    }).into_owned();

    // Cụm mũ Unicode liên tiếp ("10²³" -> "mười mũ hai mươi ba") — phải chạy
    // trước khi expand_symbols đọc rời từng ký tự "bình phương lập phương".
    current_text = RE_SUPERSCRIPT_RUN.replace_all(&current_text, |caps: &Captures| {
        let digits = sup_digits(caps.get(0).unwrap().as_str());
        format!(" mũ {} ", crate::vi_normalizer::num2vi::n2w(&digits))
    }).into_owned();

    // Công thức trần trong câu Việt: xử lý TRƯỚC các pass tách từ/nhân/khoảng
    // để cụm biến ("mc²", "4ac"), giai thừa, trừ nhị phân đọc như trong <math>.
    if !en_ctx {
        current_text = expand_inline_math(&current_text);
        current_text = RE_FACTORIAL_INLINE.replace_all(&current_text, " giai thừa ").into_owned();
        current_text = RE_NEG_FUNC.replace_all(&current_text, " âm ").into_owned();
        // "x - a", "u - v": trừ giữa hai BIẾN chữ thường đơn lẻ (dấu gạch
        // giữa hai từ thật không dính vì hai vế phải là chữ cái đơn).
        current_text = RE_MINUS_SINGLE_VARS.replace_all(&current_text, "$1 trừ $2").into_owned();
    }

    current_text = split_concatenated_terms(&current_text);

    // Biển số xe / mã định danh: phải trước pass nhân ("59X1" có chữ X) và
    // trước normalize_time ("51H" khớp mẫu giờ). Câu thuần Anh giữ nguyên
    // cho nhánh Anh xử lý.
    if !en_ctx {
        current_text = crate::vi_normalizer::misc::expand_codes_and_plates(&current_text);
        // Hệ số trước công thức hóa học: "6CO2" -> "6 CO2", "2HCl" -> "2 HCl"
        // để phần chữ đi theo nhánh acronym như khi đứng một mình.
        current_text = RE_CHEM_DIGIT_PREFIX.replace_all(&current_text, "$1 ").into_owned();
    }

    // Core normalization passes
    current_text = expand_power_of_ten(&current_text);
    current_text = RE_MULTIPLY.replace_all(&current_text, |caps: &FCaps| {
        expand_multiply_number(caps.get(0).unwrap().as_str())
    }).to_string();

    current_text = RE_CONTEXT_TRU.replace_all(&current_text, " $1 $2 trừ $3 ").into_owned();
    current_text = RE_CONTEXT_TRU_POST.replace_all(&current_text, " $1 trừ $2 $3 ").into_owned();
    current_text = RE_CONTEXT_DEN.replace_all(&current_text, " $1 $2 đến $3 ").into_owned();

    current_text = RE_EQ_MINUS.replace_all(&current_text, |caps: &Captures| {
        format!("{} trừ {} =", caps.get(1).unwrap().as_str(), caps.get(2).unwrap().as_str())
    }).into_owned();

    current_text = RE_EQ_NEG.replace_all(&current_text, |caps: &Captures| {
        format!("= âm {}", caps.get(1).unwrap().as_str())
    }).into_owned();

    current_text = crate::vi_normalizer::misc::expand_abbreviations(&current_text);
    current_text = RE_ADDR_ABBR.replace_all(&current_text, |caps: &FCaps| {
        match caps.get(1).unwrap().as_str() {
            "P" => " phường ",
            "Q" => " quận ",
            _ => " đường ",
        }.to_string()
    }).into_owned();
    current_text = expand_money_slang(&current_text);
    current_text = expand_scientific_notation(&current_text);

    current_text = RE_RANGE_PCT.replace_all(&current_text, "$1 đến $2%").into_owned();
    current_text = RE_PCT_NEG.replace_all(&current_text, " âm $1% ").into_owned();

    current_text = expand_scores(&current_text);

    current_text = normalize_date(&current_text);
    current_text = normalize_time(&current_text);

    current_text = RE_NUMERIC_DASH_GROUPS.replace_all(&current_text, |caps: &Captures| {
        let matched = caps.get(0).unwrap().as_str();
        let parts: Vec<&str> = matched.split(&['-', '\u{2013}', '\u{2014}'][..]).collect();
        parts.iter()
            .map(|&p| crate::vi_normalizer::num2vi::n2w_single(p))
            .collect::<Vec<String>>()
            .join(", ")
    }).into_owned();

    current_text = RE_LANDLINE.replace_all(&current_text, |caps: &FCaps| {
        let matched = caps.get(0).unwrap().as_str();
        let groups: Vec<String> = matched
            .split(|c: char| !c.is_ascii_digit())
            .filter(|s: &&str| !s.is_empty())
            .map(|s: &str| crate::vi_normalizer::num2vi::n2w_single(s))
            .collect();
        let prefix = if matched.contains('+') { "cộng " } else { "" };
        format!(" {}{} ", prefix, groups.join(", "))
    }).into_owned();

    current_text = RE_PHONE_SPACE.replace_all(&current_text, |caps: &Captures| {
        let matched = caps.get(0).unwrap().as_str();
        let parts: Vec<&str> = matched.split_whitespace().collect();
        parts.iter()
            .map(|&p| crate::vi_normalizer::num2vi::n2w_single(p))
            .collect::<Vec<String>>()
            .join(", ")
    }).into_owned();

    current_text = RE_PHONE_WITH_DASH.replace_all(&current_text, |caps: &Captures| {
        let p1 = caps.get(1).unwrap().as_str();
        let p2 = caps.get(2).unwrap().as_str();
        let p3 = caps.get(3).unwrap().as_str();
        format!(" {}, {}, {} ",
            crate::vi_normalizer::num2vi::n2w_single(p1),
            crate::vi_normalizer::num2vi::n2w_single(p2),
            crate::vi_normalizer::num2vi::n2w_single(p3)
        )
    }).into_owned();

    current_text = RE_HOTLINE.replace_all(&current_text, |caps: &Captures| {
        format!(" {} {} ",
            crate::vi_normalizer::num2vi::n2w_single(caps.get(1).unwrap().as_str()),
            crate::vi_normalizer::num2vi::n2w_single(caps.get(2).unwrap().as_str())
        )
    }).into_owned();

    current_text = RE_POWER_OF_TEN_IMPLICIT.replace_all(&current_text, |caps: &Captures| {
        let exp = caps.get(1).unwrap().as_str();
        if exp.starts_with('-') {
            format!("mười mũ trừ {}", crate::vi_normalizer::num2vi::n2w(&exp[1..]))
        } else {
            format!("mười mũ {}", crate::vi_normalizer::num2vi::n2w(&exp.replace('+', "")))
        }
    }).into_owned();

    current_text = RE_RANGE.replace_all(&current_text, |caps: &FCaps| {
        let n1_raw = caps.get(1).unwrap().as_str();
        let s1 = caps.get(2).unwrap().as_str();
        let s2 = caps.get(3).unwrap().as_str();
        let n2_raw = caps.get(4).unwrap().as_str();
        if !s1.is_empty() && s2.is_empty() {
            return caps.get(0).unwrap().as_str().to_string();
        }
        let n1 = n1_raw.replace(',', "").replace('.', "");
        let n2 = n2_raw.replace(',', "").replace('.', "");
        if (n1.len() as i32 - n2.len() as i32).abs() <= 1 {
            format!(" {} đến {} ", n1_raw, n2_raw)
        } else {
            format!(" {} {} ", n1_raw, n2_raw)
        }
    }).to_string();

    current_text = RE_NEG_VAR1.replace_all(&current_text, " âm $1 ").into_owned();
    current_text = RE_NEG_VAR2.replace_all(&current_text, " âm $1 ").into_owned();
    current_text = RE_MATH_MINUS_SUP.replace_all(&current_text, " trừ ").into_owned();
    current_text = RE_MATH_MINUS_COEF.replace_all(&current_text, " trừ ").into_owned();
    current_text = RE_MATH_MINUS_COEFL.replace_all(&current_text, " trừ ").into_owned();
    current_text = RE_DASH_TO_COMMA.replace_all(&current_text, ",").into_owned();
    current_text = RE_TO_SANG.replace_all(&current_text, " sang ").into_owned();

    current_text = expand_scientific_notation(&current_text);
    current_text = expand_height_weight(&current_text);
    current_text = crate::vi_normalizer::misc::expand_size_labels(&current_text);
    current_text = expand_compound_units(&current_text);
    current_text = expand_units_and_currency(&current_text);
    current_text = RE_LONG_NUM.replace_all(&current_text, |caps: &FCaps| {
        let neg = caps.get(1).unwrap().as_str();
        let num_str = caps.get(2).unwrap().as_str();
        let neg_prefix = if !neg.is_empty() { "âm " } else { "" };
        format!(" {}{} ", neg_prefix, crate::vi_normalizer::num2vi::n2w_single(num_str))
    }).to_string();

    current_text = fix_english_style_numbers(&current_text);

    current_text = RE_MULTI_COMMA.replace_all(&current_text, |caps: &Captures| {
        caps.get(1).unwrap().as_str().split(',').map(|s: &str| crate::vi_normalizer::num2vi::n2w_decimal(s)).collect::<Vec<String>>().join(" phẩy ")
    }).into_owned();

    current_text = RE_FLOAT_WITH_COMMA.replace_all(&current_text, |caps: &FCaps| {
        let int_part = crate::vi_normalizer::num2vi::n2w(&caps.get(1).unwrap().as_str().replace('.', ""));
        let dec_part = caps.get(2).unwrap().as_str().trim_end_matches('0');
        let mut res = if dec_part.is_empty() { int_part } else { format!("{} phẩy {}", int_part, crate::vi_normalizer::num2vi::n2w_decimal(dec_part)) };
        if caps.get(3).is_some() { res.push_str(" phần trăm"); }
        format!(" {} ", res)
    }).to_string();

    current_text = RE_STRIP_DOT_SEP.replace_all(&current_text, |caps: &FCaps| {
        caps.get(0).unwrap().as_str().replace('.', "")
    }).to_string();

    current_text = normalize_others(&current_text, en_ctx);
    current_text = normalize_number_vi(&current_text);

    // Văn bản đã viết sẵn "âm" mà số vẫn mang dấu trừ ("nhiệt độ âm -5 độ")
    // -> các pass số chèn thêm "âm" nữa thành "âm âm năm". Gộp lại một, nhưng
    // chỉ khi từ sau là CHỮ SỐ: "giá trị âm âm là dương" phải giữ nguyên.
    current_text = RE_DOUBLE_AM.replace_all(&current_text, |caps: &Captures| {
        let next = caps.get(1).unwrap().as_str();
        if crate::vi_normalizer::datestime::NUM_WORDS.contains(next.to_lowercase().as_str()) {
            format!("âm {}", next)
        } else {
            caps.get(0).unwrap().as_str().to_string()
        }
    }).into_owned();

    let temp_text3 = current_text.clone();
    current_text = RE_INTERNAL_EN_TAG.replace_all(&temp_text3, |caps: &Captures| {
        protect(caps.get(0).unwrap().as_str().to_string(), &mut mask_map)
    }).into_owned();

    // Câu thuần Anh: KHÔNG đọc chữ cái đơn theo tên chữ Việt ("plan B" giữ "b"
    // để G2P đọc "bi" theo ngữ cảnh Anh).
    if !en_ctx {
        current_text = expand_standalone_letters(&current_text);
    }

    if current_text.contains('.') {
        current_text = RE_DOT_BETWEEN_DIGITS.replace_all(&current_text, |caps: &Captures| {
            format!("{} chấm {}", caps.get(1).unwrap().as_str(), caps.get(2).unwrap().as_str())
        }).into_owned();
    }

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
        // Nạp dict phoneme (nếu có) để tra từ khi đọc path/URL/email kiểu Việt;
        // nạp hỏng thì bỏ qua, normalizer vẫn chạy với whitelist fallback.
        if let Some(p) = dict_path {
            crate::vi_normalizer::technical::init_norm_dict(p);
        }
        Normalizer { lang: lang.to_string() }
    }

    #[pyo3(signature = (text, punc_norm=false))]
    pub fn normalize(&self, text: &str, punc_norm: bool) -> String {
        if text.is_empty() { return String::new(); }

        let nfc_text: String = sanitize_unicode(text).nfc().collect();
        // Quy ellipsis (… ‥ ․) về "." NGAY ĐẦU để nó theo cùng đường xử lý với
        // "...". Nếu để muộn, "…" bị các pass trước đó nuốt thành dấu phân tách.
        let mut current_text = RE_ELLIPSIS.replace_all(&nfc_text, ".").into_owned();

        // Vùng <math>...</math>: tách cụm biến thành chữ rời, bỏ tag, để pipeline
        // thường đọc tên chữ + ký hiệu. Làm trước khi tách <en>. Công thức luôn
        // đọc kiểu Việt (force_vi) dù không có chữ có dấu.
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
