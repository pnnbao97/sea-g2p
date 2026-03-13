use once_cell::sync::Lazy;
use std::collections::{HashMap, HashSet};

pub static VI_LETTER_NAMES: Lazy<HashMap<String, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert("a".to_string(), "a"); m.insert("b".to_string(), "bê"); m.insert("c".to_string(), "xê");
    m.insert("d".to_string(), "đê"); m.insert("đ".to_string(), "đê"); m.insert("e".to_string(), "e");
    m.insert("ê".to_string(), "ê"); m.insert("f".to_string(), "ép"); m.insert("g".to_string(), "gờ");
    m.insert("h".to_string(), "hát"); m.insert("i".to_string(), "i"); m.insert("j".to_string(), "giây");
    m.insert("k".to_string(), "ca"); m.insert("l".to_string(), "lờ"); m.insert("m".to_string(), "mờ");
    m.insert("n".to_string(), "nờ"); m.insert("o".to_string(), "o"); m.insert("ô".to_string(), "ô");
    m.insert("ơ".to_string(), "ơ"); m.insert("p".to_string(), "pê"); m.insert("q".to_string(), "qui");
    m.insert("r".to_string(), "rờ"); m.insert("s".to_string(), "ét"); m.insert("t".to_string(), "tê");
    m.insert("u".to_string(), "u"); m.insert("ư".to_string(), "ư"); m.insert("v".to_string(), "vê");
    m.insert("w".to_string(), "đắp liu"); m.insert("x".to_string(), "ích"); m.insert("y".to_string(), "y");
    m.insert("z".to_string(), "dét");
    m
});

pub static COMMON_EMAIL_DOMAINS: Lazy<HashMap<String, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert("gmail.com".to_string(), "__start_en__gmail__end_en__ chấm com");
    m.insert("yahoo.com".to_string(), "__start_en__yahoo__end_en__ chấm com");
    m.insert("yahoo.com.vn".to_string(), "__start_en__yahoo__end_en__ chấm com chấm __start_en__v n__end_en__");
    m.insert("outlook.com".to_string(), "__start_en__outlook__end_en__ chấm com");
    m.insert("hotmail.com".to_string(), "__start_en__hotmail__end_en__ chấm com");
    m.insert("icloud.com".to_string(), "__start_en__icloud__end_en__ chấm com");
    m.insert("fpt.vn".to_string(), "__start_en__f p t__end_en__ chấm __start_en__v n__end_en__");
    m.insert("fpt.com.vn".to_string(), "__start_en__f p t__end_en__ chấm com chấm __start_en__v n__end_en__");
    m
});

pub static MEASUREMENT_KEY_VI: Lazy<HashMap<String, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert("km".to_string(), "ki lô mét"); m.insert("dm".to_string(), "đê xi mét");
    m.insert("cm".to_string(), "xen ti mét"); m.insert("mm".to_string(), "mi li mét");
    m.insert("nm".to_string(), "na nô mét"); m.insert("µm".to_string(), "mic rô mét");
    m.insert("μm".to_string(), "mic rô mét"); m.insert("m".to_string(), "mét");
    m.insert("kg".to_string(), "ki lô gam"); m.insert("g".to_string(), "gam");
    m.insert("mg".to_string(), "mi li gam"); m.insert("km2".to_string(), "ki lô mét vuông");
    m.insert("m2".to_string(), "mét vuông"); m.insert("cm2".to_string(), "xen ti mét vuông");
    m.insert("mm2".to_string(), "mi li mét vuông"); m.insert("ha".to_string(), "héc ta");
    m.insert("km3".to_string(), "ki lô mét khối"); m.insert("m3".to_string(), "mét khối");
    m.insert("cm3".to_string(), "xen ti mét khối"); m.insert("mm3".to_string(), "mi li mét khối");
    m.insert("l".to_string(), "lít"); m.insert("dl".to_string(), "đê xi lít");
    m.insert("ml".to_string(), "mi li lít"); m.insert("hl".to_string(), "héc tô lít");
    m.insert("kw".to_string(), "ki lô oát"); m.insert("mw".to_string(), "mê ga oát");
    m.insert("gw".to_string(), "gi ga oát"); m.insert("kwh".to_string(), "ki lô oát giờ");
    m.insert("mwh".to_string(), "mê ga oát giờ"); m.insert("wh".to_string(), "oát giờ");
    m.insert("hz".to_string(), "héc"); m.insert("khz".to_string(), "ki lô héc");
    m.insert("mhz".to_string(), "mê ga héc"); m.insert("ghz".to_string(), "gi ga héc");
    m.insert("pa".to_string(), "__start_en__pascal__end_en__");
    m.insert("kpa".to_string(), "__start_en__kilopascal__end_en__");
    m.insert("mpa".to_string(), "__start_en__megapascal__end_en__");
    m.insert("bar".to_string(), "__start_en__bar__end_en__");
    m.insert("mbar".to_string(), "__start_en__millibar__end_en__");
    m.insert("atm".to_string(), "__start_en__atmosphere__end_en__");
    m.insert("psi".to_string(), "__start_en__p s i__end_en__");
    m.insert("j".to_string(), "__start_en__joule__end_en__");
    m.insert("kj".to_string(), "__start_en__kilojoule__end_en__");
    m.insert("cal".to_string(), "__start_en__calorie__end_en__");
    m.insert("kcal".to_string(), "__start_en__kilocalorie__end_en__");
    m.insert("h".to_string(), "giờ"); m.insert("p".to_string(), "phút");
    m.insert("s".to_string(), "giây"); m.insert("sqm".to_string(), "mét vuông");
    m.insert("cum".to_string(), "mét khối"); m.insert("gb".to_string(), "__start_en__gigabyte__end_en__");
    m.insert("mb".to_string(), "__start_en__megabyte__end_en__");
    m.insert("kb".to_string(), "__start_en__kilobyte__end_en__");
    m.insert("tb".to_string(), "__start_en__terabyte__end_en__");
    m.insert("db".to_string(), "__start_en__decibel__end_en__");
    m.insert("oz".to_string(), "__start_en__ounce__end_en__");
    m.insert("lb".to_string(), "__start_en__pound__end_en__");
    m.insert("lbs".to_string(), "__start_en__pounds__end_en__");
    m.insert("ft".to_string(), "__start_en__feet__end_en__");
    m.insert("in".to_string(), "__start_en__inch__end_en__");
    m.insert("dpi".to_string(), "__start_en__d p i__end_en__");
    m.insert("ph".to_string(), "pê hát");
    m.insert("pH".to_string(), "pê hát"); // Added mixed case key for pH
    m.insert("gbps".to_string(), "__start_en__gigabits per second__end_en__");
    m.insert("mbps".to_string(), "__start_en__megabits per second__end_en__");
    m.insert("kbps".to_string(), "__start_en__kilobits per second__end_en__");
    m.insert("gallon".to_string(), "__start_en__gallon__end_en__");
    m.insert("mol".to_string(), "mol"); m.insert("ms".to_string(), "mi li giây");
    m.insert("M".to_string(), "triệu");
    m
});

