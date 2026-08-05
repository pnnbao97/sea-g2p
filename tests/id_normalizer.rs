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

#[test]
fn latin_unit_abbreviations_are_read_as_words() {
    assert_eq!(normalize("60 km/jam"), "enam puluh kilometer per jam");
    assert!(normalize("9,8 m/s²").contains("meter per detik kuadrat"));
    // Indonesian suffixes where Thai prefixes: meter persegi, not persegi meter
    assert_eq!(normalize("50 m2"), "lima puluh meter persegi");
    assert_eq!(normalize("2 m3"), "dua meter kubik");
    assert!(normalize("tipe 5 abc").contains("abc"));
}

#[test]
fn ordinals_are_one_word_and_leave_no_hyphen() {
    // "ke-3" reached the G2P stage as "ke- tiga", dash and all
    assert_eq!(normalize("ke-3"), "ketiga");
    assert_eq!(normalize("abad ke-20"), "abad kedua puluh");
    // a Roman ordinal loses its hyphen too
    assert_eq!(normalize("abad ke-XX"), "abad ke dua puluh");
}

#[test]
fn a_hyphen_survives_only_between_letters() {
    // reduplication needs it; COVID-19 does not, and used to keep it
    assert_eq!(normalize("COVID-19"), "COVID sembilan belas");
    assert_eq!(normalize("orang-orang"), "orang-orang");
}

#[test]
fn an_apostrophe_between_letters_closes_the_word_up() {
    // older orthography: the modern spelling simply joins. Dropping it to a
    // space, as the residual pass did, split one word into two.
    assert_eq!(normalize("do'a Jum'at"), "doa Jumat");
}

#[test]
fn prime_marks_and_ratios() {
    assert_eq!(normalize("5'6\""), "lima kaki enam inci");
    assert!(normalize("rasio 3:1").contains("tiga banding satu"));
    assert_eq!(audit_unmapped("5'"), vec!['\'']);
}

#[test]
fn e_notation_keeps_its_order_of_magnitude() {
    assert!(normalize("6,02e23").contains("kali sepuluh pangkat dua puluh tiga"));
    assert!(normalize("1e-9").contains("pangkat minus sembilan"));
}

#[test]
fn weekday_abbreviations_need_the_hari_cue() {
    assert_eq!(normalize("hari Sen"), "hari Senin");
    assert_eq!(normalize("hari Min"), "hari Minggu");
    // the full spelling is not re-expanded
    assert_eq!(normalize("hari Senin"), "hari Senin");
    // without the cue, Sen is a name or an ordinary word
    assert_eq!(normalize("Sen depan"), "Sen depan");
    // the abbreviation stage supplies the cue: hr -> hari
    assert_eq!(normalize("hr Sab"), "hari Sabtu");
}

#[test]
fn a_diacritic_folds_onto_its_letter_instead_of_eating_it() {
    // Indonesian is written in plain ASCII, so the residual stage dropped
    // anything else — taking the letter with the accent. ārati lost its
    // first sound and was then read by the English engine as "ɹˈæɾi".
    assert_eq!(normalize("ārati"), "arati");
    assert_eq!(normalize("voilà"), "voila");
    assert_eq!(normalize("khemaṁ"), "khemam");
    assert_eq!(normalize("café"), "cafe");
    // ordinary text is untouched
    assert_eq!(normalize("Saya makan nasi goreng"), "Saya makan nasi goreng");
}

#[test]
fn the_audit_reports_what_the_residual_stage_actually_deletes() {
    // `is_alphanumeric` accepts ā à ṁ and the Cyrillic г, but RE_DROP keeps
    // ASCII only — so the audit passed words the pipeline was deleting. It
    // must agree with the stage it is guarding.
    assert_eq!(audit_unmapped("ārati"), vec!['ā']);
    assert_eq!(audit_unmapped("voilà"), vec!['à']);
    // a Cyrillic homoglyph cannot fold to Latin, so it is still dropped —
    // but reported rather than silent
    assert_eq!(audit_unmapped("pasaгan"), vec!['г']);
    // and ordinary Indonesian stays quiet
    for s in ["Saya makan nasi goreng di warung", "Harga BBM naik 10%", "do'a Jum'at"] {
        assert_eq!(audit_unmapped(s), Vec::<char>::new(), "{s}");
    }
}
