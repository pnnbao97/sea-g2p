//! Foreign Latin letters folded onto their ASCII base.
//!
//! The pipeline's final whitelist replaces every unknown character with a
//! space, so an unfolded "ü" does not just lose its diaeresis — it splits the
//! word around it. These tests pin both halves of the contract: foreign
//! letters must fold, and Vietnamese letters must not.

use sea_g2p_rs::lang::vi::resources::VI_LETTERS;
use sea_g2p_rs::lang::vi::translit::fold_foreign_letters;

#[test]
fn umlauts_fold_to_the_base_letter() {
    assert_eq!(fold_foreign_letters("Müller"), "Muller");
    assert_eq!(fold_foreign_letters("München"), "Munchen");
    assert_eq!(fold_foreign_letters("Zoë"), "Zoe");
    assert_eq!(fold_foreign_letters("Ångström"), "Angstrom");
}

#[test]
fn letters_with_a_stroke_or_ligature_use_the_table() {
    // No decomposition recovers these: the mark is part of the glyph.
    assert_eq!(fold_foreign_letters("Straße"), "Strasse");
    assert_eq!(fold_foreign_letters("Łódź"), "Lódz"); // "ó" is Vietnamese, kept
    assert_eq!(fold_foreign_letters("Ørsted"), "Orsted");
    assert_eq!(fold_foreign_letters("Ærø"), "AEro");
    assert_eq!(fold_foreign_letters("ırmak"), "irmak");
}

#[test]
fn vietnamese_letters_are_never_folded() {
    // "ế" decomposes to e + circumflex + acute: without the guard this pass
    // would flatten the language it serves.
    assert_eq!(fold_foreign_letters("Tiếng Việt đường phố"), "Tiếng Việt đường phố");
    let all: String = VI_LETTERS.to_string();
    assert_eq!(fold_foreign_letters(&all), all);
}

#[test]
fn non_latin_scripts_are_left_alone() {
    // Greek has readings in SYMBOLS_MAP ("α" -> "an pha"); folding it to "a"
    // would destroy them. Cyrillic and CJK are simply not this pass's business.
    assert_eq!(fold_foreign_letters("góc α và β"), "góc α và β");
    assert_eq!(fold_foreign_letters("5 µm, 10 Ω"), "5 µm, 10 Ω");
    assert_eq!(fold_foreign_letters("Привет 日本"), "Привет 日本");
}

#[test]
fn ascii_is_returned_unchanged() {
    assert_eq!(fold_foreign_letters("plain ASCII 123"), "plain ASCII 123");
    assert_eq!(fold_foreign_letters(""), "");
}

#[test]
fn every_european_latin_letter_has_an_ascii_form() {
    // Latin-1 Supplement (U+00C0..U+00FF) and Latin Extended-A
    // (U+0100..U+017F) together cover the alphabets of western, northern,
    // central and eastern Europe. Any letter here that the fold declines would
    // be replaced by a space downstream and split the word it sits in, so the
    // range must have no holes.
    let vi: std::collections::HashSet<char> = VI_LETTERS.chars().collect();
    let mut unfolded: Vec<char> = Vec::new();
    for cp in 0x00C0u32..=0x017Fu32 {
        let c = char::from_u32(cp).unwrap();
        if !c.is_alphabetic() || vi.contains(&c) {
            continue;
        }
        let folded = fold_foreign_letters(&c.to_string());
        if !folded.chars().all(|d: char| d.is_ascii_alphabetic()) {
            unfolded.push(c);
        }
    }
    assert!(unfolded.is_empty(), "letters with no ASCII form: {:?}", unfolded);
}
