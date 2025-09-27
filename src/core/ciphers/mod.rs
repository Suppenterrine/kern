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

pub struct CipherDescriptor {
    pub name: &'static str,
    pub short: &'static str,
    pub description: &'static str,
    pub factory: fn() -> Box<dyn Cipher>,
}

const CIPHERS: &[CipherDescriptor] = &[
    CipherDescriptor {
        name: "ordinal",
        short: "or",
        description: "Ordinal (A=1..Z=26)",
        factory: || Box::new(OrdinalCipher),
    },
    CipherDescriptor {
        name: "reverse_ordinal",
        short: "ro",
        description: "Reverse Ordinal (A=26..Z=1)",
        factory: || Box::new(ReverseOrdinalCipher),
    },
    CipherDescriptor {
        name: "pythagorean",
        short: "py",
        description: "Pythagorean / Reduction",
        factory: || Box::new(PythagoreanCipher),
    },
    CipherDescriptor {
        name: "reverse_pythagorean",
        short: "rp",
        description: "Reverse Pythagorean",
        factory: || Box::new(ReversePythagoreanCipher),
    },
    CipherDescriptor {
        name: "chaldean",
        short: "ch",
        description: "Chaldean",
        factory: || Box::new(ChaldeanCipher),
    },
    CipherDescriptor {
        name: "agrippa",
        short: "ag",
        description: "Agrippa Latin",
        factory: || Box::new(AgrippaCipher),
    },
    CipherDescriptor {
        name: "primes",
        short: "pr",
        description: "Prime numbers mapping",
        factory: || Box::new(PrimesCipher),
    },
    CipherDescriptor {
        name: "fibonacci",
        short: "fi",
        description: "Fibonacci sequence mapping",
        factory: || Box::new(FibonacciCipher),
    },
    CipherDescriptor {
        name: "squares",
        short: "sq",
        description: "Square numbers mapping",
        factory: || Box::new(SquaresCipher),
    },
    CipherDescriptor {
        name: "cubes",
        short: "cu",
        description: "Cube numbers mapping",
        factory: || Box::new(CubesCipher),
    },
    CipherDescriptor {
        name: "septenary",
        short: "se",
        description: "Septenary / Base-7",
        factory: || Box::new(SeptenaryCipher),
    },
];

pub fn descriptors() -> &'static [CipherDescriptor] {
    CIPHERS
}

pub fn available_cipher_names() -> Vec<&'static str> {
    CIPHERS.iter().map(|d| d.name).collect()
}

pub fn default_cipher() -> Box<dyn Cipher> {
    (CIPHERS[0].factory)()
}

pub fn get_cipher(name: &str) -> Option<Box<dyn Cipher>> {
    let key = name.to_lowercase();
    CIPHERS
        .iter()
        .find(|descriptor| descriptor.name == key || descriptor.short == key)
        .map(|descriptor| (descriptor.factory)())
}
