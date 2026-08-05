//! Integration tests for the shared trailing-punctuation rule.

use sea_g2p_rs::punc::apply_punc_norm;
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
    // A list marker "3." read as "ba." at the start of the string: the
    // abrupt period becomes a comma so the list keeps flowing.
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
    // A very short fragment at the END keeps its period: nothing follows it.
    assert_eq!(
        apply_punc_norm("tôi đã làm xong hết mọi việc rồi. vâng."),
        "tôi đã làm xong hết mọi việc rồi. vâng."
    );
}

#[test]
fn long_middle_segment_keeps_dot() {
    // Fragments of three words or more are left alone.
    assert_eq!(
        apply_punc_norm("hôm nay trời rất đẹp. chúng tôi cùng nhau đi dạo ngoài phố."),
        "hôm nay trời rất đẹp. chúng tôi cùng nhau đi dạo ngoài phố."
    );
}

#[test]
fn abbreviation_dots_untouched() {
    // A dot with no whitespace after it is not a sentence boundary.
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
