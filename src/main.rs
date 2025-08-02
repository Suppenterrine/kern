use std::env;

fn char_to_value(ch: char) -> u32 {
    // 0-9 → direkt nehmen
    if ch.is_ascii_digit() {
        return ch as u32 - '0' as u32;
    }
    // Uppercase
    if ch.is_ascii_uppercase() {
        return ch as u32 - 'A' as u32 + 1;
    }
    // Lowercase
    if ch.is_ascii_lowercase() {
        return ch as u32 - 'a' as u32 + 1;
    }
    0
}

fn reduce_number(mut n: u32) -> u32 {
    while n > 9 && n != 11 && n != 22 && n != 33 {
        let mut sum = 0;
        let mut tmp = n;
        while tmp > 0 {
            sum += tmp % 10;
            tmp /= 10;
        }
        n = sum;
    }
    n
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Bitte mindestens ein Argument angeben!");
        return;
    }

    for arg in &args[1..] {
        let total: u32 = arg.chars().map(char_to_value).sum();
        let result = reduce_number(total);
        println!("{}: {}", arg, result);
    }
}
