use kern::core::{Bedeutung, Lang, load_bedeutungen, load_bedeutungen_lang, lookup, lookup_lang};
use std::collections::HashMap;

/// Zahlen, die in bedeutungen.yaml definiert sind
const EXPECTED_NUMBERS: [u32; 12] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 11, 22, 33];

#[test]
fn bedeutungen_has_text_light_shadow_for_known_numbers() {
    let map = load_bedeutungen();

    for &n in &EXPECTED_NUMBERS {
        let entry = map.get(&n).expect("missing entry in bedeutungen.yaml");
        assert!(entry.text.as_ref().is_some_and(|s| !s.trim().is_empty()));
        assert!(entry.licht.as_ref().is_some_and(|s| !s.trim().is_empty()));
        assert!(
            entry
                .schatten
                .as_ref()
                .is_some_and(|s| !s.trim().is_empty())
        );
    }
}

#[test]
fn every_language_is_complete() {
    for lang in Lang::ALL {
        let map = load_bedeutungen_lang(lang);

        for &n in &EXPECTED_NUMBERS {
            let entry = map
                .get(&n)
                .unwrap_or_else(|| panic!("number {n} missing in language '{lang}'"));

            for (field, value) in [
                ("bedeutung", &entry.text),
                ("lichtseite", &entry.licht),
                ("schattenseite", &entry.schatten),
            ] {
                assert!(
                    value.as_ref().is_some_and(|s| !s.trim().is_empty()),
                    "'{field}' for number {n} is empty in language '{lang}'"
                );
            }
        }
    }
}

#[test]
fn languages_have_identical_key_sets() {
    let base: std::collections::BTreeSet<u32> = load_bedeutungen_lang(Lang::De).keys().copied().collect();

    for lang in Lang::ALL {
        let keys: std::collections::BTreeSet<u32> =
            load_bedeutungen_lang(lang).keys().copied().collect();
        assert_eq!(
            keys, base,
            "language '{lang}' has a different set of numbers than the German base file"
        );
    }
}

#[test]
fn translations_are_not_copies_of_the_german_source() {
    // Fängt eine Übersetzungsdatei ab, die versehentlich deutschen Text behalten hat.
    let german = load_bedeutungen_lang(Lang::De);

    for lang in Lang::ALL.into_iter().filter(|l| *l != Lang::De) {
        let map = load_bedeutungen_lang(lang);

        for &n in &EXPECTED_NUMBERS {
            let de = german.get(&n).unwrap();
            let tr = map.get(&n).unwrap();
            assert_ne!(
                de.text, tr.text,
                "'bedeutung' for number {n} is still German in language '{lang}'"
            );
            assert_ne!(
                de.licht, tr.licht,
                "'lichtseite' for number {n} is still German in language '{lang}'"
            );
            assert_ne!(
                de.schatten, tr.schatten,
                "'schattenseite' for number {n} is still German in language '{lang}'"
            );
        }
    }
}

#[test]
fn lookup_lang_returns_the_requested_language() {
    for lang in Lang::ALL {
        let map = load_bedeutungen_lang(lang);
        let text = lookup_lang(1, &map, lang);
        assert_eq!(text, map.get(&1).unwrap().text.as_deref().unwrap());
        assert_ne!(text, lang.missing_meaning());
    }
}

#[test]
fn lookup_returns_text_field_or_default() {
    let map = load_bedeutungen();

    // Ein vorhandener Schlüssel: lookup == text
    let n = 1u32;
    let text_from_map = map.get(&n).and_then(|b| b.text.as_deref()).unwrap();
    let text_from_lookup = lookup(n, &map);
    assert_eq!(text_from_lookup, text_from_map);

    // Ein nicht vorhandener Schlüssel: liefert Standard-Text
    // (10 existiert nicht in der gelieferten YAML)
    let missing = 10u32;
    let text_missing = lookup(missing, &map);
    assert_eq!(text_missing, "- keine Bedeutung -");
}

#[test]
fn serde_aliases_for_light_shadow_are_respected() {
    // Prüfe, dass sowohl "licht"/"schatten" als auch
    // "lichtseite"/"schattenseite" befüllt werden können.

    let yaml_alt_keys = r#"
1:
  bedeutung: "Test Text"
  licht: "Licht kurz"
  schatten: "Schatten kurz"
"#;

    let parsed: HashMap<u32, Bedeutung> = serde_yaml::from_str(yaml_alt_keys).unwrap();
    let b1 = parsed.get(&1).unwrap();
    assert_eq!(b1.text.as_deref(), Some("Test Text"));
    assert_eq!(b1.licht.as_deref(), Some("Licht kurz"));
    assert_eq!(b1.schatten.as_deref(), Some("Schatten kurz"));

    let yaml_long_keys = r#"
2:
  bedeutung: "Test Lang"
  lichtseite: "Licht lang"
  schattenseite: "Schatten lang"
"#;

    let parsed2: HashMap<u32, Bedeutung> = serde_yaml::from_str(yaml_long_keys).unwrap();
    let b2 = parsed2.get(&2).unwrap();
    assert_eq!(b2.text.as_deref(), Some("Test Lang"));
    assert_eq!(b2.licht.as_deref(), Some("Licht lang"));
    assert_eq!(b2.schatten.as_deref(), Some("Schatten lang"));
}
