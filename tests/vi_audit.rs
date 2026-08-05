//! Integration tests for the Vietnamese silent-deletion audit.

use sea_g2p_rs::lang::vi::audit::audit_unmapped;
#[test]
fn letters_and_digits_are_always_safe() {
    assert!(audit_unmapped("Xin chào 123 abc").is_empty());
}

#[test]
fn previously_lost_symbols_are_now_declared() {
    // Every character here once vanished silently, or belongs to the same
    // family as one that did.
    assert!(audit_unmapped("10⁻³ Σ ∑ ∆ Δ ± ≈ ° √ ∫ µ ¥ % @ / :").is_empty());
}

#[test]
fn unknown_symbol_is_reported() {
    // U+2318 PLACE OF INTEREST SIGN has no reading and must be flagged.
    assert_eq!(audit_unmapped("phím ⌘ trên máy Mac"), vec!['⌘']);
}

#[test]
fn report_is_deduplicated_and_ordered() {
    assert_eq!(audit_unmapped("⌘ ⌥ ⌘ ⌥"), vec!['⌘', '⌥']);
}
