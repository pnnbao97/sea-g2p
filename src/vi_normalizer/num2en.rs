// Đọc số/ký hiệu kiểu TIẾNG ANH cho câu thuần Anh (en_ctx) và cụm "x2y"/"x4y".
//
// Khi câu không có chữ tiếng Việt có dấu, normalizer coi là câu tiếng Anh:
// số, %, $, đơn vị, giờ phút... đổi thành chữ Anh ("3" -> "three", "." -> "dot")
// TRƯỚC các pass tiếng Việt — các pass sau thành no-op vì không còn chữ số.

use regex::{Regex, Captures};
use once_cell::sync::Lazy;

const ONES: [&'static str; 20] = [
    "zero", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine",
    "ten", "eleven", "twelve", "thirteen", "fourteen", "fifteen", "sixteen",
    "seventeen", "eighteen", "nineteen",
];
const TENS: [&'static str; 10] = [
    "", "", "twenty", "thirty", "forty", "fifty", "sixty", "seventy", "eighty", "ninety",
];

fn two_digits(n: u64) -> String {
    if n < 20 {
        ONES[n as usize].to_string()
    } else if n % 10 == 0 {
        TENS[(n / 10) as usize].to_string()
    } else {
        format!("{} {}", TENS[(n / 10) as usize], ONES[(n % 10) as usize])
    }
}

fn three_digits(n: u64) -> String {
    if n < 100 {
        two_digits(n)
    } else if n % 100 == 0 {
        format!("{} hundred", ONES[(n / 100) as usize])
    } else {
        format!("{} hundred {}", ONES[(n / 100) as usize], two_digits(n % 100))
    }
}

pub fn n2w_en_int(mut n: u64) -> String {
    if n == 0 { return "zero".to_string(); }
    let scales: [(u64, &str); 4] = [
        (1_000_000_000_000, "trillion"),
        (1_000_000_000, "billion"),
        (1_000_000, "million"),
        (1_000, "thousand"),
    ];
    let mut parts: Vec<String> = Vec::new();
    for (v, name) in scales {
        if n >= v {
            parts.push(format!("{} {}", three_digits(n / v), name));
            n %= v;
        }
    }
    if n > 0 { parts.push(three_digits(n)); }
    parts.join(" ")
}

/// Đọc từng chữ số: "8080" -> "eight zero eight zero" (IP, port, số dài).
pub fn n2w_en_digits(s: &str) -> String {
    s.chars()
        .filter(|c: &char| c.is_ascii_digit())
        .map(|c: char| ONES[c.to_digit(10).unwrap() as usize])
        .collect::<Vec<&str>>()
        .join(" ")
}

/// Đọc số kiểu Anh: "123" -> "one hundred twenty three", "2.5" -> "two point five",
/// "1,234" -> "one thousand two hundred thirty four". Số 0 đầu -> đọc từng chữ số.
pub fn n2w_en(s: &str) -> String {
    let clean = s.replace(',', "");
    if let Some(dot) = clean.find('.') {
        let int_part = &clean[..dot];
        let frac = &clean[dot + 1..];
        let int_words = if int_part.is_empty() { "zero".to_string() } else { n2w_en(int_part) };
        if frac.is_empty() { return int_words; }
        return format!("{} point {}", int_words, n2w_en_digits(frac));
    }
    if clean.len() > 1 && clean.starts_with('0') {
        return n2w_en_digits(&clean);
    }
    match clean.parse::<u64>() {
        Ok(n) if clean.len() <= 15 => n2w_en_int(n),
        _ => n2w_en_digits(&clean),
    }
}

// "text2text" / "sale4u": số 2/4 kẹp giữa chữ thường là viết tắt to/for tiếng Anh.
// Chỉ nhận đúng MỘT chữ số 2 hoặc 4 để không đụng chuỗi mã ("abc123xyz").
static RE_SANDWICH_24: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b([a-z]+)([24])([a-z]+)\b").unwrap()
});

