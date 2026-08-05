//! Static lookup tables: letter names, units, currencies, symbols,
//! abbreviations and the keyword sets that disambiguate context.
//!
//! Every table here is data, not logic. Adding an entry is the cheapest way to
//! fix a reading — and forgetting one is how characters end up deleted in
//! silence, so a new symbol belongs in a table *and* in
//! [`crate::lang::vi::audit`].
//!
//! # Table groups
//!
//! - **Letters and units** — `VI_LETTER_NAMES`, `MEASUREMENT_KEY_VI`,
//!   `CURRENCY_KEY`, `CURRENCY_SYMBOL_MAP`, `DOMAIN_SUFFIX_MAP`.
//! - **Symbols** — `SYMBOLS_MAP`, `SUPERSCRIPTS_MAP`, `SUBSCRIPTS_MAP`. These
//!   are what the audit module checks against.
//! - **Abbreviations** — `ACRONYMS_EXCEPTIONS_VI` for Vietnamese expansions,
//!   `TECHNICAL_TERMS` for names with a fixed pronunciation, and
//!   `WORD_LIKE_ACRONYMS` for initialisms read as words (UEFA, SIM, LED)
//!   rather than spelled out.
//! - **Context keywords** — `DATE_KEYWORDS` and `DATE_LEAD_WORDS` separate
//!   dates from fractions, `ROMAN_KEYWORDS` licenses Roman numerals, and
//!   `MATH_KEYWORDS` forces mathematical readings. The date sets differ by
//!   reach: lead words must sit immediately before the number, keywords may be
//!   up to three words away.

use std::collections::{HashMap, HashSet};
use once_cell::sync::Lazy;
use crate::core::abbrev::{AbbrevTable, Reading};

