//! PRM Matrix Visualization Module
//!
//! Renders Pythagorean Resonance Matrix (PRM) with two matrix types:
//! - MATRIX 1: Horizontal layout (1,2,3 / 4,5,6 / 7,8,9)
//! - MATRIX 2: Vertical layout (1,4,7 / 2,5,8 / 3,6,9)

use crate::core::PrmMatrixData;

#[cfg(feature = "prm-colors")]
use owo_colors::OwoColorize;

/// Matrix type for PRM visualization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatrixType {
    /// Horizontal layout: rows are [1,2,3], [4,5,6], [7,8,9]
    Horizontal,
    /// Vertical layout: rows are [1,4,7], [2,5,8], [3,6,9]
    Vertical,
}

/// Color palette for matrix abbreviations (when prm-colors feature is enabled)
#[cfg(feature = "prm-colors")]
const COLORS: &[owo_colors::AnsiColors] = &[
    owo_colors::AnsiColors::Red,
    owo_colors::AnsiColors::Blue,
    owo_colors::AnsiColors::Green,
    owo_colors::AnsiColors::Yellow,
    owo_colors::AnsiColors::Cyan,
    owo_colors::AnsiColors::Magenta,
    owo_colors::AnsiColors::BrightRed,
    owo_colors::AnsiColors::BrightMagenta,
    owo_colors::AnsiColors::BrightCyan,
];

/// Calculate matrix position (row, col) for a given compartment and value
///
/// # Arguments
/// * `value` - The reduced numerological value (1-9, 11, 22, 33)
/// * `compartment` - The compartment (1-3)
/// * `matrix_type` - Horizontal or Vertical layout
///
/// # Returns
/// (row_index, col_index) in a 3×3 matrix
fn calculate_matrix_position(value: u32, _compartment: u32, matrix_type: MatrixType) -> (usize, usize) {
    // Map master numbers to their base digit for positioning
    let pos_value = match value {
        11 => 1,
        22 => 2,
        33 => 3,
        _ => value,
    };

    match matrix_type {
        MatrixType::Horizontal => {
            // Matrix 1: Horizontal
            // 1,2,3 / 4,5,6 / 7,8,9
            let row = ((pos_value - 1) / 3) as usize;
            let col = ((pos_value - 1) % 3) as usize;
            (row, col)
        }
        MatrixType::Vertical => {
            // Matrix 2: Vertical
            // 1,4,7 / 2,5,8 / 3,6,9
            let col = ((pos_value - 1) / 3) as usize;
            let row = ((pos_value - 1) % 3) as usize;
            (row, col)
        }
    }
}

/// Render a single 3×3 matrix with abbreviations placed in compartments
fn render_matrix(
    matrix_type: MatrixType,
    data: &PrmMatrixData,
) -> String {
    // Initialize 3x3 matrix with empty cells
    let mut cells: [[Option<(String, usize)>; 3]; 3] = Default::default();

    // Place first letters (not full abbreviations) in matrix
    for (idx, (input, compartment, value)) in data
        .inputs
        .iter()
        .zip(&data.compartments)
        .zip(&data.values)
        .map(|((i, c), v)| (i, c, v))
        .enumerate()
    {
        let (row, col) = calculate_matrix_position(*value, *compartment, matrix_type);
        // Use only first letter, not full abbreviation
        let first_letter = input
            .chars()
            .next()
            .unwrap_or('x')
            .to_lowercase()
            .to_string();
        cells[row][col] = Some((first_letter, idx));
    }

    // Build matrix string
    let mut output = String::new();

    // Matrix header
    let header = match matrix_type {
        MatrixType::Horizontal => "MATRIX 1",
        MatrixType::Vertical => "MATRIX 2",
    };
    output.push_str(header);
    output.push('\n');

    // Number grid
    let numbers = match matrix_type {
        MatrixType::Horizontal => vec!["1  2  3", "4  5  6", "7  8  9"],
        MatrixType::Vertical => vec!["1  4  7", "2  5  8", "3  6  9"],
    };

    for line in numbers {
        output.push_str(" ");
        output.push_str(line);
        output.push('\n');
    }

    output.push('\n');

    // Letter grid
    for row in &cells {
        output.push(' ');
        for (col_idx, cell) in row.iter().enumerate() {
            let cell_str = if let Some((letter, color_idx)) = cell {
                format_letter(letter, *color_idx)
            } else {
                "·".to_string()
            };

            output.push_str(&cell_str);

            // Add spacing between columns (consistent with number grid)
            if col_idx < 2 {
                output.push_str("  ");
            }
        }
        output.push('\n');
    }

    output
}

