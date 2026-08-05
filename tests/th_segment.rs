//! Integration tests for Thai spelling normalization and word segmentation.

use sea_g2p_rs::lang::th::segment::{normalize_spelling, segment, WordTrie};
#[test]
fn spelling_normalization() {
    assert_eq!(normalize_spelling("ทําไม"), "ทำไม");
    assert_eq!(normalize_spelling("นํ้า"), "น้ำ");
    assert_eq!(normalize_spelling("เเล้ว"), "แล้ว");
    assert_eq!(normalize_spelling("นำ้"), "น้ำ");
    assert_eq!(normalize_spelling("ท่ี"), "ที่");
    // valid text is left alone
    assert_eq!(normalize_spelling("ที่น้ำกำลังไหล"), "ที่น้ำกำลังไหล");
}

#[test]
fn segments_with_dictionary() {
    let trie = WordTrie::from_words([
        "เขา", "ฉลาด", "และ", "พอ", "ที่", "จะ", "ซ่อน", "สติปัญญา", "คน",
    ]);
    let toks = segment("เขาฉลาดและซ่อนสติปัญญา", &trie);
    let words: Vec<&str> = toks.iter().map(|t| t.text.as_str()).collect();
    assert_eq!(words, ["เขา", "ฉลาด", "และ", "ซ่อน", "สติปัญญา"]);
    assert!(toks.iter().all(|t| t.known == Some(true)));
}

#[test]
fn non_thai_runs_pass_through() {
    let trie = WordTrie::from_words(["เกม", "บน", "มือถือ"]);
    let toks = segment("เกม RoV บนมือถือ", &trie);
    let words: Vec<&str> = toks.iter().map(|t| t.text.as_str()).collect();
    assert_eq!(words, ["เกม", " RoV ", "บน", "มือถือ"]);
    assert_eq!(toks[1].known, None);
}

#[test]
fn frequency_breaks_the_tie_fewest_pieces_cannot() {
    // สากล|คน and สาก|ลคน both cost two words; only frequency separates
    // them, and สากล ("international") is far more common than สาก.
    let pairs: Vec<(&str, u64)> = vec![
        ("สากล", 5000), ("คน", 90000), ("สาก", 40), ("ลคน", 3), ("โอลิมปิก", 900),
    ];
    let trie = WordTrie::from_frequencies(pairs);
    let toks = segment("โอลิมปิกสากลคน", &trie);
    let words: Vec<&str> = toks.iter().map(|t| t.text.as_str()).collect();
    assert_eq!(words, ["โอลิมปิก", "สากล", "คน"]);
}

#[test]
fn unknown_runs_are_marked_and_merged() {
    let trie = WordTrie::from_words(["คน"]);
    let toks = segment("คนกกกก", &trie);
    assert_eq!(toks[0].text, "คน");
    assert_eq!(toks[1].known, Some(false));
    assert_eq!(toks.len(), 2);
}
