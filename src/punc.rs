//! Trailing punctuation normalization ("punc_norm").
//!
//! The goal is prosodic: give the TTS model a clean, predictable sentence
//! ending so it neither clips a phrase short nor runs two phrases together.
//! When enabled (`punc_norm = true`):
//!
//!   - A **very short** fragment — under three words — that sits at the start
//!     or in the middle of the string and ends in `.` has that `.` turned into
//!     `,`. This stops list markers from being read as complete sentences
//!     ("3." -> "ba." -> "ba,"). The final fragment always keeps a real
//!     sentence ending.
//!   - A **short** sentence — under five words — at the end of the string is
//!     forced to end in exactly one `.`, replacing whatever trailing mark it
//!     had (`,` `!` `?` `…`).
//!   - Anything longer only gains a `.` when it does not already end in one of
//!     `,` `.` `!` `?`.
//!
//! These are pure string operations with no language dependency, shared by both
//! `Normalizer` and `G2P`.

use once_cell::sync::Lazy;
use regex::Regex;

/// Word count at or below which a sentence counts as "short" (under five).
const SHORT_SENTENCE_MAX_WORDS: usize = 4;

/// Word count at or below which a fragment counts as "very short" (under three).
const SUPER_SHORT_MAX_WORDS: usize = 2;

/// A `.` that actually ends a sentence: followed by whitespace (newlines
/// included) or by the end of the string.
///
/// Requiring whitespace or EOS deliberately excludes the dots inside
/// abbreviations ("U.S.A.") and numbers ("3.5", already split by the
/// normalizer). Those are not sentence boundaries and must not be treated as
/// break points.
static RE_SENTENCE_DOT: Lazy<Regex> = Lazy::new(|| Regex::new(r"\.(\s+|$)").unwrap());

/// Trailing marks that may be replaced when a short sentence is forced to `.`.
/// Includes the single-character ellipses `…` (U+2026), `‥` (U+2025) and
/// `․` (U+2024).
fn is_trailing_punct(c: char) -> bool {
    matches!(
        c,
        ',' | '.' | '!' | '?' | ';' | ':' | '\u{2026}' | '\u{2025}' | '\u{2024}'
    )
}

/// Dấu kết thúc câu được chấp nhận cho câu dài (không cần thêm `.`).
fn is_sentence_end(c: char) -> bool {
    matches!(c, ',' | '.' | '!' | '?')
}

/// Đếm số "từ" thực — chỉ tính token có ít nhất một ký tự chữ/số, để các token
/// dấu câu đứng riêng (vd "Xin chào !") không bị tính nhầm thành một từ.
fn word_count(text: &str) -> usize {
    text.split_whitespace()
        .filter(|w| w.chars().any(|c| c.is_alphanumeric()))
        .count()
}

/// Đổi dấu `.` kết câu của các câu SIÊU NGẮN (< 3 từ) KHÔNG PHẢI câu cuối thành
/// `,`. "Câu" ở đây là đoạn văn bản giữa hai ranh giới kết câu (dấu `.` có khoảng
/// trắng/EOS theo sau). Dấu `.` cuối chuỗi (không còn nội dung thật phía sau) luôn
/// được giữ nguyên — đó là dấu kết thật của câu cuối.
fn soften_short_segments(text: &str) -> String {
    let dots: Vec<regex::Match> = RE_SENTENCE_DOT.find_iter(text).collect();
    if dots.is_empty() {
        return text.to_string();
    }

    let mut result = String::with_capacity(text.len());
    let mut last = 0usize; // byte-index đầu câu hiện tại
    for m in &dots {
        let dot_pos = m.start();
        let segment = &text[last..dot_pos]; // nội dung câu trước dấu '.'
        let ws = &text[dot_pos + 1..m.end()]; // khoảng trắng theo sau '.' (giữ nguyên)
        // Còn nội dung thật (chữ/số) phía sau dấu này? Nếu không -> đây là dấu kết
        // thật của câu cuối, không được đổi thành ','.
        let has_more = text[m.end()..].chars().any(|c: char| c.is_alphanumeric());

        result.push_str(segment);
        let wc = word_count(segment);
        if has_more && (1..=SUPER_SHORT_MAX_WORDS).contains(&wc) {
            result.push(','); // câu siêu ngắn giữa chuỗi -> làm mềm dấu kết cụt ngủn
        } else {
            result.push('.');
        }
        result.push_str(ws);
        last = m.end();
    }
    result.push_str(&text[last..]);
    result
}

