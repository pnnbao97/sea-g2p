//! Integration tests for the memory-mapped phoneme dictionary loader.
//!
//! These read the real shipped binary, so they also assert that the file in
//! `python/sea_g2p/` is a well-formed v2 with the Thai section present.

use sea_g2p_rs::core::dict::{PhonemeDict, SECTION_TH};
fn repo_bin() -> String {
    format!("{}/python/sea_g2p/sea_g2p.bin", env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn v2_legacy_tables_and_thai_section() {
    let d = PhonemeDict::new(&repo_bin()).unwrap();
    // legacy vi/en lookups still resolve
    assert!(d.lookup_merged("xin").is_some());
    assert!(d.lookup_common("go").is_some() || d.lookup_merged("go").is_some());
    // Thai section resolves with tones intact
    assert_eq!(d.lookup_section(SECTION_TH, "สวัสดี"), Some("sa˨˩ wat̚˨˩ diː˧"));
    assert_eq!(d.lookup_section(SECTION_TH, "ให้"), Some("haj˥˩"));
    assert!(d.lookup_section(SECTION_TH, "notaword").is_none());
    // absent section kind is a clean miss
    assert!(d.lookup_section(99, "สวัสดี").is_none());
}
