use std::collections::{HashMap, HashSet};
use once_cell::sync::Lazy;

pub static VI_LETTER_NAMES: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert("a", "a"); m.insert("b", "bê"); m.insert("c", "xê");
    m.insert("d", "đê"); m.insert("đ", "đê"); m.insert("e", "e");
    m.insert("ê", "ê"); m.insert("f", "ép"); m.insert("g", "gờ");
    m.insert("h", "hát"); m.insert("i", "i"); m.insert("j", "giây");
    m.insert("k", "ca"); m.insert("l", "lờ"); m.insert("m", "mờ");
    m.insert("n", "nờ"); m.insert("o", "ô"); m.insert("ô", "ô");
    m.insert("ơ", "ơ"); m.insert("p", "pê"); m.insert("q", "qui");
    m.insert("r", "rờ"); m.insert("s", "ét"); m.insert("t", "tê");
    m.insert("u", "u"); m.insert("ư", "ư"); m.insert("v", "vê");
    m.insert("w", "đắp liu"); m.insert("x", "ích"); m.insert("y", "y");
    m.insert("z", "dét");
    m
});

pub static MEASUREMENT_KEY_VI: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert("km", "ki lô mét"); m.insert("dm", "đê xi mét");
    m.insert("cm", "xen ti mét"); m.insert("mm", "mi li mét");
    m.insert("nm", "na nô mét"); m.insert("µm", "mic rô mét");
    m.insert("μm", "mic rô mét"); m.insert("m", "mét");
    m.insert("kg", "ki lô gam"); m.insert("g", "gam"); m.insert("µg", "mic rô gam");
    m.insert("mg", "mi li gam"); m.insert("km2", "ki lô mét vuông");
    m.insert("m2", "mét vuông"); m.insert("cm2", "xen ti mét vuông");
    m.insert("mm2", "mi li mét vuông"); m.insert("ha", "héc ta");
    m.insert("km3", "ki lô mét khối"); m.insert("m3", "mét khối");
    m.insert("cm3", "xen ti mét khối"); m.insert("mm3", "mi li mét khối");
    m.insert("l", "lít"); m.insert("dl", "đê xi lít");
    m.insert("ml", "mi li lít"); m.insert("hl", "héc tô lít");
    m.insert("kw", "ki lô oát"); m.insert("mw", "mê ga oát");
    m.insert("gw", "gi ga oát"); m.insert("kwh", "ki lô oát giờ"); m.insert("kWh", "ki lô oát giờ");
    m.insert("mwh", "mê ga oát giờ"); m.insert("wh", "oát giờ");
    m.insert("hz", "héc"); m.insert("khz", "ki lô héc");
    m.insert("mhz", "mê ga héc"); m.insert("ghz", "gi ga héc");
    m.insert("pa", "__start_en__pascal__end_en__"); m.insert("kpa", "__start_en__kilopascal__end_en__");
    m.insert("mpa", "__start_en__megapascal__end_en__"); m.insert("bar", "__start_en__bar__end_en__");
    m.insert("mbar", "__start_en__millibar__end_en__"); m.insert("atm", "__start_en__atmosphere__end_en__");
    m.insert("psi", "__start_en__p s i__end_en__"); m.insert("j", "__start_en__joule__end_en__");
    m.insert("kj", "__start_en__kilojoule__end_en__"); m.insert("cal", "__start_en__calorie__end_en__");
    m.insert("kcal", "__start_en__kilocalorie__end_en__"); m.insert("h", "giờ");
    m.insert("p", "phút"); m.insert("s", "giây"); m.insert("sqm", "mét vuông");
    m.insert("cum", "mét khối"); m.insert("gb", "__start_en__gigabyte__end_en__");
    m.insert("mb", "__start_en__megabyte__end_en__"); m.insert("kb", "__start_en__kilobyte__end_en__");
    m.insert("tb", "__start_en__terabyte__end_en__"); m.insert("db", "__start_en__decibel__end_en__");
    m.insert("oz", "__start_en__ounce__end_en__"); m.insert("lb", "__start_en__pound__end_en__");
    m.insert("lbs", "__start_en__pounds__end_en__"); m.insert("ft", "__start_en__feet__end_en__");
    m.insert("in", "__start_en__inch__end_en__"); m.insert("dpi", "__start_en__d p i__end_en__");
    m.insert("ph", "pê hát"); m.insert("gbps", "__start_en__gigabits per second__end_en__");
    m.insert("mbps", "__start_en__megabits per second__end_en__");
    m.insert("kbps", "__start_en__kilobits per second__end_en__");
    m.insert("gallon", "__start_en__gallon__end_en__"); m.insert("mol", "mol");
    m.insert("ms", "mi li giây"); m.insert("M", "triệu");
    m.insert("B", "tỷ");
    m
});

