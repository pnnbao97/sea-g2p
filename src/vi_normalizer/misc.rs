use fancy_regex::{Regex as FRegex, Captures as FCaps};
use regex::{Regex, Captures};
use once_cell::sync::Lazy;
use crate::vi_normalizer::num2vi::{n2w, n2w_single};
use crate::vi_normalizer::resources::{
    VI_LETTER_NAMES, DOMAIN_SUFFIX_MAP,
    ROMAN_NUMERALS, ROMAN_KEYWORDS, ABBRS, SYMBOLS_MAP, WORD_LIKE_ACRONYMS, MEASUREMENT_KEY_VI,
    CURRENCY_KEY, COMBINED_EXCEPTIONS, SUPERSCRIPTS_MAP, SUBSCRIPTS_MAP, ENGLISH_AMPERSAND
};
use crate::vi_normalizer::technical::normalize_slashes;

const VI_UPPER: &str = "ĐĂÂÊÔƠƯ";

// ─ Patterns requiring look-arounds ───────────────────────────────────────
static RE_ROMAN_NUMBER: Lazy<FRegex> = Lazy::new(|| {
    // CHỈ chữ HOA: số La Mã thật luôn viết hoa ("Chương IV", "Edward II"). Không nhận
    // chữ thường vì các âm tiết tiếng Việt viết thường ("di", "vi", "li", "cd"...) trùng
    // dạng số La Mã và gây mở rộng sai (vd "lần di chuyển" -> "lần 501 chuyển").
    FRegex::new(r"\b(?=[IVXLCDM]{2,})(?:M{0,4}(?:CM|CD|D?C{0,3})(?:XC|XL|L?X{0,3})(?:IX|IV|V?I{0,3}))(?<=[IVXLCDM])\b").unwrap()
});
// Số La Mã mở đầu dòng ở dạng SỐ THỨ TỰ ĐỀ MỤC: "I. VỀ ĐỀ NGHỊ...", "II. Về ...".
// Đọc là số ("một", "hai"...) chứ không phải chữ cái. Bắt cả trường hợp 1 ký tự (như "I")
// mà RE_ROMAN_NUMBER (yêu cầu ≥2 ký tự + từ dẫn) bỏ sót. Điều kiện: đứng đầu dòng, kèm dấu ".".
// Group: (1)=thụt đầu dòng, (2)=số La Mã, (3)=dấu chấm + khoảng trắng, (4)=chuỗi chữ HOA của
// tiêu đề theo sau (lookahead, không nuốt) — dùng để phân biệt số thứ tự với chữ viết tắt tên
// riêng ("C. Mác", "V. Nguyễn"): ký tự đơn chỉ coi là số khi tiêu đề viết HOA (≥2 chữ HOA).
static RE_ROMAN_LIST_MARKER: Lazy<FRegex> = Lazy::new(|| {
    FRegex::new(r"(?m)^([ \t]*)([IVXLCDM]+)(\.[ \t]+)(?=(\p{Lu}+))").unwrap()
});
// Đề mục La Mã thực tế không vượt quá ~XX; giá trị lớn hơn gần như chắc chắn là chữ viết tắt
// tên riêng (C=100, L=50, D=500, M=1000) nên loại ra để tránh đọc nhầm "C. Mác" -> "một trăm".
const ROMAN_MARKER_MAX: i32 = 30;
// Bỏ dấu chấm viết tắt chức danh khi theo sau là tên riêng (TS. Nguyễn -> TS Nguyễn),
// tránh dấu "." biến thành ranh giới câu gây ngắt nhịp sai.
static RE_TITLE_DOT: Lazy<FRegex> = Lazy::new(|| {
    FRegex::new(r"\b(TS|GS|BS|ThS|PGS|KS|ĐH)\.\s+(?=\p{Lu})").unwrap()
});
static RE_STANDALONE_LETTER: Lazy<FRegex> = Lazy::new(|| {
    FRegex::new(r"(?<![\''])\b([a-zA-Z])\b(\.?)").unwrap()
});
pub static RE_ACRONYM: Lazy<FRegex> = Lazy::new(|| {
    FRegex::new(&format!(r"\b(?=[A-Z{}a-z{}0-9]*[A-Z{}])(?:[A-Z{}][a-z{}]?\d*){{2,}}\b", VI_UPPER, VI_UPPER, VI_UPPER, VI_UPPER, "đăâêôơư")).unwrap()
});
static RE_VERSION: Lazy<FRegex> = Lazy::new(|| {
    FRegex::new(r"(?<![-\u2013\u2014])\b(\d+(?:\.\d+){2,})\b").unwrap()
});
static RE_PRIME: Lazy<FRegex> = Lazy::new(|| {
    // \u0110\u1ebfm s\u1ed1 d\u1ea5u ph\u1ea9y: f' -> "ph\u1ea9y", y'' -> "ph\u1ea9y ph\u1ea9y" (\u0111\u1ea1o h\u00e0m c\u1ea5p 2).
    FRegex::new(r"(\b[a-zA-Z0-9])(['\u2019]+)(?!\w)").unwrap()
});
// Gi\u00e1 tr\u1ecb tuy\u1ec7t \u0111\u1ed1i |x|, |x+1| -> "gi\u00e1 tr\u1ecb tuy\u1ec7t \u0111\u1ed1i c\u1ee7a ...". Y\u00eau c\u1ea7u n\u1ed9i dung
// kh\u00f4ng c\u00f3 kho\u1ea3ng tr\u1eafng \u1edf m\u00e9p -> kh\u00f4ng \u0111\u1ee5ng d\u1ea5u "|" c\u1ee7a b\u1ea3ng "| c\u1ed9t |".
static RE_ABS: Lazy<FRegex> = Lazy::new(|| {
    FRegex::new(r"\|(\S(?:[^|]*\S)?)\|").unwrap()
});
static RE_PRIME_DIGIT: Lazy<FRegex> = Lazy::new(|| {
    FRegex::new(r"(?<=\d)(['\u2019]+|[\x22\u201D])").unwrap()
});

