use crate::cipher::Cipher;
use crate::reduction::{
    to_ordinal, to_reverse_ordinal, to_pythagorean, to_reverse_pythagorean,
    reduce_values,
};

/// Ordinal cipher (A=1..Z=26).
pub struct Ordinal;
impl Cipher for Ordinal {
    fn name(&self) -> &'static str { "Ordinal" }
    fn calculate(&self, input: &str) -> u32 {
        let values = to_ordinal(input);
        reduce_values(&values)
    }
}

/// Reverse ordinal cipher (A=26..Z=1).
pub struct ReverseOrdinal;
impl Cipher for ReverseOrdinal {
    fn name(&self) -> &'static str { "Reverse Ordinal" }
    fn calculate(&self, input: &str) -> u32 {
        let values = to_reverse_ordinal(input);
        reduce_values(&values)
    }
}

/// Pythagorean cipher (full reduction).
pub struct Pythagorean;
impl Cipher for Pythagorean {
    fn name(&self) -> &'static str { "Pythagorean" }
    fn calculate(&self, input: &str) -> u32 {
        let values = to_pythagorean(input);
        reduce_values(&values)
    }
}

/// Reverse Pythagorean cipher.
pub struct ReversePythagorean;
impl Cipher for ReversePythagorean {
    fn name(&self) -> &'static str { "Reverse Pythagorean" }
    fn calculate(&self, input: &str) -> u32 {
        let values = to_reverse_pythagorean(input);
        reduce_values(&values)
    }
}