pub static VI_LETTER_NAMES: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert("a", "a"); m.insert("b", "bê"); m.insert("c", "xê");
    m.insert("d", "đê"); m.insert("đ", "đê"); m.insert("e", "e");
    m.insert("ê", "ê"); m.insert("f", "ép"); m.insert("g", "gờ");
    m.insert("h", "hát"); m.insert("i", "i"); m.insert("j", "giây");
    m.insert("k", "ca"); m.insert("l", "lờ"); m.insert("m", "mờ");
    m.insert("n", "nờ"); m.insert("o", "ô"); m.insert("ô", "ô");
    m.insert("ơ", "ơ"); m.insert("p", "phê"); m.insert("q", "qui");
    m.insert("r", "rờ"); m.insert("s", "ét"); m.insert("t", "tê");
    m.insert("u", "u"); m.insert("ư", "ư"); m.insert("v", "vê");
    m.insert("w", "vê kép"); m.insert("x", "ích"); m.insert("y", "y");
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
    // Bare "w" only matches with a number in front ("550 W", "320w"), so a
    // standalone letter w still reads as "vê kép".
    m.insert("w", "oát");
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
    m.insert("ph", "phê hát"); m.insert("gbps", "__start_en__gigabits per second__end_en__");
    m.insert("mbps", "__start_en__megabits per second__end_en__");
    m.insert("kbps", "__start_en__kilobits per second__end_en__");
    m.insert("gallon", "__start_en__gallon__end_en__"); m.insert("mol", "mol");
    m.insert("mmol", "mi li mol");
    m.insert("ms", "mi li giây"); m.insert("M", "triệu");
    m.insert("B", "tỷ"); m.insert("K", "nghìn");
    // Compound electrical units written camelCase (mAh/Ah); kWh/Wh/mWh above.
    m.insert("mah", "mi li am pe giờ"); m.insert("ah", "am pe giờ");
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
    m.insert("ThS", "thạc sĩ"); m.insert("Th.S", "thạc sĩ"); m.insert("BS", "bác sĩ");
    m.insert("KS", "kỹ sư");
    m.insert("UAE", "u a e"); m.insert("CUDA", "cu đa");
    // Common agency, organisation and sector abbreviations. Matched on word
    // boundaries, so they cannot fire inside a longer word.
    m.insert("QĐND", "quân đội nhân dân"); m.insert("CAND", "công an nhân dân");
    m.insert("BCH", "ban chấp hành"); m.insert("TBT", "tổng bí thư");
    m.insert("ĐHQG", "đại học quốc gia"); m.insert("KCN", "khu công nghiệp");
    m.insert("GTVT", "giao thông vận tải"); m.insert("TDTT", "thể dục thể thao");
    m.insert("BĐBP", "bộ đội biên phòng"); m.insert("KHCN", "khoa học công nghệ");
    m.insert("BV", "bệnh viện"); m.insert("BQP", "bộ quốc phòng");
    // Documents, administration, safety.
    m.insert("GPLX", "giấy phép lái xe"); m.insert("CMND", "chứng minh nhân dân");
    m.insert("CCCD", "căn cước công dân"); m.insert("PCCC", "phòng cháy chữa cháy");
    m.insert("ATGT", "an toàn giao thông"); m.insert("TNGT", "tai nạn giao thông");
    m.insert("BHYT", "bảo hiểm y tế"); m.insert("ATTP", "an toàn thực phẩm");
    m.insert("ĐKKD", "đăng ký kinh doanh"); m.insert("MST", "mã số thuế");
    // Ministries and departments, hyphenated and run-together spellings alike.
    m.insert("TN-MT", "tài nguyên môi trường"); m.insert("TNMT", "tài nguyên môi trường");
    m.insert("GD-ĐT", "giáo dục đào tạo"); m.insert("GDĐT", "giáo dục đào tạo");
    m.insert("KH-CN", "khoa học công nghệ");
    m.insert("LĐ-TB-XH", "lao động thương binh xã hội"); m.insert("LĐTBXH", "lao động thương binh xã hội");
    m.insert("NN-PTNT", "nông nghiệp phát triển nông thôn"); m.insert("NNPTNT", "nông nghiệp phát triển nông thôn");
    m.insert("VH-TT-DL", "văn hóa thể thao du lịch"); m.insert("VHTTDL", "văn hóa thể thao du lịch");
    m.insert("TT-TT", "thông tin truyền thông"); m.insert("TTTT", "thông tin truyền thông");
    // Geography, organisations, groups of people.
    m.insert("ĐBSCL", "đồng bằng sông cửu long"); m.insert("MTTQ", "mặt trận tổ quốc");
    m.insert("ĐBQH", "đại biểu quốc hội"); m.insert("VKS", "viện kiểm sát");
    m.insert("HS-SV", "học sinh sinh viên"); m.insert("HSSV", "học sinh sinh viên");
    m.insert("SV", "sinh viên"); m.insert("GV", "giáo viên");
    m.insert("CBCNV", "cán bộ công nhân viên");
    // Companies and official documents.
    m.insert("CP", "cổ phần"); m.insert("NĐ-CP", "nờ đê xê phê");
    m.insert("TT-BTC", "tê tê bê tê xê"); m.insert("QĐ-TTg", "qui đê tê tê giê");
    // Justice, land, budget, labour.
    m.insert("VKSND", "viện kiểm sát nhân dân"); m.insert("GPMB", "giải phóng mặt bằng");
    m.insert("NSNN", "ngân sách nhà nước"); m.insert("XKLĐ", "xuất khẩu lao động");
    m.insert("UBMTTQ", "uỷ ban mặt trận tổ quốc"); m.insert("HĐLĐ", "hợp đồng lao động");
    // Technology and commerce.
    m.insert("CNTT", "công nghệ thông tin"); m.insert("TMĐT", "thương mại điện tử");
    // Business and finance.
    m.insert("TNDN", "thu nhập doanh nghiệp"); m.insert("TNCN", "thu nhập cá nhân");
    m.insert("GTGT", "giá trị gia tăng"); m.insert("BCTC", "báo cáo tài chính");
    m.insert("ĐHĐCĐ", "đại hội đồng cổ đông"); m.insert("TGĐ", "tổng giám đốc");
    m.insert("PTGĐ", "phó tổng giám đốc"); m.insert("Cty", "công ty");
    m.insert("TNHH MTV", "trách nhiệm hữu hạn một thành viên");
    // Ministries written with "&". Masked early, since the special character
    // would otherwise be rewritten before the abbreviation is recognised.
    m.insert("GD&ĐT", "giáo dục đào tạo"); m.insert("TN&MT", "tài nguyên môi trường");
    m.insert("KH&CN", "khoa học công nghệ"); m.insert("LĐ-TB&XH", "lao động thương binh xã hội");
    m.insert("KH&ĐT", "kế hoạch đầu tư"); m.insert("KHĐT", "kế hoạch đầu tư");
    m.insert("TT&TT", "thông tin truyền thông"); m.insert("TTTT", "thông tin truyền thông");
    m.insert("VH-TT&DL", "văn hóa thể thao du lịch"); m.insert("VHTTDL", "văn hóa thể thao du lịch");
    m.insert("CT&XH", "chính trị xã hội");
    // Document reference numbers.
    m.insert("TT-BYT", "tê tê bê y tê"); m.insert("CT-TTg", "xê tê tê tê giê");
    m.insert("UBND-VP", "uỷ ban nhân dân vê phê");
    m.insert("QH11", "quốc hội mười một"); m.insert("QH12", "quốc hội mười hai");
    m.insert("QH13", "quốc hội mười ba"); m.insert("QH14", "quốc hội mười bốn");
    m.insert("QH15", "quốc hội mười lăm"); m.insert("QH16", "quốc hội mười sáu");
    // Note: weekday abbreviations T2..T7 and CN are resolved by context in
    // expand_weekday_abbr, which requires a time cue ("sáng", "chiều", "từ",
    // "đến"). They are deliberately absent from this table, since a hard
    // mapping would rewrite "Model T2" and "ga T3" as weekdays.
    m
});

