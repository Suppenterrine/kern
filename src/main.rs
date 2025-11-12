use chrono::{Duration, Local};
use clap::{Arg, ArgAction, Command};
use kern::core::{
    Cipher, FlowContext, FlowFlags, KernResult, Operation, Pipeline, Step, StepFlags,
    default_cipher, descriptors, get_cipher, load_bedeutungen, parse_range,
};
use serde::Deserialize;
use serde_json;
use std::collections::{HashMap, HashSet};

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
            Arg::new("pos")
                .long("pos")
                .action(ArgAction::SetTrue)
                .help("Shows positive aspects in lookup output"),
        )
        .arg(
            Arg::new("neg")
                .long("neg")
                .action(ArgAction::SetTrue)
                .help("Shows negative aspects in lookup output"),
        )
        .arg(
            Arg::new("full")
                .long("full")
                .action(ArgAction::SetTrue)
                .help("Shows complete meaning including positive and negative aspects"),
        )
        .arg(
            Arg::new("cipher")
                .long("cipher")
                .value_name("CIPHER")
                .num_args(1)
                .action(ArgAction::Append)
                .value_delimiter(',')
                .help("Cipher(s) to use (repeatable). Use cipher name, shortcode or 'all'"),
        )
        .arg(
            Arg::new("list-ciphers")
                .long("list-ciphers")
                .action(ArgAction::SetTrue)
                .help("Lists all available ciphers and exits"),
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
            Arg::new("length")
                .short('L')
                .long("length")
                .action(ArgAction::SetTrue)
                .help("Appends the character length to the result output"),
        )
        .arg(
            Arg::new("total")
                .short('t')
                .long("total")
                .action(ArgAction::SetTrue)
                .help("Shows the total sum of all reduced results at the end"),
        )
        .arg(
            Arg::new("debug")
                .long("verbose")
                .action(ArgAction::SetTrue)
                .help("Show detailed calculation trace"),
        )
        .arg(
            Arg::new("ARGS")
                .num_args(1..)
                .allow_hyphen_values(true)
                .help("Input strings to be reduced"),
        )
        .get_matches();

    let debug = matches.get_flag("debug");
    let mut show_total = matches.get_flag("total");
    let show_length = matches.get_flag("length");
    let mut show_lookup = matches.get_flag("lookup");
    let show_pos = matches.get_flag("pos");
    let show_neg = matches.get_flag("neg");
    let show_full = matches.get_flag("full");

    if matches.get_flag("list-ciphers") {
        println!("Available Ciphers:\n");
        for descriptor in descriptors() {
            println!("  {} ({}) · {}",
                descriptor.name,
                descriptor.short,
                descriptor.description
            );
        }
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
    let mut cipher_labels: Vec<String> = selected_ciphers
        .iter()
        .map(|cipher| cipher.name().to_string())
        .collect();
    let cipher_alias_map = build_cipher_alias_map(&cipher_labels);

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
                for idx in 0..inputs.len() {
                    pipeline.add_step(Step::new(idx, 0, Operation::DateReduce));
                }

                let mut ctx = FlowContext::new(FlowFlags {
                    verbose: debug,
                    ciphers: cipher_labels.clone(),
                    total: show_total,
                });

                let result_set = pipeline.run(&mut ctx, &inputs, &selected_ciphers);

                let mut results_matrix: Vec<Vec<Option<&KernResult>>> =
                    vec![vec![None; selected_ciphers.len()]; inputs.len()];

                for result in result_set.iter() {
                    if matches!(result.step.operation, Operation::DateReduce) {
                        if let Some(row) = results_matrix.get_mut(result.step.pipe_index) {
                            if let Some(slot) = row.get_mut(result.step.cipher_index) {
                                *slot = Some(result);
                            }
                        }
                    }
                }

                if ctx.global_flags.verbose {
                    for (row_index, off) in offsets.iter().enumerate() {
                        let display_date = formatted_dates[row_index];
                        println!("Datum {:+}: {}", off, display_date.format("%d.%m.%Y"));

                        if let Some(row_results) = results_matrix.get(row_index) {
                            for maybe_result in row_results {
                                if let Some(result) = maybe_result {
                                    if result.verbose {
                                        println!("[{}]", result.cipher);
                                        for line in &result.trace {
                                            println!("{line}");
                                        }
                                    }
                                }
                            }
                        }

                        println!();
                    }
                } else {
                    for (row_index, off) in offsets.iter().enumerate() {
                        let display_date = formatted_dates[row_index];
                        print!("{:+} ({})", off, display_date.format("%d.%m.%Y"));

                        if let Some(row_results) = results_matrix.get(row_index) {
                            for maybe_result in row_results {
                                if let Some(result) = maybe_result {
                                    print!("  [{}]: {}", result.cipher, result.value());
                                }
                            }
                        }
                        println!();
                    }
                }
            }
            Err(e) => eprintln!("{e}"),
        }
    }

    /* --length Flag gesetzt? -------------------------------------------- */

    if let Some(args_values) = matches.get_many::<String>("ARGS") {
        let raw_tokens: Vec<String> = args_values.map(|s| s.to_string()).collect();

        let parsed = match parse_pipeline_tokens(&raw_tokens, &cipher_alias_map) {
            Ok(data) => data,
            Err(err) => {
                eprintln!("{err}");
                return;
            }
        };

        if parsed.inputs.is_empty() {
            eprintln!("Keine weiteren Argumente angegeben.");
            return;
        }

        show_total = show_total || parsed.saw_total;
        show_lookup = show_lookup || parsed.saw_lookup;

        let args = parsed.inputs;
        let reduce_steps = parsed.steps;

        // Save original global cipher names before adding local ciphers
        let global_cipher_names = cipher_labels.clone();

        ensure_local_ciphers(&mut selected_ciphers, &mut cipher_labels, &reduce_steps);

        let mut pipeline = Pipeline::new();
        for step in reduce_steps.into_iter() {
            pipeline.add_step(step);
        }

        if show_total {
            let pipe_index = args.len().saturating_sub(1);
            pipeline.add_step(Step::new(pipe_index, 0, Operation::AggregateTotal));
        }

        if show_lookup {
            let pipe_index = args.len().saturating_sub(1);
            pipeline.add_step(Step::new(pipe_index, 0, Operation::Lookup));
        }

        let mut ctx = FlowContext::new(FlowFlags {
            verbose: debug,
            ciphers: global_cipher_names,
            total: show_total,
        });

        let result_set = pipeline.run(&mut ctx, &args, &selected_ciphers);

        let mut base_results: HashMap<(usize, usize), &KernResult> = HashMap::new();
        let mut aggregate_results: Vec<&KernResult> = Vec::new();
        let mut lookup_results: Vec<&KernResult> = Vec::new();

        for result in result_set.iter() {
            match &result.step.operation {
                Operation::Reduce | Operation::DateReduce => {
                    base_results.insert((result.step.pipe_index, result.step.cipher_index), result);
                }
                Operation::AggregateTotal => aggregate_results.push(result),
                Operation::Lookup => lookup_results.push(result),
                Operation::Custom(_) => {}
            }
        }

        let cipher_count = selected_ciphers.len();

        if cipher_count > 0 {
            for (pipe_index, arg) in args.iter().enumerate() {
                for cipher_index in 0..cipher_count {
                    if let Some(result) = base_results.get(&(pipe_index, cipher_index)) {
                        let result = *result;
                        if result.verbose {
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
            let relevant: Vec<&KernResult> = ctx
                .memory
                .iter()
                .filter(|res| {
                    matches!(
                        res.step.operation,
                        Operation::Reduce | Operation::DateReduce | Operation::Custom(_)
                    )
                })
                .collect();

            let sum: u32 = relevant.iter().map(|res| res.value()).sum();
            let totals_verbose = aggregate_results.iter().any(|res| res.verbose);
            let parts: Vec<String> = if totals_verbose {
                relevant.iter().map(|res| res.value().to_string()).collect()
            } else {
                Vec::new()
            };
            let mut printed_parts = false;

            for total_result in aggregate_results {
                if total_result.verbose {
                    if !printed_parts && !parts.is_empty() {
                        println!("\n\u{1a} Gesamtsumme: ({}) = {}", parts.join("+"), sum);
                        printed_parts = true;
                    }
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
            #[derive(Deserialize)]
            struct LookupEntry {
                value: u32,
                sources: Vec<String>,
            }

            let payload = lookup_results.last().and_then(|res| res.payload.as_deref());

            match payload {
                Some(data) => match serde_json::from_str::<Vec<LookupEntry>>(data) {
                    Ok(entries) if !entries.is_empty() => {
                        let bedeutungen = load_bedeutungen();

                        for entry in entries {
                            let bedeutung = bedeutungen.get(&entry.value);
                            let bedeutung_text = bedeutung
                                .and_then(|b| b.text.as_deref())
                                .unwrap_or("-");

                            // Header: Number · Meaning
                            println!("{} · {}", entry.value, bedeutung_text);

                            // Sources (with tree-like structure)
                            if !show_full && !entry.sources.is_empty() {
                                for (i, source) in entry.sources.iter().enumerate() {
                                    let prefix = if i == entry.sources.len() - 1 { "└─" } else { "├─" };
                                    println!("  {} {}", prefix, source);
                                }
                            }

                            // Full mode: show sources header + pos/neg
                            if show_full {
                                if !entry.sources.is_empty() {
                                    println!("  Quellen:");
                                    for (i, source) in entry.sources.iter().enumerate() {
                                        let prefix = if i == entry.sources.len() - 1 { "└─" } else { "├─" };
                                        println!("    {} {}", prefix, source);
                                    }
                                }

                                // Show positive aspects
                                if let Some(pos_text) = bedeutung.and_then(|b| b.licht.as_deref()) {
                                    println!("\n  ⊕ Positiv:");
                                    println!("    {}", pos_text);
                                }

                                // Show negative aspects
                                if let Some(neg_text) = bedeutung.and_then(|b| b.schatten.as_deref()) {
                                    println!("\n  ⊖ Negativ:");
                                    println!("    {}", neg_text);
                                }
                            }

                            // Individual pos/neg flags (only if not in full mode)
                            if !show_full && show_pos {
                                if let Some(pos_text) = bedeutung.and_then(|b| b.licht.as_deref()) {
                                    println!("\n  ⊕ Positiv:");
                                    println!("    {}", pos_text);
                                }
                            }
                            if !show_full && show_neg {
                                if let Some(neg_text) = bedeutung.and_then(|b| b.schatten.as_deref()) {
                                    println!("\n  ⊖ Negativ:");
                                    println!("    {}", neg_text);
                                }
                            }

                            println!();
                        }
                    }
                    Ok(_) => println!("Keine reduzierten Ergebnisse für Lookup."),
                    Err(_) => println!("Lookup-Auswertung konnte nicht gelesen werden."),
                },
                None => println!("Keine reduzierten Ergebnisse für Lookup."),
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
fn build_cipher_alias_map(cipher_labels: &[String]) -> HashMap<String, String> {
    let mut map = HashMap::new();

    for descriptor in descriptors() {
        map.insert(descriptor.name.to_lowercase(), descriptor.name.to_string());
        map.insert(descriptor.short.to_lowercase(), descriptor.name.to_string());
    }

    for label in cipher_labels {
        map.entry(label.to_lowercase())
            .or_insert_with(|| label.clone());
    }

    map
}

fn ensure_local_ciphers(
    selected_ciphers: &mut Vec<Box<dyn Cipher>>,
    cipher_labels: &mut Vec<String>,
    steps: &[Step],
) {
    let mut existing: HashSet<String> = selected_ciphers
        .iter()
        .map(|cipher| cipher.name().to_lowercase())
        .collect();

    for step in steps {
        if let Some(names) = &step.local_flags.ciphers {
            for name in names {
                let key = name.to_lowercase();
                if existing.contains(&key) {
                    continue;
                }

                if let Some(cipher) = get_cipher(&key) {
                    let canonical = cipher.name().to_string();
                    existing.insert(canonical.to_lowercase());
                    cipher_labels.push(canonical.clone());
                    selected_ciphers.push(cipher);
                }
            }
        }
    }
}

struct ParsedPipeline {
    inputs: Vec<String>,
    steps: Vec<Step>,
    saw_total: bool,
    saw_lookup: bool,
}

fn parse_pipeline_tokens(
    tokens: &[String],
    cipher_aliases: &HashMap<String, String>,
) -> Result<ParsedPipeline, String> {
    let mut inputs = Vec::new();
    let mut steps = Vec::new();
    let mut current_input: Option<String> = None;
    let mut current_flags = StepFlags::default();
    let mut iter = tokens.iter();
    let mut saw_total = false;
    let mut saw_lookup = false;

    while let Some(token) = iter.next() {
        match token.as_str() {
            "-v" => {
                if current_input.is_none() {
                    return Err(String::from(
                        "Lokales Flag -v muss nach einem Input angegeben werden.",
                    ));
                }
                current_flags.verbose = Some(true);
            }
            "-c" => {
                if current_input.is_none() {
                    return Err(String::from(
                        "Lokales Flag -c muss nach einem Input angegeben werden.",
                    ));
                }
                let names_token = iter.next().ok_or_else(|| {
                    String::from("Nach -c wird eine kommagetrennte Cipher-Liste erwartet.")
                })?;
                let mut resolved = Vec::new();
                for raw in names_token.split(',') {
                    let trimmed = raw.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    let key = trimmed.to_lowercase();
                    if let Some(canonical) = cipher_aliases.get(&key) {
                        if !resolved
                            .iter()
                            .any(|existing: &String| existing.eq_ignore_ascii_case(canonical))
                        {
                            resolved.push(canonical.clone());
                        }
                    } else {
                        return Err(format!("Unbekanntes Cipher für -c: {trimmed}"));
                    }
                }
                if resolved.is_empty() {
                    return Err(String::from(
                        "Nach -c wurde kein gültiges Cipher angegeben.",
                    ));
                }
                current_flags.ciphers = Some(resolved);
            }
            "-t" | "--total" => {
                saw_total = true;
            }
            "-l" | "--lookup" => {
                saw_lookup = true;
            }
            _ => {
                if let Some(input) = current_input.take() {
                    let pipe_index = inputs.len();
                    inputs.push(input);
                    let mut step = Step::new(pipe_index, 0, Operation::Reduce);
                    step.local_flags = current_flags.clone();
                    steps.push(step);
                    current_flags = StepFlags::default();
                }
                current_input = Some(token.clone());
            }
        }
    }

    if let Some(input) = current_input.take() {
        let pipe_index = inputs.len();
        inputs.push(input);
        let mut step = Step::new(pipe_index, 0, Operation::Reduce);
        step.local_flags = current_flags.clone();
        steps.push(step);
    }

    Ok(ParsedPipeline {
        inputs,
        steps,
        saw_total,
        saw_lookup,
    })
}