// ─ Simple patterns (regex crate — Thompson NFA, fast) ──────────────────
static RE_LETTER: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?i)(chữ|chữ cái|kí tự|ký tự)\s+(['"]?)([a-z])(['"]?)\b"#).unwrap()
});
static RE_ALPHANUMERIC: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b(\d+)([a-zA-Z])\b").unwrap()
});
// Cụm acronym/thương hiệu Anh nối "&" (R&D, R & D, AT&T, S&P...).
static RE_AMPERSAND_ACRONYM: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\b([a-z]{1,4})\s*&\s*([a-z]{1,4})\b").unwrap()
});
// Nhãn size quần áo: PHẢI có "size"/"cỡ" đứng trước (size M/L/XL, cỡ M).
// Khi đó S/M/L/XL... là nhãn (đọc chữ cái), không phải đơn vị (triệu/lít).
static RE_SIZE_LABEL: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\b(size|cỡ)\s+((?:xxxl|xxl|xl|xs|[sml])(?:\s*/\s*(?:xxxl|xxl|xl|xs|[sml]))*)\b").unwrap()
});

pub fn expand_size_labels(text: &str) -> String {
    RE_SIZE_LABEL.replace_all(text, |caps: &Captures| {
        let prefix = caps.get(1).unwrap().as_str();
        let spelled: Vec<String> = caps.get(2).unwrap().as_str()
            .split('/')
            .map(|s: &str| {
                let letters = s.trim().to_lowercase().chars()
                    .map(|c: char| c.to_string())
                    .collect::<Vec<String>>()
                    .join(" ");
                format!("__start_en__{}__end_en__", letters)
            })
            .collect();
        format!("{} {}", prefix, spelled.join(" "))
    }).into_owned()
}
static RE_LETTER_DIGIT: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b([a-zA-Z])(\d+)\b").unwrap()
});
static RE_BRACKETS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"[\(\[\{]\s*(.*?)\s*[\)\]\}]").unwrap()
});
static RE_STRIP_BRACKETS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"[\[\]\(\)\{\}]").unwrap()
});
static RE_TEMP_C_NEG: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)-(\d+(?:[.,]\d+)?)\s*°\s*c\b").unwrap()
});
static RE_TEMP_F_NEG: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)-(\d+(?:[.,]\d+)?)\s*°\s*f\b").unwrap()
});
static RE_TEMP_C: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(\d+(?:[.,]\d+)?)\s*°\s*c\b").unwrap()
});
static RE_TEMP_F: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(\d+(?:[.,]\d+)?)\s*°\s*f\b").unwrap()
});
static RE_DEGREE: Lazy<Regex> = Lazy::new(|| Regex::new(r"°").unwrap());
static RE_STANDARD_COLON: Lazy<FRegex> = Lazy::new(|| {
    // Lookbehind tránh khớp một phần "1.5:1"; lookahead chặn số nối tiếp/thập phân
    // nhưng CHO PHÉP dấu câu cuối câu ("2:1." vẫn là tỷ lệ "hai trên một").
    FRegex::new(r"(?<![.,\d])\b(\d+):(\d+(?:\.\d+)?)\b(?!\d)(?![.,]\d)").unwrap()
});
// Tỷ lệ >= 3 thành phần ngăn bởi ":" (1:2:3) — KHÔNG phải giờ (đã loại ở
// normalize_time). Đọc các số nối bằng "trên".
static RE_RATIO_MULTI: Lazy<FRegex> = Lazy::new(|| {
    // Chặn nối tiếp số (:\d, \d) và số thập phân (.\d/,\d) nhưng CHO PHÉP dấu câu
    // cuối câu (vd "1:2:3." vẫn là tỷ lệ, không bị cắt thành "một trên hai, ba").
    FRegex::new(r"(?<![.,\d:])\d+(?::\d+){2,}(?![\d:])(?![.,]\d)").unwrap()
});
static RE_CLEAN_OTHERS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"[^a-zA-Z0-9\sàáảãạăắằẳẵặâấầẩẫậèéẻẽẹêếềểễệìíỉĩịòóỏõọôốồổỗộơớờởỡợùúủũụưứừửữựỳýỷỹỵđÀÁẢÃẠĂẮẰẲẴẶÂẤẦẨẪẬÈÉẺẼẸÊẾỀỂỄỆÌÍỈĨỊÒÓỎÕỌÔỐỒỔỖỘƠỚỜỞỠỢÙÚỦŨỤƯỨỪỬỮỰỲÝỶỸỴĐ.,!?_\'\'-]").unwrap()
});
static RE_CLEAN_QUOTES: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"[“”„]"#).unwrap()
});
static RE_CLEAN_QUOTES_EDGES: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(^|[\s.,!?;:])[\u2018\u2019']+|[\u2018\u2019']+($|[\s.,!?;:])").unwrap()
});
static RE_COLON_SEMICOLON: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"[:;]").unwrap()
});
static RE_UNIT_POWERS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b([a-zA-Z]+)\^([-+]?\d+)\b").unwrap()
});
pub static RE_ACRONYMS_EXCEPTIONS: Lazy<Regex> = Lazy::new(|| {
    let mut keys: Vec<String> = COMBINED_EXCEPTIONS.keys().map(|k: &String| k.to_string()).collect();
    keys.sort_by_key(|b: &String| std::cmp::Reverse(b.len()));
    let pattern = keys.iter().map(|k: &String| format!(r"\b{}\b", regex::escape(k))).collect::<Vec<String>>().join("|");
    Regex::new(&pattern).unwrap()
});
pub static DOMAIN_SUFFIXES_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\.(com|vn|net|org|edu|gov|io|biz|info)\b").unwrap()
});
// Ranh giới "câu" để xét heuristic TOÀN HOA: dấu kết câu .!? HOẶC xuống dòng \n.
// Xuống dòng là ranh giới đề mục: tiêu đề viết HOA thường KHÔNG có dấu chấm cuối mà chỉ
// ngắt dòng ("...CÔNG TRẠNG\n\nHuân chương..."). Nếu không tách theo \n, đề mục HOA bị gộp
// với đoạn thường phía sau -> mất tính "toàn hoa" -> từ Việt không dấu (LAO, KHEN) bị đọc
// như tiếng Anh (<en>l a o</en>). Xem issue #177.
static RE_ACRONYMS_SPLIT: Lazy<regex::Regex> = Lazy::new(|| {
    regex::Regex::new(r"([.!?]+(?:\s+|$)|\n+)").unwrap()
});