pub static CURRENCY_KEY: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert("usd", "__start_en__u s d__end_en__"); m.insert("vnd", "việt nam đồng");
    m.insert("vnđ", "việt nam đồng"); m.insert("đ", "đồng");
    m.insert("v n d", "việt nam đồng"); m.insert("v n đ", "việt nam đồng");
    m.insert("€", "__start_en__euro__end_en__"); m.insert("euro", "__start_en__euro__end_en__");
    m.insert("eur", "__start_en__euro__end_en__"); m.insert("¥", "yên");
    m.insert("yên", "yên"); m.insert("jpy", "yên"); m.insert("%", "phần trăm");
    m
});

pub static ACRONYMS_EXCEPTIONS_VI: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert("CĐV", "cổ động viên"); m.insert("HĐND", "hội đồng nhân dân");
    m.insert("HĐQT", "hội đồng quản trị"); m.insert("TAND", "toàn án nhân dân");
    m.insert("BHXH", "bảo hiểm xã hội"); m.insert("BHTN", "bảo hiểm thất nghiệp");
    m.insert("TP.HCM", "thành phố hồ chí minh"); m.insert("VN", "việt nam");
    m.insert("UBND", "uỷ ban nhân dân"); m.insert("TP", "thành phố");
    m.insert("HCM", "hồ chí minh"); m.insert("HN", "hà nội");
    m.insert("BTC", "ban tổ chức"); m.insert("CLB", "câu lạc bộ");
    m.insert("HTX", "hợp tác xã"); m.insert("NXB", "nhà xuất bản");
    m.insert("TW", "trung ương"); m.insert("CSGT", "cảnh sát giao thông");
    m.insert("LHQ", "liên hợp quốc"); m.insert("THCS", "trung học cơ sở");
    m.insert("THPT", "trung học phổ thông"); m.insert("ĐH", "đại học");
    m.insert("HLV", "huấn luyện viên"); m.insert("GS", "giáo sư");
    m.insert("TS", "tiến sĩ"); m.insert("TNHH", "trách nhiệm hữu hạn");
    m.insert("VĐV", "vận động viên"); m.insert("TPHCM", "thành phố hồ chí minh");
    m.insert("PGS", "phó giáo sư"); m.insert("SP500", "ét pê năm trăm");
    m.insert("PGS.TS", "phó giáo sư tiến sĩ"); m.insert("GS.TS", "giáo sư tiến sĩ");
    m.insert("ThS", "thạc sĩ"); m.insert("BS", "bác sĩ");
    m.insert("UAE", "u a e"); m.insert("CUDA", "cu đa");
    m
});

pub static TECHNICAL_TERMS: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert("JSON", "__start_en__j son__end_en__");
    m.insert("VRAM", "__start_en__v ram__end_en__");
    m.insert("NVIDIA", "__start_en__n v d a__end_en__");
    m.insert("VN-Index", "__start_en__v n__end_en__ index");
    m.insert("MS DOS", "__start_en__m s dos__end_en__");
    m.insert("MS-DOS", "__start_en__m s dos__end_en__");
    m.insert("B2B", "__start_en__b two b__end_en__");
    m.insert("MI5", "__start_en__m i five__end_en__");
    m.insert("MI6", "__start_en__m i six__end_en__");
    m.insert("2FA", "__start_en__two f a__end_en__");
    m.insert("TX-0", "__start_en__t x zero__end_en__");
    m.insert("IPv6", "__start_en__i p v__end_en__ sáu");
    m.insert("IPv4", "__start_en__i p v__end_en__ bốn");
    m.insert("Washington D.C", "__start_en__washington d c__end_en__");
    m.insert("Washington DC", "__start_en__washington d c__end_en__");
    m.insert("HCN", "hát xê nờ");
    m.insert("HF", "hát ép");
    m.insert("KI", "ca i");
    m.insert("KOH", "ca ô hát");
    m
});

