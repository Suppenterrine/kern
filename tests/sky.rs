use kern::cipher::Cipher;
use kern::ciphers::{calculate_all, Ordinal};

#[test]
fn test_ordinal_cipher() {
    let c = Ordinal;
    assert_eq!(c.name(), "Ordinal");
    assert_eq!(c.calculate("feldmann"), 6);
}

#[test]
fn test_calculate_all() {
    let results = calculate_all("feldmann");
    assert_eq!(results, vec![("Ordinal", 6)]);
}