/// Tập nguyên âm tiếng Việt (kèm mọi dấu thanh), chữ thường.
const VI_VOWELS: &str = "aáàảãạăắằẳẵặâấầẩẫậeéèẻẽẹêếềểễệiíìỉĩịoóòỏõọôốồổỗộơớờởỡợuúùủũụưứừửữựyýỳỷỹỵ";

fn is_vi_vowel(c: char) -> bool {
    VI_VOWELS.contains(c)
}

/// Kiểm tra `s` có phải MỘT âm tiết tiếng Việt hợp lệ không
/// (phụ âm đầu? + nguyên âm + phụ âm cuối?). Dùng để phân biệt TỪ tiếng Việt
/// viết hoa ("CHƯƠNG","ĐƯỜNG","PHƯỜNG") với acronym/công thức gồm các chữ cái
/// (vd "ĐKVĐ" không có nguyên âm hợp lệ -> không phải âm tiết -> vẫn tách).
/// Thiên về CHẶT: khi không chắc thì trả false (hệ quả là tách ký tự như cũ,
/// an toàn hơn việc giữ nhầm một acronym).
fn is_vietnamese_syllable(s: &str) -> bool {
    let lower = s.to_lowercase();
    let chars: Vec<char> = lower.chars().collect();
    let n = chars.len();
    if n == 0 || chars.iter().any(|c: &char| !c.is_alphabetic()) {
        return false;
    }
    // Phụ âm đầu (onset): thử dài nhất trước, chỉ tách khi ngay sau là nguyên âm.
    const ONSETS: [&str; 28] = [
        "ngh", "ng", "nh", "ch", "gh", "gi", "kh", "ph", "th", "tr", "qu",
        "b", "c", "d", "đ", "g", "h", "k", "l", "m", "n", "p", "q", "r", "s", "t", "v", "x",
    ];
    let mut i = 0usize;
    for cand in ONSETS.iter() {
        let cl = cand.chars().count();
        if i + cl < n {
            let prefix: String = chars[i..i + cl].iter().collect();
            if prefix == *cand && is_vi_vowel(chars[i + cl]) {
                i += cl;
                break;
            }
        }
    }
    // Nguyên âm (nucleus): chuỗi nguyên âm liên tiếp, bắt buộc >= 1.
    let v_start = i;
    while i < n && is_vi_vowel(chars[i]) {
        i += 1;
    }
    if i == v_start {
        return false;
    }
    // Phụ âm cuối (coda): phần còn lại phải rỗng hoặc là một coda hợp lệ.
    let coda: String = chars[i..].iter().collect();
    matches!(
        coda.as_str(),
        "" | "c" | "ch" | "m" | "n" | "ng" | "nh" | "p" | "t"
    )
}

