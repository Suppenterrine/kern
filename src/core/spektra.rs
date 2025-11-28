//! SPEKTRA Template-Befüllung
//!
//! Dieses Modul befüllt das statische spektra_prompt.txt Template
//! mit Kern-berechneten Werten (11 Chiffren + Bedeutungen).

use crate::core::KernResult;
use crate::core::phase::{calculate_compartment, calculate_phase};
use regex::Regex;
use std::collections::HashMap;

/// Achsen-Typ für Spektra-Analyse
#[derive(Debug, Clone)]
pub enum AxisType {
    Resonanz,  // Gleicher reduzierter Wert (unabhängig von Phase)
    Spannung,  // Verschiedene Werte UND Phase ±1
}

/// Eine Spektra-Achse zwischen zwei Ciphers
#[derive(Debug, Clone)]
pub struct SpektraAxis {
    pub cipher_a: String,
    pub cipher_b: String,
    pub value_a: u32,
    pub value_b: u32,
    pub axis_type: AxisType,
}

/// Befüllt das SPEKTRA-Analyse-Template mit Cipher-Ergebnissen und Bedeutungen
pub fn build_spektra_prompt(
    word: &str,
    results: &[KernResult],
    bedeutungen: &HashMap<u32, crate::core::Bedeutung>,
) -> Result<String, String> {
    // Template als eingebetteter String
    let template = include_str!("../../spektra_prompt.txt");

    // Sorge Ergebnisse nach Cipher-Name
    let mut cipher_results: HashMap<String, (u32, String)> = HashMap::new();
    
    for result in results {
        let meaning = bedeutungen
            .get(&result.value)
            .and_then(|b| b.text.as_deref())
            .unwrap_or("- keine Bedeutung -")
            .to_string();
        
        cipher_results.insert(result.cipher.clone(), (result.value, meaning));
    }

    // Mapping von Template-Namen zu Cipher-Namen
    // Die Template-Namen MÜSSEN exakt dem Template entsprechen!
    let template_to_cipher = vec![
        ("Ordinal", "ordinal"),
        ("Reverse Ordinal", "reverse_ordinal"),
        ("Pythagorean", "pythagorean"),
        ("Reverse Pythagorean", "reverse_pythagorean"),
        ("Chaldean", "chaldean"),
        ("Agrippa (Latin)", "agrippa"),
        ("Primes Cipher", "primes"),
        ("Fibonacci Cipher", "fibonacci"),
        ("Squares Cipher", "squares"),
        ("Cubes Cipher", "cubes"),
        ("Septenary", "septenary"),
    ];

    let mut result_template = template.to_string();

    // Ersetze [User-Eingabe]
    result_template = result_template.replace("[User-Eingabe]", word);

    // Ersetze [Zahl] und [Bedeutung] für jede Chiffre
    // Verwende Regex für flexible Whitespace-Behandlung
    for (template_name, cipher_key) in &template_to_cipher {
        if let Some((value, meaning)) = cipher_results.get(*cipher_key) {
            // Pattern erlaubt variable Whitespace
            let pattern_str = format!(
                r"{}:\s*\n\s*-\s*Reduktion:\s*\[Zahl\]\s*\n\s*-\s*Bedeutung:\s*\[Bedeutung\]",
                regex::escape(template_name)
            );

            if let Ok(re) = Regex::new(&pattern_str) {
                let replacement = format!(
                    "{}:\n   - Reduktion: {}\n   - Bedeutung: {}",
                    template_name, value, meaning
                );
                result_template = re.replace(&result_template, replacement).to_string();
            }
        }
    }

    // Berechne Achsen aus allen Cipher-Ergebnissen
    let (resonanz_axes, spannungs_axes) = calculate_spektra_axes(results);

    // Formatiere Achsen
    let resonanz_text = format_resonanz_axes(&resonanz_axes);
    let spannungs_text = format_spannungs_axes(&spannungs_axes);

    // Ersetze Achsen-Platzhalter im Template
    result_template = result_template.replace("RESONANZACHSEN: ...", &resonanz_text);
    result_template = result_template.replace("SPANNUNGSACHSEN: ...", &spannungs_text);

    Ok(result_template)
}

