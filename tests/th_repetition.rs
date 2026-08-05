//! The mai yamok repetition mark `ๆ`, exercised through the full pipeline.
//!
//! `ๆ` repeats the **word** before it. Because Thai has no spaces, that word
//! is only identifiable after segmentation, so the rule lives in
//! `Thai::phonemize` rather than in the normalizer. These tests pin the
//! distinction: a text-level rule would repeat the whole preceding Thai run
//! and turn คนต่างๆ into "คน ต่าง คน ต่าง".

use sea_g2p_rs::core::dict::PhonemeDict;
use sea_g2p_rs::lang::th::Thai;

fn thai() -> (PhonemeDict, Thai) {
    let path = format!("{}/python/sea_g2p/sea_g2p.bin", env!("CARGO_MANIFEST_DIR"));
    let dict = PhonemeDict::new(&path).expect("shipped dictionary");
    let th = Thai::new(&dict);
    (dict, th)
}

/// Phonemes as a vector of per-syllable strings, whitespace collapsed.
fn read(th: &Thai, dict: &PhonemeDict, s: &str) -> Vec<String> {
    th.phonemize(s, dict)
        .split_whitespace()
        .map(|x| x.to_string())
        .collect()
}

#[test]
fn single_word_is_repeated() {
    let (dict, th) = thai();
    let plain = read(&th, &dict, "ต่าง");
    let repeated = read(&th, &dict, "ต่างๆ");
    assert_eq!(repeated, [plain.clone(), plain].concat());
}

#[test]
fn only_the_last_word_is_repeated() {
    let (dict, th) = thai();
    // คนต่างๆ is คน + ต่าง + ต่าง, NOT คน + ต่าง + คน + ต่าง
    let expected = [
        read(&th, &dict, "คน"),
        read(&th, &dict, "ต่าง"),
        read(&th, &dict, "ต่าง"),
    ]
    .concat();
    assert_eq!(read(&th, &dict, "คนต่างๆ"), expected);
}

#[test]
fn et_cetera_expands_then_repeats_once() {
    let (dict, th) = thai();
    // ฯลฯ -> และอื่นๆ -> และ + อื่น + อื่น
    let expected = [
        read(&th, &dict, "และ"),
        read(&th, &dict, "อื่น"),
        read(&th, &dict, "อื่น"),
    ]
    .concat();
    assert_eq!(read(&th, &dict, "ฯลฯ"), expected);
}

#[test]
fn leading_mark_has_nothing_to_repeat() {
    let (dict, th) = thai();
    // must not panic or duplicate anything that is not there
    let _ = th.phonemize("ๆ", &dict);
}

#[test]
fn compound_repeats_only_its_final_word() {
    let (dict, th) = thai();
    // เด็กเล็ก is one dictionary entry, but เด็กเล็กๆ is "dèk lék lék"
    let expected = [
        read(&th, &dict, "เด็ก"),
        read(&th, &dict, "เล็ก"),
        read(&th, &dict, "เล็ก"),
    ]
    .concat();
    assert_eq!(read(&th, &dict, "เด็กเล็กๆ"), expected);
}

#[test]
fn short_word_is_not_split_into_letters() {
    let (dict, th) = thai();
    // ต่าง must not be cut into ต่า + ง and repeat the letter name "ngo"
    let plain = read(&th, &dict, "ต่าง");
    assert_eq!(read(&th, &dict, "ต่างๆ"), [plain.clone(), plain].concat());
}