pub static TECHNICAL_TERMS: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert("JSON", "__start_en__j son__end_en__");
    m.insert("VRAM", "__start_en__v ram__end_en__");
    // "arXiv" has an uppercase letter mid-word. Without this entry the camel
    // splitter produces "ar Xiv", and "xiv" hits a junk "roman fourteen"
    // dictionary entry. Forcing the lowercase form makes it read as one word.
    m.insert("arXiv", "__start_en__arxiv__end_en__");
    // Initialisms conventionally read as words rather than spelled out.
    m.insert("TOEIC", "__start_en__toeic__end_en__");
    m.insert("UNICEF", "__start_en__unicef__end_en__");
    m.insert("ASIAD", "a si át");
    m.insert("SEA Games", "__start_en__sea games__end_en__");
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

// Domain suffixes as Vietnamese speakers actually say them: "vê nờ", "i ô",
// "ê đu", "o rờ gờ". Pure-English sentences skip this table and use English
// letter names instead.
pub static DOMAIN_SUFFIX_MAP: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert("com", "com"); m.insert("vn", "vê nờ");
    m.insert("net", "nét"); m.insert("org", "o rờ gờ");
    m.insert("edu", "ê đu"); m.insert("gov", "gờ o vê");
    m.insert("io", "i ô"); m.insert("biz", "biz");
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
    // The Vietnamese dong sign. Its absence meant "100.000₫" lost its currency
    // entirely — the most consequential omission of the set, in the one currency
    // this library is built for.
    m.insert("₫", "đồng");
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
    m.insert('√', " căn bậc hai "); m.insert('∛', " căn bậc ba "); m.insert('∜', " căn bậc bốn ");
    m.insert('→', " đến "); m.insert('÷', " chia "); m.insert('*', " sao ");
    m.insert('×', " nhân "); m.insert('^', " mũ "); m.insert('~', " khoảng ");
    m.insert('%', " phần trăm "); m.insert('$', " đô la "); m.insert('€', " ê rô ");
    m.insert('£', " bảng "); m.insert('¥', " yên "); m.insert('₩', " won ");
    m.insert('₭', " kíp "); m.insert('₱', " bê xô "); m.insert('฿', " bạc ");
    m.insert('Ω', " ôm "); m.insert('@', " a còng "); m.insert('≠', " khác ");
    m.insert('∀', " với mọi "); m.insert('∏', " tích "); m.insert('∈', " thuộc ");
    m.insert('∃', " tồn tại ");
    // Arrow family. Only "→" was mapped before, so "⇒" (implication) and the
    // bidirectional forms were deleted outright — silently changing the meaning
    // of a logical statement.
    m.insert('←', " ngược lại "); m.insert('↔', " tương đương ");
    m.insert('⇒', " suy ra "); m.insert('⇐', " ngược lại "); m.insert('⇔', " tương đương ");
    m.insert('∑', " tổng "); m.insert('∩', " giao "); m.insert('∪', " hội ");
    m.insert('¬', " phủ định "); m.insert('∞', " vô cùng "); m.insert('α', " an pha ");
    m.insert('β', " bê ta "); m.insert('γ', " ga ma "); m.insert('δ', " đen ta ");
    m.insert('ε', " ép si lon "); m.insert('ϵ', " thuộc "); m.insert('ζ', " de ta ");
    m.insert('η', " ê ta "); m.insert('θ', " thê ta "); m.insert('ι', " i ô ta ");
    m.insert('κ', " cáp ba "); m.insert('λ', " lam đa "); m.insert('ᴧ', " và ");
    m.insert('μ', " muy "); m.insert('Δ', " đen ta "); m.insert('ν', " nu ");
    // U+2206 INCREMENT is the "mathematical delta" editors often insert in
    // place of Greek Δ ("Q = mc∆t", "F = k∆l"). It belonged to no table before
    // and was therefore deleted in silence.
    m.insert('∆', " đen ta ");
    m.insert('ξ', " xi xi "); m.insert('ο', " o mi ron "); m.insert('π', " pi ");
    m.insert('ρ', " ro "); m.insert('σ', " xích ma "); m.insert('τ', " tao ");
    m.insert('υ', " úp si lon "); m.insert('φ', " phi "); m.insert('χ', " chi ");
    m.insert('ψ', " si "); m.insert('ω', " ô me ga "); m.insert('©', " bản quyền ");
    // ── Maths symbols that used to be swallowed by RE_CLEAN_OTHERS ──────────
    m.insert('∫', " tích phân "); m.insert('∮', " tích phân đường ");
    m.insert('∂', " đạo hàm riêng "); m.insert('∇', " nabla ");
    m.insert('∝', " tỉ lệ với "); m.insert('∠', " góc ");
    m.insert('⊥', " vuông góc với "); m.insert('∥', " song song với ");
    m.insert('⊂', " là tập con của "); m.insert('⊆', " là tập con của ");
    m.insert('⊃', " chứa "); m.insert('⊇', " chứa ");
    m.insert('∅', " tập rỗng "); m.insert('∉', " không thuộc ");
    m.insert('≡', " tương đương "); m.insert('≅', " đồng dạng "); m.insert('∼', " tương đương ");
    m.insert('∴', " suy ra "); m.insert('∵', " bởi vì ");
    // Capital sigma, both the Greek letter U+03A3 and the summation sign
    // U+2211. Neither was mapped, so summations vanished from the output.
    m.insert('Σ', " xích ma "); m.insert('∑', " xích ma ");
    m.insert('⋅', " nhân "); m.insert('·', " nhân "); m.insert('∓', " trừ cộng ");
    // Blackboard-bold number sets.
    m.insert('ℝ', " tập số thực "); m.insert('ℕ', " tập số tự nhiên ");
    m.insert('ℤ', " tập số nguyên "); m.insert('ℚ', " tập số hữu tỉ ");
    m.insert('ℂ', " tập số phức ");
    m.insert('½', " một phần hai "); m.insert('¼', " một phần tư "); m.insert('¾', " ba phần tư ");
    m.insert('⅓', " một phần ba "); m.insert('⅔', " hai phần ba ");
    m.insert('⅕', " một phần năm "); m.insert('⅖', " hai phần năm "); m.insert('⅗', " ba phần năm "); m.insert('⅘', " bốn phần năm ");
    m.insert('⅙', " một phần sáu "); m.insert('⅚', " năm phần sáu ");
    m.insert('⅛', " một phần tám "); m.insert('⅜', " ba phần tám ");
    m.insert('⅝', " năm phần tám "); m.insert('⅞', " bảy phần tám ");
    m
});