/// Trả về true nếu TỪ cuối cùng của `preceding` (đoạn văn ngay trước cụm La Mã)
/// nằm trong `ROMAN_KEYWORDS`. Bỏ qua dấu câu bám quanh từ.
fn has_roman_context(preceding: &str) -> bool {
    let last = preceding
        .split(|c: char| c.is_whitespace())
        .rev()
        .find(|w: &&str| !w.is_empty())
        .unwrap_or("")
        .trim_matches(|c: char| !c.is_alphanumeric())
        .to_lowercase();
    !last.is_empty() && ROMAN_KEYWORDS.contains(last.as_str())
}

/// Chuyển cụm số La Mã sang giá trị nguyên (0 nếu chứa ký tự không hợp lệ / rỗng).
pub fn roman_to_int(match_str: &str) -> i32 {
    let num = match_str.to_uppercase();
    let chars: Vec<char> = num.chars().collect();
    let mut result = 0;
    for i in 0..chars.len() {
        let val = *ROMAN_NUMERALS.get(&chars[i]).unwrap_or(&0);
        if i + 1 < chars.len() && val < *ROMAN_NUMERALS.get(&chars[i+1]).unwrap_or(&0) {
            result -= val;
        } else {
            result += val;
        }
    }
    result
}

pub fn expand_roman(match_str: &str) -> String {
    if match_str.is_empty() {
        return String::new();
    }
    let result = roman_to_int(match_str);
    if result == 0 {
        return match_str.to_string();
    }
    format!(" {} ", n2w(&result.to_string()))
}

