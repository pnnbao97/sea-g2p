//! How the Thai pipeline treats punctuation.
//!
//! The rules deliberately mirror the Vietnamese ones: a library that emits two
//! punctuation conventions would force every downstream TTS to special-case
//! the language.
//!
//! - `, . ! ?` survive as prosody and attach to the phoneme group before them;
//! - `;` and `:` become a comma — they are a pause, and dropping them erases
//!   the boundary the writer put there;
//! - any ellipsis becomes a single period;
//! - quotes, brackets and dashes are dropped;
//! - `/` is a word (ทับ), not punctuation.

use sea_g2p_rs::core::dict::PhonemeDict;
use sea_g2p_rs::lang::th::normalizer::normalize;
use sea_g2p_rs::lang::th::Thai;

fn thai() -> (PhonemeDict, Thai) {
    let path = format!("{}/python/sea_g2p/sea_g2p.bin", env!("CARGO_MANIFEST_DIR"));
    let dict = PhonemeDict::new(&path).expect("shipped dictionary");
    let th = Thai::new(&dict);
    (dict, th)
}

#[test]
fn terminators_survive_and_attach() {
    let (dict, th) = thai();
    for (input, mark) in [("เขาชอบไหม?", '?'), ("ดีมาก!", '!'), ("จบแล้ว.", '.')] {
        let out = th.phonemize(input, &dict);
        assert!(out.ends_with(mark), "{input}: {out}");
        // attached, not a whitespace-separated token of its own
        assert!(!out.ends_with(&format!(" {mark}")), "{input}: {out}");
    }
}

#[test]
fn semicolon_and_colon_become_a_comma() {
    assert_eq!(normalize("หนึ่ง; สอง: สาม"), "หนึ่ง, สอง, สาม");
}

#[test]
fn ellipsis_becomes_one_period() {
    assert_eq!(normalize("ดี…"), "ดี.");
    assert_eq!(normalize("ดี..."), "ดี.");
}

#[test]
fn quotes_brackets_and_dashes_are_dropped() {
    assert_eq!(normalize("เขาพูดว่า \"สวัสดี\""), "เขาพูดว่า สวัสดี");
    assert_eq!(normalize("เขา (คนนั้น) มา"), "เขา คนนั้น มา");
    assert_eq!(normalize("ก-ข"), "ก ข");
}

#[test]
fn slash_is_a_word() {
    assert!(normalize("ครึ่ง/หนึ่ง").contains("ทับ"));
}
