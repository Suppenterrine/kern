//! SPEKTRA Template-Befüllung
//! 
//! Dieses Modul befüllt das statische spektra_prompt.txt Template
//! mit Kern-berechneten Werten (11 Chiffren + Bedeutungen).

use crate::core::KernResult;
use regex::Regex;
use std::collections::HashMap;

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

    Ok(result_template)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_template_loading() {
        let template = include_str!("../../spektra_prompt.txt");
        assert!(template.contains("[User-Eingabe]"));
        assert!(template.contains("Ordinal:"));
        assert!(template.contains("RESONANZACHSEN:"));
    }
}
