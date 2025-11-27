//! Phase Relation Module
//!
//! Implements the phase relation system for numerological analysis.
//!
//! ## Compartment System
//! - Compartment 1: [1, 4, 7] + Masterzahl 11
//! - Compartment 2: [2, 5, 8] + Masterzahl 22
//! - Compartment 3: [3, 6, 9] + Masterzahl 33
//!
//! ## Phase Relations
//! - Phase 0: Same compartment (synchronous)
//! - Phase +1: Forward movement (1→2, 2→3, 3→1)
//! - Phase -1: Backward movement (1→3, 2→1, 3→2)

use serde::Serialize;

/// Calculate which compartment a reduced numerological value belongs to
///
/// # Arguments
/// * `value` - A reduced numerological value (1-9, 11, 22, 33)
///
/// # Returns
/// The compartment number (1, 2, or 3)
///
/// # Examples
/// ```
/// use kern::core::calculate_compartment;
/// assert_eq!(calculate_compartment(1), 1);
/// assert_eq!(calculate_compartment(5), 2);
/// assert_eq!(calculate_compartment(11), 1);  // Masterzahl
/// ```
pub fn calculate_compartment(value: u32) -> u32 {
    match value {
        // Compartment 1: [1, 4, 7] + Masterzahl 11
        1 | 4 | 7 | 11 => 1,

        // Compartment 2: [2, 5, 8] + Masterzahl 22
        2 | 5 | 8 | 22 => 2,

        // Compartment 3: [3, 6, 9] + Masterzahl 33
        3 | 6 | 9 | 33 => 3,

        _ => {
            // This should never happen with properly reduced values
            eprintln!("Warning: Unexpected value {} for compartment calculation", value);
            // Default to compartment based on modulo 3
            let remainder = value % 3;
            if remainder == 0 { 3 } else { remainder }
        }
    }
}

/// Calculate the phase relation between two compartments
///
/// # Arguments
/// * `comp_a` - Compartment of first value (1-3)
/// * `comp_b` - Compartment of second value (1-3)
///
/// # Returns
/// Phase relation: -1, 0, or +1
///
/// # Phase Logic
/// - Phase 0: Same compartment
/// - Phase +1: Forward movement (1→2, 2→3, 3→1)
/// - Phase -1: Backward movement (1→3, 2→1, 3→2)
pub fn calculate_phase(comp_a: u32, comp_b: u32) -> i32 {
    if comp_a == comp_b {
        // Same compartment: synchronous/unmoved
        0
    } else if (comp_a == 1 && comp_b == 2) ||
              (comp_a == 2 && comp_b == 3) ||
              (comp_a == 3 && comp_b == 1) {
        // Forward movement: Integration
        1
    } else {
        // Backward movement: Polarity
        -1
    }
}

/// Generate all unique pairs from a list of indices
///
/// # Arguments
/// * `count` - Number of elements
///
/// # Returns
/// Vector of (index_a, index_b) pairs where index_a < index_b
///
/// # Example
/// For count=3, generates: [(0,1), (0,2), (1,2)]
pub fn generate_matrix_pairs(count: usize) -> Vec<(usize, usize)> {
    let mut pairs = Vec::new();
    for i in 0..count {
        for j in (i + 1)..count {
            pairs.push((i, j));
        }
    }
    pairs
}

/// Result of a single phase relation calculation
#[derive(Debug, Clone, Serialize)]
pub struct PhaseRelationResult {
    /// First input string
    pub left_input: String,
    /// Second input string
    pub right_input: String,
    /// Reduced value of first input
    pub left_value: u32,
    /// Reduced value of second input
    pub right_value: u32,
    /// Compartment of first value (1-3)
    pub left_compartment: u32,
    /// Compartment of second value (1-3)
    pub right_compartment: u32,
    /// Phase relation (-1, 0, +1)
    pub phase: i32,
    /// Cipher used for calculation
    pub cipher: String,
}

impl PhaseRelationResult {
    pub fn new(
        left_input: String,
        right_input: String,
        left_value: u32,
        right_value: u32,
        cipher: String,
    ) -> Self {
        let left_compartment = calculate_compartment(left_value);
        let right_compartment = calculate_compartment(right_value);
        let phase = calculate_phase(left_compartment, right_compartment);

        Self {
            left_input,
            right_input,
            left_value,
            right_value,
            left_compartment,
            right_compartment,
            phase,
            cipher,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compartment_basic() {
        // Compartment 1: [1, 4, 7]
        assert_eq!(calculate_compartment(1), 1);
        assert_eq!(calculate_compartment(4), 1);
        assert_eq!(calculate_compartment(7), 1);

        // Compartment 2: [2, 5, 8]
        assert_eq!(calculate_compartment(2), 2);
        assert_eq!(calculate_compartment(5), 2);
        assert_eq!(calculate_compartment(8), 2);

        // Compartment 3: [3, 6, 9]
        assert_eq!(calculate_compartment(3), 3);
        assert_eq!(calculate_compartment(6), 3);
        assert_eq!(calculate_compartment(9), 3);
    }

    #[test]
    fn test_compartment_masterzahlen() {
        // Masterzahlen
        assert_eq!(calculate_compartment(11), 1);  // 11 → Compartment 1
        assert_eq!(calculate_compartment(22), 2);  // 22 → Compartment 2
        assert_eq!(calculate_compartment(33), 3);  // 33 → Compartment 3
    }

    #[test]
    fn test_phase_synchronous() {
        // Same compartment = Phase 0
        assert_eq!(calculate_phase(1, 1), 0);
        assert_eq!(calculate_phase(2, 2), 0);
        assert_eq!(calculate_phase(3, 3), 0);
    }

    #[test]
    fn test_phase_forward() {
        // Forward movement = Phase +1
        assert_eq!(calculate_phase(1, 2), 1);  // 1→2
        assert_eq!(calculate_phase(2, 3), 1);  // 2→3
        assert_eq!(calculate_phase(3, 1), 1);  // 3→1 (wrap around)
    }

    #[test]
    fn test_phase_backward() {
        // Backward movement = Phase -1
        assert_eq!(calculate_phase(1, 3), -1);  // 1→3 (backward)
        assert_eq!(calculate_phase(2, 1), -1);  // 2→1
        assert_eq!(calculate_phase(3, 2), -1);  // 3→2
    }

    #[test]
    fn test_generate_matrix_pairs() {
        // 2 elements: 1 pair
        assert_eq!(generate_matrix_pairs(2), vec![(0, 1)]);

        // 3 elements: 3 pairs
        assert_eq!(
            generate_matrix_pairs(3),
            vec![(0, 1), (0, 2), (1, 2)]
        );

        // 4 elements: 6 pairs
        assert_eq!(
            generate_matrix_pairs(4),
            vec![(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)]
        );
    }

    #[test]
    fn test_phase_relation_result() {
        let result = PhaseRelationResult::new(
            "a".to_string(),
            "b".to_string(),
            1,  // a = 1 → Compartment 1
            2,  // b = 2 → Compartment 2
            "ordinal".to_string(),
        );

        assert_eq!(result.left_compartment, 1);
        assert_eq!(result.right_compartment, 2);
        assert_eq!(result.phase, 1);  // 1→2 is forward (+1)
    }
}
