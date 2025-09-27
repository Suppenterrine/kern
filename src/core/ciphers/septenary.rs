use crate::core::ciphers::Cipher;

/// Septenary cipher: cyclic mapping into 1..7 for letters; digits interpreted as base-10 digits.
pub struct SeptenaryCipher;

impl Cipher for SeptenaryCipher {
    fn name(&self) -> &str {
        "septenary"
    }

    fn char_to_value(&self, ch: char) -> u32 {
        if ch.is_ascii_digit() {
            return ch.to_digit(10).unwrap_or(0);
        }
        let c = ch.to_ascii_uppercase();
        if ('A'..='Z').contains(&c) {
            let pos = (c as u32) - ('A' as u32); // 0..25
            // map cyclically to 1..7
            (pos % 7) + 1
        } else {
            0
        }
    }
}
