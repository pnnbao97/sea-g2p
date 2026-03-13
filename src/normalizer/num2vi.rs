pub fn get_unit_word(digit: char) -> &'static str {
    match digit {
        '0' => "không", '1' => "một", '2' => "hai", '3' => "ba", '4' => "bốn",
        '5' => "năm", '6' => "sáu", '7' => "bảy", '8' => "tám", '9' => "chín",
        _ => "",
    }
}

pub fn pre_process_n2w(number: &str) -> Option<String> {
    let clean: String = number.chars()
        .filter(|c| !matches!(c, ' ' | '-' | '.' | ','))
        .collect();
    if !clean.is_empty() && clean.chars().all(|c| c.is_ascii_digit()) {
        Some(clean)
    } else {
        None
    }
}

pub fn n2w_single(number: &str) -> String {
    let mut input_owned;
    let input = if number.starts_with("+84") {
        input_owned = "0".to_string();
        input_owned.push_str(&number[3..]);
        &input_owned
    } else {
        number
    };

    let clean = match pre_process_n2w(input) {
        Some(c) => c,
        None => return number.to_string(),
    };

    let mut res = String::with_capacity(clean.len() * 6);
    for (i, c) in clean.chars().enumerate() {
        if i > 0 { res.push(' '); }
        res.push_str(get_unit_word(c));
    }
    res
}

fn push_with_space(buffer: &mut String, s: &str, first: &mut bool) {
    if !*first { buffer.push(' '); }
    buffer.push_str(s);
    *first = false;
}

pub fn n2w_hundreds(numbers: &str, buffer: &mut String, is_larger_number: bool) {
    if numbers.is_empty() || numbers == "000" {
        return;
    }

    let padded = format!("{:0>3}", numbers);
    let mut chars = padded.chars();
    let h_digit = chars.next().unwrap();
    let t_digit = chars.next().unwrap();
    let u_digit = chars.next().unwrap();

    let mut first = buffer.is_empty();

    // Hundreds
    if h_digit != '0' {
        push_with_space(buffer, get_unit_word(h_digit), &mut first);
        buffer.push_str(" trăm");
    } else if is_larger_number || numbers.len() == 3 {
        push_with_space(buffer, "không trăm", &mut first);
    }

    // Tens
    if t_digit == '0' {
        if u_digit != '0' && (!first || is_larger_number || numbers.len() == 3) {
            push_with_space(buffer, "lẻ", &mut first);
        }
    } else if t_digit == '1' {
        push_with_space(buffer, "mười", &mut first);
    } else {
        push_with_space(buffer, get_unit_word(t_digit), &mut first);
        buffer.push_str(" mươi");
    }

    // Units
    if u_digit != '0' {
        if u_digit == '1' && t_digit != '0' && t_digit != '1' {
            push_with_space(buffer, "mốt", &mut first);
        } else if u_digit == '5' && t_digit != '0' {
            push_with_space(buffer, "lăm", &mut first);
        } else {
            push_with_space(buffer, get_unit_word(u_digit), &mut first);
        }
    }
}

pub fn n2w_large_number(numbers: &str) -> String {
    if numbers.is_empty() || numbers.chars().all(|c| c == '0') {
        return "không".to_string();
    }

    let trimmed = numbers.trim_start_matches('0');
    if trimmed.is_empty() {
        return "không".to_string();
    }

    let n_len = trimmed.len();
    let mut groups = Vec::new();
    let mut end = n_len;
    while end > 0 {
        let start = if end > 3 { end - 3 } else { 0 };
        groups.push(&trimmed[start..end]);
        end = start;
    }

    let suffixes = ["", " nghìn", " triệu", " tỷ"];
    let mut result = String::with_capacity(n_len * 10);

    // Process groups from most significant to least significant
    let mut first_group = true;
    for (i, group) in groups.iter().enumerate().rev() {
        if *group == "000" {
            continue;
        }

        let mut group_buf = String::new();
        let is_middle = !first_group;
        n2w_hundreds(group, &mut group_buf, is_middle);

        if !group_buf.is_empty() {
            if !first_group { result.push(' '); }
            result.push_str(&group_buf);

            let suffix_idx = i % 3;
            let main_suffix = if suffix_idx < suffixes.len() { suffixes[suffix_idx] } else { "" };
            result.push_str(main_suffix);

            let ty_count = i / 3;
            for _ in 0..ty_count {
                result.push_str(" tỷ");
            }
            first_group = false;
        }
    }

    if result.is_empty() {
        return "không".to_string();
    }

    result.trim().to_string()
}

pub fn n2w(number: &str) -> String {
    let clean_number = match pre_process_n2w(number) {
        Some(c) => c,
        None => return number.to_string(),
    };

    if clean_number.len() == 2 && clean_number.starts_with('0') {
        let mut res = "không ".to_string();
        res.push_str(get_unit_word(clean_number.chars().nth(1).unwrap()));
        return res;
    }

    n2w_large_number(&clean_number)
}
