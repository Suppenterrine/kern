//! Argument-Positions-Tests für die CLI.
//!
//! Das Verhalten hier war lange still falsch: Flags nach dem ersten Input
//! wurden als Wörter reduziert, `kern hello --cipher chaldean` berechnete die
//! Quersumme der Zeichenkette "--cipher" und meldete sie als Ergebnis. Kein
//! Fehler, nur eine falsche Zahl — genau der Fall, den docs/PRINCIPLES.md §1
//! ausschließt.
//!
//! Diese Tests rufen die echte Binary auf. Die Ausgabe ist JSON, weil stdout
//! beim Aufruf aus einem Test kein TTY ist.

use std::process::Command;

fn kern(args: &[&str]) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_kern"))
        .args(args)
        .output()
        .expect("failed to run the kern binary");
    String::from_utf8_lossy(&out.stdout).to_string()
}

/// Ein Flag, das als Input durchgerutscht ist, taucht als `"input"` in der
/// Ausgabe auf. Das ist die Signatur des Fehlers, den diese Datei absichert.
fn assert_not_swallowed(output: &str, flag: &str) {
    assert!(
        !output.contains(&format!("\"input\":\"{flag}\"")),
        "{flag} was reduced as a word instead of being parsed: {output}"
    );
}

#[test]
fn flags_work_before_the_input() {
    let out = kern(&["--cipher", "chaldean", "hello"]);
    assert!(out.contains("\"value\":5"), "chaldean not applied: {out}");
}

#[test]
fn flags_work_after_the_input() {
    let out = kern(&["hello", "--cipher", "chaldean"]);
    assert_not_swallowed(&out, "--cipher");
    assert!(out.contains("\"value\":5"), "chaldean not applied: {out}");
}

#[test]
fn value_flags_after_the_input_take_effect() {
    let out = kern(&["hello", "--lang", "de", "--lookup"]);
    assert_not_swallowed(&out, "--lang");
    assert!(out.contains("\"lang\":\"de\""), "language ignored: {out}");
}

#[test]
fn boolean_flags_after_the_input_take_effect() {
    let verbose = kern(&["hello", "--verbose"]);
    assert_not_swallowed(&verbose, "--verbose");
    assert!(verbose.contains("\"chain\""), "verbose ignored: {verbose}");

    let length = kern(&["hello", "-L"]);
    assert_not_swallowed(&length, "-L");
    assert!(length.contains("\"length\":5"), "length ignored: {length}");
}

#[test]
fn flags_may_be_mixed_around_the_input() {
    let out = kern(&["--lang", "de", "hello", "--lookup"]);
    assert!(out.contains("\"lang\":\"de\""), "language ignored: {out}");
    assert!(out.contains("\"number\":7"), "lookup ignored: {out}");
}

/// Die Kehrseite: das Positional akzeptiert keine führenden Bindestriche mehr.
/// Der Preis ist bewusst in Kauf genommen, weil daraus ein sichtbarer Fehler
/// wird statt eines stillen Falschergebnisses.
#[test]
fn hyphen_leading_input_needs_the_separator() {
    let out = kern(&["--", "-abc"]);
    assert!(out.contains("\"input\":\"-abc\""), "separator broken: {out}");
}

#[test]
fn date_range_still_accepts_negative_offsets() {
    let out = kern(&["-d", "-3..0"]);
    assert!(out.contains("\"offset\":-3"), "negative offset lost: {out}");
}

/// Issue #23: ohne --total wurde `"total":0` gemeldet — eine Zahl, die nie
/// berechnet wurde.
#[test]
fn total_is_absent_unless_requested() {
    let without = kern(&["hello", "world"]);
    assert!(
        !without.contains("\"total\""),
        "total reported without --total: {without}"
    );

    let with = kern(&["hello", "world", "--total"]);
    assert!(with.contains("\"total\":7"), "total missing: {with}");
}

/// Issue #23: `--total` durfte nicht zusätzlich die Items verstecken — die
/// TTY-Ausgabe zeigte immer beides.
#[test]
fn total_does_not_hide_the_items() {
    let out = kern(&["hello", "world", "--total"]);
    assert!(out.contains("\"input\":\"hello\""), "items hidden: {out}");
    assert!(out.contains("\"input\":\"world\""), "items hidden: {out}");
}

#[test]
fn unsupported_flag_combination_is_rejected_wherever_it_is_typed() {
    for args in [
        vec!["--prm", "--total", "a", "b"],
        vec!["--prm", "a", "b", "--total"],
    ] {
        let out = kern(&args);
        assert!(
            out.contains("\"code\":\"invalid_arguments\""),
            "{args:?} was not rejected: {out}"
        );
    }
}
