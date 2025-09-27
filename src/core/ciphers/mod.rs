pub trait Cipher {
    fn name(&self) -> &str;
    fn char_to_value(&self, ch: char) -> u32;
}

mod agrippa;
mod chaldean;
mod cubes;
mod fibonacci;
mod ordinal;
mod primes;
mod pythagorean;
mod reverse_ordinal;
mod reverse_pythagorean;
mod septenary;
mod squares;

pub use agrippa::AgrippaCipher;
pub use chaldean::ChaldeanCipher;
pub use cubes::CubesCipher;
pub use fibonacci::FibonacciCipher;
pub use ordinal::OrdinalCipher;
pub use primes::PrimesCipher;
pub use pythagorean::PythagoreanCipher;
pub use reverse_ordinal::ReverseOrdinalCipher;
pub use reverse_pythagorean::ReversePythagoreanCipher;
pub use septenary::SeptenaryCipher;
pub use squares::SquaresCipher;

pub fn available_cipher_names() -> &'static [&'static str] {
    &[
        "ordinal",
        "reverse_ordinal",
        "pythagorean",
        "reverse_pythagorean",
        "chaldean",
        "agrippa",
        "primes",
        "fibonacci",
        "squares",
        "cubes",
        "septenary",
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
        "chaldean" | "chald" => Some(Box::new(ChaldeanCipher)),
        "agrippa" | "agr" => Some(Box::new(AgrippaCipher)),
        "primes" | "prime" => Some(Box::new(PrimesCipher)),
        "fibonacci" | "fib" => Some(Box::new(FibonacciCipher)),
        "squares" | "square" => Some(Box::new(SquaresCipher)),
        "cubes" | "cube" => Some(Box::new(CubesCipher)),
        "septenary" | "sept" => Some(Box::new(SeptenaryCipher)),
        _ => None,
    }
}
