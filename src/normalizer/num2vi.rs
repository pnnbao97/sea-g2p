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

    let n = format!("{:0>3}", numbers);
    let chars: Vec<char> = n.chars().collect();
    let h_digit = chars[0];
    let t_digit = chars[1];
    let u_digit = chars[2];

    let mut res = Vec::new();

    // Hundreds
    if h_digit != '0' {
        res.push(format!("{} trăm", units(h_digit)));
    } else if numbers.len() == 3 {
        res.push("không trăm".to_string());
    }

    // Tens
    if t_digit == '0' {
        if u_digit != '0' && (h_digit != '0' || numbers.len() == 3) {
            res.push("lẻ".to_string());
        }
    } else if t_digit == '1' {
        res.push("mười".to_string());
    } else {
        res.push(format!("{} mươi", units(t_digit)));
    }

    // Units
    if u_digit != '0' {
        if u_digit == '1' && t_digit != '0' && t_digit != '1' {
            res.push("mốt".to_string());
        } else if u_digit == '5' && t_digit != '0' {
            res.push("lăm".to_string());
        } else {
            res.push(units(u_digit).to_string());
        }
    }

    res.join(" ")
}

pub fn n2w_large_number(numbers: &str) -> String {
    let numbers = numbers.trim_start_matches('0');
    if numbers.is_empty() {
        return units('0').to_string();
    }

    let mut groups = Vec::new();
    let n_len = numbers.len();
    let mut i = n_len as i32;
    while i > 0 {
        let start = std::cmp::max(0, i - 3) as usize;
        groups.push(&numbers[start..i as usize]);
        i -= 3;
    }

    let suffixes = ["", " nghìn", " triệu", " tỷ"];
    let mut parts = Vec::new();

    for (i, group) in groups.iter().enumerate() {
        if *group == "000" {
            continue;
        }

        let word = n2w_hundreds(group);
        if !word.is_empty() {
            let suffix_idx = i % 3;
            let main_suffix = if suffix_idx < suffixes.len() { suffixes[suffix_idx] } else { "" };
            let ty_count = i / 3;

            let mut word_with_suffix = format!("{}{}", word, main_suffix);
            for _ in 0..ty_count {
                word_with_suffix.push_str(" tỷ");
            }
            parts.push(word_with_suffix);
        }
    }

    if parts.is_empty() {
        return units('0').to_string();
    }

    parts.reverse();
    parts.join(" ").trim().to_string()
}

pub fn n2w(number: &str) -> String {
    let clean_number: String = number.chars().filter(|c: &char| c.is_ascii_digit()).collect();
    if clean_number.is_empty() {
        return number.to_string();
    }

    if clean_number.len() == 2 && clean_number.starts_with('0') {
        return format!("không {}", units(clean_number.chars().nth(1).unwrap()));
    }

    n2w_large_number(&clean_number)
}

pub fn n2w_single(number: &str) -> String {
    let mut num_str = number.to_string();
    if num_str.starts_with("+84") {
        num_str = format!("0{}", &num_str[3..]);
    }

    let res: Vec<String> = num_str.chars()
        .filter(|c: &char| c.is_ascii_digit())
        .map(|c: char| units(c).to_string())
        .collect();

    if res.is_empty() {
        return number.to_string();
    }
    res.join(" ")
}

pub fn n2w_decimal(number: &str) -> String {
    let clean_number: String = number.chars().filter(|c: &char| c.is_ascii_digit()).collect();
    if clean_number.is_empty() {
        return number.to_string();
    }

    let mut res = Vec::new();
    let chars: Vec<char> = clean_number.chars().collect();
    for (i, &d) in chars.iter().enumerate() {
        if d == '5' && i == chars.len() - 1 && i > 0 && chars[i-1] != '0' {
            res.push("lăm".to_string());
        } else {
            let u = units(d);
            if !u.is_empty() {
                res.push(u.to_string());
            }
        }
    }
    res.join(" ")
}
