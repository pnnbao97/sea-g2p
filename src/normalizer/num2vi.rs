use once_cell::sync::Lazy;
use std::collections::HashMap;

pub const UNITS: [&str; 10] = ["không", "một", "hai", "ba", "bốn", "năm", "sáu", "bảy", "tám", "chín"];

pub fn pre_process_n2w(number: &str) -> Option<String> {
    let clean: String = number.chars().filter(|c| c.is_ascii_digit()).collect();
    if clean.is_empty() {
        None
    } else {
        Some(clean)
    }
}

pub fn process_n2w_single(numbers: &str, buffer: &mut String) {
    let mut first = true;
    for c in numbers.chars() {
        if let Some(digit) = c.to_digit(10) {
            if !first {
                buffer.push(' ');
            }
            buffer.push_str(UNITS[digit as usize]);
            first = false;
        }
    }
}

pub fn n2w_hundreds(numbers: &str, buffer: &mut String, is_group: bool) {
    if numbers.is_empty() || numbers == "000" {
        return;
    }

    let n = format!("{:0>3}", numbers);
    let h_digit = n.chars().nth(0).unwrap().to_digit(10).unwrap() as usize;
    let t_digit = n.chars().nth(1).unwrap().to_digit(10).unwrap() as usize;
    let u_digit = n.chars().nth(2).unwrap().to_digit(10).unwrap() as usize;

    let mut added = false;

    // Hundreds
    if h_digit != 0 {
        buffer.push_str(UNITS[h_digit]);
        buffer.push_str(" trăm");
        added = true;
    } else if is_group {
        buffer.push_str("không trăm");
        added = true;
    }

    // Tens
    if t_digit == 0 {
        if u_digit != 0 && (h_digit != 0 || is_group) {
            if added { buffer.push(' '); }
            buffer.push_str("lẻ");
            added = true;
        }
    } else if t_digit == 1 {
        if added { buffer.push(' '); }
        buffer.push_str("mười");
        added = true;
    } else {
        if added { buffer.push(' '); }
        buffer.push_str(UNITS[t_digit]);
        buffer.push_str(" mươi");
        added = true;
    }

    // Units
    if u_digit != 0 {
        if added { buffer.push(' '); }
        if u_digit == 1 && t_digit > 1 {
            buffer.push_str("mốt");
        } else if u_digit == 5 && t_digit != 0 {
            buffer.push_str("lăm");
        } else {
            buffer.push_str(UNITS[u_digit]);
        }
    }
}

pub fn n2w_large_number(numbers: &str, buffer: &mut String) {
    let numbers = numbers.trim_start_matches('0');
    if numbers.is_empty() {
        buffer.push_str(UNITS[0]);
        return;
    }

    let n_len = numbers.len();
    let mut groups = Vec::new();
    let mut i = n_len as i32;
    while i > 0 {
        let start = std::cmp::max(0, i - 3) as usize;
        let end = i as usize;
        groups.push(&numbers[start..end]);
        i -= 3;
    }

    let suffixes = ["", " nghìn", " triệu", " tỷ"];
    let mut parts = Vec::new();

    for (idx, &group) in groups.iter().enumerate() {
        if group == "000" {
            continue;
        }

        let mut group_buf = String::new();
        // is_group should be true if it's not the most significant group
        n2w_hundreds(group, &mut group_buf, idx < groups.len() - 1);

        if !group_buf.is_empty() {
            let suffix_idx = idx % 3;
            let ty_count = idx / 3;

            let mut full_suffix = String::new();
            if suffix_idx < suffixes.len() {
                full_suffix.push_str(suffixes[suffix_idx]);
            }
            for _ in 0..ty_count {
                full_suffix.push_str(" tỷ");
            }

            group_buf.push_str(&full_suffix);
            parts.push(group_buf);
        }
    }

    if parts.is_empty() {
        buffer.push_str(UNITS[0]);
        return;
    }

    parts.reverse();
    for (i, p) in parts.iter().enumerate() {
        if i > 0 {
            buffer.push(' ');
        }
        buffer.push_str(p);
    }
}

pub fn n2w(number: &str) -> String {
    let mut buffer = String::new();
    let clean_opt = pre_process_n2w(number);
    if clean_opt.is_none() {
        return number.to_string();
    }
    let clean_number = clean_opt.unwrap();

    if clean_number.len() == 2 && clean_number.starts_with('0') {
        buffer.push_str("không ");
        let digit = clean_number.chars().nth(1).unwrap().to_digit(10).unwrap() as usize;
        buffer.push_str(UNITS[digit]);
        return buffer;
    }

    n2w_large_number(&clean_number, &mut buffer);
    buffer.trim().to_string()
}

pub fn n2w_single(number: &str) -> String {
    let mut input = number;
    let mut owned_input;
    if input.starts_with("+84") {
        owned_input = "0".to_string();
        owned_input.push_str(&input[3..]);
        input = &owned_input;
    }

    let clean_opt = pre_process_n2w(input);
    if clean_opt.is_none() {
        return number.to_string();
    }
    let clean_number = clean_opt.unwrap();

    let mut buffer = String::new();
    process_n2w_single(&clean_number, &mut buffer);
    buffer
}
