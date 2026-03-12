use crate::normalizer::vi_resources::VI_LETTER_NAMES_MAP;

pub fn n2w_units(numbers: &str) -> String {
    if numbers.is_empty() {
        return "".to_string();
    }
    VI_LETTER_NAMES_MAP.get(numbers).cloned().unwrap_or(numbers).to_string()
}

pub fn pre_process_n2w(number: &str) -> Option<String> {
    let clean: String = number.chars().filter(|c| !"-., ".contains(*c)).collect();
    if !clean.is_empty() && clean.chars().all(|c| c.is_ascii_digit()) {
        Some(clean)
    } else {
        None
    }
}

pub fn process_n2w_single(numbers: &str) -> String {
    numbers.chars()
        .filter_map(|c| VI_LETTER_NAMES_MAP.get(c.to_string().as_str()).cloned())
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn n2w_hundreds(numbers: &str) -> String {
    if numbers.is_empty() || numbers == "000" {
        return "".to_string();
    }

    let n = format!("{:0>3}", numbers);
    let h_digit = n.chars().nth(0).unwrap();
    let t_digit = n.chars().nth(1).unwrap();
    let u_digit = n.chars().nth(2).unwrap();

    let mut res = Vec::new();

    // Hundreds
    if h_digit != '0' {
        res.push(format!("{} trăm", VI_LETTER_NAMES_MAP.get(h_digit.to_string().as_str()).unwrap()));
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
        res.push(format!("{} mươi", VI_LETTER_NAMES_MAP.get(t_digit.to_string().as_str()).unwrap()));
    }

    // Units
    if u_digit != '0' {
        if u_digit == '1' && t_digit != '0' && t_digit != '1' {
            res.push("mốt".to_string());
        } else if u_digit == '5' && (t_digit != '0' || (h_digit != '0' || numbers.len() == 3)) {
            res.push("lăm".to_string());
        } else {
            res.push(VI_LETTER_NAMES_MAP.get(u_digit.to_string().as_str()).unwrap().to_string());
        }
    }

    res.join(" ")
}

pub fn n2w_large_number(numbers: &str) -> String {
    let numbers = numbers.trim_start_matches('0');
    if numbers.is_empty() {
        return VI_LETTER_NAMES_MAP.get("0").unwrap().to_string();
    }

    let n_len = numbers.len();
    let mut groups = Vec::new();
    let mut i = n_len as i32;
    while i > 0 {
        let start = std::cmp::max(0, i - 3) as usize;
        groups.push(&numbers[start..i as usize]);
        i -= 3;
    }

    let suffixes = ["", " nghìn", " triệu", " tỷ"];
    let mut parts = Vec::new();

    for (i, &group) in groups.iter().enumerate() {
        if group == "000" {
            continue;
        }

        let word = n2w_hundreds(group);
        if !word.is_empty() {
            let suffix_idx = i % 3;
            let main_suffix = if suffix_idx < suffixes.len() { suffixes[suffix_idx] } else { "" };
            let ty_count = i / 3;

            let mut full_suffix = main_suffix.to_string();
            for _ in 0..ty_count {
                full_suffix.push_str(" tỷ");
            }
            parts.push(format!("{}{}", word, full_suffix));
        }
    }

    if parts.is_empty() {
        return VI_LETTER_NAMES_MAP.get("0").unwrap().to_string();
    }

    parts.reverse();
    parts.join(" ").trim().to_string()
}

pub fn n2w(number: &str) -> String {
    if let Some(clean_number) = pre_process_n2w(number) {
        if clean_number.len() == 2 && clean_number.starts_with('0') {
            return format!("không {}", VI_LETTER_NAMES_MAP.get(&clean_number[1..2]).unwrap());
        }
        n2w_large_number(&clean_number)
    } else {
        number.to_string()
    }
}

pub fn n2w_single(number: &str) -> String {
    let mut number_str = number.to_string();
    if number_str.starts_with("+84") {
        number_str = format!("0{}", &number_str[3..]);
    }
    if let Some(clean_number) = pre_process_n2w(&number_str) {
        process_n2w_single(&clean_number)
    } else {
        number_str
    }
}
