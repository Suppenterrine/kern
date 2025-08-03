use std::{collections::HashMap, fs, path::Path};

use chrono::{NaiveDate, Duration, Local};
use clap::{Arg, ArgAction, Command};
use prettytable::{Table, row};
use regex::Regex;
use serde::Deserialize;

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
        _ => 0,
    }
}

fn reduce_number(mut n: u32) -> u32 {
    while n > 9 && !matches!(n, 11 | 22 | 33) {
        n = n.to_string().chars().map(|c| c.to_digit(10).unwrap()).sum();
    }
    n
}

fn load_bedeutungen(path: &Path) -> HashMap<u32, Bedeutung> {
    let yaml = fs::read_to_string(path).expect("bedeutungen.yaml nicht gefunden");
    serde_yaml::from_str(&yaml).expect("YAML konnte nicht geparst werden")
}

fn lookup_row(table: &mut Table, zahl: u32, map: &HashMap<u32, Bedeutung>) {
    if let Some(txt) = map.get(&zahl).and_then(|b| b.text.as_deref()) {
        table.add_row(row![zahl, txt]);
    } else {
        table.add_row(row![zahl, "– keine Bedeutung –"]);
    }
}

fn main() {
    let version = env!("CARGO_PKG_VERSION");
    let about = format!(r#"
┌────────────────────────┐
│   KERN™CODE - v{version}   │
└────────────────────────┘

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
      "#);
    let matches = Command::new("kern")
        .version(env!("CARGO_PKG_VERSION"))
        .about(about)
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
            Arg::new("date")
                .short('d')
                .long("date")
                .value_name("RANGE")
                .allow_hyphen_values(true)
                .help(r#"Datums-Offset/Range:
    -3, +2, 0+3, 0-3, -5..4, 3..-2, etc.
    28.07.2025, 26.07.2025..02.08.2025"#),
        )
        .arg(
            Arg::new("ARGS")
                .num_args(1..)
                .help("Strings oder Zahlen zur Quersummen-Berechnung"),
        )
        .get_matches();

    /* --lookup: sofort Tabelle ausgeben ---------------------------------- */
    if let Some(list) = matches.get_many::<String>("lookup") {
        let map = load_bedeutungen(Path::new("bedeutungen.yaml"));
        let mut t = Table::new();
        t.add_row(row!["Zahl", "Bedeutung"]);

        for raw in list {
            for part in raw.split(',') {
                let s = part.trim();
                if s.is_empty() {
                    continue;
                }

                match s.parse::<u32>() {
                    Ok(n) => lookup_row(&mut t, n, &map),
                    Err(_) => eprintln!("Ignoriere ungültigen Wert: {s}"),
                }
            }
        }
        t.printstd();
        return;
    }

    fn parse_range(spec: &str) -> Result<Vec<i32>, String> {
        let today = Local::now().date_naive();

        // A) Datums-Range: dd.mm.yyyy..dd.mm.yyyy
        if let Some((start, end)) = spec.split_once("..") {
            if let (Ok(sd), Ok(ed)) = (
                NaiveDate::parse_from_str(start, "%d.%m.%Y"),
                NaiveDate::parse_from_str(end, "%d.%m.%Y"),
            ) {
                let s = (sd - today).num_days() as i32;
                let e = (ed - today).num_days() as i32;
                let mut v = Vec::new();
                if s <= e {
                    for i in s..=e {
                        v.push(i);
                    }
                } else {
                    for i in (e..=s).rev() {
                        v.push(i);
                    }
                }
                return Ok(v);
            }
        }

        // B) Einzel-Datum
        if let Ok(d) = NaiveDate::parse_from_str(spec, "%d.%m.%Y") {
            let off = (d - today).num_days() as i32;
            return Ok(vec![off]);
        }

        // A)  -5..4   oder   3..-2
        if let Some((a, b)) = spec.split_once("..") {
            let s: i32 = a.parse().map_err(|_| "Ungültiger Start")?;
            let e: i32 = b.parse().map_err(|_| "Ungültiges Ende")?;
            let mut v = Vec::new();
            if s <= e {
                for i in s..=e {
                    v.push(i);
                }
            } else {
                for i in (e..=s).rev() {
                    v.push(i);
                }
            }
            return Ok(v);
        }

        // B) alte Syntax  0+3 / 0-3
        let re = Regex::new(r"^([+-]?\d+)([+-])(\d+)$").unwrap();
        if let Some(c) = re.captures(spec) {
            let start: i32 = c[1].parse().unwrap();
            let end_off: i32 = c[3].parse().unwrap();
            let end = if &c[2] == "+" { end_off } else { -end_off };
            let mut v = Vec::new();
            if start <= end {
                for i in start..=end {
                    v.push(i);
                }
            } else {
                for i in (end..=start).rev() {
                    v.push(i);
                }
            }
            return Ok(v);
        }

        // C) Einzelwert
        spec.parse::<i32>()
            .map(|v| vec![v])
            .map_err(|_| "Ungültige Range-Angabe".into())
    }

    if let Some(dspec) = matches.get_one::<String>("date") {
        match parse_range(dspec) {
            Ok(offsets) => {
                let map = load_bedeutungen(Path::new("bedeutungen.yaml"));
                let mut t = Table::new();
                t.add_row(row!["Offset", "Datum", "Summe", "Bedeutung"]);

                let today = Local::now().date_naive();

                for off in offsets {
                    let date = today + Duration::days(off as i64);
                    let num = reduce_number(
                        date.format("%d%m%Y")
                            .to_string()
                            .chars()
                            .map(|c| c.to_digit(10).unwrap() as u32)
                            .sum(),
                    );
                    let text = map.get(&num).and_then(|b| b.text.as_deref()).unwrap_or("–");
                    t.add_row(row![
                        format!("{:+}", off),
                        date.format("%d.%m.%Y"),
                        num,
                        text
                    ]);
                }
                t.printstd();
            }
            Err(e) => eprintln!("{e}"),
        }
        // kein return; Berechnung von ARGS darf danach weiterlaufen
    }

    /* ­--length Flag gesetzt? -------------------------------------------- */
    let show_length = matches.get_flag("length");

    /* Standard-Modus: Strings berechnen ---------------------------------- */
    if let Some(args) = matches.get_many::<String>("ARGS") {
        for arg in args {
            let total: u32 = arg.chars().map(char_to_value).sum();
            let reduced = reduce_number(total);

            if show_length {
                let len = arg.chars().count();
                println!("{arg}: {reduced} ({len})");
            } else {
                println!("{arg}: {reduced}");
            }
        }
    } else {
        eprintln!("Keine weiteren Argumente angegeben.");
    }
}
