//! Output formatting functions for KERN CLI
//!
//! All output formatting is centralized here for consistency.

use super::*;
use crate::core::{Bedeutung, KernResult};
use std::collections::HashMap;

// ============================================================================
// REDUCE OUTPUT
// ============================================================================

/// Format a single reduction result
pub fn format_reduce_single(input: &str, value: u32, cipher: &str) {
    println!("{} {} {} [{}]", input, ARROW_RIGHT, value, cipher);
}

/// Format grouped reduction results (multiple ciphers per input)
pub fn format_reduce_grouped(input: &str, results: &[(String, u32)]) {
    if results.is_empty() {
        return;
    }

    if results.len() == 1 {
        // Single cipher - use compact format
        format_reduce_single(input, results[0].1, &results[0].0);
    } else {
        // Multiple ciphers - use grouped format
        println!("{}", input);
        for (cipher, value) in results {
            println!("{}{}  {} {}", INDENT_BASE, cipher, ARROW_RIGHT, value);
        }
    }
}

// ============================================================================
// VERBOSE REDUCTION OUTPUT
// ============================================================================

/// Format verbose reduction trace
pub fn format_verbose_reduction(input: &str, cipher: &str, trace: &[String], final_value: u32) {
    println!("{} [{}]", input, cipher);

    // Print all trace lines except the last "Quersumme" line
    for line in trace.iter().take(trace.len().saturating_sub(1)) {
        // Remove leading "test → " or similar if present
        let clean_line = if line.starts_with(&format!("{} {} ", input, ARROW_RIGHT)) {
            line.trim_start_matches(&format!("{} {} ", input, ARROW_RIGHT))
        } else {
            line
        };

        // Remove "Quersumme:" prefix if present
        let clean_line = clean_line.trim_start_matches("Quersumme: ");

        println!("{}{}", INDENT_BASE, clean_line);
    }

    // Final result arrow
    println!("{}{} {}", INDENT_BASE, ARROW_RIGHT, final_value);
}

// ============================================================================
// TOTAL OUTPUT
// ============================================================================

/// Format simple total output
pub fn format_total_simple(sum: u32, reduced: u32) {
    spacing(SPACING_SECTION);

    if sum == reduced {
        println!("Total: {}", sum);
    } else {
        println!("Total: {} {} {}", sum, ARROW_RIGHT, reduced);
    }
}

/// Format verbose total calculation
pub fn format_total_verbose(parts: &[u32], sum: u32, trace: &[String], final_value: u32) {
    spacing(SPACING_SECTION);
    println!("Total Calculation");

    // Show the sum calculation
    if !parts.is_empty() {
        let parts_str: Vec<String> = parts.iter().map(|p| p.to_string()).collect();
        println!("{}{}  = {}", INDENT_BASE, parts_str.join("+"), sum);
    }

    // Show reduction steps if sum > 9
    if sum > 9 && sum != final_value {
        // Show all trace lines except the last one (which is "→ final_value")
        for line in trace.iter().take(trace.len().saturating_sub(1)) {
            // Skip lines that start with arrows (they're just the final result marker)
            if !line.starts_with('→') {
                println!("{}{}", INDENT_BASE, line);
            }
        }
    }

    // Final result
    println!("{}{} {}", INDENT_BASE, ARROW_RIGHT, final_value);
}

// ============================================================================
// LOOKUP OUTPUT
// ============================================================================

/// Format standard lookup entry
pub fn format_lookup_entry(
    value: u32,
    meaning: &str,
    sources: &[String],
    bedeutung: Option<&Bedeutung>,
    show_pos: bool,
    show_neg: bool,
    show_full: bool,
) {
    // Header: Number · Meaning
    println!("{} {} {}", value, MIDDOT, meaning);

    // Sources
    if show_full && !sources.is_empty() {
        println!("{}Sources:", INDENT_BASE);
        for (i, source) in sources.iter().enumerate() {
            let is_last = i == sources.len() - 1;
            println!("{}", tree_item(source, is_last));
        }
    } else if !sources.is_empty() {
        for (i, source) in sources.iter().enumerate() {
            let is_last = i == sources.len() - 1;
            println!("{}", tree_item(source, is_last));
        }
    }

    // Positive and negative aspects
    if let Some(bed) = bedeutung {
        // Full mode or individual flags
        let show_positive = show_full || show_pos;
        let show_negative = show_full || show_neg;

        if show_positive {
            if let Some(pos) = &bed.licht {
                println!();
                println!("{}{} Positive", INDENT_BASE, SYMBOL_POSITIVE);
                let wrapped = wrap_text(pos, 4, None);
                println!("{}", wrapped);
            }
        }

        if show_negative {
            if let Some(neg) = &bed.schatten {
                println!();
                println!("{}{} Negative", INDENT_BASE, SYMBOL_NEGATIVE);
                let wrapped = wrap_text(neg, 4, None);
                println!("{}", wrapped);
            }
        }
    }

    // Blank line after each entry
    spacing(SPACING_SECTION);
}

