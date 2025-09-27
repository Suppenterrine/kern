use crate::core::ciphers::Cipher;

/// Fibonacci cipher: A->1, B->1, C->2, D->3, E->5, ... sequence for letters.
pub struct FibonacciCipher;

fn first_n_fib(n: usize) -> Vec<u32> {
    let mut v = Vec::with_capacity(n);
    v.push(1);
    v.push(1);
    while v.len() < n {
        let l = v.len();
        let next = v[l - 1] + v[l - 2];
        v.push(next);
    }
    v
}

impl Cipher for FibonacciCipher {
    fn name(&self) -> &str {
        "fibonacci"
    }

    fn char_to_value(&self, ch: char) -> u32 {
        if ch.is_ascii_digit() {
            return ch.to_digit(10).unwrap_or(0);
        }
        let c = ch.to_ascii_uppercase();
        if ('A'..='Z').contains(&c) {
            let idx = (c as u8 - b'A') as usize; // 0-based
            let fib = first_n_fib(26);
            fib[idx]
        } else {
            0
        }
    }
}
