use chrono::{Duration, Local, Utc};
use clap::{Arg, ArgAction, Command, value_parser};
use kern::core::sky;
use kern::core::{
    Cipher, KernResult, ResultSet, Step, default_cipher, descriptors, get_cipher, load_bedeutungen,
    parse_range,
};
use prettytable::{Cell, Row, Table};
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
                .action(ArgAction::SetTrue)
                .help("Prints lookup meanings for all reduced results"),
        )
        .arg(
            Arg::new("list-ciphers")
                .long("list-ciphers")
                .action(ArgAction::SetTrue)
                .help("Lists all available ciphers and exits"),
        )
        .arg(
            Arg::new("cipher")
                .short('c')
                .long("cipher")
                .value_name("CIPHER")
                .action(ArgAction::Append)
                .value_delimiter(',')
                .help("Cipher(s) to use (repeatable). Use cipher name, shortcode or 'all'"),
        )
        .arg(
            Arg::new("light")
                .long("light")
                .action(ArgAction::SetTrue)
                .help("Shows the light meaning column in lookup output"),
        )
        .arg(
            Arg::new("shadow")
                .long("shadow")
                .action(ArgAction::SetTrue)
                .help("Shows the shadow meaning column in lookup output"),
        )
        .arg(
            Arg::new("length")
                .short('L')
                .long("length")
                .action(ArgAction::SetTrue)
                .help("Appends the character length to the result output"),
        )
        .arg(
            Arg::new("date")
                .short('d')
                .long("date")
                .value_name("RANGE")
                .allow_hyphen_values(true)
                .help(
                    r#"Date-Offset/Range:
    -3, +2, 0+3, 0-3, -5..4, 3..-2, etc.
    28.07.2025, 26.07.2025..02.08.2025"#,
                ),
        )
        .arg(
            Arg::new("debug")
                .short('v')
                .long("verbose")
                .action(ArgAction::SetTrue)
                .help("Show detailed calculation trace"),
        )
        .arg(
            Arg::new("total")
                .short('t')
                .long("total")
                .action(ArgAction::SetTrue)
                .help("Shows the total sum of all reduced results at the end"),
        )
        .arg(
            Arg::new("ARGS")
                .num_args(1..)
                .help("Input strings to be reduced"),
        )
        .subcommand(
            Command::new("sky")
                .about("Fetches sky data for given location and time")
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

    let debug = matches.get_flag("debug");
    let show_total = matches.get_flag("total");
    let show_length = matches.get_flag("length");
    let show_lookup = matches.get_flag("lookup");
    let show_light = matches.get_flag("light");
    let show_shadow = matches.get_flag("shadow");

    if matches.get_flag("list-ciphers") {
        let mut table = Table::new();
        table.add_row(Row::new(vec![
            Cell::new("Name"),
            Cell::new("Short"),
            Cell::new("Description"),
        ]));

        for descriptor in descriptors() {
            table.add_row(Row::new(vec![
                Cell::new(descriptor.name),
                Cell::new(descriptor.short),
                Cell::new(descriptor.description),
            ]));
        }

        table.printstd();
        return;
    }

    let mut selected_ciphers: Vec<Box<dyn Cipher>> = Vec::new();
    if let Some(values) = matches.get_many::<String>("cipher") {
        let values_vec: Vec<String> = values.map(|s| s.to_string()).collect();

        if values_vec.iter().any(|v| v.eq_ignore_ascii_case("all")) {
            for descriptor in descriptors() {
                selected_ciphers.push((descriptor.factory)());
            }
        } else {
            for value in values_vec {
                match get_cipher(&value) {
                    Some(cipher) => selected_ciphers.push(cipher),
                    None => {
                        let available: Vec<String> = descriptors()
                            .iter()
                            .map(|d| format!("{} ({})", d.name, d.short))
                            .collect();
                        eprintln!(
                            "Unknown cipher: {}. Available: {}",
                            value,
                            available.join(", "),
                        );
                    }
                }
            }
        }
    }

    if selected_ciphers.is_empty() {
        selected_ciphers.push(default_cipher());
    }
    let cipher_labels: Vec<String> = selected_ciphers
        .iter()
        .map(|cipher| cipher.name().to_string())
        .collect();

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

    if let Some(dspec) = matches.get_one::<String>("date") {
        match parse_range(dspec) {
            Ok(offsets) => {
                let mut t = Table::new();

                let mut header = vec![Cell::new("Offset"), Cell::new("Datum")];
                for label in &cipher_labels {
                    header.push(Cell::new(label));
                }
                t.add_row(Row::new(header));

                let today = Local::now().date_naive();

                for (row_index, off) in offsets.iter().enumerate() {
                    let date = today + Duration::days(*off as i64);
                    let date_str = date.format("%d%m%Y").to_string();

                    if debug {
                        println!("Datum {:+}: {}", off, date.format("%d.%m.%Y"));
                    }

                    let mut row_cells = vec![
                        Cell::new(&format!("{:+}", off)),
                        Cell::new(&date.format("%d.%m.%Y").to_string()),
                    ];

                    for (cipher_index, cipher) in selected_ciphers.iter().enumerate() {
                        let step = Step::new(row_index, cipher_index, "date::reduce");
                        let result =
                            KernResult::from_input(&date_str, debug, cipher.as_ref(), step);

                        if debug {
                            println!("[{}]", result.cipher);
                            for line in &result.trace {
                                println!("{line}");
                            }
                        } else {
                            row_cells.push(Cell::new(&result.value().to_string()));
                        }
                    }

                    if debug {
                        println!();
                    } else {
                        t.add_row(Row::new(row_cells));
                    }
                }

                if !debug {
                    t.printstd();
                }
            }
            Err(e) => eprintln!("{e}"),
        }
    }

    /* --length Flag gesetzt? -------------------------------------------- */

    if let Some(args_values) = matches.get_many::<String>("ARGS") {
        let args: Vec<String> = args_values.map(|s| s.to_string()).collect();
        let mut result_set = ResultSet::new(); // linear flow collects all calculation results

        for (pipe_index, arg) in args.iter().enumerate() {
            for (cipher_index, cipher) in selected_ciphers.iter().enumerate() {
                let step = Step::new(pipe_index, cipher_index, "reduce");
                let calc_result = KernResult::from_input(arg, debug, cipher.as_ref(), step);

                if debug {
                    println!("{} [{}]", arg, calc_result.cipher);
                    for line in &calc_result.trace {
                        println!("{line}");
                    }
                    println!();
                } else if show_length {
                    let len = arg.chars().count();
                    println!(
                        "{arg} [{}]: {} ({len})",
                        calc_result.cipher,
                        calc_result.value()
                    );
                } else {
                    println!("{arg} [{}]: {}", calc_result.cipher, calc_result.value());
                }

                result_set.add(calc_result);
            }
        }

        if show_total {
            let sum: u32 = result_set.total();
            if debug {
                let parts: Vec<String> = result_set.values().map(|v| v.to_string()).collect();
                println!("\n\u{1a} Gesamtsumme: ({}) = {}", parts.join("+"), sum);
            }

            let total_step =
                Step::new(result_set.len(), selected_ciphers.len(), "aggregate::total");
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

        if show_lookup {
            use std::collections::{BTreeSet, HashMap};

            let mut grouped_order: Vec<u32> = Vec::new();
            let mut grouped_map: HashMap<u32, Vec<&KernResult>> = HashMap::new();

            for result in result_set.iter() {
                let value = result.value();
                if let Some(entries) = grouped_map.get_mut(&value) {
                    entries.push(result);
                } else {
                    grouped_order.push(value);
                    grouped_map.insert(value, vec![result]);
                }
            }

            if grouped_order.is_empty() {
                println!("Keine reduzierten Ergebnisse für Lookup.");
            } else {
                let bedeutungen = load_bedeutungen();
                let mut table = Table::new();
                let mut header = vec![
                    Cell::new("Quellen"),
                    Cell::new("Zahl"),
                    Cell::new("Bedeutung"),
                ];
                if show_light {
                    header.push(Cell::new("Light"));
                }
                if show_shadow {
                    header.push(Cell::new("Shadow"));
                }
                table.add_row(Row::new(header));

                for value in &grouped_order {
                    if let Some(results) = grouped_map.get(value) {
                        let entry = bedeutungen.get(value);

                        let mut sources = BTreeSet::new();
                        for res in results {
                            sources.insert(format!("{} [{}]", res.source, res.cipher));
                        }

                        let mut cells = vec![
                            Cell::new(&sources.into_iter().collect::<Vec<_>>().join(", ")),
                            Cell::new(&value.to_string()),
                            Cell::new(entry.and_then(|b| b.text.as_deref()).unwrap_or("-")),
                        ];

                        if show_light {
                            cells.push(Cell::new(
                                entry.and_then(|b| b.licht.as_deref()).unwrap_or("-"),
                            ));
                        }
                        if show_shadow {
                            cells.push(Cell::new(
                                entry.and_then(|b| b.schatten.as_deref()).unwrap_or("-"),
                            ));
                        }

                        table.add_row(Row::new(cells));
                    }
                }

                table.printstd();
            }
        }
        if std::env::var("KERN_DUMP_RESULTSET").is_ok() {
            if let Ok(debug_json) = serde_json::to_string_pretty(&result_set) {
                eprintln!("[KERN DEBUG] ResultSet = {debug_json}");
            }
        }
    } else {
        eprintln!("Keine weiteren Argumente angegeben.");
    }
}