pub static SUPERSCRIPTS_MAP: Lazy<HashMap<char, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert('⁰', " không "); m.insert('¹', " một "); m.insert('²', " bình phương ");
    m.insert('³', " lập phương ");
    // ⁴-⁹ are read as "mũ X". ² and ³ keep the idiomatic Vietnamese forms
    // "bình phương" and "lập phương".
    m.insert('⁴', " mũ bốn "); m.insert('⁵', " mũ năm ");
    m.insert('⁶', " mũ sáu "); m.insert('⁷', " mũ bảy "); m.insert('⁸', " mũ tám ");
    m.insert('⁹', " mũ chín ");
    m.insert('ⁿ', " mũ n "); m.insert('ⁱ', " mũ i ");
    // A lone superscript sign; the "⁻³" cluster is handled earlier by the
    // signed-exponent pass.
    m.insert('⁻', " trừ "); m.insert('⁺', " cộng ");
    // Modifier-letter exponents ("2ˣ = 32", "eˣ"), previously deleted.
    m.insert('ˣ', " mũ x "); m.insert('ʸ', " mũ y "); m.insert('ᵏ', " mũ k ");
    m.insert('ᵗ', " mũ t ");
    m
});

pub static SUBSCRIPTS_MAP: Lazy<HashMap<char, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert('₀', " không "); m.insert('₁', " một "); m.insert('₂', " hai ");
    m.insert('₃', " ba "); m.insert('₄', " bốn "); m.insert('₅', " năm ");
    m.insert('₆', " sáu "); m.insert('₇', " bảy "); m.insert('₈', " tám ");
    m.insert('₉', " chín ");
    // Letter subscripts (aᵢ, xₙ) read by letter name; previously deleted.
    m.insert('ᵢ', " i "); m.insert('ⱼ', " j "); m.insert('ₐ', " a "); m.insert('ₑ', " e ");
    m.insert('ₒ', " o "); m.insert('ₓ', " x "); m.insert('ₕ', " h "); m.insert('ₖ', " k ");
    m.insert('ₗ', " l "); m.insert('ₘ', " m "); m.insert('ₙ', " n "); m.insert('ₚ', " p ");
    m.insert('ₛ', " s "); m.insert('ₜ', " t "); m.insert('ᵣ', " r "); m.insert('ᵤ', " u ");
    m.insert('ᵥ', " v "); m.insert('₊', " cộng "); m.insert('₋', " trừ ");
    m
});