// ============================================================================
// DATE OUTPUT
// ============================================================================

/// Format simple date output
pub fn format_date_simple(offset: i32, date_str: &str, value: u32, cipher: &str) {
    print!("{:+} {} {} {} {} [{}]",
        offset,
        MIDDOT,
        date_str,
        ARROW_RIGHT,
        value,
        cipher
    );
}

/// Format verbose date reduction
pub fn format_date_verbose(
    offset: i32,
    date_str: &str,
    cipher: &str,
    trace: &[String],
    final_value: u32,
) {
    println!("{:+} {} {} [{}]", offset, MIDDOT, date_str, cipher);

    // Print all calculation steps except the last line (which is "→ final_value")
    for line in trace.iter().take(trace.len().saturating_sub(1)) {
        if !line.starts_with('→') {
            println!("{}{}", INDENT_BASE, line);
        }
    }

    // Final result
    println!("{}{} {}", INDENT_BASE, ARROW_RIGHT, final_value);
}

// ============================================================================
// LIST CIPHERS OUTPUT
// ============================================================================

/// Format cipher list with alignment
pub fn format_cipher_list(ciphers: &[(String, String, String)]) {
    println!("Available Ciphers:");
    spacing(SPACING_SECTION);

    // Find max width for alignment
    let max_name_width = ciphers.iter().map(|(name, short, _)| {
        format!("{} ({})", name, short).len()
    }).max().unwrap_or(0);

    for (name, short, description) in ciphers {
        let label = format!("{} ({})", name, short);
        let padding = " ".repeat(max_name_width.saturating_sub(label.len()));
        println!("{}{}{}  {} {}", INDENT_BASE, label, padding, MIDDOT, description);
    }
}

// ============================================================================
// HELPER: Format multiple cipher results for one input
// ============================================================================

pub fn group_results_by_input<'a>(
    results: &'a [&KernResult],
) -> HashMap<&'a str, Vec<(&'a str, u32)>> {
    let mut grouped: HashMap<&str, Vec<(&str, u32)>> = HashMap::new();

    for result in results {
        grouped
            .entry(result.source.as_str())
            .or_insert_with(Vec::new)
            .push((result.cipher.as_str(), result.value()));
    }

    grouped
}

// ============================================================================
// SPEKTRA OUTPUT
// ============================================================================

/// Format and output SPEKTRA analysis prompt
/// Copies to clipboard if available, otherwise prints to stdout
pub fn format_spektra_output(prompt: &str) {
    // Try to copy to clipboard if clipboard feature is enabled
    #[cfg(feature = "clipboard")]
    {
        match arboard::Clipboard::new() {
            Ok(mut clipboard) => {
                match clipboard.set_text(prompt.to_string()) {
                    Ok(_) => {
                        // Show minimal confirmation
                        println!("{} Prompt in Zwischenablage kopiert", SYMBOL_POSITIVE);
                        return;
                    }
                    Err(_) => {
                        // Fall through to print the prompt
                    }
                }
            }
            Err(_) => {
                // Fall through to print the prompt
            }
        }
    }

    // Clipboard not available or failed - print the prompt to stdout
    #[cfg(not(feature = "clipboard"))]
    println!("{} SPEKTRA Prompt (Zwischenablage nicht verfügbar):", SYMBOL_POSITIVE);

    #[cfg(feature = "clipboard")]
    println!("{} SPEKTRA Prompt (Zwischenablage fehlgeschlagen):", SYMBOL_POSITIVE);

    println!();
    println!("{}", prompt);
}