pub fn expand_unit_powers(text: &str) -> String {
    RE_UNIT_POWERS.replace_all(text, |caps: &Captures| {
        let base = caps.get(1).unwrap().as_str();
        let power = caps.get(2).unwrap().as_str();
        let power_norm = if power.starts_with('-') {
            format!("trừ {}", n2w(&power[1..]))
        } else {
            n2w(&power.replace('+', ""))
        };
        let base_lower = base.to_lowercase();
        let full_base = MEASUREMENT_KEY_VI.get(base_lower.as_str())
            .or(CURRENCY_KEY.get(base_lower.as_str()))
            .copied()
            .unwrap_or(base);
        format!(" {} mũ {} ", full_base, power_norm)
    }).to_string()
}

pub fn expand_letter(text: &str) -> String {
    RE_LETTER.replace_all(text, |caps: &Captures| {
        let prefix = caps.get(1).unwrap().as_str();
        let char = caps.get(3).unwrap().as_str();
        if let Some(name) = VI_LETTER_NAMES.get(char.to_lowercase().as_str()) {
            format!("{} {} ", prefix, name)
        } else {
            caps.get(0).unwrap().as_str().to_string()
        }
    }).to_string()
}

pub fn expand_abbreviations(text: &str) -> String {
    let mut result = text.to_string();
    for (k, v) in ABBRS.iter() {
        result = result.replace(k, v);
    }
    result
}

pub fn expand_standalone_letters(text: &str) -> String {
    RE_STANDALONE_LETTER.replace_all(text, |caps: &FCaps| {
        let char_raw = caps.get(1).unwrap().as_str();
        let char_lower = char_raw.to_lowercase();
        let dot = caps.get(2).unwrap().as_str();
        if let Some(name) = VI_LETTER_NAMES.get(char_lower.as_str()) {
            if char_raw.chars().next().unwrap().is_uppercase() && dot == "." {
                format!(" {} ", name)
            } else {
                format!(" {}{} ", name, dot)
            }
        } else {
            caps.get(0).unwrap().as_str().to_string()
        }
    }).to_string()
}