pub static DOMAIN_SUFFIX_MAP: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert("com", "com"); m.insert("vn", "__start_en__v n__end_en__");
    m.insert("net", "nét"); m.insert("org", "o rờ gờ");
    m.insert("edu", "__start_en__edu__end_en__"); m.insert("gov", "gờ o vê");
    m.insert("io", "__start_en__i o__end_en__"); m.insert("biz", "biz");
    m.insert("info", "info");
    m
});

pub static CURRENCY_SYMBOL_MAP: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert("$", "__start_en__u s d__end_en__");
    m.insert("€", "__start_en__euro__end_en__");
    m.insert("¥", "yên");
    m.insert("£", "__start_en__pound__end_en__");
    m.insert("₩", "won");
    m
});

pub static ROMAN_NUMERALS: Lazy<HashMap<char, i32>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert('I', 1); m.insert('V', 5); m.insert('X', 10);
    m.insert('L', 50); m.insert('C', 100); m.insert('D', 500);
    m.insert('M', 1000);
    m
});

pub static ABBRS: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert("v.v", " vân vân"); m.insert("v/v", " về việc");
    m.insert("đ/c", "địa chỉ");
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
    m.insert('%', " phần trăm "); m.insert('$', " đô la "); m.insert('€', " ê rô ");
    m.insert('£', " bảng "); m.insert('¥', " yên "); m.insert('₩', " won ");
    m.insert('₭', " kíp "); m.insert('₱', " bê xô "); m.insert('฿', " bạc ");
    m.insert('Ω', " ôm "); m.insert('@', " a còng "); m.insert('≠', " khác ");
    m.insert('∀', " với mọi "); m.insert('∏', " tích "); m.insert('∈', " thuộc ");
    m.insert('∑', " tổng "); m.insert('∩', " giao "); m.insert('∪', " hội ");
    m.insert('¬', " phủ định "); m.insert('∞', " vô cùng "); m.insert('α', " an pha ");
    m.insert('β', " bê ta "); m.insert('γ', " ga ma "); m.insert('δ', " đen ta ");
    m.insert('ε', " ép si lon "); m.insert('ϵ', " thuộc "); m.insert('ζ', " de ta ");
    m.insert('η', " ê ta "); m.insert('θ', " thê ta "); m.insert('ι', " i ô ta ");
    m.insert('κ', " cáp ba "); m.insert('λ', " lam đa "); m.insert('ᴧ', " và ");
    m.insert('μ', " muy "); m.insert('Δ', " đen ta "); m.insert('ν', " nu ");
    m.insert('ξ', " xi xi "); m.insert('ο', " o mi ron "); m.insert('π', " pi ");
    m.insert('ρ', " ro "); m.insert('σ', " xích ma "); m.insert('τ', " tao ");
    m.insert('υ', " úp si lon "); m.insert('φ', " phi "); m.insert('χ', " chi ");
    m.insert('ψ', " si "); m.insert('ω', " ô me ga "); m.insert('©', " bản quyền ");
    m.insert('½', " một phần hai "); m.insert('¼', " một phần tư "); m.insert('¾', " ba phần tư ");
    m.insert('⅓', " một phần ba "); m.insert('⅔', " hai phần ba ");
    m.insert('⅕', " một phần năm "); m.insert('⅖', " hai phần năm "); m.insert('⅗', " ba phần năm "); m.insert('⅘', " bốn phần năm ");
    m.insert('⅚', " năm phần sáu "); m.insert('⅚', " năm phần sáu ");
    m
});

