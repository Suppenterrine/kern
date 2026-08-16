use chrono::{Duration, Local};
use clap::{Arg, ArgAction, Command};
use kern::core::{
    Cipher, FlowContext, FlowFlags, KernResult, Lang, Operation, Pipeline, Step, StepMetadata,
    ErrorCode, default_cipher, descriptors, generate_matrix_pairs, get_cipher,
    load_bedeutungen_lang, parse_range,
};
use kern::ui;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;

// ============================================================================
// JSON Output Structures (for piping support)
// ============================================================================

/// Output mode detection based on TTY
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OutputMode {
    Tty,    // Terminal: human-readable output
    Piped,  // Piped: JSON output
}

impl OutputMode {
    fn detect() -> Self {
        if ui::is_tty() {
            Self::Tty
        } else {
            Self::Piped
        }
    }
}

/// Error response for JSON mode
#[derive(Serialize)]
struct ErrorResponse {
    code: ErrorCode,
    error: String,
}

/// Cipher result in multi-cipher mode
#[derive(Serialize)]
struct CipherResult {
    name: String,
    code: String,
    value: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    chain: Option<Vec<String>>,
}

/// Single reduce item
#[derive(Serialize)]
struct ReduceItem {
    input: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    length: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ciphers: Option<Vec<CipherResult>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    value: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    chain: Option<Vec<String>>,
}

/// Reduce mode response
///
/// `total` is absent unless `--total` was given. It used to be a plain `u32`
/// filled with `unwrap_or(0)`, so every call without `--total` reported
/// `"total": 0` — a number that was never computed (issue #23).
#[derive(Serialize)]
struct ReduceResponse {
    items: Vec<ReduceItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    total: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    total_chain: Option<Vec<String>>,
}

/// One entry of a Lookup step's payload: the reduced value plus the inputs that
/// produced it. The Lookup `KernResult` itself carries no value (`value` is 0)
/// — the actual data lives in its JSON payload, so it must always be read from
/// here rather than from `KernResult::value`.
#[derive(Deserialize)]
struct LookupEntry {
    value: u32,
    sources: Vec<String>,
}

/// Lookup response for a single number
#[derive(Serialize)]
struct LookupResponse {
    number: u32,
    meaning: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    positive: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    negative: Option<String>,
}

/// Lookup mode response (multiple numbers)
#[derive(Serialize)]
struct LookupListResponse {
    lang: &'static str,
    items: Vec<LookupResponse>,
}

/// Single date item
#[derive(Serialize)]
struct DateItem {
    offset: i32,
    date: String,
    value: u32,
    meaning: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    chain: Option<Vec<String>>,
}

/// Date mode response
#[derive(Serialize)]
struct DateResponse {
    lang: &'static str,
    dates: Vec<DateItem>,
}

/// Phase relation item
#[derive(Serialize)]
struct PhaseRelationItem {
    left_input: String,
    right_input: String,
    left_value: u32,
    right_value: u32,
    left_compartment: u32,
    right_compartment: u32,
    phase: i32,
    cipher: String,
}

/// Phase relation mode response
#[derive(Serialize)]
struct PhaseResponse {
    relations: Vec<PhaseRelationItem>,
}

/// Spektra mode response
#[derive(Serialize)]
struct SpektraResponse {
    word: String,
    prompt: String,
}

/// Single alphabet-index entry (letter -> position, A=1)
#[derive(Serialize)]
struct IndexEntry {
    letter: String,
    index: u32,
}

/// Alphabet-index response for a single input
#[derive(Serialize)]
struct IndexResponse {
    input: String,
    entries: Vec<IndexEntry>,
}

/// Alphabet-index response for multiple inputs (piped/JSON mode)
#[derive(Serialize)]
struct IndexListResponse {
    items: Vec<IndexResponse>,
}

