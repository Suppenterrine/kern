use chrono::{Duration, Local};
use clap::{Arg, ArgAction, Command};
use kern::core::{
    Cipher, KernResult, Operation, Pipeline, Step, default_cipher, descriptors, get_cipher,
    load_bedeutungen, parse_range,
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

    if let Some(dspec) = matches.get_one::<String>("date") {
        match parse_range(dspec) {
            Ok(offsets) => {
                let today = Local::now().date_naive();
                let formatted_dates: Vec<_> = offsets
                    .iter()
                    .map(|off| today + Duration::days(*off as i64))
                    .collect();
                let inputs: Vec<String> = formatted_dates
                    .iter()
                    .map(|date| date.format("%d%m%Y").to_string())
                    .collect();

                let mut pipeline = Pipeline::new();
                pipeline.add_step(Step::new(0, 0, Operation::DateReduce));

                let result_set = pipeline.run(&inputs, &selected_ciphers, debug);

                let mut results_matrix: Vec<Vec<Option<&KernResult>>> =
                    vec![vec![None; selected_ciphers.len()]; offsets.len()];

                for result in result_set.iter() {
                    if matches!(result.step.operation, Operation::DateReduce) {
                        let row = result.step.pipe_index;
                        let col = result.step.cipher_index;
                        if row < results_matrix.len() {
                            if let Some(slot) = results_matrix[row].get_mut(col) {
                                *slot = Some(result);
                            }
                        }
                    }
                }

                if debug {
                    for (row_index, off) in offsets.iter().enumerate() {
                        let display_date = formatted_dates[row_index];
                        println!("Datum {:+}: {}", off, display_date.format("%d.%m.%Y"));

                        if let Some(row_results) = results_matrix.get(row_index) {
                            for maybe_result in row_results {
                                if let Some(result) = maybe_result {
                                    println!("[{}]", result.cipher);
                                    for line in &result.trace {
                                        println!("{line}");
                                    }
                                }
                            }
                        }

                        println!();
                    }
                } else {
                    let mut table = Table::new();
                    let mut header = vec![Cell::new("Offset"), Cell::new("Datum")];
                    for label in &cipher_labels {
                        header.push(Cell::new(label));
                    }
                    table.add_row(Row::new(header));

                    for (row_index, off) in offsets.iter().enumerate() {
                        let display_date = formatted_dates[row_index];
                        let mut row_cells = vec![
                            Cell::new(&format!("{:+}", off)),
                            Cell::new(&display_date.format("%d.%m.%Y").to_string()),
                        ];

                        if let Some(row_results) = results_matrix.get(row_index) {
                            for maybe_result in row_results {
                                let value = maybe_result
                                    .map(|result| result.value().to_string())
                                    .unwrap_or_else(|| "-".to_string());
                                row_cells.push(Cell::new(&value));
                            }
                        }

                        table.add_row(Row::new(row_cells));
                    }

                    table.printstd();
                }
            }
            Err(e) => eprintln!("{e}"),
        }
    }

    /* --length Flag gesetzt? -------------------------------------------- */

    if let Some(args_values) = matches.get_many::<String>("ARGS") {
        let args: Vec<String> = args_values.map(|s| s.to_string()).collect();

        let mut pipeline = Pipeline::new();
        pipeline.add_step(Step::new(0, 0, Operation::Reduce));
        if show_total {
            pipeline.add_step(Step::new(0, 0, Operation::AggregateTotal));
        }

        let result_set = pipeline.run(&args, &selected_ciphers, debug);

        let mut base_results: Vec<&KernResult> = Vec::new();
        let mut aggregate_results: Vec<&KernResult> = Vec::new();

        for result in result_set.iter() {
            match &result.step.operation {
                Operation::Reduce => base_results.push(result),
                Operation::AggregateTotal => aggregate_results.push(result),
                _ => {}
            }
        }

        let cipher_count = selected_ciphers.len();

        if cipher_count > 0 {
            for (pipe_index, arg) in args.iter().enumerate() {
                for cipher_index in 0..cipher_count {
                    let base_idx = pipe_index * cipher_count + cipher_index;
                    if let Some(result) = base_results.get(base_idx) {
                        if debug {
                            println!("{} [{}]", arg, result.cipher);
                            for line in &result.trace {
                                println!("{line}");
                            }
                            println!();
                        } else if show_length {
                            let len = arg.chars().count();
                            println!("{arg} [{}]: {} ({len})", result.cipher, result.value());
                        } else {
                            println!("{arg} [{}]: {}", result.cipher, result.value());
                        }
                    }
                }
            }
        }

        if show_total && !aggregate_results.is_empty() {
            let sum: u32 = base_results.iter().map(|r| r.value()).sum();

            if debug && !base_results.is_empty() {
                let parts: Vec<String> =
                    base_results.iter().map(|r| r.value().to_string()).collect();
                println!("\n\u{1a} Gesamtsumme: ({}) = {}", parts.join("+"), sum);
            }

            for total_result in aggregate_results {
                if debug {
                    for line in &total_result.trace {
                        println!("{line}");
                    }
                    println!("\u{1a} Gesamtsumme: {sum} \u{1a} {}", total_result.value());
                } else {
                    println!("Gesamtsumme: {sum} \u{1a} {}", total_result.value());
                }
            }
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
