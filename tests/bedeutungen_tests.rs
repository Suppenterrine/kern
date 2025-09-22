use kern::core::{load_bedeutungen, lookup, Bedeutung};
use std::collections::HashMap;

#[test]
fn bedeutungen_has_text_light_shadow_for_known_numbers() {
    let map = load_bedeutungen();

    // Zahlen, die in bedeutungen.yaml definiert sind
    let expected = [1u32,2,3,4,5,6,7,8,9,11,22,33];
    for &n in &expected {
        let entry = map.get(&n).expect("missing entry in bedeutungen.yaml");
        assert!(entry.text.as_ref().is_some_and(|s| !s.trim().is_empty()));
        assert!(entry.licht.as_ref().is_some_and(|s| !s.trim().is_empty()));
        assert!(entry.schatten.as_ref().is_some_and(|s| !s.trim().is_empty()));
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

