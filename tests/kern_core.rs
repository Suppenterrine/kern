//! Kern-Unittests (werden von `cargo test` automatisch gefunden)

use kern::core::*;
use chrono::{Local};

//
// ── 1.  Einzel-Char-Werte ────────────────────────────────────────────────────
//
#[test]
fn test_char_to_value_digits() {
    assert_eq!(char_to_value('0'), 0);
    assert_eq!(char_to_value('7'), 7);
}

#[test]
fn test_char_to_value_letters() {
    assert_eq!(char_to_value('A'), 1);
    assert_eq!(char_to_value('Z'), 26);
    assert_eq!(char_to_value('a'), 1);
    assert_eq!(char_to_value('z'), 26);
}

//
// ── 2.  Quersummen-Reduktion (ohne Debugtext) ────────────────────────────────
//
#[test]
fn test_reduce_number_verbose_simple() {
    // "11" ist Masterzahl → bleibt 11
    assert_eq!(reduce_number_verbose("11", false), 11);

    // 654 -> 6+5+4 = 15 -> 1+5 = 6
    assert_eq!(reduce_number_verbose("654", false), 6);

    // "feldmann" erwartete Endsumme = 6
    assert_eq!(reduce_number_verbose("feldmann", false), 6);
}

//
// ── 3.  Datums-Offset-Parser ────────────────────────────────────────────────
//
#[test]
fn test_parse_range_absolute_date() {
    let today = Local::now().date_naive();
    let date_str = today.format("%d.%m.%Y").to_string();          // Offset 0
    assert_eq!(parse_range(&date_str).unwrap(), vec![0]);
}

#[test]
fn test_parse_range_relative() {
    assert_eq!(parse_range("-2").unwrap(), vec![-2]);
    assert_eq!(parse_range("0+2").unwrap(), vec![0, 1, 2]);
    assert_eq!(parse_range("-3..1").unwrap(), vec![-3, -2, -1, 0, 1]);
}
