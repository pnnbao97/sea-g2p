//! Thai number-to-words conversion.
//!
//! Two reading modes, mirroring the Vietnamese module:
//!
//!   - **cardinal** ([`n2w`]) for quantities — "1250" becomes
//!     "หนึ่งพันสองร้อยห้าสิบ";
//!   - **digit by digit** ([`n2w_single`]) for identifiers such as phone
//!     numbers and codes, where the figures carry no arithmetic meaning.
//!
//! Thai has three obligatory alternations that a naive digit-by-position
//! mapping gets wrong:
//!
//!   - the tens digit 1 is **สิบ**, not หนึ่งสิบ ("10" -> สิบ, "15" -> สิบห้า);
//!   - the tens digit 2 is **ยี่สิบ**, not สองสิบ ("20" -> ยี่สิบ);
//!   - a final 1 after any tens digit is **เอ็ด**, not หนึ่ง ("21" ->
//!     ยี่สิบเอ็ด, "101" -> หนึ่งร้อยเอ็ด).
//!
//! Above six digits Thai counts in **ล้าน** (millions) and stacks them:
//! 10^8 is สิบล้าน, 10^12 is ล้านล้าน. Grouping therefore runs in blocks of
//! six, not the Western three.

const DIGITS: [&str; 10] = [
    "ศูนย์", "หนึ่ง", "สอง", "สาม", "สี่", "ห้า", "หก", "เจ็ด", "แปด", "เก้า",
];
/// Place names within a six-digit block, from the units place upward.
const PLACES: [&str; 6] = ["", "สิบ", "ร้อย", "พัน", "หมื่น", "แสน"];

pub fn digit_word(d: char) -> &'static str {
    match d {
        '0'..='9' => DIGITS[d as usize - '0' as usize],
        _ => "",
    }
}

/// Read each figure separately: "0812" -> "ศูนย์ แปด หนึ่ง สอง".
pub fn n2w_single(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_ascii_digit())
        .map(|c| digit_word(c).to_string())
        .collect::<Vec<_>>()
        .join(" ")
}

/// Read one block of at most six digits as a cardinal.
fn block_to_words(block: &str) -> String {
    let digits: Vec<char> = block.chars().collect();
    let n = digits.len();
    let mut out = String::new();
    for (i, d) in digits.iter().enumerate() {
        if *d == '0' {
            continue;
        }
        let place = n - 1 - i; // 0 = units, 1 = tens, ...
        let is_last = place == 0;
        let word = match (place, *d) {
            // tens: 1 is bare สิบ, 2 is ยี่สิบ
            (1, '1') => "สิบ".to_string(),
            (1, '2') => "ยี่สิบ".to_string(),
            (1, d) => format!("{}สิบ", digit_word(d)),
            // units: 1 becomes เอ็ด when anything precedes it
            (0, '1') if n > 1 && digits[..i].iter().any(|c| *c != '0') => "เอ็ด".to_string(),
            (0, d) => digit_word(d).to_string(),
            (p, d) => format!("{}{}", digit_word(d), PLACES[p]),
        };
        let _ = is_last;
        out.push_str(&word);
    }
    out
}

/// Cardinal reading of a digit string: "1250" -> "หนึ่งพันสองร้อยห้าสิบ".
///
/// Blocks of six are joined with ล้าน, so 10^8 reads สิบล้าน and 10^12
/// ล้านล้าน — the stacking Thai actually uses.
pub fn n2w(s: &str) -> String {
    let digits: String = s.chars().filter(|c| c.is_ascii_digit()).collect();
    let digits = digits.trim_start_matches('0');
    if digits.is_empty() {
        return DIGITS[0].to_string();
    }
    let chars: Vec<char> = digits.chars().collect();
    // split into six-digit blocks, most significant first
    let mut blocks: Vec<String> = Vec::new();
    let mut i = chars.len();
    while i > 0 {
        let start = i.saturating_sub(6);
        blocks.push(chars[start..i].iter().collect());
        i = start;
    }
    blocks.reverse();
    let mut out = String::new();
    for (idx, b) in blocks.iter().enumerate() {
        let words = block_to_words(b);
        if !words.is_empty() {
            out.push_str(&words);
        }
        let remaining = blocks.len() - 1 - idx;
        for _ in 0..remaining.min(1) {
            out.push_str("ล้าน");
        }
        // stacked ล้าน for very large magnitudes (10^12 and beyond)
        if remaining > 1 {
            for _ in 1..remaining {
                out.push_str("ล้าน");
            }
        }
    }
    out
}

/// Decimal reading: the integer part as a cardinal, then จุด, then the
/// fractional digits one by one — "3.14" -> "สามจุดหนึ่งสี่".
pub fn n2w_decimal(int_part: &str, frac_part: &str) -> String {
    let mut out = n2w(int_part);
    out.push_str("จุด");
    for c in frac_part.chars().filter(|c| c.is_ascii_digit()) {
        out.push_str(digit_word(c));
    }
    out
}