pub fn expand_sandwich_digits(text: &str) -> String {
    if !text.chars().any(|c: char| c == '2' || c == '4') { return text.to_string(); }
    RE_SANDWICH_24.replace_all(text, |caps: &Captures| {
        // Tên hàm toán + số + biến ("cos2x", "sin2a", "log2n") KHÔNG phải kiểu
        // "text2text" -> giữ nguyên cho nhánh công thức đọc "cos hai ích".
        let left = caps.get(1).unwrap().as_str();
        if matches!(left, "sin" | "cos" | "tan" | "cot" | "log" | "lg" | "ln" | "lim") {
            return caps.get(0).unwrap().as_str().to_string();
        }
        let d = if caps.get(2).unwrap().as_str() == "2" { "two" } else { "four" };
        // Vế 1 chữ cái ("b2b") bọc <en> để không bị pass chữ-cái-đơn đọc tên
        // chữ Việt ("bê two bê"); vế nhiều chữ để trần cho G2P tra dict.
        let wrap = |s: &str| -> String {
            if s.chars().count() == 1 {
                format!("__start_en__{}__end_en__", s)
            } else {
                s.to_string()
            }
        };
        format!("{} {} {}", wrap(caps.get(1).unwrap().as_str()), d, wrap(caps.get(3).unwrap().as_str()))
    }).into_owned()
}

// ── Các pass cho câu thuần Anh ──────────────────────────────────────────────
static RE_EN_TIME: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b(\d{1,2}):([0-5]\d)\b").unwrap()
});
static RE_EN_PERCENT: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(\d+(?:\.\d+)?)\s*%").unwrap()
});
static RE_EN_DOLLAR: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\$\s*(\d+(?:,\d{3})*(?:\.\d+)?)").unwrap()
});
static RE_EN_UNIT: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\b(\d+(?:\.\d+)?)\s*(tb|gb|mb|kb|ghz|mhz|khz|hz|kg|km|cm|mm|ml|mah|dpi|fps|mph)\b").unwrap()
});
static RE_EN_SLASH_NUM: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b(\d{1,4})/(\d{1,4})\b").unwrap()
});
// Token trộn chữ-số còn lại (4K, 23H2, 1080p): số đọc kiểu Anh, chữ giữ nguyên.
// regex crate không có lookahead -> match rộng rồi lọc trong closure.
static RE_EN_ALNUM: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b[A-Za-z0-9]+\b").unwrap()
});
static RE_EN_SUB_TOKENS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"[A-Za-z]+|\d+").unwrap()
});
static RE_EN_NUMBER: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\d+(?:,\d{3})*(?:\.\d+)?").unwrap()
});

fn en_unit_name(unit: &str, singular: bool) -> String {
    let base = match unit {
        "tb" => "terabyte", "gb" => "gigabyte", "mb" => "megabyte", "kb" => "kilobyte",
        "ghz" => "gigahertz", "mhz" => "megahertz", "khz" => "kilohertz", "hz" => "hertz",
        "kg" => "kilogram", "km" => "kilometer", "cm" => "centimeter", "mm" => "millimeter",
        "ml" => "milliliter", "mah" => "milliamp hour", "dpi" => "d p i", "fps" => "f p s",
        "mph" => "mile per hour",
        _ => unit,
    };
    // Đơn vị dạng đánh vần / đã có "per" thì không thêm "s".
    let no_plural = matches!(unit, "hz" | "ghz" | "mhz" | "khz" | "dpi" | "fps" | "mph");
    if singular || no_plural { base.to_string() } else { format!("{}s", base) }
}