pub static SUPERSCRIPTS_MAP: Lazy<HashMap<char, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert('⁰', " không "); m.insert('¹', " một "); m.insert('²', " bình phương ");
    m.insert('³', " lập phương "); m.insert('⁴', " bốn "); m.insert('⁵', " năm ");
    m.insert('⁶', " sáu "); m.insert('⁷', " bảy "); m.insert('⁸', " tám ");
    m.insert('⁹', " chín ");
    m
});

pub static SUBSCRIPTS_MAP: Lazy<HashMap<char, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert('₀', " không "); m.insert('₁', " một "); m.insert('₂', " hai ");
    m.insert('₃', " ba "); m.insert('₄', " bốn "); m.insert('₅', " năm ");
    m.insert('₆', " sáu "); m.insert('₇', " bảy "); m.insert('₈', " tám ");
    m.insert('₉', " chín ");
    m
});

pub static WORD_LIKE_ACRONYMS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    let mut s = HashSet::new();
    let words = [
        "UNESCO", "NASA", "NATO", "ASEAN", "OPEC", "SARS", "FIFA", "UNIC", "RAM", "VRAM", "COVID", "IELTS", "STEM",
        "SWAT", "SEAL", "WASP", "COBOL", "BASIC", "OLED", "COVAX", "BRICS", "APEC", "VUCA", "PERMA", "DINK",
        "MENA", "EPIC", "OASIS", "BASE", "DART", "IDEA", "CHAOS", "SMART", "FANG", "BLEU", "REST", "ERROR",
        "SELECT", "FROM", "WHERE", "ORDER", "BY", "LIMIT", "OFFSET", "GROUP", "HAVING", "JOIN", "LEFT", "RIGHT", 
        "INNER", "OUTER", "ON", "AS", "AND", "OR", "NOT", "IN", "BETWEEN", "LIKE", "IS", "NULL", "TRUE", "FALSE", 
        "CASE", "WHEN", "THEN", "ELSE", "END", "UNION", "INTERSECT", "EXCEPT", "DESC"
    ];
    for w in words { s.insert(w); }
    s
});

pub static COMMON_EMAIL_DOMAINS: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert("gmail.com", "__start_en__gmail__end_en__ chấm com");
    m.insert("yahoo.com", "__start_en__yahoo__end_en__ chấm com");
    m.insert("yahoo.com.vn", "__start_en__yahoo__end_en__ chấm com chấm __start_en__v n__end_en__");
    m.insert("outlook.com", "__start_en__outlook__end_en__ chấm com");
    m.insert("hotmail.com", "__start_en__hotmail__end_en__ chấm com");
    m.insert("icloud.com", "__start_en__icloud__end_en__ chấm com");
    m.insert("fpt.vn", "__start_en__f p t__end_en__ chấm __start_en__v n__end_en__");
    m.insert("fpt.com.vn", "__start_en__f p t__end_en__ chấm com chấm __start_en__v n__end_en__");
    m
});

pub static COMBINED_EXCEPTIONS: Lazy<HashMap<String, String>> = Lazy::new(|| {
    let mut m = HashMap::new();
    for (k, v) in ACRONYMS_EXCEPTIONS_VI.iter() {
        m.insert(k.to_string(), v.to_string());
    }
    for (k, v) in TECHNICAL_TERMS.iter() {
        m.insert(k.to_string(), v.to_string());
    }
    m
});

pub static DATE_KEYWORDS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    let mut s = HashSet::new();
    let words = [
        "vào", "ngày", "hôm", "hôm nay", "hôm qua", "hôm kia", "mai", "ngày mai", "ngày kia",
        "sinh", "sinh nhật", "kỷ niệm", "lễ", "tết", "diễn ra", "tổ chức", "thứ", "tuần", "tháng", "năm"
    ];
    for w in words { s.insert(w); }
    s
});

pub static MATH_KEYWORDS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    let mut s = HashSet::new();
    let words = [
        "cộng", "trừ", "nhân", "chia", "bằng", "sin", "cos", "tan", "log", "sqrt", "xác suất", "tỷ lệ", "tỉ lệ"
    ];
    for w in words { s.insert(w); }
    s
});