pub fn normalize_acronyms(text: &str) -> String {
    let mut result = Vec::new();
    let re_split = &*RE_ACRONYMS_SPLIT;

    let mut last = 0;
    let mut final_parts = Vec::new();
    for mat in re_split.find_iter(text) {
        final_parts.push(&text[last..mat.start()]);
        final_parts.push(mat.as_str());
        last = mat.end();
    }
    final_parts.push(&text[last..]);

    for i in (0..final_parts.len()).step_by(2) {
        let s = final_parts[i];
        let sep = if i + 1 < final_parts.len() { final_parts[i+1] } else { "" };
        if s.is_empty() {
            result.push(sep.to_string());
            continue;
        }

        let words: Vec<&str> = s.split_whitespace().collect();
        // Quyết định "toàn chữ hoa" (heading/câu hét -> đọc như prose tiếng Việt):
        //   - Token thuần số/dấu câu ("4", "06") KHÔNG tính, nếu không "CHƯƠNG 4"
        //     sẽ bị coi là không-toàn-hoa và từ Việt viết hoa bị spell thành ký tự.
        //   - Token có chữ cái LẪN chữ số (CO2, H2O, B2B) là công thức/mã, KHÔNG
        //     phải prose -> để nhánh acronym xử lý (CO2 -> "xê ô hai").
        let letter_tokens: Vec<&&str> = words.iter()
            .filter(|w: &&&str| w.chars().any(|c: char| c.is_alphabetic()))
            .collect();
        let is_all_caps = !letter_tokens.is_empty()
            && letter_tokens.iter().all(|w: &&&str| {
                !w.chars().any(|c: char| c.is_numeric())
                    && w.chars().filter(|c: &char| c.is_alphabetic()).all(|c: char| c.is_uppercase())
            });

        let mut processed_s = s.to_string();
        if !is_all_caps {
            processed_s = RE_ACRONYM.replace_all(&processed_s, |caps: &FCaps| {
                let word = caps.get(0).unwrap().as_str();
                if word.chars().all(|c: char| c.is_ascii_digit()) { return word.to_string(); }
                if WORD_LIKE_ACRONYMS.contains(word) {
                    return format!("__start_en__{}__end_en__", word.to_lowercase());
                }

                let has_vi_letter = word.chars().any(|c: char| !c.is_ascii() && c.is_alphabetic());
                let is_mixed_case = word.chars().any(|c: char| c.is_lowercase()) && word.chars().any(|c: char| c.is_uppercase());
                let has_subscript = word.chars().any(|c: char| c >= '₀' && c <= '₉');

                // Từ tiếng Việt viết hoa toàn bộ tạo thành MỘT âm tiết hợp lệ là TỪ
                // (không phải acronym/công thức) -> giữ nguyên, không tách ký tự.
                // vd "CHƯƠNG"->"chương" giữa câu chữ thường; còn "ĐKVĐ" vẫn tách.
                if has_vi_letter && is_vietnamese_syllable(word) {
                    return word.to_lowercase();
                }

                if has_vi_letter || is_mixed_case || has_subscript {
                    let mut parts = Vec::new();
                    for c in word.chars() {
                        let cl = c.to_lowercase().to_string();
                        if c.is_ascii_digit() { parts.push(n2w_single(&c.to_string())); }
                        else if let Some(name) = VI_LETTER_NAMES.get(cl.as_str()) { parts.push(name.to_string()); }
                        else if let Some(sub_name) = SUBSCRIPTS_MAP.get(&c) { parts.push(sub_name.trim().to_string()); }
                        else if c.is_alphabetic() { parts.push(cl); }
                    }
                    return parts.join(" ");
                }

                if word.chars().any(|c: char| c.is_ascii_digit() || (c >= '₀' && c <= '₉')) {
                    let res: Vec<String> = word.chars().map(|c: char| {
                        if c.is_ascii_digit() { n2w_single(&c.to_string()) }
                        else if let Some(sub_name) = SUBSCRIPTS_MAP.get(&c) { sub_name.trim().to_string() }
                        else { VI_LETTER_NAMES.get(c.to_lowercase().to_string().as_str()).cloned().unwrap_or(c.to_string().as_str()).to_string() }
                    }).collect();
                    return res.join(" ");
                }

                let spaced_word = word.chars().filter(|c: &char| c.is_alphanumeric()).map(|c: char| c.to_lowercase().to_string()).collect::<Vec<String>>().join(" ");
                if !spaced_word.is_empty() { format!("__start_en__{}__end_en__", spaced_word) } else { word.to_string() }
            }).to_string();
        }
        result.push(processed_s + sep);
    }
    result.join("")
}

pub fn expand_alphanumeric(text: &str) -> String {
    RE_ALPHANUMERIC.replace_all(text, |caps: &Captures| {
        let num = caps.get(1).unwrap().as_str();
        let char = caps.get(2).unwrap().as_str().to_lowercase();
        if let Some(name) = VI_LETTER_NAMES.get(char.as_str()) {
            let mut pronunciation = name.to_string();
            if char == "d" && (text.to_lowercase().contains("quốc lộ") || text.to_lowercase().contains("ql")) {
                pronunciation = "đê".to_string();
            }
            format!("{} {}", num, pronunciation)
        } else {
            caps.get(0).unwrap().as_str().to_string()
        }
    }).into_owned()
}

