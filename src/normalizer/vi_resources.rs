use std::collections::HashMap;
use once_cell::sync::Lazy;

pub const VI_LETTER_NAMES: &[(&str, &str)] = &[
    ("a", "a"), ("b", "bê"), ("c", "xê"), ("d", "đê"), ("đ", "đê"), ("e", "e"), ("ê", "ê"),
    ("f", "ép"), ("g", "gờ"), ("h", "hát"), ("i", "i"), ("j", "giây"), ("k", "ca"), ("l", "lờ"),
    ("m", "mờ"), ("n", "nờ"), ("o", "o"), ("ô", "ô"), ("ơ", "ơ"), ("p", "pê"), ("q", "qui"),
    ("r", "rờ"), ("s", "ét"), ("t", "tê"), ("u", "u"), ("ư", "ư"), ("v", "vê"), ("w", "đắp liu"),
    ("x", "ích"), ("y", "y"), ("z", "dét"),
    ("0", "không"), ("1", "một"), ("2", "hai"), ("3", "ba"), ("4", "bốn"),
    ("5", "năm"), ("6", "sáu"), ("7", "bảy"), ("8", "tám"), ("9", "chín")
];

pub static VI_LETTER_NAMES_MAP: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    VI_LETTER_NAMES.iter().cloned().collect()
});

pub static VI_LETTER_NAMES_MAP_MAP: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    VI_LETTER_NAMES.iter().cloned().collect()
});

pub const COMMON_EMAIL_DOMAINS: &[(&str, &str)] = &[
    ("gmail.com", "__start_en__gmail__end_en__ chấm com"),
    ("yahoo.com", "__start_en__yahoo__end_en__ chấm com"),
    ("yahoo.com.vn", "__start_en__yahoo__end_en__ chấm com chấm __start_en__v n__end_en__"),
    ("outlook.com", "__start_en__outlook__end_en__ chấm com"),
    ("hotmail.com", "__start_en__hotmail__end_en__ chấm com"),
    ("icloud.com", "__start_en__icloud__end_en__ chấm com"),
    ("fpt.vn", "__start_en__f p t__end_en__ chấm __start_en__v n__end_en__"),
    ("fpt.com.vn", "__start_en__f p t__end_en__ chấm com chấm __start_en__v n__end_en__"),
];

pub static COMMON_EMAIL_DOMAINS_MAP: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    COMMON_EMAIL_DOMAINS.iter().cloned().collect()
});

pub const MEASUREMENT_KEY_VI: &[(&str, &str)] = &[
    ("km", "ki lô mét"), ("dm", "đê xi mét"), ("cm", "xen ti mét"), ("mm", "mi li mét"),
    ("nm", "na nô mét"), ("µm", "mic rô mét"), ("μm", "mic rô mét"), ("m", "mét"),
    ("kg", "ki lô gam"), ("g", "gam"), ("mg", "mi li gam"),
    ("km2", "ki lô mét vuông"), ("m2", "mét vuông"), ("cm2", "xen ti mét vuông"), ("mm2", "mi li mét vuông"),
    ("ha", "héc ta"),
    ("km3", "ki lô mét khối"), ("m3", "mét khối"), ("cm3", "xen ti mét khối"), ("mm3", "mi li mét khối"),
    ("l", "lít"), ("dl", "đê xi lít"), ("ml", "mi li lít"), ("hl", "héc tô lít"),
    ("kw", "ki lô oát"), ("mw", "mê ga oát"), ("gw", "gi ga oát"),
    ("kwh", "ki lô oát giờ"), ("mwh", "mê ga oát giờ"), ("wh", "oát giờ"),
    ("hz", "héc"), ("khz", "ki lô héc"), ("mhz", "mê ga héc"), ("ghz", "gi ga héc"),
    ("pa", "__start_en__pascal__end_en__"), ("kpa", "__start_en__kilopascal__end_en__"), ("mpa", "__start_en__megapascal__end_en__"),
    ("bar", "__start_en__bar__end_en__"), ("mbar", "__start_en__millibar__end_en__"), ("atm", "__start_en__atmosphere__end_en__"), ("psi", "__start_en__p s i__end_en__"),
    ("j", "__start_en__joule__end_en__"), ("kj", "__start_en__kilojoule__end_en__"),
    ("cal", "__start_en__calorie__end_en__"), ("kcal", "__start_en__kilocalorie__end_en__"),
    ("h", "giờ"), ("p", "phút"), ("s", "giây"),
    ("sqm", "mét vuông"), ("cum", "mét khối"),
    ("gb", "__start_en__gigabyte__end_en__"), ("mb", "__start_en__megabyte__end_en__"), ("kb", "__start_en__kilobyte__end_en__"), ("tb", "__start_en__terabyte__end_en__"),
    ("db", "__start_en__decibel__end_en__"), ("oz", "__start_en__ounce__end_en__"), ("lb", "__start_en__pound__end_en__"), ("lbs", "__start_en__pounds__end_en__"),
    ("ft", "__start_en__feet__end_en__"), ("in", "__start_en__inch__end_en__"), ("dpi", "__start_en__d p i__end_en__"), ("pH", "pê hát"),
    ("gbps", "__start_en__gigabits per second__end_en__"), ("mbps", "__start_en__megabits per second__end_en__"), ("kbps", "__start_en__kilobits per second__end_en__"),
    ("gallon", "__start_en__gallon__end_en__"), ("mol", "mol"), ("ms", "mi li giây"), ("M", "triệu")
];