/// Berechnet Spektra-Achsen aus den 11 Cipher-Ergebnissen eines einzelnen Wortes
///
/// Vergleicht alle Paare (55 Kombinationen) und identifiziert:
/// - Resonanzachsen: Paare mit gleichem reduziertem Wert
/// - Spannungsachsen: Paare mit verschiedenen Werten UND Phase ±1
///
/// Returns: (resonanz_axes, spannungs_axes)
fn calculate_spektra_axes(
    results: &[KernResult],
) -> (Vec<SpektraAxis>, Vec<SpektraAxis>) {
    let mut resonanz_axes = Vec::new();
    let mut spannungs_axes = Vec::new();

    // Generiere alle eindeutigen Paare (55 bei 11 Ciphers)
    for i in 0..results.len() {
        for j in (i + 1)..results.len() {
            let res_a = &results[i];
            let res_b = &results[j];

            // Prüfe auf Resonanzachse: gleicher reduzierter Wert
            if res_a.value == res_b.value {
                resonanz_axes.push(SpektraAxis {
                    cipher_a: res_a.cipher.clone(),
                    cipher_b: res_b.cipher.clone(),
                    value_a: res_a.value,
                    value_b: res_b.value,
                    axis_type: AxisType::Resonanz,
                });
            } else {
                // Prüfe auf Spannungsachse: verschiedene Werte UND Phase ±1
                let comp_a = calculate_compartment(res_a.value);
                let comp_b = calculate_compartment(res_b.value);
                let phase = calculate_phase(comp_a, comp_b);

                if phase == 1 || phase == -1 {
                    spannungs_axes.push(SpektraAxis {
                        cipher_a: res_a.cipher.clone(),
                        cipher_b: res_b.cipher.clone(),
                        value_a: res_a.value,
                        value_b: res_b.value,
                        axis_type: AxisType::Spannung,
                    });
                }
            }
        }
    }

    (resonanz_axes, spannungs_axes)
}

/// Formatiert Cipher-Namen für die Ausgabe (interner Name → Display-Name)
fn format_cipher_name(cipher: &str) -> &str {
    match cipher {
        "ordinal" => "Ordinal",
        "reverse_ordinal" => "Reverse Ordinal",
        "pythagorean" => "Pythagorean",
        "reverse_pythagorean" => "Reverse Pythagorean",
        "chaldean" => "Chaldean",
        "agrippa" => "Agrippa (Latin)",
        "primes" => "Primes Cipher",
        "fibonacci" => "Fibonacci Cipher",
        "squares" => "Squares Cipher",
        "cubes" => "Cubes Cipher",
        "septenary" => "Septenary",
        _ => cipher,
    }
}

/// Formatiert Resonanzachsen für die Template-Ausgabe
fn format_resonanz_axes(axes: &[SpektraAxis]) -> String {
    if axes.is_empty() {
        return "RESONANZACHSEN: Keine gefunden".to_string();
    }

    let mut output = String::from("RESONANZACHSEN:");
    for axis in axes {
        output.push_str(&format!(
            "\n  - {} ↔ {} (beide: {})",
            format_cipher_name(&axis.cipher_a),
            format_cipher_name(&axis.cipher_b),
            axis.value_a
        ));
    }
    output
}

