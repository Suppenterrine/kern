/// Numerical helper functions used by ciphers.

/// Sum of all values in the slice.
pub fn sum(values: &[u32]) -> u32 {
    values.iter().sum()
}

/// Sum of all decimal digits of `n`.
pub fn digit_sum(mut n: u32) -> u32 {
    let mut total = 0;
    while n > 0 {
        total += n % 10;
        n /= 10;
    }
    total
}

/// Reduce `n` by repeatedly applying [digit_sum] until a single digit or
/// a master number (11, 22, 33) remains.
pub fn reduce(mut n: u32) -> u32 {
    while n > 9 && !matches!(n, 11 | 22 | 33) {
        n = digit_sum(n);
    }
    n
}

/// Convenience function: sum the slice and then [reduce] the result.
pub fn reduce_values(values: &[u32]) -> u32 {
    let s = sum(values);
    reduce(s)
}
