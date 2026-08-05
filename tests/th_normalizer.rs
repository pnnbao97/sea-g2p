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

#[test]
fn roman_numerals_need_a_cue() {
    // Thai writes reign names this way constantly
    assert!(normalize("รัชกาลที่ IX").contains("เก้า"), "reign IX");
    // without a cue, letters stay letters: CD is a disc, not four hundred
    assert_eq!(normalize("แผ่น CD"), "แผ่น CD");
}

#[test]
fn identifiers_need_a_cue_too() {
    // after a cue the digits are figures, not a quantity
    let p = normalize("ทะเบียน กก 1234");
    assert!(p.contains("หนึ่ง สอง สาม สี่"), "{p}");
    // without one, 1234 is still one thousand two hundred and thirty-four
    assert!(normalize("มี 1234 ตัว").contains("หนึ่งพัน"), "no cue");
}

#[test]
fn markup_tags_are_stripped_not_read() {
    // "<math>" was being read as "less than math greater than"
    let m = normalize("<math>b² - 4ac</math>");
    assert!(!m.contains("math"), "{m}");
    assert!(m.contains("กำลังสอง") && m.contains("ลบ"), "{m}");
}

#[test]
fn latin_unit_abbreviations_are_read_as_words() {
    // "60 km/h" read as "km ทับ h": the abbreviation unread and the slash
    // voiced as punctuation rather than as "per".
    assert_eq!(normalize("60 km/h"), "หกสิบ กิโลเมตร ต่อ ชั่วโมง");
    assert!(normalize("9.8 m/s²").contains("เมตร ต่อ วินาที กำลังสอง"));
    // a trailing 2 or 3 on a unit is an exponent, not a count
    assert_eq!(normalize("50 m2"), "ห้าสิบ ตารางเมตร");
    assert_eq!(normalize("2 m3"), "สอง ลูกบาศก์เมตร");
    // a letter run that is NOT a unit must be left alone
    assert!(normalize("รุ่น 5 abc").contains("abc"));
}

#[test]
fn prime_marks_next_to_a_digit_are_measurements() {
    // '5\'6"' used to read "ห้า หก" — both marks deleted in silence
    assert_eq!(normalize("5'6\""), "ห้า ฟุต หก นิ้ว");
    let c = normalize("13° 45' 30\"");
    assert!(c.contains("ลิปดา") && c.contains("ฟิลิปดา"), "{c}");
    // a prime nothing reads IS reported, and a plain quotation mark is not
    assert_eq!(audit_unmapped("5'"), vec!['\'']);
    assert_eq!(audit_unmapped("\"คำ\""), Vec::<char>::new());
}

#[test]
fn a_ratio_is_not_a_clock_time() {
    assert!(normalize("อัตรา 3:1").contains("สาม ต่อ หนึ่ง"));
    // the time pass still has the prior claim
    assert!(normalize("14:30").contains("นาฬิกา"));
}

#[test]
fn e_notation_keeps_its_order_of_magnitude() {
    // "1.5e10" read as "…e สิบ": ten orders of magnitude lost inaudibly
    let s = normalize("1.5e10");
    assert!(s.contains("คูณ สิบ ยกกำลัง สิบ"), "{s}");
    assert!(normalize("1e-9").contains("ยกกำลัง ลบ เก้า"));
}

#[test]
fn weekday_abbreviations_need_the_day_cue() {
    assert_eq!(normalize("วันจ."), "วันจันทร์");
    assert_eq!(normalize("วันอา."), "วันอาทิตย์");
    assert_eq!(normalize("วันพฤ."), "วันพฤหัสบดี");
    // ศ. after วัน is Friday; on its own it is still professor, which is why
    // the cue exists — "วันศ." used to read "วัน ศาสตราจารย์"
    assert!(normalize("วันศ. ที่ 5").contains("วันศุกร์"));
    assert!(normalize("ศ.ดร.สมชาย").contains("ศาสตราจารย์"));
}

#[test]
fn a_lone_latin_letter_is_spelled_in_thai() {
    // "วิตามิน C" phonemised through the English engine, putting an
    // out-of-inventory token in the middle of a Thai sentence
    assert_eq!(normalize("วิตามิน C"), "วิตามิน ซี");
    assert_eq!(normalize("กระดาษ A4"), "กระดาษ เอ สี่");
    // Thai runs on without spaces, so the letter need not be space-separated
    assert_eq!(normalize("กระดาษA4"), "กระดาษ เอ สี่");
    // but a letter inside an English phrase belongs to that phrase
    assert_eq!(normalize("Grade A student"), "Grade A student");
    // and English words are still left for the code-switching path
    assert!(normalize("ผมใช้ iPhone").contains("iPhone"));
    assert!(normalize("แผ่น CD").contains("CD"));
}

#[test]
fn the_audit_is_quiet_on_ordinary_thai() {
    // Tone marks and thanthakhat are combining marks, not letters, so an
    // `is_alphanumeric` whitelist reported them as dropped — on text that
    // keeps them. Thai marks tone on nearly every word, so the audit fired
    // on essentially every real sentence and buried its own findings.
    for s in [
        "เขาฉลาดพอที่จะซ่อนสติปัญญา",
        "น้ำท่วมหนักที่จังหวัดเชียงใหม่",
        "ผมใช้ iPhone ราคา ฿1,250 ที่กรุงเทพฯ",
    ] {
        assert_eq!(audit_unmapped(s), Vec::<char>::new(), "{s}");
    }
    // every mark in the Thai block survives, so none may be reported
    for c in '\u{0E01}'..='\u{0E4E}' {
        let s: String = ['ก', c, 'ก'].iter().collect();
        assert_eq!(audit_unmapped(&s), Vec::<char>::new(), "{c:?} U+{:04X}", c as u32);
    }
    // and a genuinely undeclared symbol is still caught
    assert_eq!(audit_unmapped("∮ ก"), vec!['∮']);
}

#[test]
fn the_audit_reports_scripts_the_pipeline_deletes() {
    // stage_residual keeps Thai, ASCII and kept punctuation — nothing else.
    // `is_alphanumeric` accepts ā ṁ Ω and CJK, so the audit passed text the
    // pipeline was busy dropping. Fixing the combining-mark false positives
    // without this left the guard blind in the direction that hides losses.
    assert_eq!(audit_unmapped("ก ā ก"), vec!['ā']);
    assert_eq!(audit_unmapped("ก Ω ก"), vec!['Ω']);
    assert_eq!(audit_unmapped("ก 日本 ก"), vec!['日', '本']);
    // and ordinary Thai, including code-switched Latin, stays quiet
    for s in ["เขาฉลาดพอที่จะซ่อนสติปัญญา", "ผมใช้ iPhone ราคา ฿1,250"] {
        assert_eq!(audit_unmapped(s), Vec::<char>::new(), "{s}");
    }
}
