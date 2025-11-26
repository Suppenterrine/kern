use chrono::{Duration, Local};
use clap::{Arg, ArgAction, Command};
use kern::core::{
    Cipher, FlowContext, FlowFlags, KernResult, Operation, Pipeline, Step, StepFlags,
    default_cipher, descriptors, get_cipher, load_bedeutungen, parse_range,
};
use kern::ui;
use serde::Deserialize;
use serde_json;
use std::collections::{HashMap, HashSet};

fn main() {
    let version = env!("CARGO_PKG_VERSION");
    let about = format!(
        r#"
┌───────────────────┐
│   KERN - v{version}   │
└───────────────────┘
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
            Arg::new("spektra")
                .long("spektra")
                .action(ArgAction::SetTrue)
                .help("Generate SPEKTRA analysis prompt (uses all ciphers automatically)"),
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
    let show_spektra = matches.get_flag("spektra");

    if matches.get_flag("list-ciphers") {
        let cipher_list: Vec<(String, String, String)> = descriptors()
            .into_iter()
            .map(|d| (d.name.to_string(), d.short.to_string(), d.description.to_string()))
            .collect();
        ui::output::format_cipher_list(&cipher_list);
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

                // Add lookup operation if requested
                if show_lookup {
                    pipeline.add_step(Step::new(0, 0, Operation::Lookup));
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
                        let date_str = display_date.format("%d.%m.%Y").to_string();

                        if let Some(row_results) = results_matrix.get(row_index) {
                            for maybe_result in row_results {
                                if let Some(result) = maybe_result {
                                    if result.verbose {
                                        ui::output::format_date_verbose(
                                            *off,
                                            &date_str,
                                            &result.cipher,
                                            &result.trace,
                                            result.value(),
                                        );
                                        ui::spacing(ui::SPACING_SECTION);
                                    }
                                }
                            }
                        }
                    }
                } else {
                    for (row_index, off) in offsets.iter().enumerate() {
                        let display_date = formatted_dates[row_index];
                        let date_str = display_date.format("%d.%m.%Y").to_string();

                        if let Some(row_results) = results_matrix.get(row_index) {
                            for maybe_result in row_results {
                                if let Some(result) = maybe_result {
                                    ui::output::format_date_simple(
                                        *off,
                                        &date_str,
                                        result.value(),
                                        &result.cipher,
                                    );
                                    println!();
                                }
                            }
                        }
                    }
                }

                // Handle lookup if requested
                if show_lookup {
                    #[derive(Deserialize)]
                    struct LookupEntry {
                        value: u32,
                        sources: Vec<String>,
                    }

                    let lookup_results: Vec<&KernResult> = result_set
                        .iter()
                        .filter(|r| matches!(r.step.operation, Operation::Lookup))
                        .collect();

                    let payload = lookup_results.last().and_then(|res| res.payload.as_deref());

                    match payload {
                        Some(data) => match serde_json::from_str::<Vec<LookupEntry>>(data) {
                            Ok(entries) if !entries.is_empty() => {
                                let bedeutungen = load_bedeutungen();

                                ui::spacing(ui::SPACING_MODE);

                                for entry in entries {
                                    let bedeutung = bedeutungen.get(&entry.value);
                                    let bedeutung_text = bedeutung
                                        .and_then(|b| b.text.as_deref())
                                        .unwrap_or("-");

                                    ui::output::format_lookup_entry(
                                        entry.value,
                                        bedeutung_text,
                                        &entry.sources,
                                        bedeutung,
                                        show_pos,
                                        show_neg,
                                        show_full,
                                    );
                                }
                            }
                            _ => {}
                        },
                        None => {}
                    }
                }
            }
            Err(e) => eprintln!("{e}"),
        }
        return; // Date processing complete, exit early
    }

    /* --spektra Flag gesetzt? -------------------------------------------- */

    if show_spektra {
        if let Some(args_values) = matches.get_many::<String>("ARGS") {
            let raw_tokens: Vec<String> = args_values.map(|s| s.to_string()).collect();
            
            if raw_tokens.is_empty() {
                eprintln!("--spektra requires a word argument");
                return;
            }

            // Only use the first word for spektra analysis
            let word = &raw_tokens[0];

            // Force all ciphers for spektra
            let mut spektra_ciphers: Vec<Box<dyn Cipher>> = Vec::new();
            for descriptor in descriptors() {
                spektra_ciphers.push((descriptor.factory)());
            }

            let cipher_names: Vec<String> = spektra_ciphers
                .iter()
                .map(|cipher| cipher.name().to_string())
                .collect();

            // Build and execute pipeline
            let mut pipeline = Pipeline::new();
            let step = Step::new(0, 0, Operation::Reduce);
            pipeline.add_step(step);
            
            // Add lookup for meanings
            let lookup_step = Step::new(0, 0, Operation::Lookup);
            pipeline.add_step(lookup_step);

            let mut ctx = FlowContext::new(FlowFlags {
                verbose: debug,
                ciphers: cipher_names,
                total: false,
            });

            let _result_set = pipeline.run(&mut ctx, &[word.clone()], &spektra_ciphers);

            // Collect results from memory (all reduce operations)
            let reduce_results: Vec<KernResult> = ctx
                .memory
                .iter()
                .filter(|res| matches!(res.step.operation, Operation::Reduce))
                .cloned()
                .collect();

            // Load meanings
            let bedeutungen = load_bedeutungen();

            // Build spektra prompt
            match kern::core::spektra::build_spektra_prompt(word, &reduce_results, &bedeutungen) {
                Ok(prompt) => {
                    ui::output::format_spektra_output(&prompt);
                }
                Err(e) => {
                    eprintln!("Error building spektra prompt: {}", e);
                }
            }
        }
        return;
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
            // Group results by input
            let mut grouped_by_input: HashMap<&str, Vec<(&str, u32, bool, &[String])>> = HashMap::new();

            for (pipe_index, arg) in args.iter().enumerate() {
                for cipher_index in 0..cipher_count {
                    if let Some(result) = base_results.get(&(pipe_index, cipher_index)) {
                        grouped_by_input
                            .entry(arg.as_str())
                            .or_insert_with(Vec::new)
                            .push((
                                result.cipher.as_str(),
                                result.value(),
                                result.verbose,
                                result.trace.as_slice(),
                            ));
                    }
                }
            }

            // Output results by input order
            for arg in &args {
                if let Some(results) = grouped_by_input.get(arg.as_str()) {
                    // Check if any result is verbose
                    let any_verbose = results.iter().any(|(_, _, verbose, _)| *verbose);

                    if any_verbose {
                        // Verbose mode - show each cipher separately
                        for (cipher, value, verbose, trace) in results {
                            if *verbose {
                                ui::output::format_verbose_reduction(arg, cipher, trace, *value);
                                ui::spacing(ui::SPACING_SECTION);
                            }
                        }
                    } else if show_length {
                        // Length mode - show inline format
                        let len = arg.chars().count();
                        if results.len() == 1 {
                            println!("{} {} {} [{}] ({})", arg, ui::ARROW_RIGHT, results[0].1, results[0].0, len);
                        } else {
                            println!("{} ({})", arg, len);
                            for (cipher, value, _, _) in results {
                                println!("{}{}  {} {}", ui::INDENT_BASE, cipher, ui::ARROW_RIGHT, value);
                            }
                        }
                    } else {
                        // Standard mode - use grouped format
                        let simple_results: Vec<(String, u32)> = results
                            .iter()
                            .map(|(cipher, value, _, _)| (cipher.to_string(), *value))
                            .collect();
                        ui::output::format_reduce_grouped(arg, &simple_results);
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
            let parts: Vec<u32> = relevant.iter().map(|res| res.value()).collect();

            for total_result in aggregate_results {
                if total_result.verbose {
                    ui::output::format_total_verbose(
                        &parts,
                        sum,
                        &total_result.trace,
                        total_result.value(),
                    );
                } else {
                    ui::output::format_total_simple(sum, total_result.value());
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

                        ui::spacing(ui::SPACING_MODE);

                        for entry in entries {
                            let bedeutung = bedeutungen.get(&entry.value);
                            let bedeutung_text = bedeutung
                                .and_then(|b| b.text.as_deref())
                                .unwrap_or("-");

                            ui::output::format_lookup_entry(
                                entry.value,
                                bedeutung_text,
                                &entry.sources,
                                bedeutung,
                                show_pos,
                                show_neg,
                                show_full,
                            );
                        }
                    }
                    _ => {}
                },
                None => {}
            }
        }
        if std::env::var("KERN_DUMP_RESULTSET").is_ok() {
            if let Ok(debug_json) = serde_json::to_string_pretty(&result_set) {
                eprintln!("[KERN DEBUG] ResultSet = {debug_json}");
            }
        }
    }
    // Note: If no ARGS provided, quietly exit (no error message needed)
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
