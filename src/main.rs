use chrono::{Duration, Local, Utc};
use clap::{Arg, ArgAction, Command, value_parser};
use kern::core::sky;
use kern::core::{
    KernResult, ResultSet, Step, load_bedeutungen, lookup, parse_range, reduce_number_verbose,
};
use prettytable::{Cell, Row, Table, row};
use serde_json;

fn main() {
    let version = env!("CARGO_PKG_VERSION");
    let about = format!(
        r#"
┌────────────────────────┐
│   KERN™CODE - v{version}   │
└────────────────────────┘

> SOMA CORE MODULES:
   [ HALTEKRAFT.PROCESSOR ] ......... OK
   [ TRAUMSCHATTEN.EXE ] ............ OK
   [ STIMULUS_MONITOR ] ............. OK (Caution: Overload Risk)
   [ MEMORY.DRIFT.REGULATOR ] ....... FAILED (Recovering)
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
            Arg::new("licht")
                .long("licht")
                .action(ArgAction::SetTrue)
                .help("Zeigt auch die Lichtseite in der Lookup-Tabelle"),
        )
        .arg(
            Arg::new("schatten")
                .long("schatten")
                .action(ArgAction::SetTrue)
                .help("Zeigt auch die Schattenseite in der Lookup-Tabelle"),
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
        .subcommand(
            Command::new("sky")
                .about("Wetter + Sonnenstand kombiniert")
                .arg(
                    Arg::new("lat")
                        .long("lat")
                        .required(true)
                        .value_parser(value_parser!(f64)),
                )
                .arg(
                    Arg::new("lon")
                        .long("lon")
                        .required(true)
                        .value_parser(value_parser!(f64)),
                )
                .arg(
                    Arg::new("time").long("time").value_name("ISO8601"), // optional, z.B. 2025-08-12T07:30:00+00:00
                ),
        )
        .get_matches();

    if let Some((cmd, sub_m)) = matches.subcommand() {
        match cmd {
            "sky" => {
                let lat = *sub_m.get_one::<f64>("lat").unwrap();
                let lon = *sub_m.get_one::<f64>("lon").unwrap();
                let dt = if let Some(t) = sub_m.get_one::<String>("time") {
                    chrono::DateTime::parse_from_rfc3339(t)
                        .map(|d| d.with_timezone(&Utc))
                        .unwrap()
                } else {
                    Utc::now()
                };
                match sky::report(lat, lon, Some(dt)) {
                    Ok(r) => println!("{}", serde_json::to_string_pretty(&r).unwrap()),
                    Err(e) => eprintln!("Fehler: {e}"),
                }
                return;
            }
            _ => {}
        }
    }

    /* --lookup: sofort Tabelle ausgeben ---------------------------------- */
    if let Some(list) = matches.get_many::<String>("lookup") {
        let map = load_bedeutungen();
        let mut t = Table::new();
        let show_licht = matches.get_flag("licht");
        let show_schatten = matches.get_flag("schatten");

        let mut header = vec![Cell::new("Zahl"), Cell::new("Bedeutung")];
        if show_licht {
            header.push(Cell::new("Lichtseite"));
        }
        if show_schatten {
            header.push(Cell::new("Schattenseite"));
        }
        t.add_row(Row::new(header));

        for raw in list {
            for part in raw.split(',') {
                let s = part.trim();
                if s.is_empty() {
                    continue;
                }

                match s.parse::<u32>() {
                    Ok(n) => {
                        let text = lookup(n, &map);
                        let entry = map.get(&n);
                        let mut cells = vec![Cell::new(&n.to_string()), Cell::new(text)];
                        if show_licht {
                            let l = entry.and_then(|b| b.licht.as_deref()).unwrap_or("-");
                            cells.push(Cell::new(l));
                        }
                        if show_schatten {
                            let s = entry.and_then(|b| b.schatten.as_deref()).unwrap_or("-");
                            cells.push(Cell::new(s));
                        }
                        t.add_row(Row::new(cells));
                    }
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
                let map = load_bedeutungen();
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
        let mut result_set = ResultSet::new(); // linear flow collects all calculation results

        for (pipe_index, arg) in args.enumerate() {
            let step = Step::new(pipe_index, 0, "reduce");
            let calc_result = KernResult::from_input_default(arg, debug, step);

            if debug {
                for line in &calc_result.trace {
                    println!("{line}");
                }
            } else if show_length {
                let len = arg.chars().count();
                println!("{arg}: {} ({len})", calc_result.value());
            } else {
                println!("{arg}: {}", calc_result.value());
            }

            result_set.add(calc_result);
        }

        if show_total {
            let sum: u32 = result_set.total();
            if debug {
                let parts: Vec<String> = result_set.values().map(|v| v.to_string()).collect();
                println!("\n\u{1a} Gesamtsumme: ({}) = {}", parts.join("+"), sum);
            }

            let total_step = Step::new(result_set.len(), 0, "aggregate::total");
            let total_result = KernResult::from_numeric_value_default(sum, debug, total_step);

            if debug {
                for line in &total_result.trace {
                    println!("{line}");
                }
                println!("\u{1a} Gesamtsumme: {sum} \u{1a} {}", total_result.value());
            } else {
                println!("Gesamtsumme: {sum} \u{1a} {}", total_result.value());
            }

            result_set.add(total_result);
        }
    } else {
        eprintln!("Keine weiteren Argumente angegeben.");
    }
}
