pub trait Cipher {
    fn name(&self) -> &str;
    fn char_to_value(&self, ch: char) -> u32;
}

mod ordinal;
mod pythagorean;
mod reverse_ordinal;
mod reverse_pythagorean;

pub use ordinal::OrdinalCipher;
pub use pythagorean::PythagoreanCipher;
pub use reverse_ordinal::ReverseOrdinalCipher;
pub use reverse_pythagorean::ReversePythagoreanCipher;

pub fn available_cipher_names() -> &'static [&'static str] {
    &[
        "ordinal",
        "reverse_ordinal",
        "pythagorean",
        "reverse_pythagorean",
    ]
}

pub fn default_cipher() -> Box<dyn Cipher> {
    Box::new(OrdinalCipher)
}

pub fn get_cipher(name: &str) -> Option<Box<dyn Cipher>> {
    let key = name.to_lowercase();
    match key.as_str() {
        "ordinal" | "ord" => Some(Box::new(OrdinalCipher)),
        "reverse_ordinal" | "reverse-ordinal" | "rev_ord" | "rev-ord" => {
            Some(Box::new(ReverseOrdinalCipher))
        }
        "pythagorean" | "pyth" => Some(Box::new(PythagoreanCipher)),
        "reverse_pythagorean" | "reverse-pythagorean" | "rev_pyth" | "rev-pyth" => {
            Some(Box::new(ReversePythagoreanCipher))
        }
        _ => None,
    }
}
