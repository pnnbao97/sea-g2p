//! Integration tests for the Indonesian normalizer.

use sea_g2p_rs::lang::id::normalizer::{audit_unmapped, normalize};

#[test]
fn period_groups_thousands_and_comma_is_the_decimal_mark() {
    // Indonesian writes 1.250,75 where English writes 1,250.75. Reading it
    // with the English convention turns a thousand into "one point two".
    assert!(normalize("1.250").contains("seribu dua ratus lima puluh"));
    let d = normalize("3,14");
    assert!(d.contains("koma"), "{d}");
    assert!(d.starts_with("tiga"), "{d}");
}

#[test]
fn rupiah_reads_the_amount_then_the_unit() {
    let m = normalize("Rp1.250.000");
    assert!(m.contains("rupiah"), "{m}");
    assert!(m.contains("juta"), "{m}");
}

#[test]
fn chat_contractions_expand() {
    // yg and dgn look like pronounceable words to a rule engine
    assert_eq!(normalize("yg penting tdk lupa dgn tugasnya"),
               "yang penting tidak lupa dengan tugasnya");
}

#[test]
fn initialisms_are_spelled_with_indonesian_letter_names() {
    assert_eq!(normalize("DPR"), "de pe er");
    assert_eq!(normalize("NKRI"), "en ka er i");
}

#[test]
fn a_cue_word_is_not_repeated() {
    // "pukul 14:30" was coming out as "pukul pukul empat belas"
    assert_eq!(normalize("pukul 14:30"), "pukul empat belas lewat tiga puluh menit");
    assert!(normalize("jam 14:30").starts_with("jam"));
    // with no cue at all, one is supplied
    assert!(normalize("14:30").starts_with("pukul"));
}

#[test]
fn dates_read_day_month_year() {
    let d = normalize("17/8/1945");
    assert!(d.contains("Agustus"), "{d}");
    assert!(d.contains("tujuh belas"), "{d}");
    assert!(d.contains("seribu sembilan ratus empat puluh lima"), "{d}");
}

#[test]
fn nothing_is_deleted_in_silence() {
    let inventory = "&+=<>±≈≠×÷/%°@©→~$€£¥";
    assert_eq!(audit_unmapped(inventory), Vec::<char>::new());
    assert_eq!(audit_unmapped("∮"), vec!['∮']);
}

#[test]
fn plain_text_is_untouched() {
    assert_eq!(normalize("saya makan nasi goreng"), "saya makan nasi goreng");
}
