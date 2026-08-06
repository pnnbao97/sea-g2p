//! Thai lookup tables: abbreviations, symbols, units, calendar names.
//!
//! The abbreviation table is an instance of [`crate::core::abbrev::AbbrevTable`],
//! the same type the Vietnamese module uses, so both languages express "how is
//! this read" as data rather than as branches in the code.
//!
//! Thai abbreviations are written with periods inside them (ม.ค. = January,
//! รธน. = constitution), which is why they must be expanded **before** any
//! sentence-splitting or number pass — otherwise the periods read as sentence
//! boundaries and the letters get spelled out one by one.

use crate::core::abbrev::{AbbrevTable, Reading};
use once_cell::sync::Lazy;
use std::collections::HashMap;

/// One alternate reading of a heteronym, and the words that license it.
///
/// Thai has คำพ้องรูป — one spelling, several pronunciations, several
/// meanings — and the dictionary can hold only one reading per key, so the
/// frequent one lives there and the rest live here with the context that
/// selects them. เพลา is /pʰeː laː/ "time" and /pʰlaw/ "axle"; nothing in
/// the spelling separates them, and no segmenter can, because it is one
/// token either way.
pub struct Heteronym {
    /// Used instead of the dictionary reading when a cue is adjacent.
    pub reading: &'static str,
    /// Words looked for within two tokens either side. Kept deliberately
    /// narrow: a cue that fires too easily is worse than no cue, because the
    /// dictionary default is already the more frequent reading.
    pub cues: &'static [&'static str],
}

/// Heteronyms whose alternate reading is worth selecting for.
///
/// Deliberately small. 1,109 Thai words have more than one recorded reading
/// (7.33% of corpus tokens), but most of that is not ambiguity a listener
/// would notice: 2.89% is sandhi — the linking form a Pali/Sanskrit morpheme
/// takes inside a compound, which the dictionary already carries on the
/// compound itself — and 1.03% differs only in tone between two accepted
/// pronunciations. What is left, where picking wrong says a different word,
/// is a short list, and its head is shorter still: เพลา at 27 per million,
/// everything else near 1.
pub static TH_HETERONYMS: Lazy<HashMap<&'static str, &'static [Heteronym]>> = Lazy::new(|| {
    [
        // เพลา: the dictionary holds /pʰlaw/ "axle". The "time" reading is
        // the one in เพลาเช้า, เพลาบ่าย, เพลาค่ำ.
        (
            "เพลา",
            &[Heteronym {
                reading: "pʰeː˧ laː˧",
                cues: &["เช้า", "สาย", "บ่าย", "เย็น", "ค่ำ", "กลางวัน", "กลางคืน", "ยาม"],
            }][..],
        ),
        // แหน: /hɛːn/ by default — that is the reading inside หวงแหน "to
        // cherish". As the water fern the ห is silent: จอกแหน is /tɕɔːk nɛː/.
        (
            "แหน",
            &[Heteronym {
                reading: "nɛː˩˩˦",
                cues: &["จอก", "สาหร่าย", "บัว", "สระ", "บ่อ", "น้ำ"],
            }][..],
        ),
    ]
    .into_iter()
    .collect()
});

/// Thai letter names, used when an isolated Latin or Thai letter has to be
/// spelled out. Thai spells its own letters as "CV + ชื่อ" (ก = กอ ไก่); the
/// short form is what a reader actually says in an initialism.
pub static TH_LETTER_NAMES: Lazy<HashMap<char, &'static str>> = Lazy::new(|| {
    [
        ('ก', "กอ"), ('ข', "ขอ"), ('ค', "คอ"), ('ง', "งอ"), ('จ', "จอ"),
        ('ฉ', "ฉอ"), ('ช', "ชอ"), ('ซ', "ซอ"), ('ญ', "ยอ"), ('ด', "ดอ"),
        ('ต', "ตอ"), ('ถ', "ถอ"), ('ท', "ทอ"), ('ธ', "ทอ"), ('น', "นอ"),
        ('บ', "บอ"), ('ป', "ปอ"), ('ผ', "ผอ"), ('ฝ', "ฝอ"), ('พ', "พอ"),
        ('ฟ', "ฟอ"), ('ภ', "พอ"), ('ม', "มอ"), ('ย', "ยอ"), ('ร', "รอ"),
        ('ล', "ลอ"), ('ว', "วอ"), ('ศ', "สอ"), ('ษ', "สอ"), ('ส', "สอ"),
        ('ห', "หอ"), ('ฬ', "ลอ"), ('อ', "ออ"), ('ฮ', "ฮอ"),
    ].into_iter().collect()
});