/// Formatiert Spannungsachsen für die Template-Ausgabe
fn format_spannungs_axes(axes: &[SpektraAxis]) -> String {
    if axes.is_empty() {
        return "SPANNUNGSACHSEN: Keine gefunden".to_string();
    }

    let mut output = String::from("SPANNUNGSACHSEN:");
    for axis in axes {
        output.push_str(&format!(
            "\n  - {} ({}) ⟷ {} ({})",
            format_cipher_name(&axis.cipher_a),
            axis.value_a,
            format_cipher_name(&axis.cipher_b),
            axis.value_b
        ));
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_loading() {
        let template = include_str!("../../spektra_prompt.txt");
        assert!(template.contains("[User-Eingabe]"));
        assert!(template.contains("Ordinal:"));
        assert!(template.contains("RESONANZACHSEN:"));
    }

    #[test]
    fn test_calculate_spektra_axes_resonanz() {
        use crate::core::{Operation, Step};

        // Erstelle Mock-Ergebnisse mit einigen gleichen Werten
        let results = vec![
            KernResult {
                source: "test".to_string(),
                value: 7,
                cipher: "ordinal".to_string(),
                step: Step {
                    pipe_index: 0,
                    cipher_index: 0,
                    operation: Operation::Reduce,
                    metadata: None,
                },
                verbose: false,
                trace: vec![],
                payload: None,
            },
            KernResult {
                source: "test".to_string(),
                value: 7,
                cipher: "pythagorean".to_string(),
                step: Step {
                    pipe_index: 0,
                    cipher_index: 1,
                    operation: Operation::Reduce,
                    metadata: None,
                },
                verbose: false,
                trace: vec![],
                payload: None,
            },
            KernResult {
                source: "test".to_string(),
                value: 3,
                cipher: "chaldean".to_string(),
                step: Step {
                    pipe_index: 0,
                    cipher_index: 2,
                    operation: Operation::Reduce,
                    metadata: None,
                },
                verbose: false,
                trace: vec![],
                payload: None,
            },
        ];

        let (resonanz, _spannung) = calculate_spektra_axes(&results);

        // Ordinal (7) und Pythagorean (7) sollten eine Resonanzachse bilden
        assert_eq!(resonanz.len(), 1);
        assert_eq!(resonanz[0].value_a, 7);
        assert_eq!(resonanz[0].value_b, 7);
    }

    #[test]
    fn test_calculate_spektra_axes_spannung() {
        use crate::core::{Operation, Step};

        // Erstelle Mock-Ergebnisse mit Spannungs-Phasen
        // Compartment 1 = [1, 4, 7], Compartment 2 = [2, 5, 8]
        // Phase von 1→2 ist +1 (Spannung)
        let results = vec![
            KernResult {
                source: "test".to_string(),
                value: 1, // Compartment 1
                cipher: "ordinal".to_string(),
                step: Step {
                    pipe_index: 0,
                    cipher_index: 0,
                    operation: Operation::Reduce,
                    metadata: None,
                },
                verbose: false,
                trace: vec![],
                payload: None,
            },
            KernResult {
                source: "test".to_string(),
                value: 2, // Compartment 2
                cipher: "pythagorean".to_string(),
                step: Step {
                    pipe_index: 0,
                    cipher_index: 1,
                    operation: Operation::Reduce,
                    metadata: None,
                },
                verbose: false,
                trace: vec![],
                payload: None,
            },
        ];

        let (_resonanz, spannung) = calculate_spektra_axes(&results);

        // 1 und 2 sollten eine Spannungsachse bilden (Phase +1)
        assert_eq!(spannung.len(), 1);
        assert_eq!(spannung[0].value_a, 1);
        assert_eq!(spannung[0].value_b, 2);
    }

    #[test]
    fn test_format_cipher_name() {
        assert_eq!(format_cipher_name("ordinal"), "Ordinal");
        assert_eq!(format_cipher_name("reverse_ordinal"), "Reverse Ordinal");
        assert_eq!(format_cipher_name("pythagorean"), "Pythagorean");
        assert_eq!(format_cipher_name("agrippa"), "Agrippa (Latin)");
    }

    #[test]
    fn test_format_axes() {
        let resonanz = vec![SpektraAxis {
            cipher_a: "ordinal".to_string(),
            cipher_b: "pythagorean".to_string(),
            value_a: 7,
            value_b: 7,
            axis_type: AxisType::Resonanz,
        }];

        let output = format_resonanz_axes(&resonanz);
        assert!(output.contains("RESONANZACHSEN:"));
        assert!(output.contains("Ordinal"));
        assert!(output.contains("Pythagorean"));
        assert!(output.contains("(beide: 7)"));
    }

    #[test]
    fn test_integration_axes_in_template() {
        use crate::core::{Bedeutung, Operation, Step};
        use std::collections::HashMap;

        // Erstelle Test-Ergebnisse mit mehreren Resonanz- und Spannungsachsen
        let results = vec![
            // Resonanzachse: ordinal (7) = pythagorean (7)
            KernResult {
                source: "test".to_string(),
                value: 7,
                cipher: "ordinal".to_string(),
                step: Step {
                    pipe_index: 0,
                    cipher_index: 0,
                    operation: Operation::Reduce,
                    metadata: None,
                },
                verbose: false,
                trace: vec![],
                payload: None,
            },
            KernResult {
                source: "test".to_string(),
                value: 7,
                cipher: "pythagorean".to_string(),
                step: Step {
                    pipe_index: 0,
                    cipher_index: 1,
                    operation: Operation::Reduce,
                    metadata: None,
                },
                verbose: false,
                trace: vec![],
                payload: None,
            },
            // Spannungsachse: 1 (Comp 1) -> 2 (Comp 2) = Phase +1
            KernResult {
                source: "test".to_string(),
                value: 1,
                cipher: "chaldean".to_string(),
                step: Step {
                    pipe_index: 0,
                    cipher_index: 2,
                    operation: Operation::Reduce,
                    metadata: None,
                },
                verbose: false,
                trace: vec![],
                payload: None,
            },
            KernResult {
                source: "test".to_string(),
                value: 2,
                cipher: "agrippa".to_string(),
                step: Step {
                    pipe_index: 0,
                    cipher_index: 3,
                    operation: Operation::Reduce,
                    metadata: None,
                },
                verbose: false,
                trace: vec![],
                payload: None,
            },
        ];

        let mut bedeutungen = HashMap::new();
        bedeutungen.insert(
            7,
            Bedeutung {
                text: Some("Test Bedeutung 7".to_string()),
                licht: None,
                schatten: None,
            },
        );
        bedeutungen.insert(
            1,
            Bedeutung {
                text: Some("Test Bedeutung 1".to_string()),
                licht: None,
                schatten: None,
            },
        );
        bedeutungen.insert(
            2,
            Bedeutung {
                text: Some("Test Bedeutung 2".to_string()),
                licht: None,
                schatten: None,
            },
        );

        let output = build_spektra_prompt("TestWort", &results, &bedeutungen).unwrap();

        // Prüfe, dass Achsen im Output enthalten sind
        assert!(output.contains("RESONANZACHSEN:"));
        assert!(output.contains("SPANNUNGSACHSEN:"));

        // Prüfe Resonanzachse
        assert!(output.contains("Ordinal ↔ Pythagorean (beide: 7)"));

        // Prüfe Spannungsachse
        assert!(output.contains("Chaldean (1) ⟷ Agrippa (Latin) (2)"));

        // Optional: Print für manuelle Verifikation
        println!("\n=== SPEKTRA AXES OUTPUT TEST ===\n");
        // Extrahiere die Achsen-Sektion
        if let Some(start) = output.find("RESONANZACHSEN:") {
            // Finde das Ende der Achsen-Sektion (nächster doppelter Newline oder Ende)
            let section = &output[start..];
            let end_pos = section.find("\n\n").unwrap_or(section.len());
            println!("{}", &section[..end_pos]);
        } else {
            println!("RESONANZACHSEN nicht gefunden im Output!");
        }
        println!("\n================================\n");
    }
}
