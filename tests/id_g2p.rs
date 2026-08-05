//! Integration tests for Indonesian pronunciation: the dictionary, the rules
//! that read what it does not have, and the split with English.

use sea_g2p_rs::core::dict::PhonemeDict;
use sea_g2p_rs::g2p::G2PEngine;
use sea_g2p_rs::lang::id::num2id::{n2w, n2w_decimal, n2w_single};
use sea_g2p_rs::lang::id::rules::g2p_word;
use sea_g2p_rs::lang::id::Indonesian;

fn engine() -> G2PEngine {
    let path = format!("{}/python/sea_g2p/sea_g2p.bin", env!("CARGO_MANIFEST_DIR"));
    G2PEngine::new(&path).expect("shipped dictionary")
}

fn read(eng: &G2PEngine, id: &Indonesian, s: &str) -> String {
    id.phonemize_with(s, &eng.dict, |latin| eng.phonemize(latin))
}

#[test]
fn number_irregularities() {
    // 11-19 use belas, and a leading 1 contracts to se-
    assert_eq!(n2w("10"), "sepuluh");
    assert_eq!(n2w("11"), "sebelas");
    assert_eq!(n2w("12"), "dua belas");
    assert_eq!(n2w("100"), "seratus");
    assert_eq!(n2w("200"), "dua ratus");
    assert_eq!(n2w("1000"), "seribu");
    assert_eq!(n2w("1250"), "seribu dua ratus lima puluh");
    assert_eq!(n2w("0"), "nol");
    assert_eq!(n2w_single("081"), "nol delapan satu");
    assert_eq!(n2w_decimal("3", "14"), "tiga koma satu empat");
}

#[test]
fn digraphs_and_diphthongs() {
    assert_eq!(g2p_word("nyanyi").join(" "), "ɲ a ɲ i");
    assert_eq!(g2p_word("syarat").join(" "), "ʃ a r a t");
    assert_eq!(g2p_word("khusus").join(" "), "x u s u s");
    assert_eq!(g2p_word("bangun").join(" "), "b a ŋ u n");
    // ai is a diphthong at the end of a word, hiatus before a consonant
    assert_eq!(g2p_word("pandai").join(" "), "p a n d a j");
    assert_eq!(g2p_word("main").join(" "), "m a i n");
    assert_eq!(g2p_word("pulau").join(" "), "p u l a w");
}

#[test]
fn pre_1972_spellings_in_names() {
    // the reform changed these, but names kept the old forms
    assert_eq!(g2p_word("soeharto").join(" "), "s u h a r t o");
    assert_eq!(g2p_word("gadjah").join(" "), "ɡ a d͡ʒ a h");
    assert_eq!(g2p_word("achmad").join(" "), "a x m a d");
    // nj is NOT among them: menjadi is meN- + jadi, a morpheme boundary
    assert_eq!(g2p_word("menjadi").join(" "), "m ə n d͡ʒ a d i");
}

#[test]
fn schwa_is_the_default_and_the_dictionary_overrides_it() {
    let eng = engine();
    let id = Indonesian::new();
    // by rule every ⟨e⟩ is schwa
    assert_eq!(g2p_word("cerdas").join(" "), "t͡ʃ ə r d a s");
    // …and the dictionary knows where it is not. "sepeda" is /səpeda/: the
    // first ⟨e⟩ is schwa, the second is not, which no rule can predict.
    assert_eq!(read(&eng, &id, "sepeda"), "sə pe da");
}

#[test]
fn code_switching_goes_to_the_english_engine() {
    let eng = engine();
    let id = Indonesian::new();
    let out = read(&eng, &id, "saya pakai iPhone dan Facebook");
    assert!(!out.contains("iphone") && !out.contains("facebook"), "{out}");
    // the Indonesian around it still reads as Indonesian
    assert!(out.starts_with("sa ja"), "{out}");
}

#[test]
fn punctuation_attaches_to_the_group_before_it() {
    let eng = engine();
    let id = Indonesian::new();
    let out = read(&eng, &id, "apa kabar?");
    assert!(out.ends_with('?'), "{out}");
    assert!(!out.ends_with(" ?"), "{out}");
}

#[test]
fn english_routing_is_a_dictionary_test_not_an_engine_test() {
    let eng = engine();
    let id = Indonesian::new();
    // The English engine segments any unknown string and always returns
    // something, so routing on "did it produce output" sent the Indonesian
    // name Gadjah to be read as the English "gad jah".
    assert_eq!(read(&eng, &id, "gadjah"), "ɡa d͡ʒah");
    // a word the English dictionary really has still goes there
    let out = read(&eng, &id, "iphone");
    assert!(out.contains('ˈ'), "expected English stress marks, got {out}");
}

#[test]
fn place_names_are_read_by_rule() {
    let eng = engine();
    let id = Indonesian::new();
    for (name, want) in [
        ("cianjur", "t͡ʃi an d͡ʒur"),
        ("purworejo", "pur wo rə d͡ʒo"),
        ("karanganyar", "ka ra ŋa ɲar"),
    ] {
        assert_eq!(read(&eng, &id, name), want, "{name}");
    }
}
