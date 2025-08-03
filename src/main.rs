use std::{path::Path};
use chrono::{Duration, Local};
use clap::{Arg, ArgAction, Command};
use prettytable::{Table, row};
use kern::core::*;


fn main() {
    let version = env!("CARGO_PKG_VERSION");
    let about = format!(
        r#"
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
      "#
    );
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
                .help(
                    r#"Datums-Offset/Range:
    -3, +2, 0+3, 0-3, -5..4, 3..-2, etc.
    28.07.2025, 26.07.2025..02.08.2025"#,
                ),
        )
        .arg(
            Arg::new("debug")
                .short('v')
                .long("verbose")
                .action(ArgAction::SetTrue)
                .help("Zeigt die vollständige Reduktionskette für jede Eingabe"),
        )
        .arg(
            Arg::new("total")
                .short('t')
                .long("total")
                .action(ArgAction::SetTrue)
                .help("Zeigt die Gesamtsumme aller Ergebnisse (reduziert)"),
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
                    Ok(n) => {
                        let text = lookup(n, &map);
                        t.add_row(row![n, text]);
                    },
                    Err(_) => eprintln!("Ignoriere ungültigen Wert: {s}"),
                }
            }
        }
        t.printstd();
        return;
    }

    

    let debug = matches.get_flag("debug");

    if let Some(dspec) = matches.get_one::<String>("date") {
        match parse_range(dspec) {
            Ok(offsets) => {
                let map = load_bedeutungen(Path::new("bedeutungen.yaml"));
                let mut t = Table::new();
                t.add_row(row!["Offset", "Datum", "Summe", "Bedeutung"]);

                let today = Local::now().date_naive();

                for off in offsets {
                    let date = today + Duration::days(off as i64);

                    // Debug/Normal trennen
                    let date_str = date.format("%d%m%Y").to_string();
                    let num = reduce_number_verbose(&date_str, debug);

                    if !debug {
                        let text = map.get(&num).and_then(|b| b.text.as_deref()).unwrap_or("–");
                        t.add_row(row![
                            format!("{:+}", off),
                            date.format("%d.%m.%Y"),
                            num,
                            text
                        ]);
                    }
                }

                if !debug {
                    t.printstd();
                }
            }
            Err(e) => eprintln!("{e}"),
        }
    }

    /* ­--length Flag gesetzt? -------------------------------------------- */
    let show_total = matches.get_flag("total");
    let show_length = matches.get_flag("length");
    let debug = matches.get_flag("debug");

    /* Standard-Modus: Strings berechnen ---------------------------------- */
    if let Some(args) = matches.get_many::<String>("ARGS") {
        let mut results = Vec::new(); // ← Ergebnisse sammeln

        for arg in args {
            if debug {
                // Debugmodus → nur Reduktionskette
                let val = reduce_number_verbose(arg, true);
                results.push(val);
            } else {
                // Normale Ausgabe
                let reduced = reduce_number_verbose(arg, false);
                if show_length {
                    let len = arg.chars().count();
                    println!("{arg}: {reduced} ({len})");
                } else {
                    println!("{arg}: {reduced}");
                }
                results.push(reduced);
            }
        }

        // Nach allen Argumenten → Gesamtsumme
        if show_total {
            let sum: u32 = results.iter().sum();
            if debug {
                // Gesamtkette aus den Einzelergebnissen zeigen
                let parts: Vec<String> = results.iter().map(|v| v.to_string()).collect();
                println!(
                    "\n→ Gesamtsumme: ({}) = {}",
                    parts.join("+"),
                    sum
                );
            }

            let reduced_total = reduce_number_verbose(&sum.to_string(), debug);
            if debug {
                println!("→ Gesamtsumme: {sum} → {reduced_total}");
            } else {
                println!("Gesamtsumme: {sum} → {reduced_total}");
            }
        }
    } else {
        eprintln!("Keine weiteren Argumente angegeben.");
    }
}
