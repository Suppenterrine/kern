//! UI module for KERN CLI output formatting
//!
//! This module provides consistent formatting across all output modes.
//! Design principles: Fast, Direct, Concrete, Weighty, Precise

pub mod output;
pub mod theme;

// ============================================================================
// SPACING CONSTANTS
// ============================================================================

/// No blank lines - for related items in same group
pub const SPACING_NONE: usize = 0;

/// One blank line - between different logical sections
pub const SPACING_SECTION: usize = 1;

/// Two blank lines - between major mode changes
pub const SPACING_MODE: usize = 2;

// ============================================================================
// INDENTATION CONSTANTS
// ============================================================================

/// Base indentation unit (2 spaces)
pub const INDENT_BASE: &str = "  ";

/// Double indent for nested content
pub const INDENT_NESTED: &str = "    ";

// ============================================================================
// UNICODE SYMBOLS
// ============================================================================

/// Rightwards arrow for transformations (input → output)
pub const ARROW_RIGHT: &str = "→";

/// Leftwards arrow for reverse/results
pub const ARROW_LEFT: &str = "←";

/// Downwards arrow for flow
pub const ARROW_DOWN: &str = "↓";

/// Middle dot for separation
pub const MIDDOT: &str = "·";

/// Tree branch with siblings below
pub const TREE_BRANCH: &str = "├─";

/// Tree final branch
pub const TREE_LAST: &str = "└─";

/// Tree vertical continuation
pub const TREE_VERTICAL: &str = "│";

/// Circled plus for positive aspects
pub const SYMBOL_POSITIVE: &str = "⊕";

/// Circled minus for negative aspects
pub const SYMBOL_NEGATIVE: &str = "⊖";

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Print blank lines according to spacing constant
pub fn spacing(lines: usize) {
    for _ in 0..lines {
        println!();
    }
}

/// Check if stdout is a TTY (for color/formatting decisions)
pub fn is_tty() -> bool {
    use std::io::IsTerminal;
    std::io::stdout().is_terminal()
}

/// Get terminal width, fallback to 80 if not detectable
pub fn terminal_width() -> usize {
    if let Some((w, _)) = term_size::dimensions() {
        w
    } else {
        80
    }
}

/// Wrap text to fit terminal width
pub fn wrap_text(text: &str, indent: usize, max_width: Option<usize>) -> String {
    let width = max_width.unwrap_or_else(terminal_width);
    let available = width.saturating_sub(indent);

    if text.len() <= available {
        return text.to_string();
    }

    let mut result = String::new();
    let indent_str = " ".repeat(indent);
    let mut current_line = String::new();

    for word in text.split_whitespace() {
        if current_line.is_empty() {
            current_line = word.to_string();
        } else if current_line.len() + word.len() + 1 <= available {
            current_line.push(' ');
            current_line.push_str(word);
        } else {
            if !result.is_empty() {
                result.push('\n');
            }
            result.push_str(&indent_str);
            result.push_str(&current_line);
            current_line = word.to_string();
        }
    }

    if !current_line.is_empty() {
        if !result.is_empty() {
            result.push('\n');
        }
        result.push_str(&indent_str);
        result.push_str(&current_line);
    }

    result
}

/// Format a tree item (branch or last)
pub fn tree_item(content: &str, is_last: bool) -> String {
    let prefix = if is_last { TREE_LAST } else { TREE_BRANCH };
    format!("{}{} {}", INDENT_BASE, prefix, content)
}

/// Format a header with consistent style
pub fn format_header(text: &str) -> String {
    text.to_string()
}

/// Format an indented line
pub fn indent(text: &str, level: usize) -> String {
    let indent = INDENT_BASE.repeat(level);
    format!("{}{}", indent, text)
}
