use crate::core::utils::{char_to_value_ordinal, normalize_char};

use super::Cipher;

#[derive(Debug, Clone)]
pub struct ReverseOrdinalCipher;

impl Cipher for ReverseOrdinalCipher {
    fn name(&self) -> &str {
        "reverse_ordinal"
    }

    fn char_to_value(&self, ch: char) -> u32 {
        match normalize_char(ch) {
            Some(c) if c.is_ascii_digit() => char_to_value_ordinal(c),
            Some(c @ 'A'..='Z') => ('Z' as u32) - (c as u32) + 1,
            _ => 0,
        }
    }
}
