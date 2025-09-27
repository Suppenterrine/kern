use crate::core::utils::{char_to_value_pythagorean, normalize_char};

use super::Cipher;

#[derive(Debug, Clone)]
pub struct ReversePythagoreanCipher;

impl Cipher for ReversePythagoreanCipher {
    fn name(&self) -> &str {
        "reverse_pythagorean"
    }

    fn char_to_value(&self, ch: char) -> u32 {
        match normalize_char(ch) {
            Some(c) if c.is_ascii_digit() => char_to_value_pythagorean(c),
            Some(c @ 'A'..='Z') => {
                let offset = ('Z' as u32) - (c as u32);
                (offset % 9) + 1
            }
            _ => 0,
        }
    }
}
