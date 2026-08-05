//! Indonesian lookup tables.
//!
//! The abbreviation table is an instance of the shared
//! [`crate::core::abbrev::AbbrevTable`], as the Vietnamese and Thai ones are,
//! so all three express "how is this read" as data.
//!
//! Indonesian writing abbreviates heavily, and in two distinct registers that
//! both reach a TTS front end: formal initialisms (DPR, NKRI, KTP) and the
//! chat contractions that pervade informal text (yg, dgn, tdk, utk). The
//! second group is what a naive pipeline mangles, since the letters look like
//! a pronounceable word.

use crate::core::abbrev::{AbbrevTable, Reading};
use once_cell::sync::Lazy;
use std::collections::HashMap;

pub const ID_MONTHS: [&str; 12] = [
    "Januari", "Februari", "Maret", "April", "Mei", "Juni",
    "Juli", "Agustus", "September", "Oktober", "November", "Desember",
];

/// Unit abbreviations, matched only after a digit. Most are spoken as the
/// full Indonesian word, so the entry is a genuine expansion rather than a
/// pass-through; `jam` and `menit` map to themselves because the written form
/// is already the word and the table is what licenses a `/` to read "per".
pub static ID_UNITS: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    [
        ("km", "kilometer"), ("m", "meter"), ("cm", "sentimeter"),
        ("mm", "milimeter"), ("nm", "nanometer"), ("ha", "hektar"),
        ("kg", "kilogram"), ("g", "gram"), ("mg", "miligram"),
        ("l", "liter"), ("ml", "mililiter"),
        ("jam", "jam"), ("j", "jam"), ("menit", "menit"), ("mnt", "menit"),
        ("detik", "detik"), ("dtk", "detik"), ("s", "detik"),
        ("w", "watt"), ("kw", "kilowatt"), ("mw", "megawatt"),
        ("v", "volt"), ("kv", "kilovolt"),
        ("hz", "hertz"), ("khz", "kilohertz"), ("mhz", "megahertz"),
        ("ghz", "gigahertz"), ("kb", "kilobita"), ("mb", "megabita"),
        ("gb", "gigabita"), ("tb", "terabita"), ("kkal", "kilokalori"),
    ].into_iter().collect()
});

pub static ID_SYMBOLS: Lazy<HashMap<char, &'static str>> = Lazy::new(|| {
    [
        ('&', " dan "), ('+', " plus "), ('=', " sama dengan "),
        ('<', " kurang dari "), ('>', " lebih dari "), ('±', " kurang lebih "),
        ('≈', " kira-kira "), ('≠', " tidak sama dengan "),
        ('×', " kali "), ('÷', " bagi "), ('/', " garis miring "),
        ('%', " persen "), ('°', " derajat "), ('@', " at "),
        ('©', " hak cipta "), ('→', " ke "), ('~', " kira-kira "),
        ('$', " dolar "), ('€', " euro "), ('£', " pound "), ('¥', " yen "),
    ].into_iter().collect()
});

pub static ID_ABBREV: Lazy<AbbrevTable> = Lazy::new(|| {
    let mut t = AbbrevTable::new();
    // Chat and note-taking contractions. These are the dangerous ones: "yg"
    // and "dgn" look like words a rule engine will happily mispronounce.
    let expand: &[(&'static str, &'static str)] = &[
        ("yg", "yang"), ("dgn", "dengan"), ("dg", "dengan"),
        ("tdk", "tidak"), ("tsb", "tersebut"), ("utk", "untuk"),
        ("dlm", "dalam"), ("dr", "dari"), ("krn", "karena"),
        ("sdh", "sudah"), ("blm", "belum"), ("jd", "jadi"),
        ("bhw", "bahwa"), ("spt", "seperti"), ("dpt", "dapat"),
        ("hrs", "harus"), ("byk", "banyak"), ("org", "orang"),
        ("thn", "tahun"), ("bln", "bulan"), ("hr", "hari"),
        ("jl", "jalan"), ("no", "nomor"), ("tgl", "tanggal"),
        ("kpd", "kepada"), ("ttg", "tentang"), ("sbg", "sebagai"),
        ("pd", "pada"), ("ybs", "yang bersangkutan"),
        ("dll", "dan lain-lain"), ("dsb", "dan sebagainya"),
        ("dkk", "dan kawan-kawan"), ("yth", "yang terhormat"),
        // titles and honorifics
        ("bpk", "bapak"), ("ibu", "ibu"), ("sdr", "saudara"),
        ("drs", "doktorandus"), ("ir", "insinyur"), ("prof", "profesor"),
        // institutions read as words, not spelled
        ("pt", "perseroan terbatas"), ("cv", "commanditaire vennootschap"),
        ("ri", "Republik Indonesia"),
        ("kel", "kelurahan"), ("kec", "kecamatan"), ("kab", "kabupaten"),
        ("prov", "provinsi"),
    ];
    for (k, v) in expand {
        t.insert(k, Reading::Expand(v));
    }
    // Initialisms spoken as words rather than spelled — Indonesian does this
    // far more than English does.
    for k in ["ASEAN", "UNESCO", "SIM", "KTP", "PAUD", "SIMPEG", "BUMN"] {
        t.insert(k, Reading::WordEn);
    }
    // Spelled with Indonesian letter names. Deliberately absent: single- and
    // two-letter forms whose reading depends on context.
    for k in ["DPR", "MPR", "NKRI", "KPK", "TNI", "PNS", "SMA", "SMP", "SD",
              "PLN", "KTT", "HUT", "RT", "RW"] {
        t.insert(k, Reading::LettersNative);
    }
    t
});

/// Indonesian letter names, for initialisms read letter by letter.
pub static ID_LETTER_NAMES: Lazy<HashMap<char, &'static str>> = Lazy::new(|| {
    [
        ('a', "a"), ('b', "be"), ('c', "ce"), ('d', "de"), ('e', "e"),
        ('f', "ef"), ('g', "ge"), ('h', "ha"), ('i', "i"), ('j', "je"),
        ('k', "ka"), ('l', "el"), ('m', "em"), ('n', "en"), ('o', "o"),
        ('p', "pe"), ('q', "ki"), ('r', "er"), ('s', "es"), ('t', "te"),
        ('u', "u"), ('v', "ve"), ('w', "we"), ('x', "eks"), ('y', "ye"),
        ('z', "zet"),
    ].into_iter().collect()
});