/// Latin letter names as Thai speakers say them, for initialisms and URL
/// schemes: "https" is เอช-ที-ที-พี-เอส.
pub static TH_LATIN_LETTERS: Lazy<HashMap<char, &'static str>> = Lazy::new(|| {
    [
        ('a', "เอ"), ('b', "บี"), ('c', "ซี"), ('d', "ดี"), ('e', "อี"),
        ('f', "เอฟ"), ('g', "จี"), ('h', "เอช"), ('i', "ไอ"), ('j', "เจ"),
        ('k', "เค"), ('l', "แอล"), ('m', "เอ็ม"), ('n', "เอ็น"), ('o', "โอ"),
        ('p', "พี"), ('q', "คิว"), ('r', "อาร์"), ('s', "เอส"), ('t', "ที"),
        ('u', "ยู"), ('v', "วี"), ('w', "ดับเบิลยู"), ('x', "เอ็กซ์"),
        ('y', "วาย"), ('z', "แซด"),
    ].into_iter().collect()
});

/// Month names by number, and the abbreviated spellings that appear in dates.
pub static TH_MONTHS: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    [
        ("ม.ค.", "มกราคม"), ("ก.พ.", "กุมภาพันธ์"), ("มี.ค.", "มีนาคม"),
        ("เม.ย.", "เมษายน"), ("พ.ค.", "พฤษภาคม"), ("มิ.ย.", "มิถุนายน"),
        ("ก.ค.", "กรกฎาคม"), ("ส.ค.", "สิงหาคม"), ("ก.ย.", "กันยายน"),
        ("ต.ค.", "ตุลาคม"), ("พ.ย.", "พฤศจิกายน"), ("ธ.ค.", "ธันวาคม"),
    ].into_iter().collect()
});

/// Units and currency symbols, keyed by the written form.
pub static TH_UNITS: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    [
        ("กม.", "กิโลเมตร"), ("ซม.", "เซนติเมตร"), ("มม.", "มิลลิเมตร"),
        ("กก.", "กิโลกรัม"), ("ก.ก.", "กิโลกรัม"), ("ล.", "ลิตร"),
        ("บ.", "บาท"), ("ตร.ม.", "ตารางเมตร"), ("ตร.กม.", "ตารางกิโลเมตร"),
        ("ชม.", "ชั่วโมง"), ("นาที", "นาที"), ("วิ.", "วินาที"),
        ("%", "เปอร์เซ็นต์"), ("฿", "บาท"), ("$", "ดอลลาร์"),
        ("€", "ยูโร"), ("£", "ปอนด์"), ("¥", "เยน"), ("₩", "วอน"),
        ("°", "องศา"), ("°C", "องศาเซลเซียส"), ("°F", "องศาฟาเรนไฮต์"),
    ].into_iter().collect()
});

/// Latin unit abbreviations, which Thai writing uses as freely as the Thai
/// ones above — `60 km/h`, `50 m2`, `9.8 m/s²`.
///
/// Kept apart from [`TH_UNITS`] because these are matched only after a digit
/// and never carry a trailing period, so they must not join the abbreviation
/// table: `m` and `g` are ordinary letters everywhere else.
pub static TH_LATIN_UNITS: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    [
        ("km", "กิโลเมตร"), ("m", "เมตร"), ("cm", "เซนติเมตร"),
        ("mm", "มิลลิเมตร"), ("nm", "นาโนเมตร"), ("ha", "เฮกตาร์"),
        ("kg", "กิโลกรัม"), ("g", "กรัม"), ("mg", "มิลลิกรัม"),
        ("l", "ลิตร"), ("ml", "มิลลิลิตร"),
        ("h", "ชั่วโมง"), ("hr", "ชั่วโมง"), ("min", "นาที"),
        ("s", "วินาที"), ("sec", "วินาที"), ("ms", "มิลลิวินาที"),
        ("w", "วัตต์"), ("kw", "กิโลวัตต์"), ("mw", "เมกะวัตต์"),
        // "a" and "t" are deliberately absent: ampere and tonne are rare in
        // running text while the bare letters are common, so claiming them
        // costs more than it earns.
        ("v", "โวลต์"), ("kv", "กิโลโวลต์"),
        ("hz", "เฮิรตซ์"), ("khz", "กิโลเฮิรตซ์"), ("mhz", "เมกะเฮิรตซ์"),
        ("ghz", "กิกะเฮิรตซ์"), ("kb", "กิโลไบต์"), ("mb", "เมกะไบต์"),
        ("gb", "กิกะไบต์"), ("tb", "เทระไบต์"), ("kcal", "กิโลแคลอรี"),
    ].into_iter().collect()
});