pub static CURRENCY_KEY: Lazy<HashMap<String, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert("usd".to_string(), "__start_en__u s d__end_en__"); m.insert("vnd".to_string(), "đồng");
    m.insert("đ".to_string(), "đồng"); m.insert("v n d".to_string(), "đồng");
    m.insert("v n đ".to_string(), "đồng"); m.insert("€".to_string(), "__start_en__euro__end_en__");
    m.insert("euro".to_string(), "__start_en__euro__end_en__"); m.insert("eur".to_string(), "__start_en__euro__end_en__");
    m.insert("¥".to_string(), "yên"); m.insert("yên".to_string(), "yên");
    m.insert("jpy".to_string(), "yên"); m.insert("%".to_string(), "phần trăm");
    m
});

pub static ACRONYMS_EXCEPTIONS_VI: Lazy<HashMap<String, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert("CĐV".to_string(), "cổ động viên"); m.insert("HĐND".to_string(), "hội đồng nhân dân");
    m.insert("HĐQT".to_string(), "hội đồng quản trị"); m.insert("TAND".to_string(), "tòa án nhân dân");
    m.insert("BHXH".to_string(), "bảo hiểm xã hội"); m.insert("BHTN".to_string(), "bảo hiểm thất nghiệp");
    m.insert("TP.HCM".to_string(), "thành phố hồ chí minh"); m.insert("VN".to_string(), "việt nam");
    m.insert("UBND".to_string(), "uỷ ban nhân dân"); m.insert("TP".to_string(), "thành phố");
    m.insert("HCM".to_string(), "hồ chí minh"); m.insert("HN".to_string(), "hà nội");
    m.insert("BTC".to_string(), "ban tổ chức"); m.insert("CLB".to_string(), "câu lạc bộ");
    m.insert("HTX".to_string(), "hợp tác xã"); m.insert("NXB".to_string(), "nhà xuất bản");
    m.insert("TW".to_string(), "trung ương"); m.insert("CSGT".to_string(), "cảnh sát giao thông");
    m.insert("LHQ".to_string(), "liên hợp quốc"); m.insert("THCS".to_string(), "trung học cơ sở");
    m.insert("THPT".to_string(), "trung học phổ thông"); m.insert("ĐH".to_string(), "đại học");
    m.insert("HLV".to_string(), "huấn luyện viên"); m.insert("GS".to_string(), "giáo sư");
    m.insert("TS".to_string(), "tiến sĩ"); m.insert("TNHH".to_string(), "trách nhiệm hữu hạn");
    m.insert("VĐV".to_string(), "vận động viên"); m.insert("TPHCM".to_string(), "thành phố hồ chí minh");
    m.insert("PGS".to_string(), "phó giáo sư"); m.insert("SP500".to_string(), "ét pê năm trăm");
    m.insert("PGS.TS".to_string(), "phó giáo sư tiến sĩ"); m.insert("GS.TS".to_string(), "giáo sư tiến sĩ");
    m.insert("ThS".to_string(), "thạc sĩ"); m.insert("BS".to_string(), "bác sĩ");
    m.insert("UAE".to_string(), "u a e"); m.insert("CUDA".to_string(), "cu đa");
    m
});

