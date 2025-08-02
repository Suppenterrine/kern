use std::{fs, path::Path, collections::HashMap};

use clap::{Arg, Command};
use serde::Deserialize;
use prettytable::{Table, row};   // cell-Macro nicht genutzt → raus

#[derive(Debug, Deserialize)]
struct Bedeutung {
    #[serde(alias = "bedeutung")]
    text: Option<String>,
}

fn char_to_value(ch: char) -> u32 {
    match ch {
        '0'..='9' => ch as u32 - '0' as u32,
        'A'..='Z' => ch as u32 - 'A' as u32 + 1,
        'a'..='z' => ch as u32 - 'a' as u32 + 1,
        _         => 0,
    }
}

fn reduce_number(mut n: u32) -> u32 {
    while n > 9 && !matches!(n, 11 | 22 | 33) {
        n = n.to_string()
             .chars()
             .map(|c| c.to_digit(10).unwrap())
             .sum();
    }
    n
}

fn load_bedeutungen(path: &Path) -> HashMap<u32, Bedeutung> {
    let yaml = fs::read_to_string(path)
        .expect("bedeutungen.yaml nicht gefunden");
    serde_yaml::from_str(&yaml)
        .expect("YAML konnte nicht geparst werden")
}

fn lookup(bedeutungen: &HashMap<u32, Bedeutung>, zahl: u32) {
    if let Some(b) = bedeutungen.get(&zahl) {
        let mut t = Table::new();
        t.add_row(row!["Zahl", "Bedeutung"]);
        if let Some(txt) = &b.text {
            t.add_row(row![zahl, txt]);
        }
        t.printstd();
    } else {
        println!("Keine Bedeutung für {} gefunden!", zahl);
    }
}

fn main() {
    let matches = Command::new("kern")
        .version("0.1.0")
        .about("Numerologie-Tool")
        .arg(
            Arg::new("lookup")
                .short('l')
                .long("lookup")
                .value_name("ZAHL")
                .help("Bedeutung einer Zahl anzeigen"),
        )
        .arg(
            Arg::new("ARGS")
                .num_args(1..)
                .help("Strings oder Zahlen zur Quersummen-Berechnung"),
        )
        .get_matches();

    if let Some(n) = matches.get_one::<String>("lookup") {
        let zahl: u32 = n.parse().expect("Ungültige Zahl für Lookup");
        let map = load_bedeutungen(Path::new("bedeutungen.yaml"));
        lookup(&map, zahl);
        return;
    }

    if let Some(args) = matches.get_many::<String>("ARGS") {
        for arg in args {
            let total: u32 = arg.chars().map(char_to_value).sum();
            println!("{arg}: {}", reduce_number(total));
        }
    } else {
        eprintln!("Bitte Argumente oder --lookup angeben!");
    }
}
