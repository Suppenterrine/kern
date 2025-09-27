pub fn normalize_char(ch: char) -> Option<char> {
    if ch.is_ascii_digit() {
        Some(ch)
    } else if ch.is_ascii_alphabetic() {
        Some(ch.to_ascii_uppercase())
    } else {
        None
    }
}

pub fn char_to_value_ordinal(ch: char) -> u32 {
    match normalize_char(ch) {
        Some(c) if c.is_ascii_digit() => (c as u32) - ('0' as u32),
        Some(c @ 'A'..='Z') => (c as u32) - ('A' as u32) + 1,
        _ => 0,
    }
}

pub fn char_to_value_pythagorean(ch: char) -> u32 {
    match normalize_char(ch) {
        Some(c) if c.is_ascii_digit() => (c as u32) - ('0' as u32),
        Some(c @ 'A'..='Z') => {
            let offset = (c as u32) - ('A' as u32);
            (offset % 9) + 1
        }
        _ => 0,
    }
}
