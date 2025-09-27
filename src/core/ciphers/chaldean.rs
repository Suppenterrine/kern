use crate::core::ciphers::Cipher;

/// Chaldean numerology mapping (common simplified variant)
pub struct ChaldeanCipher;

impl Cipher for ChaldeanCipher {
    fn name(&self) -> &str {
        "chaldean"
    }

    fn char_to_value(&self, ch: char) -> u32 {
        // Simplified Chaldean map for letters; digits are returned as their numeric value
        match ch.to_ascii_lowercase() {
            'a' | 'i' | 'j' | 'q' | 'y' => 1,
            'b' | 'k' | 'r' => 2,
            'c' | 'g' | 'l' => 3,
            'd' | 'm' | 't' => 4,
            'e' | 'h' | 'n' | 'x' => 5,
            'u' | 'v' | 'w' => 6,
            'o' | 'z' => 7,
            'f' | 'p' => 8,
            ch if ch.is_ascii_digit() => ch.to_digit(10).unwrap_or(0),
            _ => 0,
        }
    }
}