pub static MEASUREMENT_KEY_VI_MAP: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    MEASUREMENT_KEY_VI.iter().cloned().collect()
});

pub const CURRENCY_KEY: &[(&str, &str)] = &[
    ("usd", "__start_en__u s d__end_en__"),
    ("vnd", "đồng"), ("đ", "đồng"), ("v n d", "đồng"), ("v n đ", "đồng"), ("€", "__start_en__euro__end_en__"), ("euro", "__start_en__euro__end_en__"), ("eur", "__start_en__euro__end_en__"),
    ("¥", "yên"), ("yên", "yên"), ("jpy", "yên"), ("%", "phần trăm")
];

pub static CURRENCY_KEY_MAP: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    CURRENCY_KEY.iter().cloned().collect()
});

pub const ACRONYMS_EXCEPTIONS_VI: &[(&str, &str)] = &[
    ("CĐV", "cổ động viên"), ("HĐND", "hội đồng nhân dân"), ("HĐQT", "hội đồng quản trị"), ("TAND", "toàn án nhân dân"),
    ("BHXH", "bảo hiểm xã hội"), ("BHTN", "bảo hiểm thất nghiệp"), ("TP.HCM", "thành phố hồ chí minh"),
    ("VN", "việt nam"), ("UBND", "uỷ ban nhân dân"), ("TP", "thành phố"), ("HCM", "hồ chí minh"),
    ("HN", "hà nội"), ("BTC", "ban tổ chức"), ("CLB", "câu lạc bộ"), ("HTX", "hợp tác xã"),
    ("NXB", "nhà xuất bản"), ("TW", "trung ương"), ("CSGT", "cảnh sát giao thông"), ("LHQ", "liên hợp quốc"),
    ("THCS", "trung học cơ sở"), ("THPT", "trung học phổ thông"), ("ĐH", "đại học"), ("HLV", "huấn luyện viên"),
    ("GS", "giáo sư"), ("TS", "tiến sĩ"), ("TNHH", "trách nhiệm hữu hạn"), ("VĐV", "vận động viên"),
    ("TPHCM", "thành phố hồ chí minh"), ("PGS", "phó giáo sư"), ("SP500", "ét pê năm trăm"),
    ("PGS.TS", "phó giáo sư tiến sĩ"), ("GS.TS", "giáo sư tiến sĩ"), ("ThS", "thạc sĩ"), ("BS", "bác sĩ"),
    ("UAE", "u a e"), ("CUDA", "cu đa")
];

