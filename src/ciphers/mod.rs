use crate::cipher::Cipher;

mod simple;
mod advanced;

pub use self::simple::*;
pub use self::advanced::*;

/// Registry of all available ciphers. More can be added later without
/// changing the public API.
pub static ALL_CIPHERS: &[&dyn Cipher] = &[
    &simple::Ordinal,
    // &simple::ReverseOrdinal,
    // &simple::Pythagorean,
    // &simple::ReversePythagorean,
    // &advanced::Primes,
    // &advanced::Fibonacci,
    // &advanced::Chaldean,
    // &advanced::Agrippa,
];

/// Calculate all registered ciphers for the given input.
pub fn calculate_all(input: &str) -> Vec<(&'static str, u32)> {
    ALL_CIPHERS
        .iter()
        .map(|c| (c.name(), c.calculate(input)))
        .collect()
}