/// Chuyển số/ký hiệu trong câu THUẦN ANH thành chữ Anh. Chạy sau khi URL/email
/// đã được mask, trước mọi pass tiếng Việt.
pub fn english_prenormalize(text: &str) -> String {
    let mut t = text.to_string();

    // Giờ phút: 10:30 -> "ten thirty", 10:05 -> "ten oh five", 10:00 -> "ten o'clock".
    t = RE_EN_TIME.replace_all(&t, |caps: &Captures| {
        let h: u64 = caps.get(1).unwrap().as_str().parse().unwrap_or(0);
        let m: u64 = caps.get(2).unwrap().as_str().parse().unwrap_or(0);
        if m == 0 {
            format!("{} o'clock", n2w_en_int(h))
        } else if m < 10 {
            format!("{} oh {}", n2w_en_int(h), n2w_en_int(m))
        } else {
            format!("{} {}", n2w_en_int(h), n2w_en_int(m))
        }
    }).into_owned();

    t = RE_EN_PERCENT.replace_all(&t, |caps: &Captures| {
        format!("{} percent", n2w_en(caps.get(1).unwrap().as_str()))
    }).into_owned();

    t = RE_EN_DOLLAR.replace_all(&t, |caps: &Captures| {
        let num = caps.get(1).unwrap().as_str();
        let unit = if num == "1" { "dollar" } else { "dollars" };
        format!("{} {}", n2w_en(num), unit)
    }).into_owned();

    t = RE_EN_UNIT.replace_all(&t, |caps: &Captures| {
        let num = caps.get(1).unwrap().as_str();
        let unit = caps.get(2).unwrap().as_str().to_lowercase();
        format!("{} {}", n2w_en(num), en_unit_name(&unit, num == "1"))
    }).into_owned();

    t = RE_EN_SLASH_NUM.replace_all(&t, |caps: &Captures| {
        format!("{} slash {}", n2w_en(caps.get(1).unwrap().as_str()), n2w_en(caps.get(2).unwrap().as_str()))
    }).into_owned();

    t = RE_EN_ALNUM.replace_all(&t, |caps: &Captures| {
        let tok = caps.get(0).unwrap().as_str();
        let has_digit = tok.chars().any(|c: char| c.is_ascii_digit());
        let has_letter = tok.chars().any(|c: char| c.is_ascii_alphabetic());
        if !has_digit || !has_letter { return tok.to_string(); }
        RE_EN_SUB_TOKENS.find_iter(tok).map(|m: regex::Match| {
            let s = m.as_str();
            if s.chars().all(|c: char| c.is_ascii_digit()) { n2w_en(s) } else { s.to_string() }
        }).collect::<Vec<String>>().join(" ")
    }).into_owned();

    // "-5" -> "minus five", "+84" -> "plus eight four" (trước pass số).
    t = {
        static RE_EN_MINUS: Lazy<Regex> = Lazy::new(|| Regex::new(r"(^|[\s(])-(\d)").unwrap());
        RE_EN_MINUS.replace_all(&t, "${1}minus $2").into_owned()
    };
    t = {
        static RE_EN_PLUS: Lazy<Regex> = Lazy::new(|| Regex::new(r"\+\s*(\d)").unwrap());
        RE_EN_PLUS.replace_all(&t, "plus $1").into_owned()
    };

    t = RE_EN_NUMBER.replace_all(&t, |caps: &Captures| {
        n2w_en(caps.get(0).unwrap().as_str())
    }).into_owned();

    // Dấu chấm KẸP GIỮA chữ cái (TP.HCM) -> "dot". Chấm cuối câu có khoảng
    // trắng theo sau không bị đụng. Chạy lặp vì regex crate không có lookahead
    // ("A.B.C" cần 2 lượt cho các cặp xen kẽ).
    {
        static RE_EN_INNER_DOT: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r"([A-Za-z])\.([A-Za-z])").unwrap()
        });
        for _ in 0..3 {
            let next = RE_EN_INNER_DOT.replace_all(&t, "$1 dot $2").into_owned();
            if next == t { break; }
            t = next;
        }
    }

    // Ký hiệu rời phổ biến.
    t = t.replace(" & ", " and ").replace(" + ", " plus ").replace(" = ", " equals ");
    t
}
