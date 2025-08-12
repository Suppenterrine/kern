use kern::reduction::{
    to_ordinal, to_reverse_ordinal, to_pythagorean, to_reverse_pythagorean,
    reduce, reduce_values,
};

#[test]
fn test_letter_mappings() {
    assert_eq!(to_ordinal("abc"), vec![1, 2, 3]);
    assert_eq!(to_reverse_ordinal("abc"), vec![26, 25, 24]);
    assert_eq!(to_pythagorean("j"), vec![1]); // J -> 10 -> 1
    assert_eq!(to_reverse_pythagorean("z"), vec![1]);
}

#[test]
fn test_number_reduction() {
    assert_eq!(reduce(654), 6);
    assert_eq!(reduce(11), 11); // master number
    let values = to_ordinal("feldmann");
    assert_eq!(reduce_values(&values), 6);
}

#[test]
fn test_edge_cases() {
    assert!(to_ordinal("!@#").is_empty());
    let values = to_ordinal("");
    assert_eq!(reduce_values(&values), 0);
}
