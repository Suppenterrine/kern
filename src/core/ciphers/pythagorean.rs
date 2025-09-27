use crate::core::utils::char_to_value_pythagorean;

use super::Cipher;

#[derive(Debug, Clone)]
pub struct PythagoreanCipher;

impl Cipher for PythagoreanCipher {
    fn name(&self) -> &str {
        "pythagorean"
    }

    fn char_to_value(&self, ch: char) -> u32 {
        char_to_value_pythagorean(ch)
    }
}
