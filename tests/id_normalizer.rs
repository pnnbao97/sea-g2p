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

#[test]
fn mathematical_notation_is_not_deleted_in_silence() {
    assert!(normalize("suhu -5 derajat").contains("minus"), "minus sign");
    // "10^6" read as "sepuluh enam" before this: six orders of magnitude
    let p = normalize("10^6 orang");
    assert!(p.contains("pangkat"), "{p}");
    assert!(normalize("10⁻³").contains("minus"), "negative exponent");
    assert!(normalize("5 m²").contains("kuadrat"), "squared");
    assert!(normalize("H₂O").contains("dua"), "subscript");
    assert!(normalize("usia 10-20 tahun").contains("sampai"), "range");
    assert!(normalize("3 x 4").contains("kali"), "multiplication");
    assert!(normalize("1/2").contains("per"), "fraction");
}

#[test]
fn the_audit_verifies_numeric_hyphens_rather_than_assuming() {
    assert_eq!(audit_unmapped("-5"), Vec::<char>::new());
    assert_eq!(audit_unmapped("10-20"), Vec::<char>::new());
    // reduplication hyphens are ordinary and stay unreported
    assert_eq!(audit_unmapped("orang-orang"), Vec::<char>::new());
}

#[test]
fn emails_and_urls_are_read_whole() {
    let u = normalize("lihat https://www.google.com");
    assert!(u.contains("titik"), "{u}");
    // The scheme is READ, not dropped — text that says https:// means it —
    // and spelled with Indonesian letter names, as Vietnamese spells it
    // "hát tê tê phê ét" rather than leaving it for the G2P stage.
    assert!(u.contains("ha te te pe es"), "{u}");
    assert!(u.contains("titik dua"), "{u}");
    assert!(u.matches("garis miring").count() >= 2, "{u}");
    assert!(!u.contains("//"), "{u}");
    let e = normalize("kirim ke admin@example.com");
    assert!(e.contains("at") && e.contains("titik"), "{e}");
    // a path is spoken with its separator
    assert!(normalize("www.abc.com/berita").contains("garis miring"));
}

#[test]
fn roman_numerals_need_a_cue() {
    assert!(normalize("Perang Dunia II").contains("dua"), "WWII");
    assert_eq!(normalize("cakram CD"), "cakram CD");
}

#[test]
fn identifiers_need_a_cue_too() {
    let p = normalize("plat B 1234 XYZ");
    assert!(p.contains("satu dua tiga empat"), "{p}");
    assert!(normalize("ada 1234 ekor").contains("seribu"), "no cue");
}

#[test]
fn markup_tags_are_stripped_not_read() {
    let m = normalize("<math>b² - 4ac</math>");
    assert!(!m.contains("math"), "{m}");
    assert!(m.contains("kuadrat") && m.contains("minus"), "{m}");
}

#[test]
fn reduplication_hyphens_survive_the_math_stage() {
    // a compound hyphen is not a minus sign: both sides must have spaces
    assert_eq!(normalize("orang-orang"), "orang-orang");
    assert_eq!(normalize("anak-anak"), "anak-anak");
}