/// Áp dụng chuẩn hóa dấu câu cuối lên `text`.
pub fn apply_punc_norm(text: &str) -> String {
    let trimmed = text.trim_end();
    if trimmed.is_empty() {
        return trimmed.to_string();
    }

    // Bước 1: làm mềm dấu '.' cụt ngủn của các câu siêu ngắn ở đầu/giữa chuỗi.
    let softened = soften_short_segments(trimmed);
    let trimmed = softened.trim_end();

    // Bước 2: chuẩn hóa dấu kết của CẢ CHUỖI (câu cuối) như cũ.
    if word_count(trimmed) <= SHORT_SENTENCE_MAX_WORDS {
        // Câu siêu ngắn: ép dấu cuối về đúng một `.` bất kể đang là dấu gì.
        let stripped = trimmed
            .trim_end_matches(|c: char| is_trailing_punct(c) || c.is_whitespace());
        if stripped.is_empty() {
            // Toàn dấu câu -> trả về một dấu `.`.
            return ".".to_string();
        }
        format!("{}.", stripped)
    } else {
        // Câu dài: chỉ thêm `.` nếu chưa kết thúc bằng , . ! ?
        let last_char = trimmed.chars().next_back().unwrap();
        if is_sentence_end(last_char) {
            trimmed.to_string()
        } else {
            format!("{}.", trimmed)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::apply_punc_norm;

    #[test]
    fn long_sentence_gets_dot_when_missing() {
        assert_eq!(
            apply_punc_norm("tôi đi học mỗi ngày vào buổi sáng"),
            "tôi đi học mỗi ngày vào buổi sáng."
        );
    }

    #[test]
    fn long_sentence_keeps_existing_terminator() {
        assert_eq!(
            apply_punc_norm("hôm nay trời đẹp quá phải không?"),
            "hôm nay trời đẹp quá phải không?"
        );
        assert_eq!(
            apply_punc_norm("anh ấy chạy rất nhanh trên đường!"),
            "anh ấy chạy rất nhanh trên đường!"
        );
    }

    #[test]
    fn short_sentence_forced_to_dot() {
        assert_eq!(apply_punc_norm("xin chào"), "xin chào.");
        assert_eq!(apply_punc_norm("xin chào!"), "xin chào.");
        assert_eq!(apply_punc_norm("xin chào?"), "xin chào.");
        assert_eq!(apply_punc_norm("ừ…"), "ừ.");
        assert_eq!(apply_punc_norm("xin chào !"), "xin chào.");
    }

    #[test]
    fn idempotent() {
        assert_eq!(apply_punc_norm("xin chào."), "xin chào.");
    }

    #[test]
    fn empty_stays_empty() {
        assert_eq!(apply_punc_norm("   "), "");
    }

    #[test]
    fn leading_short_segment_dot_becomes_comma() {
        // List-marker "3." -> "ba." đầu chuỗi: dấu '.' cụt ngủn -> ','.
        assert_eq!(
            apply_punc_norm("ba. công ty cổ phần green travel việt nam là doanh nghiệp lớn."),
            "ba, công ty cổ phần green travel việt nam là doanh nghiệp lớn."
        );
    }

    #[test]
    fn middle_short_segment_dot_becomes_comma() {
        assert_eq!(
            apply_punc_norm("vâng. tôi sẽ đến ngay bây giờ và gặp anh."),
            "vâng, tôi sẽ đến ngay bây giờ và gặp anh."
        );
    }

    #[test]
    fn final_short_sentence_keeps_dot() {
        // Câu siêu ngắn ở CUỐI chuỗi giữ nguyên dấu kết (không có nội dung phía sau).
        assert_eq!(
            apply_punc_norm("tôi đã làm xong hết mọi việc rồi. vâng."),
            "tôi đã làm xong hết mọi việc rồi. vâng."
        );
    }

    #[test]
    fn long_middle_segment_keeps_dot() {
        // Câu ≥3 từ ở giữa chuỗi KHÔNG bị làm mềm.
        assert_eq!(
            apply_punc_norm("hôm nay trời rất đẹp. chúng tôi cùng nhau đi dạo ngoài phố."),
            "hôm nay trời rất đẹp. chúng tôi cùng nhau đi dạo ngoài phố."
        );
    }

    #[test]
    fn abbreviation_dots_untouched() {
        // Dấu chấm dính liền (không có khoảng trắng theo sau) không phải ranh giới câu.
        assert_eq!(
            apply_punc_norm("U.S.A là một quốc gia rộng lớn nằm ở bắc mỹ."),
            "U.S.A là một quốc gia rộng lớn nằm ở bắc mỹ."
        );
    }

    #[test]
    fn multiple_short_segments_all_softened() {
        assert_eq!(
            apply_punc_norm("một. hai. ba là những con số đầu tiên trong dãy đếm."),
            "một, hai, ba là những con số đầu tiên trong dãy đếm."
        );
    }
}
