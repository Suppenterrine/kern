use std::{fs, path::Path, collections::HashMap};

use clap::{Arg, ArgAction, Command};
use serde::Deserialize;
use prettytable::{Table, row};

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

fn lookup_row(table: &mut Table,
              zahl: u32,
              map: &HashMap<u32, Bedeutung>) {
    if let Some(txt) = map.get(&zahl).and_then(|b| b.text.as_deref()) {
        table.add_row(row![zahl, txt]);
    } else {
        table.add_row(row![zahl, "– keine Bedeutung –"]);
    }
}

fn main() {
    let matches = Command::new("kern")
        .version("0.1.0")
        .about(r#"
┌───────────────────────────┐
│   KERN v0.1.0 — PROTOKOLL │
└───────────────────────────┘

> decoding symbolic integers...
> interfacing with resonance layer...
> parsing STRUCTUR 83...

> SOMA CORE MODULES:
   [ HALTEKRAFT.PROCESSOR ] ......... OK
   [ TRAUMSCHATTEN.EXE ] ............ OK
   [ STIMULUS_MONITOR ] ............. OK (Caution: Overload Risk)
   [ MEMORY.DRIFT.REGULATOR ] ....... FAILED (Recovering)

> AUTHENTICATING USER: "WICKFELD_507"
   Retinal Echo Match ✓
   Pulse Resonance ✓
   Dream Residue ✓
      "#)
        .arg(
            Arg::new("lookup")
                .short('l')
                .long("lookup")
                .value_name("ZAHL")
                .num_args(1..)
                .value_delimiter(',')
                .help("Bedeutung einer Zahl anzeigen"),
        )
        .arg(
            Arg::new("length")
                .short('L')
                .long("length")
                .action(ArgAction::SetTrue)
                .help("Hängt die Zeichenlänge an die Ergebnis-Ausgabe an"),
        )
        .arg(
            Arg::new("ARGS")
                .num_args(1..)
                .help("Strings oder Zahlen zur Quersummen-Berechnung"),
        )
        .get_matches();

    /* --lookup: sofort Tabelle ausgeben ---------------------------------- */
    if let Some(list) = matches.get_many::<String>("lookup") {
        let map   = load_bedeutungen(Path::new("bedeutungen.yaml"));
        let mut t = Table::new();
        t.add_row(row!["Zahl", "Bedeutung"]);

        for raw in list {
            for part in raw.split(',') {
                let s = part.trim();
                if s.is_empty() { continue; }

                match s.parse::<u32>() {
                    Ok(n) => lookup_row(&mut t, n, &map),
                    Err(_) => eprintln!("Ignoriere ungültigen Wert: {s}"),
                }
            }
        }
        t.printstd();
        return;
    }

    /* ­--length Flag gesetzt? -------------------------------------------- */
    let show_length = matches.get_flag("length");

    /* Standard-Modus: Strings berechnen ---------------------------------- */
    if let Some(args) = matches.get_many::<String>("ARGS") {
        for arg in args {
            let total: u32 = arg.chars().map(char_to_value).sum();
            let reduced    = reduce_number(total);

            if show_length {
                let len = arg.chars().count();
                println!("{arg}: {reduced} ({len})");
            } else {
                println!("{arg}: {reduced}");
            }
        }
    } else {
        eprintln!("Bitte Argumente oder --lookup angeben!");
    }
}