pub static TECHNICAL_TERMS: Lazy<HashMap<String, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert("JSON".to_string(), "__start_en__j son__end_en__"); m.insert("VRAM".to_string(), "__start_en__v ram__end_en__");
    m.insert("NVIDIA".to_string(), "__start_en__n v d a__end_en__"); m.insert("VN-Index".to_string(), "__start_en__v n__end_en__ index");
    m.insert("MS DOS".to_string(), "__start_en__m s dos__end_en__"); m.insert("MS-DOS".to_string(), "__start_en__m s dos__end_en__");
    m.insert("B2B".to_string(), "__start_en__b two b__end_en__"); m.insert("MI5".to_string(), "__start_en__m i five__end_en__");
    m.insert("MI6".to_string(), "__start_en__m i six__end_en__"); m.insert("2FA".to_string(), "__start_en__two f a__end_en__");
    m.insert("TX-0".to_string(), "__start_en__t x zero__end_en__"); m.insert("IPv6".to_string(), "__start_en__i p v__end_en__ sáu");
    m.insert("IPv4".to_string(), "__start_en__i p v__end_en__ bốn");
    m
});

pub static DOMAIN_SUFFIX_MAP: Lazy<HashMap<String, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert("com".to_string(), "com"); m.insert("vn".to_string(), "__start_en__v n__end_en__");
    m.insert("net".to_string(), "nét"); m.insert("org".to_string(), "o rờ gờ");
    m.insert("edu".to_string(), "__start_en__edu__end_en__"); m.insert("gov".to_string(), "gờ o vê");
    m.insert("io".to_string(), "__start_en__i o__end_en__"); m.insert("biz".to_string(), "biz");
    m.insert("info".to_string(), "info");
    m
});

pub static CURRENCY_SYMBOL_MAP: Lazy<HashMap<String, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert("$".to_string(), "__start_en__u s d__end_en__"); m.insert("€".to_string(), "__start_en__euro__end_en__");
    m.insert("¥".to_string(), "yên"); m.insert("£".to_string(), "__start_en__pound__end_en__");
    m.insert("₩".to_string(), "won");
    m
});

pub static ROMAN_NUMERALS: Lazy<HashMap<char, i32>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert('I', 1); m.insert('V', 5); m.insert('X', 10); m.insert('L', 50);
    m.insert('C', 100); m.insert('D', 500); m.insert('M', 1000);
    m
});

pub static ABBRS: Lazy<HashMap<String, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert("v.v".to_string(), " vân vân"); m.insert("v/v".to_string(), " về việc");
    m.insert("đ/c".to_string(), "địa chỉ");
    m
});

pub static SYMBOLS_MAP: Lazy<HashMap<char, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert('&', " và "); m.insert('+', " cộng "); m.insert('=', " bằng ");
    m.insert('#', " thăng "); m.insert('>', " lớn hơn "); m.insert('<', " nhỏ hơn ");
    m.insert('≥', " lớn hơn hoặc bằng "); m.insert('≤', " nhỏ hơn hoặc bằng ");
    m.insert('±', " cộng trừ "); m.insert('≈', " xấp xỉ "); m.insert('/', " trên ");
    m.insert('→', " đến "); m.insert('÷', " chia "); m.insert('*', " sao ");
    m.insert('×', " nhân "); m.insert('^', " mũ "); m.insert('~', " khoảng ");
    m
});

pub static WORD_LIKE_ACRONYMS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    let mut s = HashSet::new();
    s.insert("UNESCO"); s.insert("NASA"); s.insert("NATO"); s.insert("ASEAN");
    s.insert("OPEC"); s.insert("SARS"); s.insert("FIFA"); s.insert("UNIC");
    s.insert("RAM"); s.insert("VRAM"); s.insert("COVID"); s.insert("IELTS");
    s.insert("STEM"); s.insert("SWAT"); s.insert("SEAL"); s.insert("WASP");
    s.insert("COBOL"); s.insert("BASIC"); s.insert("OLED"); s.insert("COVAX");
    s.insert("BRICS"); s.insert("APEC"); s.insert("VUCA"); s.insert("PERMA");
    s.insert("DINK"); s.insert("MENA"); s.insert("EPIC"); s.insert("OASIS");
    s.insert("BASE"); s.insert("DART"); s.insert("IDEA"); s.insert("CHAOS");
    s.insert("SMART"); s.insert("FANG"); s.insert("BLEU"); s.insert("REST");
    s.insert("ERROR"); s.insert("SELECT"); s.insert("FROM"); s.insert("WHERE");
    s
});

pub static COMBINED_EXCEPTIONS: Lazy<HashMap<String, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();
    for (k, v) in ACRONYMS_EXCEPTIONS_VI.iter() { m.insert(k.clone(), *v); }
    for (k, v) in TECHNICAL_TERMS.iter() { m.insert(k.clone(), *v); }
    m
});