/// Mathematical and typographic symbols that must become words rather than be
/// deleted. Mirrors the Vietnamese `SYMBOLS_MAP`; the audit test keeps the two
/// in step.
pub static TH_SYMBOLS: Lazy<HashMap<char, &'static str>> = Lazy::new(|| {
    [
        ('&', " และ "), ('+', " บวก "), ('=', " เท่ากับ "), ('<', " น้อยกว่า "),
        ('>', " มากกว่า "), ('≤', " น้อยกว่าหรือเท่ากับ "), ('≥', " มากกว่าหรือเท่ากับ "),
        ('±', " บวกลบ "), ('≈', " ประมาณ "), ('≠', " ไม่เท่ากับ "),
        ('×', " คูณ "), ('÷', " หาร "), ('/', " ทับ "), ('^', " ยกกำลัง "),
        ('√', " รากที่สอง "), ('∞', " อนันต์ "), ('π', " พาย "),
        ('@', " แอท "), ('©', " ลิขสิทธิ์ "), ('→', " ถึง "), ('~', " ประมาณ "),
    ].into_iter().collect()
});

/// The Thai digits ๐–๙ map onto ASCII before any numeric pass runs.
pub fn thai_digit_to_ascii(c: char) -> Option<char> {
    match c {
        '๐'..='๙' => Some((b'0' + (c as u32 - '๐' as u32) as u8) as char),
        _ => None,
    }
}

/// Abbreviations, each carrying how it is read.
///
/// `Expand` entries are replaced by their full Thai words; the surrounding
/// pipeline then segments and looks those words up like any other text.
/// Initialisms with no accepted expansion are spelled with Thai letter names
/// via `LettersVi` (the enum variant is named for its first user, Vietnamese;
/// it means "spell using this language's letter names").
pub static TH_ABBREV: Lazy<AbbrevTable> = Lazy::new(|| {
    let mut t = AbbrevTable::new();
    let expand: &[(&'static str, &'static str)] = &[
        // government, law, administration
        ("รธน.", "รัฐธรรมนูญ"),
        ("ครม.", "คณะรัฐมนตรี"),
        ("สนช.", "สภานิติบัญญัติแห่งชาติ"),
        ("ส.ส.", "สมาชิกสภาผู้แทนราษฎร"),
        ("ส.ว.", "สมาชิกวุฒิสภา"),
        ("กทม.", "กรุงเทพมหานคร"),
        // จ. is deliberately absent: it abbreviates จังหวัด (province) AND
        // จันทร์ (Monday), and mapping it to either misreads half its
        // occurrences. The Vietnamese table makes the same call for T2-T7,
        // which need a time cue before they expand.
        ("ต.", "ตำบล"),
        ("ถ.", "ถนน"),
        ("ร.ร.", "โรงเรียน"),
        ("รพ.", "โรงพยาบาล"),
        // Deliberately absent: ม. (มหาวิทยาลัย / หมู่ / มัธยม) and อ.
        // (อำเภอ / อาจารย์). Both readings of each are common, and a hard
        // mapping would misread half the occurrences — the same call the
        // Vietnamese table makes for weekday abbreviations, which need a
        // context cue before they expand.
        ("บจก.", "บริษัทจำกัด"),
        ("บมจ.", "บริษัทมหาชนจำกัด"),
        ("หจก.", "ห้างหุ้นส่วนจำกัด"),
        // titles
        ("ดร.", "ดอกเตอร์"),
        ("ศ.", "ศาสตราจารย์"),
        ("รศ.", "รองศาสตราจารย์"),
        ("ผศ.", "ผู้ช่วยศาสตราจารย์"),
        ("นพ.", "นายแพทย์"),
        ("พญ.", "แพทย์หญิง"),
        ("พล.อ.", "พลเอก"),
        ("พ.ต.ท.", "พันตำรวจโท"),
        // calendar and time
        ("พ.ศ.", "พุทธศักราช"),
        ("ค.ศ.", "คริสต์ศักราช"),
        ("น.", "นาฬิกา"),
        // everyday
        ("ฯลฯ", "และอื่นๆ"),
        ("ฯพณฯ", "ฯพณฯ"),
        ("โทร.", "โทรศัพท์"),
        ("เลขที่", "เลขที่"),
    ];
    for (k, v) in expand {
        t.insert(k, Reading::Expand(v));
    }
    for (k, v) in TH_MONTHS.iter() {
        t.insert(k, Reading::Expand(v));
    }
    for (k, v) in TH_UNITS.iter() {
        if k.chars().all(|c| !c.is_ascii_punctuation() || c == '.') && k.ends_with('.') {
            t.insert(k, Reading::Expand(v));
        }
    }
    t
});
