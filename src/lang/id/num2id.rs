//! Indonesian number-to-words.
//!
//! Two obligatory irregularities a digit-by-position mapping gets wrong:
//!
//!   - **11-19 use belas**: 11 is *sebelas*, 12 *dua belas*, not "satu satu";
//!   - **a leading 1 is se-, not satu**: 100 is *seratus*, 1000 *seribu*,
//!     10 *sepuluh* — but 200 is *dua ratus*, so the contraction only
//!     applies to one.
//!
//! Above that the system is regular and groups in thousands: ribu, juta,
//! miliar, triliun.

const UNITS: [&str; 10] = [
    "nol", "satu", "dua", "tiga", "empat", "lima", "enam", "tujuh",
    "delapan", "sembilan",
];
const SCALES: [&str; 5] = ["", "ribu", "juta", "miliar", "triliun"];

pub fn digit_word(d: char) -> &'static str {
    match d {
        '0'..='9' => UNITS[d as usize - '0' as usize],
        _ => "",
    }
}

/// Read each figure separately, for identifiers rather than quantities.
pub fn n2w_single(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_ascii_digit())
        .map(|c| digit_word(c).to_string())
        .collect::<Vec<_>>()
        .join(" ")
}

/// Read a group of at most three digits.
fn triple(n: u32) -> String {
    let mut parts: Vec<String> = Vec::new();
    let (h, rest) = (n / 100, n % 100);
    if h == 1 {
        parts.push("seratus".into()); // not "satu ratus"
    } else if h > 1 {
        parts.push(format!("{} ratus", UNITS[h as usize]));
    }
    let (t, u) = (rest / 10, rest % 10);
    if t == 1 {
        // the teens are formed with belas, and 11 contracts to sebelas
        parts.push(match u {
            0 => "sepuluh".into(),
            1 => "sebelas".into(),
            _ => format!("{} belas", UNITS[u as usize]),
        });
    } else {
        if t > 1 {
            parts.push(format!("{} puluh", UNITS[t as usize]));
        }
        if u > 0 {
            parts.push(UNITS[u as usize].into());
        }
    }
    parts.join(" ")
}

/// Cardinal reading: "1250" -> "seribu dua ratus lima puluh".
pub fn n2w(s: &str) -> String {
    let digits: String = s.chars().filter(|c| c.is_ascii_digit()).collect();
    let trimmed = digits.trim_start_matches('0');
    if trimmed.is_empty() {
        return UNITS[0].into();
    }
    let n: u64 = match trimmed.parse() {
        Ok(v) => v,
        Err(_) => return n2w_single(trimmed), // too large to be a quantity
    };
    let mut groups: Vec<u32> = Vec::new();
    let mut rest = n;
    while rest > 0 {
        groups.push((rest % 1000) as u32);
        rest /= 1000;
    }
    let mut parts: Vec<String> = Vec::new();
    for (i, g) in groups.iter().enumerate().rev() {
        if *g == 0 {
            continue;
        }
        let scale = SCALES.get(i).copied().unwrap_or("");
        if i == 1 && *g == 1 {
            parts.push("seribu".into()); // not "satu ribu"
        } else if scale.is_empty() {
            parts.push(triple(*g));
        } else {
            parts.push(format!("{} {}", triple(*g), scale));
        }
    }
    parts.join(" ")
}

/// "3.14" -> "tiga koma satu empat": the fractional digits are read one by
/// one, as they are in Vietnamese and Thai.
pub fn n2w_decimal(int_part: &str, frac_part: &str) -> String {
    let mut out = n2w(int_part);
    out.push_str(" koma");
    for c in frac_part.chars().filter(|c| c.is_ascii_digit()) {
        out.push(' ');
        out.push_str(digit_word(c));
    }
    out
}
