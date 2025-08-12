/// Trait for gematria ciphers.
///
/// Ciphers map an input string to a numerical value using a
/// particular reduction strategy. Implementations must be pure and
/// side‑effect free so they can easily be executed in parallel.
pub trait Cipher: Sync {
    /// Name of the cipher.
    fn name(&self) -> &'static str;

    /// Calculate the value for the given input.
    fn calculate(&self, input: &str) -> u32;
}
