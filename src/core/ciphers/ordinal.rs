use crate::core::utils::char_to_value_ordinal;

use super::Cipher;

#[derive(Debug, Clone)]
pub struct OrdinalCipher;

impl Cipher for OrdinalCipher {
    fn name(&self) -> &str {
        "ordinal"
    }

    fn char_to_value(&self, ch: char) -> u32 {
        char_to_value_ordinal(ch)
    }
}
