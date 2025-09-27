use crate::core::ciphers::Cipher;

/// Squares cipher: letter position squared. A=1->1, B=2->4, C=3->9, ...
pub struct SquaresCipher;

impl Cipher for SquaresCipher {
    fn name(&self) -> &str {
        "squares"
    }

    fn char_to_value(&self, ch: char) -> u32 {
        if ch.is_ascii_digit() {
            return ch.to_digit(10).unwrap_or(0);
        }
        let c = ch.to_ascii_uppercase();
        if ('A'..='Z').contains(&c) {
            let pos = (c as u32) - ('A' as u32) + 1;
            pos * pos
        } else {
            0
        }
    }
}
