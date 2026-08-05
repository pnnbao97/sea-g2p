//! Integration tests for the rule-based Thai G2P (the OOV fallback).

use sea_g2p_rs::lang::th::rules::g2p_word;
fn read(w: &str) -> String {
    g2p_word(w).join(" ")
}

#[test]
fn ordinary_words() {
    assert_eq!(read("สวัสดี"), "sa˨˩ wat̚˨˩ diː˧");   // อักษรนำ: ส leads วัส
    assert_eq!(read("ครับ"), "kʰrap̚˦˥");             // true cluster
    assert_eq!(read("เขา"), "kʰaw˩˩˦");               // เ-า diphthong
    assert_eq!(read("ผู้คน"), "pʰuː˥˩ kʰon˧");        // no coda stealing
    assert_eq!(read("คน"), "kʰon˧");                  // inherent /o/
    assert_eq!(read("เดิน"), "dɤːn˧");
    assert_eq!(read("ไทย"), "tʰaj˧");                 // silent ย
}

#[test]
fn tone_mark_position() {
    // the mark is written before า, so it must not be read as a coda
    assert_eq!(read("ข้าว"), "kʰaːw˥˩");
    assert_eq!(read("แม่"), "mɛː˥˩");
}

#[test]
fn silent_letters_and_ro_han() {
    assert_eq!(read("หลอก"), "lɔːk̚˨˩");   // silent ห lends high class
    assert_eq!(read("ให้"), "haj˥˩");
    assert_eq!(read("ธรรม"), "tʰam˧");      // ro han
    assert_eq!(read("ละคร"), "la˦˥ kʰɔːn˧"); // lone ร final takes /ɔː/
}

#[test]
fn never_panics_on_junk() {
    for w in ["", "ๆ", "abc", "ก", "กกกกก", "๛", "เเ"] {
        let _ = g2p_word(w);
    }
}