pub static WORD_LIKE_ACRONYMS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    let mut s = HashSet::new();
    let words = [
        "UNESCO", "NASA", "NATO", "ASEAN", "OPEC", "SARS", "FIFA", "UNIC", "RAM", "VRAM", "COVID", "IELTS", "STEM",
        "ISO",
        // Sport, organisations, examinations, everyday life — all read as words.
        "UEFA", "EURO", "VAR", "ASIAD", "INTERPOL", "UNICEF",
        "TOEFL", "PISA", "STEAM", "SAT", "GMAT",
        "AIDS", "MERS", "ECMO", "LASIK",
        "FED", "NASDAQ", "UPCOM", "FOMO", "YOLO", "ASAP",
        "RADAR", "LASER", "LIDAR", "SONAR", "SCUBA", "GIF", "JPEG", "UNIX", "WIFI",
        "SIM", "LED", "VIP", "SPA", "GYM", "POS",
        "SWAT", "SEAL", "WASP", "COBOL", "BASIC", "OLED", "COVAX", "BRICS", "APEC", "VUCA", "PERMA", "DINK",
        "MENA", "EPIC", "OASIS", "BASE", "DART", "IDEA", "CHAOS", "SMART", "FANG", "BLEU", "REST", "ERROR",
        "SELECT", "FROM", "WHERE", "ORDER", "BY", "LIMIT", "OFFSET", "GROUP", "HAVING", "JOIN", "LEFT", "RIGHT", 
        "INNER", "OUTER", "ON", "AS", "AND", "OR", "NOT", "IN", "BETWEEN", "LIKE", "IS", "NULL", "TRUE", "FALSE", 
        "CASE", "WHEN", "THEN", "ELSE", "END", "UNION", "INTERSECT", "EXCEPT", "DESC"
    ];
    for w in words { s.insert(w); }
    s
});