/// "R&D"/"R & D" -> "<en>r and d</en>" cho các acronym tiếng Anh đã biết.
/// Cụm không nằm trong danh sách (vd "A & B") giữ nguyên để "&" -> "và" như cũ.
pub fn expand_english_ampersand(text: &str) -> String {
    RE_AMPERSAND_ACRONYM.replace_all(text, |caps: &Captures| {
        let l = caps.get(1).unwrap().as_str();
        let r = caps.get(2).unwrap().as_str();
        let key = format!("{}{}", l.to_uppercase(), r.to_uppercase());
        if ENGLISH_AMPERSAND.contains(key.as_str()) {
            let spell = |s: &str| s.chars()
                .map(|c: char| c.to_lowercase().to_string())
                .collect::<Vec<String>>()
                .join(" ");
            format!(" __start_en__{} and {}__end_en__ ", spell(l), spell(r))
        } else {
            caps.get(0).unwrap().as_str().to_string()
        }
    }).into_owned()
}

pub fn expand_symbols(text: &str) -> String {
    let res = text.replace("<>", " khác ");
    let mut result = String::with_capacity(res.len());
    for c in res.chars() {
        if let Some(v) = SYMBOLS_MAP.get(&c) {
            result.push_str(v);
        } else if let Some(v) = SUPERSCRIPTS_MAP.get(&c) {
            result.push_str(v);
        } else if let Some(v) = SUBSCRIPTS_MAP.get(&c) {
            result.push_str(v);
        } else {
            result.push(c);
        }
    }
    result
}

pub fn expand_prime(text: &str) -> String {
    let res = RE_PRIME.replace_all(text, |caps: &FCaps| {
        let val = caps.get(1).unwrap().as_str().to_lowercase();
        let name = if val.chars().next().unwrap().is_ascii_digit() {
            n2w_single(&val)
        } else {
            VI_LETTER_NAMES.get(val.as_str()).cloned().unwrap_or(&val).to_string()
        };
        let count = caps.get(2).unwrap().as_str().chars().count();
        let phay = vec!["phẩy"; count].join(" ");
        format!("{} {}", name, phay)
    }).to_string();

    RE_PRIME_DIGIT.replace_all(&res, |caps: &FCaps| {
        let q = caps.get(1).unwrap().as_str();
        if q == "\"" || q == "\u{201D}" || q.len() > 1 {
            " phẩy phẩy ".to_string()
        } else {
            " phẩy ".to_string()
        }
    }).to_string()
}

pub fn expand_temperatures(text: &str) -> String {
    let mut res = RE_TEMP_C_NEG.replace_all(text, "âm $1 độ xê").into_owned();
    res = RE_TEMP_F_NEG.replace_all(&res, "âm $1 độ ép").into_owned();
    res = RE_TEMP_C.replace_all(&res, "$1 độ xê").into_owned();
    res = RE_TEMP_F.replace_all(&res, "$1 độ ép").into_owned();
    RE_DEGREE.replace_all(&res, " độ ").into_owned()
}

