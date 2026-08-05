//! Thai text that switches into Latin script mid-sentence.
//!
//! Thai writing code-switches with English constantly — brand names, tech
//! terms, game titles, whole English clauses — so a Thai pipeline that only
//! reads Thai leaves raw letters in its phoneme output. These tests pin the
//! composition: Thai tokens come from the Thai dictionary, Latin runs from
//! the same engine that serves English elsewhere, and the result is one
//! phoneme string with no script left unread.

use sea_g2p_rs::core::dict::PhonemeDict;
use sea_g2p_rs::g2p::G2PEngine;
use sea_g2p_rs::lang::th::Thai;

fn engine() -> G2PEngine {
    let path = format!("{}/python/sea_g2p/sea_g2p.bin", env!("CARGO_MANIFEST_DIR"));
    G2PEngine::new(&path).expect("shipped dictionary")
}

fn read(eng: &G2PEngine, th: &Thai, s: &str) -> String {
    th.phonemize_with(s, &eng.dict, |latin| eng.phonemize(latin))
}

fn is_thai(c: char) -> bool {
    ('\u{0E01}'..='\u{0E4E}').contains(&c)
}

/// Nothing may survive unread.
///
/// Thai characters in the output always mean a token was passed through. Bare
/// Latin letters cannot be checked the same way — IPA is itself written with
/// Latin letters — so instead every Latin word of the INPUT must be absent
/// from the output.
fn assert_fully_read(out: &str, input: &str) {
    let thai_left: Vec<char> = out.chars().filter(|c| is_thai(*c)).collect();
    assert!(thai_left.is_empty(), "unread Thai in {input:?}: {out:?}");

    for word in input.split_whitespace() {
        if word.chars().any(is_thai) {
            continue;
        }
        let w: String = word.chars().filter(|c| c.is_ascii_alphabetic()).collect();
        if w.len() >= 3 {
            assert!(
                !out.contains(&w),
                "{w:?} was passed through unread in {input:?}: {out:?}"
            );
        }
    }
}

#[test]
fn brand_names_are_read_as_english() {
    let eng = engine();
    let th = Thai::new(&eng.dict);
    let out = read(&eng, &th, "ผมใช้ iPhone และ Facebook");
    // the Thai around them still reads as Thai
    assert!(out.contains("pʰom"), "{out}");
    // and the Latin is phonemised, not passed through
    assert!(!out.contains("iPhone") && !out.contains("Facebook"), "{out}");
    assert_fully_read(&out, "ผมใช้ iPhone และ Facebook");
}

#[test]
fn mixed_script_leaves_nothing_unread() {
    let eng = engine();
    let th = Thai::new(&eng.dict);
    for s in [
        "เด็กเล่นเกม RoV บนมือถือ",
        "โควิด-19 ระบาดในปี 2020",
        "ซื้อของที่ Central และ Lotus",
    ] {
        assert_fully_read(&read(&eng, &th, s), s);
    }
}

#[test]
fn pure_thai_is_unaffected_by_the_latin_path() {
    let eng = engine();
    let th = Thai::new(&eng.dict);
    let with_cb = read(&eng, &th, "เขาฉลาดพอที่จะซ่อนสติปัญญา");
    let without = th.phonemize("เขาฉลาดพอที่จะซ่อนสติปัญญา", &eng.dict);
    assert_eq!(with_cb, without);
}

#[test]
fn digits_survive_the_thai_number_pass_not_the_latin_one() {
    let eng = engine();
    let th = Thai::new(&eng.dict);
    // 19 must be read in Thai (สิบเก้า), not handed to the English engine
    let out = read(&eng, &th, "โควิด-19");
    assert!(out.contains("kaːw"), "{out}");
}
