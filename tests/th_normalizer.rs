//! Integration tests for the Thai text normalizer.

use sea_g2p_rs::lang::th::normalizer::{audit_unmapped, normalize};
#[test]
fn thai_digits_become_numbers() {
    assert_eq!(normalize("๑๒๓"), "หนึ่งร้อยยี่สิบสาม");
    assert_eq!(normalize("พ.ศ. ๒๕๖๐"), "พุทธศักราช สองพันห้าร้อยหกสิบ");
}

#[test]
fn repetition_mark_is_left_for_the_token_stage() {
    // ๆ repeats the preceding WORD, and word boundaries only exist after
    // segmentation — see tests/th_repetition.rs for the behaviour itself.
    assert_eq!(normalize("ต่างๆ"), "ต่างๆ");
    // ฯลฯ expands with its ๆ attached so the segmenter sees และ|อื่น|ๆ
    assert_eq!(normalize("ฯลฯ"), "และอื่นๆ");
}

#[test]
fn abbreviations_expand_and_do_not_vanish() {
    // รธน. used to disappear entirely — the exact failure the audit exists for
    assert!(normalize("ตาม รธน. มาตรา 44").contains("รัฐธรรมนูญ"));
    assert!(normalize("วันที่ 6 ม.ค. 2560").contains("มกราคม"));
    assert!(normalize("ส.ส. คนใหม่").contains("สมาชิกสภาผู้แทนราษฎร"));
}

#[test]
fn money_percent_and_temperature() {
    assert_eq!(normalize("฿500"), "ห้าร้อย บาท");
    assert!(normalize("ราคา 1,250 บาท").contains("หนึ่งพันสองร้อยห้าสิบ"));
    assert!(normalize("50%").contains("เปอร์เซ็นต์"));
    assert!(normalize("30°C").contains("องศาเซลเซียส"));
}

#[test]
fn dates_and_times() {
    let d = normalize("6/1/2560");
    assert!(d.contains("มกราคม") && d.contains("สองพันห้าร้อยหกสิบ"), "{d}");
    // the era marker must arrive already expanded: this stage runs after the
    // abbreviation pass, so an emitted "พ.ศ." would read as letters + periods
    assert!(d.contains("พุทธศักราช"), "{d}");
    assert!(!d.contains("พ.ศ."), "{d}");
    assert!(normalize("14:30").contains("นาฬิกา"));
}

#[test]
fn nothing_is_deleted_in_silence() {
    // every symbol we claim to support must be declared somewhere
    let inventory = "฿$€£¥₩%°&+=<>≤≥±≈≠×÷/^√∞π@©→~ๆฯ๐๑๒๓๔๕๖๗๘๙";
    assert_eq!(audit_unmapped(inventory), Vec::<char>::new());
    // an undeclared symbol IS reported
    assert_eq!(audit_unmapped("∮"), vec!['∮']);
}

#[test]
fn plain_text_is_untouched() {
    assert_eq!(normalize("เขาฉลาด"), "เขาฉลาด");
}

#[test]
fn decimal_is_not_a_clock_time() {
    // "3.14 เมตร" used to read as "3 นาฬิกา 14 นาที": a period between digits
    // is a decimal point unless a นาฬิกา / น. cue says otherwise.
    let d = normalize("3.14 เมตร");
    assert!(d.contains("จุด"), "{d}");
    assert!(!d.contains("นาฬิกา"), "{d}");
    // with the cue, the dotted form IS a time
    assert!(normalize("14.30 น.").contains("นาฬิกา"));
}

#[test]
fn phone_numbers_are_read_figure_by_figure() {
    // as a cardinal "081" reads "eighty-one" and the leading zero vanishes
    let p = normalize("โทร 081-234-5678");
    assert!(p.contains("ศูนย์"), "{p}");
    assert!(!p.contains("แปดสิบเอ็ด"), "{p}");
    // a lone leading zero marks an identifier anywhere, not just in phones
    assert_eq!(normalize("รหัส 007"), "รหัส ศูนย์ ศูนย์ เจ็ด");
}

#[test]
fn a_clause_comma_is_not_a_thousands_separator() {
    // "ISBN 3211812164, ISBN ..." used to absorb the sentence comma, look like
    // grouped digits, and be read as a cardinal in the billions.
    let out = normalize("ISBN 3211812164, ISBN 9783211812167");
    assert!(!out.contains("ล้าน"), "{out}");
    assert!(out.contains("สาม สอง หนึ่ง"), "{out}");
    // a real thousands separator still groups
    assert!(normalize("1,250").contains("หนึ่งพันสองร้อยห้าสิบ"));
}

#[test]
fn markup_runs_are_not_operators() {
    // wiki headings were read as "equals equals References equals equals"
    let out = normalize("== อ้างอิง ==");
    assert!(!out.contains("เท่ากับ"), "{out}");
    assert!(out.contains("อ้างอิง"), "{out}");
    // a single = is still an operator
    assert!(normalize("a = b").contains("เท่ากับ"));
}

#[test]
fn mathematical_notation_is_not_deleted_in_silence() {
    // each of these used to vanish, changing the meaning with no audible cue
    assert!(normalize("อุณหภูมิ -5 องศา").contains("ลบ"), "minus sign");
    assert!(normalize("10⁻³").contains("ลบ"), "negative exponent");
    assert!(normalize("5 m²").contains("กำลังสอง"), "squared");
    assert!(normalize("2 m³").contains("กำลังสาม"), "cubed");
    assert!(normalize("H₂O").contains("สอง"), "subscript");
    assert!(normalize("10-20 ปี").contains("ถึง"), "range");
    assert!(normalize("3 x 4").contains("คูณ"), "multiplication");
    assert!(normalize("1/2").contains("ส่วน"), "fraction");
}

#[test]
fn the_audit_verifies_numeric_hyphens_rather_than_assuming() {
    // Declaring '-' intentionally dropped is what let "-5" lose its sign
    // while the audit stayed silent. Now that the math stage reads it, the
    // audit must stay quiet — and speak up only if that stage ever stops
    // handling it, which is why it verifies rather than assumes.
    assert_eq!(audit_unmapped("-5"), Vec::<char>::new());
    assert_eq!(audit_unmapped("10-20"), Vec::<char>::new());
    // …and not when it is an ordinary hyphen between letters
    assert_eq!(audit_unmapped("ก-ข"), Vec::<char>::new());
}

#[test]
fn emails_and_urls_are_read_whole() {
    // "https://www.google.com" came out as "https, ทับ ทับ www.google.com":
    // the separators voiced, the domain never read. The scheme is READ, not
    // dropped — text that says https:// means it — and spelled with Thai
    // letter names, as Vietnamese spells it "hát tê tê phê ét".
    let u = normalize("ดู https://www.google.com");
    assert!(u.contains("จุด"), "{u}");              // the dots are spoken
    assert!(u.contains("เอช ที ที พี เอส"), "{u}"); // h-t-t-p-s
    assert!(u.contains("ทวิภาค"), "{u}");           // colon
    assert!(u.matches("ทับ").count() >= 2, "{u}");  // slash slash
    assert!(!u.contains("//"), "{u}");              // never as raw characters
    let e = normalize("ส่งไป admin@example.com");
    assert!(e.contains("แอท") && e.contains("จุด"), "{e}");
}