pub fn normalize_others(text: &str) -> String {
    let text = RE_TITLE_DOT.replace_all(text, "$1 ").into_owned();
    let text = RE_ABS.replace_all(&text, " giá trị tuyệt đối của $1 ").into_owned();
    let mut res = RE_ACRONYMS_EXCEPTIONS.replace_all(&text, |caps: &Captures| {
        COMBINED_EXCEPTIONS.get(caps.get(0).unwrap().as_str()).cloned().unwrap_or(caps.get(0).unwrap().as_str().to_string())
    }).into_owned();

    res = normalize_slashes(&res);
    res = DOMAIN_SUFFIXES_RE.replace_all(&res, |caps: &Captures| {
        let suffix = DOMAIN_SUFFIX_MAP.get(caps.get(1).unwrap().as_str().to_lowercase().as_str()).copied().unwrap_or("");
        format!(" chấm {} ", if suffix.is_empty() { caps.get(1).unwrap().as_str() } else { suffix })
    }).into_owned();

    // Số thứ tự đề mục La Mã đầu dòng ("I. VỀ ...", "II. Về ...") -> đọc là số.
    // Giữ nguyên dấu "." (đóng vai trò ngắt nhịp cho đề mục), chỉ thay phần số La Mã.
    res = RE_ROMAN_LIST_MARKER.replace_all(&res, |caps: &FCaps| {
        let lead = caps.get(1).unwrap().as_str();
        let roman = caps.get(2).unwrap().as_str();
        let tail = caps.get(3).unwrap().as_str();
        let head_upper = caps.get(4).map(|m| m.as_str()).unwrap_or("");
        let value = roman_to_int(roman);
        let single = roman.chars().count() == 1;
        // Loại số quá lớn (chữ viết tắt tên: C/L/D/M) và ký tự đơn mà tiêu đề không viết HOA
        // toàn bộ (dễ nhầm chữ viết tắt tên riêng: "V. Nguyễn", "I. Trần").
        if value <= 0 || value > ROMAN_MARKER_MAX
            || (single && head_upper.chars().count() < 2)
        {
            return caps.get(0).unwrap().as_str().to_string();
        }
        format!("{}{}{}", lead, n2w(&value.to_string()), tail)
    }).to_string();

    // Chỉ mở rộng số La Mã khi có từ dẫn ngay trước (thế kỷ/chương/phần/đời/vua...).
    // Nếu không, để nguyên cụm để nhánh acronym xử lý (vd "CD","MC","XL" -> <en>).
    let roman_src = res.clone();
    res = RE_ROMAN_NUMBER.replace_all(&roman_src, |caps: &FCaps| {
        let m = caps.get(0).unwrap();
        if has_roman_context(&roman_src[..m.start()]) {
            expand_roman(m.as_str())
        } else {
            m.as_str().to_string()
        }
    }).to_string();

    res = expand_letter(&res);
    res = expand_alphanumeric(&res);
    res = RE_LETTER_DIGIT.replace_all(&res, |caps: &Captures| {
        let l = caps.get(1).unwrap().as_str().to_lowercase();
        let d = caps.get(2).unwrap().as_str();
        if let Some(name) = VI_LETTER_NAMES.get(l.as_str()) {
            format!("{} {}", name, n2w(d))
        } else {
            caps.get(0).unwrap().as_str().to_string()
        }
    }).into_owned();
    res = expand_prime(&res);
    res = expand_unit_powers(&res);
    res = RE_CLEAN_QUOTES.replace_all(&res, "").into_owned();
    res = RE_CLEAN_QUOTES_EDGES.replace_all(&res, "$1 $2").into_owned();
    res = expand_english_ampersand(&res);
    res = expand_symbols(&res);
    res = RE_BRACKETS.replace_all(&res, ", $1, ").into_owned();
    res = RE_STRIP_BRACKETS.replace_all(&res, " ").into_owned();
    res = expand_temperatures(&res);
    res = normalize_acronyms(&res);

    res = RE_VERSION.replace_all(&res, |caps: &FCaps| {
        let parts: Vec<String> = caps.get(1).unwrap().as_str().split('.').map(|s: &str| {
            s.chars().map(|c: char| n2w_single(&c.to_string())).collect::<Vec<String>>().join(" ")
        }).collect();
        parts.join(" chấm ")
    }).to_string();

    // Tỷ lệ nhiều thành phần "1:2:3" -> "một trên hai trên ba" (trước ratio 2 số).
    res = RE_RATIO_MULTI.replace_all(&res, |caps: &FCaps| {
        let parts: Vec<String> = caps.get(0).unwrap().as_str()
            .split(':')
            .map(|p: &str| n2w(p))
            .collect();
        format!(" {} ", parts.join(" trên "))
    }).to_string();

    // Handle numeric ratios/versions like 2:1 or 9001:2015
    res = RE_STANDARD_COLON.replace_all(&res, |caps: &FCaps| {
        let n1 = caps.get(1).unwrap().as_str();
        let n2 = caps.get(2).unwrap().as_str();
        let n1_val = n1.parse::<u64>().unwrap_or(0);

        // Heuristic: Use "trên" ONLY for pure integer-integer ratios where n1 is small.
        // Use a comma for EVERYTHING else (years, map scales 1:50.000, odds 1:2.5, etc.)
        if n1_val < 1000 && !n2.contains('.') {
            format!(" {} trên {} ", n1, n2)
        } else {
            format!("{}, {}", n1, n2)
        }
    }).to_string();

    res = RE_COLON_SEMICOLON.replace_all(&res, ", ").into_owned();
    RE_CLEAN_OTHERS.replace_all(&res, " ").into_owned()
}
