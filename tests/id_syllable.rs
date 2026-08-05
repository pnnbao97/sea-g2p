//! Indonesian syllabification, and the output convention it exists to keep.
//!
//! The library emits one space per syllable boundary in every language.
//! Indonesian phonemes arrive from the dictionary one per token, so without
//! this pass `saya makan` came out as nine groups where Vietnamese and Thai
//! would emit four — one library, two formats.

use sea_g2p_rs::core::dict::PhonemeDict;
use sea_g2p_rs::lang::id::syllable::syllabify;
use sea_g2p_rs::lang::id::Indonesian;

fn syl(s: &str) -> String {
    let phones: Vec<String> = s.split_whitespace().map(str::to_string).collect();
    syllabify(&phones).join(" ")
}

#[test]
fn one_consonant_between_vowels_opens_the_next_syllable() {
    assert_eq!(syl("s a j a"), "sa ja");
    assert_eq!(syl("b a t͡ʃ a"), "ba t͡ʃa");
}

#[test]
fn two_consonants_split() {
    assert_eq!(syl("m a n d i"), "man di");
    assert_eq!(syl("b a ŋ k u"), "baŋ ku");
}

#[test]
fn a_valid_onset_cluster_stays_together() {
    // loan clusters only; native roots have no complex onsets
    assert_eq!(syl("i n f r a"), "in fra");
}

#[test]
fn adjacent_vowels_are_separate_syllables() {
    assert_eq!(syl("d i a"), "di a");
    assert_eq!(syl("m a i n"), "ma in");
}

#[test]
fn a_glide_closes_a_syllable_when_no_vowel_follows() {
    // /j/ between vowels opens a syllable, at the end it closes one
    assert_eq!(syl("p a n d a j"), "pan daj");
    assert_eq!(syl("p u l a w"), "pu law");
}

#[test]
fn output_groups_match_syllable_count() {
    let path = format!("{}/python/sea_g2p/sea_g2p.bin", env!("CARGO_MANIFEST_DIR"));
    let dict = PhonemeDict::new(&path).expect("shipped dictionary");
    let id = Indonesian::new();
    // saya makan is sa-ya ma-kan: four syllables, four groups
    assert_eq!(id.phonemize("saya makan", &dict).split_whitespace().count(), 4);
    assert_eq!(id.phonemize("menyembunyikan", &dict).split_whitespace().count(), 5);
}
