pub fn units(digit: char) -> &'static str {
    match digit {
        '0' => "không",
        '1' => "một",
        '2' => "hai",
        '3' => "ba",
        '4' => "bốn",
        '5' => "năm",
        '6' => "sáu",
        '7' => "bảy",
        '8' => "tám",
        '9' => "chín",
        _ => "",
    }
}

pub fn n2w_hundreds(numbers: &str) -> String {
    if numbers.is_empty() || numbers == "000" {
        return String::new();
    }

    let n_bytes = numbers.as_bytes();
    let (h_digit, t_digit, u_digit) = match n_bytes.len() {
        3 => (n_bytes[0] as char, n_bytes[1] as char, n_bytes[2] as char),
        2 => ('0', n_bytes[0] as char, n_bytes[1] as char),
        1 => ('0', '0', n_bytes[0] as char),
        _ => ('0', '0', '0'),
    };

    let mut res = String::with_capacity(32);

    // Hundreds
    if h_digit != '0' {
        res.push_str(units(h_digit));
        res.push_str(" trăm");
    } else if numbers.len() == 3 {
        res.push_str("không trăm");
    }

    // Tens
    if t_digit == '0' {
        if u_digit != '0' && (h_digit != '0' || numbers.len() == 3) {
            if !res.is_empty() { res.push(' '); }
            res.push_str("lẻ");
        }
    } else if t_digit == '1' {
        if !res.is_empty() { res.push(' '); }
        res.push_str("mười");
    } else {
        if !res.is_empty() { res.push(' '); }
        res.push_str(units(t_digit));
        res.push_str(" mươi");
    }

    // Units
    if u_digit != '0' {
        if u_digit == '1' && t_digit != '0' && t_digit != '1' {
            if !res.is_empty() { res.push(' '); }
            res.push_str("mốt");
        } else if u_digit == '5' && t_digit != '0' {
            if !res.is_empty() { res.push(' '); }
            res.push_str("lăm");
        } else {
            let u = units(u_digit);
            if !u.is_empty() {
                if !res.is_empty() { res.push(' '); }
                res.push_str(u);
            }
        }
    }

    res
}

pub fn n2w_large_number(numbers: &str) -> String {
    let numbers = numbers.trim_start_matches('0');
    if numbers.is_empty() {
        return units('0').to_string();
    }

    let n_len = numbers.len();
    let mut i = n_len as i32;
    let mut groups = Vec::with_capacity((n_len + 2) / 3);
    while i > 0 {
        let start = std::cmp::max(0, i - 3) as usize;
        groups.push(&numbers[start..i as usize]);
        i -= 3;
    }

    let suffixes = ["", " nghìn", " triệu", " tỷ"];
    let mut parts = Vec::with_capacity(groups.len());

    for (i, group) in groups.iter().enumerate() {
        if *group == "000" {
            continue;
        }

        let mut word = n2w_hundreds(group);
        if !word.is_empty() {
            let suffix_idx = i % 3;
            let main_suffix = if suffix_idx < suffixes.len() { suffixes[suffix_idx] } else { "" };
            let ty_count = i / 3;

            word.push_str(main_suffix);
            for _ in 0..ty_count {
                word.push_str(" tỷ");
            }
            parts.push(word);
        }
    }

    if parts.is_empty() {
        return units('0').to_string();
    }

    parts.reverse();
    parts.join(" ")
}

pub fn n2w(number: &str) -> String {
    if number.is_empty() { return String::new(); }

    let is_all_digits = number.chars().all(|c| c.is_ascii_digit());
    let clean_number: std::borrow::Cow<str> = if is_all_digits {
        std::borrow::Cow::Borrowed(number)
    } else {
        std::borrow::Cow::Owned(number.chars().filter(|c: &char| c.is_ascii_digit()).collect())
    };

    if clean_number.is_empty() {
        return number.to_string();
    }

    if clean_number.len() == 2 && clean_number.starts_with('0') {
        return format!("không {}", units(clean_number.chars().nth(1).unwrap()));
    }

    n2w_large_number(&clean_number)
}

pub fn n2w_single(number: &str) -> String {
    if number.is_empty() { return String::new(); }

    let mut num_str_owned;
    let num_str = if number.starts_with("+84") {
        num_str_owned = String::with_capacity(number.len() - 2);
        num_str_owned.push('0');
        num_str_owned.push_str(&number[3..]);
        &num_str_owned
    } else {
        number
    };

    let mut res = String::with_capacity(num_str.len() * 4);
    for c in num_str.chars() {
        if c.is_ascii_digit() {
            let u = units(c);
            if !u.is_empty() {
                if !res.is_empty() { res.push(' '); }
                res.push_str(u);
            }
        }
    }

    if res.is_empty() {
        return number.to_string();
    }
    res
}

pub fn n2w_decimal(number: &str) -> String {
    if number.is_empty() { return String::new(); }

    let mut res = String::with_capacity(number.len() * 4);
    let chars: Vec<char> = number.chars().filter(|c| c.is_ascii_digit()).collect();

    if chars.is_empty() {
        return number.to_string();
    }

    for (i, &d) in chars.iter().enumerate() {
        if !res.is_empty() { res.push(' '); }
        if d == '5' && i == chars.len() - 1 && i > 0 && chars[i-1] != '0' {
            res.push_str("lăm");
        } else {
            let u = units(d);
            if !u.is_empty() {
                res.push_str(u);
            }
        }
    }
    res
}
