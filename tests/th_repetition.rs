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

#[test]
fn paiyannoi_is_silent_and_never_leaks() {
    let (dict, th) = thai();
    // ฯ marks an elided remainder and is not pronounced. The rule fallback
    // used to push the raw character into the phoneme string, so Bangkok
    // came out as "kruŋ˧ tʰeːp̚˥˩ ฯ" — a token no voice was trained on.
    for s in ["กรุงเทพฯ", "สหรัฐฯ", "โปรดเกล้าฯ", "ผมอยู่กรุงเทพฯ ครับ"] {
        let out = th.phonemize(s, &dict);
        assert!(!out.contains('ฯ'), "{s} -> {out}");
        assert!(!out.trim().is_empty(), "{s}");
    }
    assert!(th.phonemize("กรุงเทพฯ", &dict).contains("kruŋ"));
    // ฯลฯ is expanded by the normalizer and must still read
    assert!(th.phonemize("ฯลฯ", &dict).contains("ʔɯːn"));
}

#[test]
fn an_empty_dictionary_entry_is_not_a_pronunciation() {
    let (dict, th) = thai();
    // 2,767 keys in the shipped Thai section carry an empty pronunciation —
    // segmentation fragments of transliterated names. Taking the lookup at
    // its word deleted every one of them from the phoneme stream. The rules
    // can read them, so an empty entry must count as a miss.
    for s in ["มปิก", "ดยุค", "ทเทิล", "รปภ.", "ปชช."] {
        let out = th.phonemize(s, &dict);
        assert!(
            out.chars().any(|c| !c.is_whitespace() && !matches!(c, ',' | '.' | '!' | '?')),
            "{s} read as {out:?}"
        );
        assert!(!out.chars().any(|c| ('\u{0E01}'..='\u{0E5B}').contains(&c)), "{s} -> {out}");
    }
    // A word whose every consonant carries thanthakhat really is silent, and
    // must stay silent rather than fall back to the raw characters.
    for s in ["ร์", "น์"] {
        let out = th.phonemize(s, &dict);
        assert!(out.trim().is_empty(), "{s} -> {out:?}");
    }
    // ordinary words are unaffected
    assert!(th.phonemize("โอลิมปิก", &dict).contains("lim"));
}

#[test]
fn a_heteronym_is_chosen_by_its_neighbours() {
    let (dict, th) = thai();
    // เพลา is /pʰeː laː/ "time" and /pʰlaw/ "axle". One token either way, so
    // no segmenter can separate them — only the words around it can.
    assert!(th.phonemize("เพลาเช้า", &dict).starts_with("pʰeː˧ laː˧"), "morning");
    assert!(th.phonemize("เพลาบ่าย", &dict).starts_with("pʰeː˧ laː˧"), "afternoon");
    // with no cue, the dictionary's reading stands
    assert!(th.phonemize("เพลารถยนต์", &dict).starts_with("pʰlaw˧"), "axle");
    assert!(th.phonemize("เพลา", &dict).starts_with("pʰlaw˧"), "bare");

    // แหน: the ห is silent in the water fern, read in หวงแหน "to cherish"
    assert!(th.phonemize("จอกแหน", &dict).contains("nɛː˩˩˦"));
    assert!(th.phonemize("หวงแหน", &dict).contains("hɛːn˩˩˦"));
}

#[test]
fn two_dictionary_entries_that_disagreed_with_gold() {
    let (dict, th) = thai();
    // ปรัก was stored as prak̚˨˩; the only human transcription is pa˨˩rak̚˦˥,
    // and ปรักหักพัง carried the wrong tone on its second syllable.
    assert_eq!(th.phonemize("ปรัก", &dict), "pa˨˩ rak̚˦˥");
    assert!(th.phonemize("ปรักหักพัง", &dict).starts_with("pa˨˩ rak̚˦˥"));
}