// ============================================================================
// Main Entry Point
// ============================================================================

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
            Arg::new("index")
                .short('i')
                .long("index")
                .action(ArgAction::SetTrue)
                .help(
                    "Shows the alphabet position of each letter (A=1, B=2, ...). \
                     Cipher-independent pure lookup, special characters are skipped, \
                     duplicate letters are deduplicated. Example: kern -i kassel",
                ),
        )
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
            Arg::new("lang")
                .long("lang")
                .value_name("CODE")
                .num_args(1)
                .help(
                    "Content language: en (default), de, fr. Meanings exist in all three; \
                     SPEKTRA and RTAP prompts only in en and de, and an unavailable \
                     language is rejected rather than substituted. \
                     Calculations are language independent",
                ),
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
            Arg::new("rtap")
                .long("rtap")
                .value_name("PART")
                .help("Show RTAP (Rethinking Thoughts And Positions) prompt. Values: 1 or 2"),
        )
        .arg(
            Arg::new("phase-relation-matrix")
                .long("phase-relation-matrix")
                .visible_alias("prm")
                .action(ArgAction::SetTrue)
                .help("Calculate phase relation matrix for all input pairs"),
        )
        .arg(
            Arg::new("ARGS")
                .num_args(1..)
                .help("Input strings to be reduced"),
        )
        .get_matches();

    // Detect output mode (TTY vs piped)
    let output_mode = OutputMode::detect();
    let is_tty = matches!(output_mode, OutputMode::Tty);

    let debug = matches.get_flag("debug");
    let show_total = matches.get_flag("total");
    let show_length = matches.get_flag("length");
    let show_lookup = matches.get_flag("lookup");
    let show_pos = matches.get_flag("pos");
    let show_neg = matches.get_flag("neg");
    let show_full = matches.get_flag("full");
    let show_spektra = matches.get_flag("spektra");
    let show_pmr = matches.get_flag("phase-relation-matrix");
    let show_index = matches.get_flag("index");

    // Meanings language. An unknown code aborts instead of silently falling
    // back, mirroring the server's 400 response.
    let lang = match matches.get_one::<String>("lang") {
        Some(raw) => match raw.parse::<Lang>() {
            Ok(lang) => lang,
            Err(msg) => output_error(ErrorCode::UnsupportedLanguage, &msg, is_tty),
        },
        None => Lang::default(),
    };

    // --index: dedicated alphabet-position lookup, independent of any cipher
    if show_index {
        match matches.get_many::<String>("ARGS") {
            Some(values) => {
                let inputs: Vec<String> =
                    values.map(|s| s.to_string()).collect();
                if inputs.is_empty() {
                    output_error(ErrorCode::InputMissing, "index requires at least one input", is_tty);
                }
                let mut json_items = Vec::new();
                for input in &inputs {
                    let entries = kern::core::alphabet_index(input);
                    if !is_tty {
                        json_items.push(IndexResponse {
                            input: input.clone(),
                            entries: entries
                                .iter()
                                .map(|(ch, idx)| IndexEntry {
                                    letter: ch.to_string(),
                                    index: *idx,
                                })
                                .collect(),
                        });
                    } else {
                        let parts: Vec<String> = entries
                            .iter()
                            .map(|(ch, idx)| format!("{ch}={idx}"))
                            .collect();
                        println!("{} {} {}", input, ui::ARROW_RIGHT, parts.join(" "));
                    }
                }
                if !is_tty {
                    let response = IndexListResponse { items: json_items };
                    if let Ok(json) = serde_json::to_string(&response) {
                        println!("{}", json);
                    }
                }
            }
            None => output_error(ErrorCode::InputMissing, "index requires at least one input", is_tty),
        }
        return;
    }

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
                        output_error(
                            ErrorCode::UnknownCipher,
                            &format!("unknown cipher: {}. available: {}", value, available.join(", ")),
                            is_tty,
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

                // JSON output mode for dates
                if !is_tty {
                    // Collect first result from each row for JSON output
                    let flat_results: Vec<Option<&KernResult>> = results_matrix
                        .iter()
                        .map(|row| row.first().copied().flatten())
                        .collect();

                    let bedeutungen = load_bedeutungen_lang(lang);
                    output_date_json(
                        &offsets,
                        &formatted_dates,
                        &flat_results,
                        &bedeutungen,
                        lang,
                        debug,
                    );
                    return;
                }

                // TTY output mode
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
                    let lookup_results: Vec<&KernResult> = result_set
                        .iter()
                        .filter(|r| matches!(r.step.operation, Operation::Lookup))
                        .collect();

                    let payload = lookup_results.last().and_then(|res| res.payload.as_deref());

                    match payload {
                        Some(data) => match serde_json::from_str::<Vec<LookupEntry>>(data) {
                            Ok(entries) if !entries.is_empty() => {
                                let bedeutungen = load_bedeutungen_lang(lang);

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
            Err(e) => output_error(ErrorCode::InvalidRange, &e, is_tty),
        }
        return; // Date processing complete, exit early
    }

    /* --spektra Flag gesetzt? -------------------------------------------- */

    if show_spektra {
        if let Some(args_values) = matches.get_many::<String>("ARGS") {
            let raw_tokens: Vec<String> = args_values.map(|s| s.to_string()).collect();
            
            if raw_tokens.is_empty() {
                output_error(ErrorCode::WordMissing, "--spektra requires a word argument", is_tty);
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

            // The SPEKTRA prompt exists in German and English only. No silent
            // substitution — an unavailable language aborts.
            if !lang.has_prompts() {
                output_error(
                    ErrorCode::LanguageNotAvailable,
                    &format!(
                        "SPEKTRA prompt is not available in '{lang}'. available: {}",
                        Lang::prompt_langs()
                    ),
                    is_tty,
                );
            }
            let bedeutungen = load_bedeutungen_lang(lang);

            // Build spektra prompt
            match kern::core::spektra::build_spektra_prompt(
                word,
                &reduce_results,
                &bedeutungen,
                lang,
            ) {
                Ok(prompt) => {
                    if is_tty {
                        ui::output::format_spektra_output(&prompt, is_tty);
                    } else {
                        output_spektra_json(word, &prompt);
                    }
                }
                Err(e) => {
                    output_error(ErrorCode::SpektraFailed, &format!("error building spektra prompt: {e}"), is_tty);
                }
            }
        }
        return;
    }

    /* --rtap Flag gesetzt? -------------------------------------------- */

    if let Some(rtap_part) = matches.get_one::<String>("rtap") {
        let part_num = match rtap_part.parse::<u8>() {
            Ok(1) | Ok(2) => rtap_part.parse::<u8>().unwrap(),
            _ => {
                output_error(
                    ErrorCode::InvalidRtapPart,
                    &format!("invalid part number: {rtap_part}. must be 1 or 2"),
                    is_tty,
                );
            }
        };

        // No silent substitution: a language without RTAP prompts aborts.
        let prompts = match kern::core::load_rtap_prompts_lang(lang) {
            Some(prompts) => prompts,
            None => output_error(
                ErrorCode::LanguageNotAvailable,
                &format!(
                    "RTAP prompts are not available in '{lang}'. available: {}",
                    Lang::prompt_langs()
                ),
                is_tty,
            ),
        };

        match kern::core::get_rtap_prompt(part_num, &prompts) {
            Some(prompt) => {
                if is_tty {
                    println!("{}", prompt);
                } else {
                    output_rtap_json(prompt, part_num);
                }
            }
            None => {
                output_error(
                    ErrorCode::RtapPromptMissing,
                    &format!("RTAP prompt {part_num} not found in configuration"),
                    is_tty,
                );
            }
        }
        return;
    }

    /* --length Flag gesetzt? -------------------------------------------- */

    if let Some(args_values) = matches.get_many::<String>("ARGS") {
        let raw_tokens: Vec<String> = args_values.map(|s| s.to_string()).collect();

        let parsed = match parse_pipeline_tokens(
            &raw_tokens,
            &cipher_alias_map,
            show_pmr,
            show_total,
            show_lookup,
        ) {
            Ok(data) => data,
            Err((code, err)) => {
                output_error(code, &err, is_tty);
            }
        };

        if parsed.inputs.is_empty() {
            return;
        }

        let args = parsed.inputs;
        let reduce_steps = parsed.steps;

        // All global cipher names apply uniformly
        let global_cipher_names = cipher_labels.clone();

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

        // Check if we're in phase relation mode
        let is_phase_mode = parsed.is_phase_mode;

        if is_phase_mode {
            // Phase relation mode: output phase results
            if is_tty {
                ui::output::format_phase_relation_results(&ctx.phase_results, &selected_ciphers);
            } else {
                output_phase_json(&ctx.phase_results);
            }

            if std::env::var("KERN_DUMP_RESULTSET").is_ok() {
                if let Ok(debug_json) = serde_json::to_string_pretty(&ctx.phase_results) {
                    eprintln!("[KERN DEBUG] PhaseResults = {debug_json}");
                }
            }
            return;
        }

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
                Operation::PhaseRelation => {}, // Handled separately
                Operation::Custom(_) => {}
            }
        }

        let cipher_count = selected_ciphers.len();

        // JSON output mode for reduce/lookup
        if !is_tty {
            if show_lookup {
                // Lookup mode: only output lookup results
                let payload = lookup_results.last().and_then(|res| res.payload.as_deref());
                if let Some(data) = payload {
                    if let Ok(entries) = serde_json::from_str::<Vec<LookupEntry>>(data) {
                        if !entries.is_empty() {
                            let bedeutungen = load_bedeutungen_lang(lang);
                            output_lookup_json(
                                &entries,
                                &bedeutungen,
                                lang,
                                show_pos,
                                show_neg,
                                show_full,
                            );
                        }
                    }
                }
            } else {
                // Reduce mode: output reduce results
                output_reduce_json(&args, &result_set, &selected_ciphers, debug, show_length);
            }
            return;
        }

        // TTY output mode
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
            let payload = lookup_results.last().and_then(|res| res.payload.as_deref());

            match payload {
                Some(data) => match serde_json::from_str::<Vec<LookupEntry>>(data) {
                    Ok(entries) if !entries.is_empty() => {
                        let bedeutungen = load_bedeutungen_lang(lang);

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

// ============================================================================
// Error Handling Helper
// ============================================================================

/// Output error message and exit
/// In TTY mode: prints to stderr
/// In pipe mode: prints JSON error to stdout
/// Aborts with `message`. In pipe mode the JSON carries the same `code` field
/// the server emits, so a consumer can branch identically no matter which one
/// it called — see docs/PRINCIPLES.md §3.
fn output_error(code: ErrorCode, message: &str, is_tty: bool) -> ! {
    if is_tty {
        eprintln!("{}", message);
        std::process::exit(1);
    } else {
        // Pipe mode: JSON to stdout
        let error = ErrorResponse {
            code,
            error: message.to_string(),
        };
        if let Ok(json) = serde_json::to_string(&error) {
            println!("{}", json);
        }
        std::process::exit(1);
    }
}

// ============================================================================
// JSON Output Functions (for piping mode)
// ============================================================================

/// Output spektra response as JSON
fn output_spektra_json(word: &str, prompt: &str) {
    let response = SpektraResponse {
        word: word.to_string(),
        prompt: prompt.to_string(),
    };
    if let Ok(json) = serde_json::to_string(&response) {
        println!("{}", json);
    }
}

/// Output RTAP response as JSON
fn output_rtap_json(prompt: &str, part: u8) {
    #[derive(Serialize)]
    struct RtapResponse {
        prompt: String,
        part: u8,
    }

    let response = RtapResponse {
        prompt: prompt.to_string(),
        part,
    };

    if let Ok(json) = serde_json::to_string(&response) {
        println!("{}", json);
    }
}

/// Output phase relation results as JSON
fn output_phase_json(phase_results: &[kern::core::PhaseRelationResult]) {
    let relations: Vec<PhaseRelationItem> = phase_results
        .iter()
        .map(|pr| PhaseRelationItem {
            left_input: pr.left_input.clone(),
            right_input: pr.right_input.clone(),
            left_value: pr.left_value,
            right_value: pr.right_value,
            left_compartment: pr.left_compartment,
            right_compartment: pr.right_compartment,
            phase: pr.phase,
            cipher: pr.cipher.clone(),
        })
        .collect();

    let response = PhaseResponse { relations };
    if let Ok(json) = serde_json::to_string(&response) {
        println!("{}", json);
    }
}

/// Output date results as JSON
fn output_date_json(
    offsets: &[i32],
    formatted_dates: &[chrono::NaiveDate],
    results: &[Option<&KernResult>],
    bedeutungen: &HashMap<u32, kern::core::Bedeutung>,
    lang: Lang,
    debug: bool,
) {
    let mut dates = Vec::new();

    for (i, off) in offsets.iter().enumerate() {
        if let Some(result) = results.get(i).and_then(|r| *r) {
            let date = formatted_dates.get(i).map(|d| d.format("%d.%m.%Y").to_string()).unwrap_or_default();
            let meaning = bedeutungen
                .get(&result.value)
                .and_then(|b| b.text.as_deref())
                .unwrap_or_else(|| lang.missing_meaning())
                .to_string();

            dates.push(DateItem {
                offset: *off,
                date,
                value: result.value,
                meaning,
                chain: if debug { Some(result.trace.clone()) } else { None },
            });
        }
    }

    let response = DateResponse {
        lang: lang.code(),
        dates,
    };
    if let Ok(json) = serde_json::to_string(&response) {
        println!("{}", json);
    }
}

/// Output lookup results as JSON
fn output_lookup_json(
    lookup_entries: &[LookupEntry],
    bedeutungen: &HashMap<u32, kern::core::Bedeutung>,
    lang: Lang,
    show_pos: bool,
    show_neg: bool,
    show_full: bool,
) {
    let mut items = Vec::new();

    for result in lookup_entries {
        let entry = bedeutungen.get(&result.value);
        let meaning = entry
            .and_then(|b| b.text.as_deref())
            .unwrap_or_else(|| lang.missing_meaning())
            .to_string();

        let positive = if show_pos || show_full {
            entry.and_then(|b| b.licht.clone())
        } else {
            None
        };

        let negative = if show_neg || show_full {
            entry.and_then(|b| b.schatten.clone())
        } else {
            None
        };

        items.push(LookupResponse {
            number: result.value,
            meaning,
            positive,
            negative,
        });
    }

    let response = LookupListResponse {
        lang: lang.code(),
        items,
    };
    if let Ok(json) = serde_json::to_string(&response) {
        println!("{}", json);
    }
}

/// Output reduce results as JSON
fn output_reduce_json(
    args: &[String],
    result_set: &kern::core::ResultSet,
    selected_ciphers: &[Box<dyn Cipher>],
    debug: bool,
    show_length: bool,
) {
    let multi_cipher = selected_ciphers.len() > 1;
    let mut items = Vec::new();

    // Group results by input
    let mut input_map: HashMap<String, Vec<&KernResult>> = HashMap::new();
    for result in &result_set.results {
        if matches!(result.step.operation, Operation::Reduce) {
            input_map.entry(result.source.clone()).or_default().push(result);
        }
    }

    // Build items in order of args
    for arg in args {
        if let Some(results) = input_map.get(arg) {
            let length = if show_length { Some(arg.chars().count()) } else { None };

            if multi_cipher {
                // Multi-cipher mode: include ciphers array
                let ciphers: Vec<CipherResult> = results
                    .iter()
                    .map(|r| {
                        let descriptor = descriptors()
                            .into_iter()
                            .find(|d| d.name == r.cipher)
                            .unwrap();

                        CipherResult {
                            name: descriptor.name.to_string(),
                            code: descriptor.short.to_string(),
                            value: r.value,
                            chain: if debug { Some(r.trace.clone()) } else { None },
                        }
                    })
                    .collect();

                items.push(ReduceItem {
                    input: arg.clone(),
                    length,
                    ciphers: Some(ciphers),
                    value: None,
                    chain: None,
                });
            } else {
                // Single cipher mode: direct value and chain
                if let Some(result) = results.first() {
                    items.push(ReduceItem {
                        input: arg.clone(),
                        length,
                        ciphers: None,
                        value: Some(result.value),
                        chain: if debug { Some(result.trace.clone()) } else { None },
                    });
                }
            }
        }
    }

    // Only reported when it was actually computed — no invented zero.
    let total = result_set.results
        .iter()
        .find(|r| matches!(r.step.operation, Operation::AggregateTotal))
        .map(|r| r.value);

    let total_chain = if debug {
        result_set.results
            .iter()
            .find(|r| matches!(r.step.operation, Operation::AggregateTotal))
            .map(|r| r.trace.clone())
    } else {
        None
    };

    // Items are always reported. Hiding them under --total made one flag do two
    // things, and made the piped output disagree with the TTY output, which has
    // always shown both (issue #23).
    let response = ReduceResponse {
        items,
        total,
        total_chain,
    };

    if let Ok(json) = serde_json::to_string(&response) {
        println!("{}", json);
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

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

struct ParsedPipeline {
    inputs: Vec<String>,
    steps: Vec<Step>,
    is_phase_mode: bool,
}

/// Parse tokens into a list of input strings and steps.
/// If phase_mode is true, creates PhaseRelation steps for all pairs.
/// Errors carry their own [`ErrorCode`] rather than collapsing into one, so the
/// CLI reports the same code the server would for the same situation
/// (docs/PRINCIPLES.md §6).
fn parse_pipeline_tokens(
    tokens: &[String],
    _cipher_aliases: &HashMap<String, String>,
    phase_mode: bool,
    show_total: bool,
    show_lookup: bool,
) -> Result<ParsedPipeline, (ErrorCode, String)> {
    // Every token is an input. Flags never reach here — clap parses them
    // wherever they appear. This function used to fish `-t` and `-l` out of the
    // token stream by hand, which is exactly why those two worked after the
    // input while every other flag was silently reduced as a word.
    let inputs: Vec<String> = tokens.to_vec();
    let mut steps = Vec::new();

    // If we're in phase relation mode
    if phase_mode {
        if inputs.len() < 2 {
            // Same code the server returns for this situation.
            return Err((
                ErrorCode::InsufficientInputs,
                "phase relation matrix requires at least 2 inputs".to_string(),
            ));
        }

        // --total and --lookup are not supported with phase relations (for now).
        // Read from the parsed flags rather than from the token stream, so the
        // rejection no longer depends on where the flag was typed.
        if show_total {
            return Err((
                ErrorCode::InvalidArguments,
                "--total is not supported with phase relation mode".to_string(),
            ));
        }
        if show_lookup {
            return Err((
                ErrorCode::InvalidArguments,
                "--lookup is not supported with phase relation mode".to_string(),
            ));
        }

        // Generate matrix pairs from all inputs
        let pairs = generate_matrix_pairs(inputs.len());

        // Create PhaseRelation steps for each pair
        for (left_idx, right_idx) in pairs {
            let step = Step::new(0, 0, Operation::PhaseRelation)
                .with_metadata(StepMetadata::PhaseRelation {
                    left_index: left_idx,
                    right_index: right_idx,
                });
            steps.push(step);
        }

        return Ok(ParsedPipeline {
            inputs,
            steps,
            is_phase_mode: true,
        });
    }

    // Regular mode: create reduce steps
    for (idx, _input) in inputs.iter().enumerate() {
        let step = Step::new(idx, 0, Operation::Reduce);
        steps.push(step);
    }

    Ok(ParsedPipeline {
        inputs,
        steps,
        is_phase_mode: false,
    })
}
