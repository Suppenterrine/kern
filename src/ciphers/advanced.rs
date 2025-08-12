use crate::cipher::Cipher;

/// Placeholder for a prime number based cipher.
pub struct Primes;
impl Cipher for Primes {
    fn name(&self) -> &'static str { "Primes" }
    fn calculate(&self, _input: &str) -> u32 { 0 }
}

/// Placeholder for a Fibonacci based cipher.
pub struct Fibonacci;
impl Cipher for Fibonacci {
    fn name(&self) -> &'static str { "Fibonacci" }
    fn calculate(&self, _input: &str) -> u32 { 0 }
}

/// Placeholder for a Chaldean cipher.
pub struct Chaldean;
impl Cipher for Chaldean {
    fn name(&self) -> &'static str { "Chaldean" }
    fn calculate(&self, _input: &str) -> u32 { 0 }
}

/// Placeholder for an Agrippa cipher.
pub struct Agrippa;
impl Cipher for Agrippa {
    fn name(&self) -> &'static str { "Agrippa" }
    fn calculate(&self, _input: &str) -> u32 { 0 }
}
