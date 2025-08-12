/// Utilities for converting characters into numerical values.
///
/// Input is normalised to ASCII uppercase letters and any non A‑Z
/// characters are ignored.

fn normalize(input: &str) -> impl Iterator<Item = char> + '_ {
    input
        .chars()
        .filter(|c| c.is_ascii_alphabetic())
        .map(|c| c.to_ascii_uppercase())
}

/// Map the input string to ordinal values (A=1..Z=26).
pub fn to_ordinal(input: &str) -> Vec<u32> {
    normalize(input)
        .map(|c| (c as u8 - b'A' + 1) as u32)
        .collect()
}

/// Map the input string to reverse ordinal values (A=26..Z=1).
pub fn to_reverse_ordinal(input: &str) -> Vec<u32> {
    normalize(input)
        .map(|c| (26 - (c as u8 - b'A') as u32))
        .collect()
}

/// Map the input string using the Pythagorean (full reduction) scheme.
/// Values wrap every nine (A=1, B=2 .. I=9, J=1, ...).
pub fn to_pythagorean(input: &str) -> Vec<u32> {
    normalize(input)
        .map(|c| ((c as u8 - b'A') % 9 + 1) as u32)
        .collect()
}

/// Reverse variant of [to_pythagorean].
pub fn to_reverse_pythagorean(input: &str) -> Vec<u32> {
    normalize(input)
        .map(|c| ((25 - (c as u8 - b'A')) % 9 + 1) as u32)
        .collect()
}
