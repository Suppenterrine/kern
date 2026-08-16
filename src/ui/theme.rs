//! Color theme system for KERN CLI
//!
//! Infrastructure is prepared but colors are NOT yet activated in output.
//! This will be enabled in a future update.

use std::env;

/// Color theme for CLI output
#[derive(Debug, Clone)]
pub struct Theme {
    pub enabled: bool,
    pub number: &'static str,
    pub meaning: &'static str,
    pub cipher: &'static str,
    pub structure: &'static str,
    pub positive: &'static str,
    pub negative: &'static str,
    pub reset: &'static str,
}

impl Theme {
    /// Create a theme with colors disabled
    pub fn no_color() -> Self {
        Self {
            enabled: false,
            number: "",
            meaning: "",
            cipher: "",
            structure: "",
            positive: "",
            negative: "",
            reset: "",
        }
    }

    /// Create default theme (for future activation)
    pub fn default_theme() -> Self {
        Self {
            enabled: true,
            number: "\x1b[1;36m",        // Bold Cyan
            meaning: "\x1b[0m",          // Normal
            cipher: "\x1b[2m",           // Dim
            structure: "\x1b[90m",       // Dark Gray
            positive: "\x1b[32m",        // Green
            negative: "\x1b[31m",        // Red
            reset: "\x1b[0m",            // Reset
        }
    }

    /// Detect if colors should be enabled based on environment
    pub fn auto_detect() -> Self {
        // Check NO_COLOR environment variable
        if env::var("NO_COLOR").is_ok() {
            return Self::no_color();
        }

        // Check if stdout is a TTY
        if super::is_tty() {
            Self::default_theme()
        } else {
            Self::no_color()
        }
    }
}

/// Global theme - currently always disabled
pub fn current_theme() -> Theme {
    // TODO: Enable in future update with --color flag support
    Theme::no_color()
}
