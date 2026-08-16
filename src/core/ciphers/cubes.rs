use crate::core::ciphers::Cipher;

/// Cubes cipher: letter position cubed. A=1->1, B=2->8, C=3->27, ...
pub struct CubesCipher;

impl Cipher for CubesCipher {
    fn name(&self) -> &str {
        "cubes"
    }

    fn char_to_value(&self, ch: char) -> u32 {
        if ch.is_ascii_digit() {
            return ch.to_digit(10).unwrap_or(0);
        }
        let c = ch.to_ascii_uppercase();
        if c.is_ascii_uppercase() {
            let pos = (c as u32) - ('A' as u32) + 1;
            pos * pos * pos
        } else {
            0
        }
    }
}