pub const TECHNICAL_TERMS: &[(&str, &str)] = &[
    ("JSON", "__start_en__j son__end_en__"),
    ("VRAM", "__start_en__v ram__end_en__"),
    ("NVIDIA", "__start_en__n v d a__end_en__"),
    ("VN-Index", "__start_en__v n__end_en__ index"),
    ("MS DOS", "__start_en__m s dos__end_en__"),
    ("MS-DOS", "__start_en__m s dos__end_en__"),
    ("B2B", "__start_en__b two b__end_en__"),
    ("MI5", "__start_en__m i five__end_en__"),
    ("MI6", "__start_en__m i six__end_en__"),
    ("2FA", "__start_en__two f a__end_en__"),
    ("TX-0", "__start_en__t x zero__end_en__"),
    ("IPv6", "__start_en__i p v__end_en__ sáu"),
    ("IPv4", "__start_en__i p v__end_en__ bốn"),
];

pub static COMBINED_EXCEPTIONS: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();
    for (k, v) in ACRONYMS_EXCEPTIONS_VI {
        m.insert(*k, *v);
    }
    for (k, v) in TECHNICAL_TERMS {
        m.insert(*k, *v);
    }
    m
});

pub const DOMAIN_SUFFIX_MAP: &[(&str, &str)] = &[
    ("com", "com"),
    ("vn", "__start_en__v n__end_en__"),
    ("net", "nét"),
    ("org", "o rờ gờ"),
    ("edu", "__start_en__edu__end_en__"),
    ("gov", "gờ o vê"),
    ("io", "__start_en__i o__end_en__"),
    ("biz", "biz"),
    ("info", "info"),
];

pub static DOMAIN_SUFFIX_MAP_MAP: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    DOMAIN_SUFFIX_MAP.iter().cloned().collect()
});

pub const CURRENCY_SYMBOL_MAP: &[(&str, &str)] = &[
    ("$", "__start_en__u s d__end_en__"),
    ("€", "__start_en__euro__end_en__"),
    ("¥", "yên"),
    ("£", "__start_en__pound__end_en__"),
    ("₩", "won"),
];

pub static CURRENCY_SYMBOL_MAP_MAP: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    CURRENCY_SYMBOL_MAP.iter().cloned().collect()
});

pub const CURRENCY_SYMBOLS_RE: &str = "[$€¥£₩]";

pub const ROMAN_NUMERALS: &[(&str, u32)] = &[
    ("I", 1), ("V", 5), ("X", 10), ("L", 50), ("C", 100), ("D", 500), ("M", 1000)
];

pub static ROMAN_NUMERALS_MAP: Lazy<HashMap<char, u32>> = Lazy::new(|| {
    let mut m = HashMap::new();
    for (k, v) in ROMAN_NUMERALS {
        m.insert(k.chars().next().unwrap(), *v);
    }
    m
});

pub const ABBRS: &[(&str, &str)] = &[
    ("v.v", " vân vân"), ("v/v", " về việc"), ("đ/c", "địa chỉ")
];

pub const SYMBOLS_MAP: &[(&str, &str)] = &[
    ("&", " và "), ("+", " cộng "), ("=", " bằng "), ("#", " thăng "),
    (">", " lớn hơn "), ("<", " nhỏ hơn "),
    ("≥", " lớn hơn hoặc bằng "), ("≤", " nhỏ hơn hoặc bằng "),
    ("±", " cộng trừ "), ("≈", " xấp xỉ "),
    ("/", " trên "), ("→", " đến "), ("÷", " chia "),
    ("*", " sao "), ("×", " nhân "), ("^", " mũ "), ("~", " khoảng ")
];

pub static SYMBOLS_MAP_MAP: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    SYMBOLS_MAP.iter().cloned().collect()
});

pub const WORD_LIKE_ACRONYMS: &[&str] = &[
    "UNESCO", "NASA", "NATO", "ASEAN", "OPEC", "SARS", "FIFA", "UNIC", "RAM", "VRAM", "COVID", "IELTS", "STEM",
    "SWAT", "SEAL", "WASP", "COBOL", "BASIC", "OLED", "COVAX", "BRICS", "APEC", "VUCA", "PERMA", "DINK",
    "MENA", "EPIC", "OASIS", "BASE", "DART", "IDEA", "CHAOS", "SMART", "FANG", "BLEU", "REST", "ERROR",
    "SELECT", "FROM", "WHERE"
];

pub static WORD_LIKE_ACRONYMS_SET: Lazy<std::collections::HashSet<&'static str>> = Lazy::new(|| {
    WORD_LIKE_ACRONYMS.iter().cloned().collect()
});