/// English acronyms and brands joined by "&": the "&" is read "and" and the
/// whole span is tagged English ("R&D" -> "<en>r and d</en>"). Keys are the
/// uppercase run with the "&" removed.
///
/// The list is curated rather than pattern-based so that an ordinary
/// "A & B" ("option A and B") still takes the Vietnamese reading "và".
pub static ENGLISH_AMPERSAND: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    let mut s = HashSet::new();
    let words = [
        "RD", "MA", "SP", "PG", "ATT", "HM", "FB", "QA", "RB", "TC",
        "JJ", "PL", "MM", "BW", "GT", "AE", "BQ", "DG", "SM", "BB", "PC",
    ];
    for w in words { s.insert(w); }
    s
});

pub static COMMON_EMAIL_DOMAINS: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();
    // The domain name is left bare for G2P to look up in the English
    // dictionary; only the suffix takes a Vietnamese reading.
    m.insert("gmail.com", "gmail chấm com");
    m.insert("yahoo.com", "yahoo chấm com");
    m.insert("yahoo.com.vn", "yahoo chấm com chấm vê nờ");
    m.insert("outlook.com", "outlook chấm com");
    m.insert("hotmail.com", "hotmail chấm com");
    m.insert("icloud.com", "icloud chấm com");
    m.insert("fpt.vn", "ép phê tê chấm vê nờ");
    m.insert("fpt.com.vn", "ép phê tê chấm com chấm vê nờ");
    m
});

/// Acronyms spelled with ENGLISH letter names. Doubles as the veto list for
/// the dictionary-arbiter fallback: several of these are valid Vietnamese
/// syllables whose lowercase form is a dictionary word (LA, CA, IT, AM, PM),
/// and without an entry here they would be read as words.
pub static ACRONYMS_SPELL_EN: &[&str] = &[
    "LA", "CA", "IT", "US", "UK", "AI", "ID", "IP", "PC", "TV", "CD", "DVD",
    "USB", "GPS", "SUV", "CEO", "CFO", "CTO", "GDP", "FBI", "CIA", "NBA",
    "DJ", "PR", "HR", "IQ", "EQ", "MV", "EP", "URL", "SEO", "AM", "PM",
    "NY", "HP", "SEA", "AN",
    // Foreign universities, organisations and brands. Unlike BA/CO/MA (which
    // compete with real Vietnamese words), these have no Vietnamese reading
    // to lose, so pinning them costs nothing. Listed even when the fallback
    // already spells them correctly: several are one dictionary change away
    // from breaking — "MIT" sat in the common table and was read "mít".
    "MIT", "UCLA", "NYU", "USC", "RMIT", "NUS", "NTU", "ANU", "LSE",
    "UN", "WHO", "WTO", "IMF", "ADB", "OECD", "UNDP", "UNHCR", "ILO", "IAEA",
    "IBM", "BMW", "LG", "MSI", "AMD", "TSMC", "VW", "GM",
    "BBC", "CNN", "HBO", "MTV", "ESPN", "NFL", "MLB", "UFC", "PSG",
    "HSBC", "ANZ", "UOB", "DBS", "KFC",
];

/// Acronyms conventionally spelled with VIETNAMESE letter names.
pub static ACRONYMS_SPELL_VI: &[&str] = &["VTV", "VTC", "HTV", "EU", "QĐ"];

