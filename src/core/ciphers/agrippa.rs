use crate::core::ciphers::Cipher;

/// Agrippa (Cornelius Agrippa) - a historical Latin/Esoteric mapping.
/// We'll use a simplified mapping where A=1..Z=26 but with a small twist: map J=10 and U=20
pub struct AgrippaCipher;

impl Cipher for AgrippaCipher {
    fn name(&self) -> &str {
        "agrippa"
    }

    fn char_to_value(&self, ch: char) -> u32 {
        let c = ch.to_ascii_uppercase();
        if c.is_ascii_digit() {
            return c.to_digit(10).unwrap_or(0);
        }

        match c {
            'A'..='Z' => (c as u32) - ('A' as u32) + 1,
            _ => 0,
        }
    }
}
