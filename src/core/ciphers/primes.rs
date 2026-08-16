use crate::core::ciphers::Cipher;

/// Primes cipher: map A..Z to the sequence of prime numbers starting at 2.
/// A -> 2, B -> 3, C -> 5, D -> 7, ...
pub struct PrimesCipher;

fn nth_primes() -> [u32; 26] {
    // first 26 primes
    [
        2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71, 73, 79, 83, 89,
        97, 101,
    ]
}

impl Cipher for PrimesCipher {
    fn name(&self) -> &str {
        "primes"
    }

    fn char_to_value(&self, ch: char) -> u32 {
        if ch.is_ascii_digit() {
            return ch.to_digit(10).unwrap_or(0);
        }

        let c = ch.to_ascii_uppercase();
        if c.is_ascii_uppercase() {
            let idx = (c as u8 - b'A') as usize;
            nth_primes()[idx]
        } else {
            0
        }
    }
}