/// The Vietnamese abbreviation table: one lookup façade with an explicit
/// reading mode per entry, assembled from the data groups above
/// (`ACRONYMS_EXCEPTIONS_VI` = Expand, `TECHNICAL_TERMS` = Fixed,
/// `WORD_LIKE_ACRONYMS` = WordEn, the two spell lists = Letters*). Thai and
/// Indonesian instantiate the same [`AbbrevTable`] type with their own data.
pub static VI_ABBREV: Lazy<AbbrevTable> = Lazy::new(|| {
    // First insert wins: Expand and Fixed carry hand-written replacements and
    // outrank the mode-only groups — ASIAD and VRAM sit in WORD_LIKE_ACRONYMS
    // too, and must keep their Fixed readings ("a si át", "v ram").
    let mut t = AbbrevTable::new();
    for (k, v) in ACRONYMS_EXCEPTIONS_VI.iter() {
        t.insert(k, Reading::Expand(v));
    }
    for (k, v) in TECHNICAL_TERMS.iter() {
        if !t.contains(k) { t.insert(k, Reading::Fixed(v)); }
    }
    for k in WORD_LIKE_ACRONYMS.iter() {
        if !t.contains(k) { t.insert(*k, Reading::WordEn); }
    }
    for k in ACRONYMS_SPELL_EN {
        if !t.contains(k) { t.insert(k, Reading::LettersEn); }
    }
    for k in ACRONYMS_SPELL_VI {
        if !t.contains(k) { t.insert(k, Reading::LettersNative); }
    }
    t
});

pub static COMBINED_EXCEPTIONS: Lazy<HashMap<String, String>> = Lazy::new(|| {
    VI_ABBREV
        .replacement_keys()
        .map(|k| (k.to_string(), VI_ABBREV.replacement(k).unwrap().to_string()))
        .collect()
});

pub static DATE_KEYWORDS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    let mut s = HashSet::new();
    let words = [
        "vào", "ngày", "hôm", "hôm nay", "hôm qua", "hôm kia", "mai", "ngày mai", "ngày kia",
        "sinh", "sinh nhật", "kỷ niệm", "lễ", "tết", "diễn ra", "tổ chức", "thứ", "tuần", "tháng", "năm",
        "phiên", "mùng", "mồng"
    ];
    for w in words { s.insert(w); }
    s
});

/// Date lead words, counted ONLY when immediately before a "d/m" cluster —
/// unlike [`DATE_KEYWORDS`], which scans a three-word window on either side.
///
/// The adjacency requirement is what separates "chiều 17/10", a date, from
/// "chiều dài 3/4 mét", a fraction: in the latter, "dài" sits in between.
pub static DATE_LEAD_WORDS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    let mut s = HashSet::new();
    let words = [
        // Parts of the day: "chiều 17/10", "sáng 5/9", "đêm 30/4".
        "sáng", "trưa", "chiều", "tối", "đêm", "khuya", "rạng",
        // Time markers: "trước 30/4", "từ 1/8", "hết 31/8", "hạn 20/11".
        "trước", "từ", "hết", "hạn", "đợt", "nghỉ", "lúc",
    ];
    for w in words { s.insert(w); }
    s
});

/// Cue words that must sit immediately before a Roman numeral for it to be read
/// as a number ("thế kỷ XXI", "chương IV").
///
/// Without a cue, any abbreviation or English word built from I V X L C D M
/// would be converted — "CD", "MC", "XL", "MIX". Only the word directly before
/// the numeral is checked, so multi-word cues need just their final word here:
/// "thế kỷ" -> "kỷ", "đại hội" -> "hội", "thế chiến" -> "chiến".
pub static ROMAN_KEYWORDS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    let mut s = HashSet::new();
    let words = [
        "kỷ", "kỉ", "chương", "phần", "hồi", "quyển", "tập", "kỳ", "kì",
        "khoản", "điều", "mục", "đời", "vua", "chiến", "hội", "khóa", "khoá",
        "đệ", "triều", "quý", "lần", "vòng", "thứ",
        // Names of monarchs and popes, which regularly carry Roman numerals.
        // Safe to list because only an uppercase numeral directly after them
        // can match.
        "louis", "napoléon", "napoleon", "henry", "george", "charles",
        "elizabeth", "edward", "william", "james", "richard", "john",
        "philip", "philippe", "frederick", "ferdinand", "peter", "pierre",
        "catherine", "pius", "benedict", "leo", "gregory", "otto",
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