/// Format a letter with optional color
#[cfg(feature = "prm-colors")]
fn format_letter(letter: &str, color_idx: usize) -> String {
    let color = COLORS[color_idx % COLORS.len()];
    letter.color(color).bold().to_string()
}

#[cfg(not(feature = "prm-colors"))]
fn format_letter(letter: &str, _color_idx: usize) -> String {
    letter.to_string()
}

/// Render the legend showing only colored first letters
fn render_legend(data: &PrmMatrixData) -> String {
    let mut legend = String::new();

    for (idx, input) in data.inputs.iter().enumerate() {
        if idx > 0 {
            legend.push(' ');
        }

        // Extract first letter from input
        let first_letter = input
            .chars()
            .next()
            .unwrap_or('x')
            .to_lowercase()
            .to_string();

        legend.push_str(&format_letter(&first_letter, idx));
    }

    legend
}

/// Render complete PRM matrix visualization with only horizontal matrix
pub fn render_prm_matrices(data: &PrmMatrixData) -> String {
    // Render only MATRIX 1 (Horizontal), without legend
    render_matrix(MatrixType::Horizontal, data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matrix_position_horizontal() {
        // Value 1 should be at (0,0)
        assert_eq!(calculate_matrix_position(1, 1, MatrixType::Horizontal), (0, 0));
        // Value 5 should be at (1,1) - middle
        assert_eq!(calculate_matrix_position(5, 2, MatrixType::Horizontal), (1, 1));
        // Value 9 should be at (2,2) - bottom right
        assert_eq!(calculate_matrix_position(9, 3, MatrixType::Horizontal), (2, 2));
    }

    #[test]
    fn test_matrix_position_vertical() {
        // Value 1 should be at (0,0)
        assert_eq!(calculate_matrix_position(1, 1, MatrixType::Vertical), (0, 0));
        // Value 5 should be at (1,1) - middle
        assert_eq!(calculate_matrix_position(5, 2, MatrixType::Vertical), (1, 1));
        // Value 9 should be at (2,2) - bottom right
        assert_eq!(calculate_matrix_position(9, 3, MatrixType::Vertical), (2, 2));
    }

    #[test]
    fn test_matrix_position_master_numbers() {
        // Master number 11 should map to position of 1
        assert_eq!(calculate_matrix_position(11, 1, MatrixType::Horizontal), (0, 0));
        // Master number 22 should map to position of 2
        assert_eq!(calculate_matrix_position(22, 2, MatrixType::Horizontal), (0, 1));
        // Master number 33 should map to position of 3
        assert_eq!(calculate_matrix_position(33, 3, MatrixType::Horizontal), (0, 2));
    }

    #[test]
    fn test_render_legend() {
        let data = PrmMatrixData {
            inputs: vec!["zucker".to_string(), "wasser".to_string()],
            abbreviations: vec!["z1".to_string(), "w2".to_string()],
            compartments: vec![1, 2],
            values: vec![1, 2],
            cipher: "ordinal".to_string(),
        };

        let legend = render_legend(&data);
        // Legend should now only contain first letters (z w), not full words
        // May have ANSI color codes if prm-colors feature enabled
        assert!(legend.contains("z") || legend.contains("\x1b"));
        assert!(legend.contains("w") || legend.contains("\x1b"));
        // Should NOT contain full words anymore
        assert!(!legend.contains("zucker"));
        assert!(!legend.contains("wasser"));
    }

    #[test]
    fn test_render_matrix_basic() {
        let data = PrmMatrixData {
            inputs: vec!["test".to_string()],
            abbreviations: vec!["t1".to_string()],
            compartments: vec![2],
            values: vec![5],
            cipher: "ordinal".to_string(),
        };

        let matrix = render_matrix(MatrixType::Horizontal, &data);
        assert!(matrix.contains("MATRIX 1"));
        assert!(matrix.contains("1  2  3"));
        // Matrix now shows only first letter, not full abbreviation
        assert!(matrix.contains("t") || matrix.contains("\x1b")); // 't' or ANSI codes
    }
}
