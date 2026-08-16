use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
};
use chrono::{Duration, Local};
use kern::core::{
    Bedeutung, Cipher, ErrorCode, FlowContext, FlowFlags, KernResult, Lang, Operation, Pipeline,
    Step, StepMetadata, alphabet_index, descriptors, generate_matrix_pairs, load_all_bedeutungen,
    lookup_lang, parse_range, reduce_number_steps, reduce_number_verbose,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, net::SocketAddr, sync::Arc};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const PKG_NAME: &str = env!("CARGO_PKG_NAME");

#[derive(Clone)]
struct AppState {
    maps: Arc<HashMap<Lang, HashMap<u32, Bedeutung>>>,
}

impl AppState {
    /// Meanings for `lang`. Every variant is loaded at startup, so the fallback
    /// to the default language is unreachable in practice.
    fn map(&self, lang: Lang) -> &HashMap<u32, Bedeutung> {
        self.maps
            .get(&lang)
            .or_else(|| self.maps.get(&Lang::default()))
            .expect("default language is always loaded")
    }
}

/// Resolves the optional `lang` query parameter: missing or empty falls back to
/// the default language, an unknown tag is rejected with 400. Rejecting is
/// deliberate — silently answering a typo in the wrong language is worse than
/// an explicit error.
fn resolve_lang(raw: Option<&str>) -> Result<Lang, ApiError> {
    match raw.map(str::trim).filter(|s| !s.is_empty()) {
        None => Ok(Lang::default()),
        Some(tag) => Lang::parse(tag).ok_or_else(|| {
            bad_request(
                ErrorCode::UnsupportedLanguage,
                format!(
                    "unsupported language '{tag}'. supported: {}",
                    Lang::supported()
                ),
            )
        }),
    }
}

/// Resolves `lang` for the prompt endpoints. A language the API supports for
/// content but that has no prompt (French) is **rejected** here, with its own
/// code so clients can tell it apart from an unknown language. Answering in a
/// different language than asked would be a wrong answer disguised as a
/// successful one — see docs/PRINCIPLES.md.
fn resolve_prompt_lang(raw: Option<&str>) -> Result<Lang, ApiError> {
    let lang = resolve_lang(raw)?;
    if !lang.has_prompts() {
        return Err(bad_request(
            ErrorCode::LanguageNotAvailable,
            format!(
                "prompts are not available in '{}'. available: {}",
                lang.code(),
                Lang::prompt_langs()
            ),
        ));
    }
    Ok(lang)
}

// ============================================================================
// Root Endpoint - Service Descriptor
// ============================================================================

/// Deliberately small: `/` is what health checks and monitoring poll, so it
/// stays a cheap identity probe. The full endpoint listing lives at `/help`.
#[derive(Serialize)]
struct ServiceInfo {
    name: &'static str,
    version: &'static str,
    /// Language codes accepted by the `lang` parameter, in no particular order.
    /// The default is named separately by `default_language`.
    languages: Vec<&'static str>,
    default_language: &'static str,
    documentation: &'static str,
}

async fn root_handler() -> Json<ServiceInfo> {
    Json(ServiceInfo {
        name: "KERN API",
        version: VERSION,
        languages: Lang::ALL.iter().map(|l| l.code()).collect(),
        default_language: Lang::default().code(),
        documentation: "/help",
    })
}

// ============================================================================
// Help Endpoint - Full API Overview
// ============================================================================

#[derive(Serialize)]
struct EndpointInfo {
    path: &'static str,
    method: &'static str,
    description: &'static str,
}

#[derive(Serialize)]
struct ApiOverview {
    name: &'static str,
    version: &'static str,
    /// Language codes accepted by the `lang` parameter, in no particular order.
    /// The default is named separately by `default_language`.
    languages: Vec<&'static str>,
    default_language: &'static str,
    endpoints: Vec<EndpointInfo>,
    examples: HashMap<&'static str, &'static str>,
}

async fn help_handler() -> Json<ApiOverview> {
    let mut examples = HashMap::new();
    examples.insert("reduce_simple", "/reduce?input=Wickfeld");
    examples.insert("reduce_debug", "/reduce?input=Wickfeld&debug=true");
    examples.insert("reduce_multi", "/reduce?input=Test,Love,Life");
    examples.insert("reduce_cipher", "/reduce?input=Test&cipher=ord,py");
    examples.insert("reduce_all_ciphers", "/reduce?input=Test&cipher=all");
    examples.insert("lookup_single", "/lookup/7?parts=full");
    examples.insert("lookup_all", "/lookup");
    examples.insert("lookup_multi", "/lookup?numbers=1,7,11&parts=full");
    examples.insert("date_range", "/date?range=0..7&debug=true");
    examples.insert("spektra", "/spektra?word=Love");
    examples.insert("phase_simple", "/phase?inputs=a,b");
    examples.insert("phase_multi", "/phase?inputs=a,b,c&cipher=all");
    examples.insert("rtap_single", "/rtap?part=1");
    examples.insert("rtap_both", "/rtap?part=both");
    examples.insert("index_single", "/index?input=kassel");
    examples.insert("index_multi", "/index?input=Wickfeld,Love");
    examples.insert("lookup_german", "/lookup/7?parts=full&lang=de");
    examples.insert("lookup_french", "/lookup?numbers=1,7,11&parts=full&lang=fr");
    examples.insert("date_german", "/date?range=0..7&lang=de");

    Json(ApiOverview {
        name: "KERN API",
        version: VERSION,
        languages: Lang::ALL.iter().map(|l| l.code()).collect(),
        default_language: Lang::default().code(),
        endpoints: vec![
            EndpointInfo {
                path: "/",
                method: "GET",
                description: "Service descriptor: name, version, languages, link to this help",
            },
            EndpointInfo {
                path: "/help",
                method: "GET",
                description: "This endpoint listing with examples",
            },
            EndpointInfo {
                path: "/version",
                method: "GET",
                description: "Version information",
            },
            EndpointInfo {
                path: "/reduce",
                method: "GET",
                description: "Reduce text to numerology values (supports multiple inputs and ciphers)",
            },
            EndpointInfo {
                path: "/lookup/:number",
                method: "GET",
                description: "Get meaning of a single number (supports lang=de|en|fr)",
            },
            EndpointInfo {
                path: "/lookup",
                method: "GET",
                description: "Get meanings of multiple numbers (supports lang=de|en|fr)",
            },
            EndpointInfo {
                path: "/date",
                method: "GET",
                description: "Analyze date range with numerology (supports lang=de|en|fr)",
            },
            EndpointInfo {
                path: "/spektra",
                method: "GET",
                description: "Multi-cipher spectral analysis for LLM prompt generation",
            },
            EndpointInfo {
                path: "/phase",
                method: "GET",
                description: "Calculate phase relation matrix for multiple inputs",
            },
            EndpointInfo {
                path: "/rtap",
                method: "GET",
                description: "Get RTAP (Rethinking Thoughts And Positions) prompts",
            },
            EndpointInfo {
                path: "/index",
                method: "GET",
                description:
                    "Alphabet-position lookup (A=1, B=2, ...). Cipher-independent, special \
                     characters skipped, duplicate letters deduplicated.",
            },
        ],
        examples,
    })
}

// ============================================================================
// Version Endpoint
// ============================================================================

#[derive(Serialize)]
struct VersionResponse {
    name: &'static str,
    version: &'static str,
}

async fn version_handler() -> Json<VersionResponse> {
    Json(VersionResponse {
        name: PKG_NAME,
        version: VERSION,
    })
}

// ============================================================================
// Reduce Endpoint - WITH Multi-Cipher Support
// ============================================================================

#[derive(Deserialize)]
struct ReduceParams {
    input: Option<String>,
    #[serde(default)]
    debug: bool,
    #[serde(default)]
    length: bool,
    /// Adds the aggregate total. It used to be computed and returned always,
    /// which meant every caller paid for it and none could tell whether it had
    /// been asked for. Mirrors the CLI's `--total`.
    #[serde(default)]
    total: bool,
    cipher: Option<String>, // comma-separated cipher codes or "all"
}

#[derive(Serialize)]
struct CipherResult {
    name: String,
    code: String,
    value: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    chain: Option<Vec<String>>,
}

#[derive(Serialize)]
struct ReduceItem {
    input: String, // NEW: Always include input text
    #[serde(skip_serializing_if = "Option::is_none")]
    length: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ciphers: Option<Vec<CipherResult>>, // NEW: Multi-cipher results
    #[serde(skip_serializing_if = "Option::is_none")]
    value: Option<u32>, // Legacy: single value (when no cipher param)
    #[serde(skip_serializing_if = "Option::is_none")]
    chain: Option<Vec<String>>, // Legacy: single chain (when no cipher param)
}

#[derive(Serialize)]
struct ReduceResponse {
    items: Vec<ReduceItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    total: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    total_chain: Option<Vec<String>>,
}

/// Error payload. `code` is a stable identifier meant for branching in client
/// code; `error` is human-readable prose and may be reworded at any time, so
/// clients must not match on it. Error messages are always English regardless
/// of the `lang` parameter — `lang` selects the language of the *content*, not
/// of the protocol.
#[derive(Serialize)]
struct ErrorResponse {
    code: ErrorCode,
    error: String,
}

type ApiError = (StatusCode, Json<ErrorResponse>);

fn bad_request(code: ErrorCode, error: impl Into<String>) -> ApiError {
    (
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
            code,
            error: error.into(),
        }),
    )
}

fn server_error(code: ErrorCode, error: impl Into<String>) -> ApiError {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse {
            code,
            error: error.into(),
        }),
    )
}

async fn reduce_handler(
    Query(params): Query<ReduceParams>,
) -> Result<Json<ReduceResponse>, ApiError> {
    let input = params
        .input
        .as_ref()
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| bad_request(ErrorCode::InputMissing, "input parameter missing"))?;

    // Parse inputs (comma-separated)
    let inputs: Vec<&str> = input
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    if inputs.is_empty() {
        return Err(bad_request(ErrorCode::NoValidInputs, "no valid inputs provided"));
    }

    // Parse cipher parameter
    let use_multi_cipher = params.cipher.is_some();
    let cipher_codes = if let Some(ref cipher_param) = params.cipher {
        let cipher_param = cipher_param.trim();
        if cipher_param == "all" {
            // Use all available ciphers
            descriptors()
                .iter()
                .map(|d| d.short.to_string())
                .collect()
        } else {
            // Parse comma-separated cipher codes
            cipher_param
                .split(',')
                .map(|s| s.trim().to_lowercase())
                .filter(|s| !s.is_empty())
                .collect()
        }
    } else {
        vec![] // Empty if no cipher param (use default behavior)
    };

    // Validate cipher codes if provided
    if use_multi_cipher && cipher_codes.is_empty() {
        return Err(bad_request(
            ErrorCode::NoValidCiphers,
            "cipher parameter provided but no valid cipher codes found",
        ));
    }

    // Build cipher instances if using multi-cipher mode
    let ciphers: Vec<Box<dyn Cipher>> = if use_multi_cipher {
        let all_descriptors = descriptors();
        let mut result = Vec::new();
        for code in &cipher_codes {
            match all_descriptors.iter().find(|d| d.short.to_lowercase() == *code) {
                Some(descriptor) => result.push((descriptor.factory)()),
                None => {
                    return Err(bad_request(
                        ErrorCode::UnknownCipher,
                        format!("unknown cipher code: {code}"),
                    ));
                }
            }
        }
        result
    } else {
        vec![]
    };

    let mut results = Vec::new();
    let mut items = Vec::new();

    for word in &inputs {
        if use_multi_cipher {
            // Multi-cipher mode: calculate value for each cipher
            let mut cipher_results = Vec::new();

            for cipher in &ciphers {
                let (value, chain) = reduce_number_steps_with_cipher(word, cipher.as_ref());
                cipher_results.push(CipherResult {
                    name: cipher.name().to_string(),
                    code: descriptors()
                        .iter()
                        .find(|d| d.name == cipher.name())
                        .map(|d| d.short.to_string())
                        .unwrap_or_else(|| "unknown".to_string()),
                    value,
                    chain: if params.debug { Some(chain) } else { None },
                });
            }

            // Use first cipher's value for total calculation (or could sum all)
            let first_value = cipher_results.first().map(|cr| cr.value).unwrap_or(0);
            results.push(first_value);

            {
                items.push(ReduceItem {
                    input: word.to_string(),
                    length: if params.length {
                        Some(word.chars().count())
                    } else {
                        None
                    },
                    ciphers: Some(cipher_results),
                    value: None,
                    chain: None,
                });
            }
        } else {
            // Legacy single-cipher mode (default Ordinal)
            let (value, chain) = reduce_number_steps(word);
            results.push(value);
            {
                items.push(ReduceItem {
                    input: word.to_string(),
                    length: if params.length {
                        Some(word.chars().count())
                    } else {
                        None
                    },
                    ciphers: None,
                    value: Some(value),
                    chain: if params.debug { Some(chain) } else { None },
                });
            }
        }
    }

    // Only computed when asked for. Reporting a total nobody requested made it
    // impossible to tell a real total from a default (issue #23).
    let (total, total_chain) = if params.total {
        let sum: u32 = results.iter().sum();
        if params.debug {
            let (val, chain) = reduce_number_steps(&sum.to_string());
            (Some(val), Some(chain))
        } else {
            (Some(reduce_number_verbose(&sum.to_string(), false)), None)
        }
    } else {
        (None, None)
    };

    let response = ReduceResponse {
        items,
        total,
        total_chain,
    };

    Ok(Json(response))
}

// Helper function to reduce with a specific cipher
fn reduce_number_steps_with_cipher(s: &str, cipher: &dyn Cipher) -> (u32, Vec<String>) {
    let mut chain = Vec::new();
    let mut current = s.to_string();
    chain.push(current.clone());

    // Calculate initial sum using cipher
    let mut sum: u32 = current
        .chars()
        .filter_map(|ch| {
            let val = cipher.char_to_value(ch);
            if val > 0 { Some(val) } else { None }
        })
        .sum();

    // Reduce until we hit a master number or single digit
    loop {
        if sum < 10 || sum == 11 || sum == 22 || sum == 33 {
            break;
        }
        current = sum.to_string();
        chain.push(current.clone());
        sum = current.chars().filter_map(|c| c.to_digit(10)).sum();
    }

    if sum >= 10 {
        chain.push(sum.to_string());
    }

    (sum, chain)
}

// ============================================================================
// Lookup Endpoints
// ============================================================================

#[derive(Serialize)]
struct LookupResponse {
    number: u32,
    lang: &'static str,
    meaning: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    positive: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    negative: Option<String>,
}

#[derive(Deserialize)]
struct LookupPartsParam {
    parts: Option<String>, // "pos", "neg", "both", "full" (also supports legacy: "light", "shadow")
    lang: Option<String>,  // "de" (default), "en", "fr"
}

async fn lookup_handler(
    Path(number): Path<u32>,
    Query(param): Query<LookupPartsParam>,
    State(state): State<AppState>,
) -> Result<Json<LookupResponse>, ApiError> {
    let lang = resolve_lang(param.lang.as_deref())?;
    let map = state.map(lang);

    let meaning = lookup_lang(number, map, lang).to_string();
    let entry = map.get(&number);
    let sel = param.parts.as_deref();

    // Support both new ("pos"/"neg"/"full") and legacy ("light"/"shadow") parameter names
    let want_positive = matches!(sel, Some("pos") | Some("light") | Some("both") | Some("full"));
    let want_negative = matches!(sel, Some("neg") | Some("shadow") | Some("both") | Some("full"));

    let positive = if want_positive {
        entry.and_then(|b| b.licht.clone())
    } else {
        None
    };
    let negative = if want_negative {
        entry.and_then(|b| b.schatten.clone())
    } else {
        None
    };
    Ok(Json(LookupResponse {
        number,
        lang: lang.code(),
        meaning,
        positive,
        negative,
    }))
}

#[derive(Deserialize)]
struct LookupParams {
    numbers: Option<String>, // optional: if omitted, returns all meanings
    parts: Option<String>,   // optional: "pos", "neg", "both", "full" (also supports legacy: "light", "shadow")
    lang: Option<String>,    // optional: "de" (default), "en", "fr"
}

#[derive(Serialize)]
struct LookupItem {
    number: u32,
    meaning: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    positive: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    negative: Option<String>,
}

#[derive(Serialize)]
struct LookupListResponse {
    lang: &'static str,
    items: Vec<LookupItem>,
}

async fn lookup_multi_handler(
    Query(params): Query<LookupParams>,
    State(state): State<AppState>,
) -> Result<Json<LookupListResponse>, ApiError> {
    let lang = resolve_lang(params.lang.as_deref())?;
    let map = state.map(lang);

    let mut items = Vec::new();
    let sel = params.parts.as_deref();

    // Support both new and legacy parameter names
    let want_positive = matches!(sel, Some("pos") | Some("light") | Some("both") | Some("full"));
    let want_negative = matches!(sel, Some("neg") | Some("shadow") | Some("both") | Some("full"));

    // A `numbers` parameter that is absent, empty or blank means "all meanings".
    // Matching on it directly avoids re-checking and then unwrapping.
    let requested = params
        .numbers
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());

    let Some(numbers) = requested else {
        let mut all_numbers: Vec<u32> = map.keys().copied().collect();
        all_numbers.sort();

        for n in all_numbers {
            let meaning = lookup_lang(n, map, lang).to_string();
            items.push(LookupItem {
                number: n,
                meaning,
                positive: None,
                negative: None,
            });
        }

        return Ok(Json(LookupListResponse {
            lang: lang.code(),
            items,
        }));
    };

    {
        // Parse specific numbers from parameter
        for part in numbers.split(',') {
            let s = part.trim();
            if s.is_empty() {
                continue;
            }
            if let Ok(n) = s.parse::<u32>() {
                let meaning = lookup_lang(n, map, lang).to_string();
                let entry = map.get(&n);
                let positive = if want_positive {
                    entry.and_then(|b| b.licht.clone())
                } else {
                    None
                };
                let negative = if want_negative {
                    entry.and_then(|b| b.schatten.clone())
                } else {
                    None
                };
                items.push(LookupItem {
                    number: n,
                    meaning,
                    positive,
                    negative,
                });
            }
        }
    }
    Ok(Json(LookupListResponse {
        lang: lang.code(),
        items,
    }))
}

// ============================================================================
// Alphabet Index Endpoint
// ============================================================================

#[derive(Deserialize)]
struct IndexParams {
    input: Option<String>, // comma-separated words; letters mapped to A=1, B=2, ...
}

#[derive(Serialize)]
struct IndexEntry {
    letter: String,
    index: u32,
}

#[derive(Serialize)]
struct IndexItem {
    input: String,
    entries: Vec<IndexEntry>,
}

#[derive(Serialize)]
struct IndexListResponse {
    items: Vec<IndexItem>,
}

async fn index_handler(
    Query(params): Query<IndexParams>,
) -> Result<Json<IndexListResponse>, ApiError> {
    let input = params
        .input
        .as_ref()
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| bad_request(ErrorCode::InputMissing, "input parameter missing"))?;

    let mut items = Vec::new();
    for word in input.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()) {
        let entries: Vec<IndexEntry> = alphabet_index(word)
            .into_iter()
            .map(|(ch, idx)| IndexEntry {
                letter: ch.to_string(),
                index: idx,
            })
            .collect();
        items.push(IndexItem {
            input: word.to_string(),
            entries,
        });
    }

    Ok(Json(IndexListResponse { items }))
}

// ============================================================================
// Date Endpoint
// ============================================================================

#[derive(Deserialize)]
struct DateParams {
    range: String,
    #[serde(default)]
    debug: bool,
    /// Meanings are lookup information, so they require asking for a lookup —
    /// the same rule the CLI's `--lookup` follows.
    #[serde(default)]
    lookup: bool,
    lang: Option<String>, // optional: "de" (default), "en", "fr"
}

#[derive(Serialize)]
struct DateItem {
    offset: i32,
    date: String,
    value: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    meaning: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    chain: Option<Vec<String>>,
}

#[derive(Serialize)]
struct DateResponse {
    lang: &'static str,
    dates: Vec<DateItem>,
}

async fn date_handler(
    Query(params): Query<DateParams>,
    State(state): State<AppState>,
) -> Result<Json<DateResponse>, ApiError> {
    let lang = resolve_lang(params.lang.as_deref())?;
    let map = state.map(lang);

    let offsets = parse_range(&params.range).map_err(|e| bad_request(ErrorCode::InvalidRange, e))?;
    let today = Local::now().date_naive();
    let mut dates = Vec::new();
    for off in offsets {
        let date = today + Duration::days(off as i64);
        let date_str = date.format("%d.%m.%Y").to_string();
        let raw = date.format("%d%m%Y").to_string();
        let (num, chain) = reduce_number_steps(&raw);
        dates.push(DateItem {
            offset: off,
            date: date_str,
            value: num,
            meaning: if params.lookup {
                Some(lookup_lang(num, map, lang).to_string())
            } else {
                None
            },
            chain: if params.debug { Some(chain) } else { None },
        });
    }
    Ok(Json(DateResponse {
        lang: lang.code(),
        dates,
    }))
}

// ============================================================================
// Spektra Endpoint
// ============================================================================

#[derive(Deserialize)]
struct SpektraParams {
    word: Option<String>,
    lang: Option<String>, // "en" (default) or "de"; fr falls back to en
}

#[derive(Serialize)]
struct SpektraResponse {
    lang: &'static str,
    prompt: String,
}

async fn spektra_handler(
    Query(params): Query<SpektraParams>,
    State(state): State<AppState>,
) -> Result<Json<SpektraResponse>, ApiError> {
    let word = params
        .word
        .as_ref()
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| bad_request(ErrorCode::WordMissing, "word parameter missing"))?;

    // The prompt exists in German and English only; anything else is rejected.
    // The meanings woven in must match the prompt language.
    let prompt_lang = resolve_prompt_lang(params.lang.as_deref())?;

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
        verbose: false,
        ciphers: cipher_names,
        total: false,
    });

    let _result_set = pipeline.run(&mut ctx, std::slice::from_ref(word), &spektra_ciphers);

    // Collect results from memory (all reduce operations)
    let reduce_results: Vec<KernResult> = ctx
        .memory
        .iter()
        .filter(|res| matches!(res.step.operation, Operation::Reduce))
        .cloned()
        .collect();

    match kern::core::spektra::build_spektra_prompt(
        word,
        &reduce_results,
        state.map(prompt_lang),
        prompt_lang,
    ) {
        Ok(prompt) => Ok(Json(SpektraResponse {
            lang: prompt_lang.code(),
            prompt,
        })),
        Err(e) => Err(server_error(
            ErrorCode::SpektraFailed,
            format!("error building spektra prompt: {e}"),
        )),
    }
}

// ============================================================================
// Phase Relation Matrix Endpoint
// ============================================================================

#[derive(Deserialize)]
struct PhaseParams {
    inputs: String,         // Comma-separated inputs
    cipher: Option<String>, // Optional cipher codes (comma-separated or "all")
}

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

#[derive(Serialize)]
struct PhaseResponse {
    relations: Vec<PhaseRelationItem>,
}

// ============================================================================
// RTAP Endpoint
// ============================================================================

#[derive(Deserialize)]
struct RtapParams {
    part: Option<String>, // "1", "2", or "both"
    lang: Option<String>, // "en" (default) or "de"; fr falls back to en
}

#[derive(Serialize)]
struct RtapResponse {
    lang: &'static str,
    prompt: String,
    part: u8,
}

#[derive(Serialize)]
struct RtapBothResponse {
    lang: &'static str,
    prompts: Vec<RtapResponse>,
}

async fn phase_handler(
    Query(params): Query<PhaseParams>,
) -> Result<Json<PhaseResponse>, ApiError> {
    // Parse inputs
    let inputs: Vec<String> = params
        .inputs
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if inputs.len() < 2 {
        return Err(bad_request(
            ErrorCode::InsufficientInputs,
            "phase relation matrix requires at least 2 inputs",
        ));
    }

    // Parse cipher parameter
    let cipher_codes = if let Some(ref cipher_param) = params.cipher {
        let cipher_param = cipher_param.trim();
        if cipher_param == "all" {
            // Use all available ciphers
            descriptors()
                .iter()
                .map(|d| d.short.to_string())
                .collect()
        } else {
            // Parse comma-separated cipher codes
            cipher_param
                .split(',')
                .map(|s| s.trim().to_lowercase())
                .filter(|s| !s.is_empty())
                .collect()
        }
    } else {
        vec!["or".to_string()] // Default to ordinal
    };

    // Build cipher instances
    let all_descriptors = descriptors();
    let mut ciphers: Vec<Box<dyn Cipher>> = Vec::new();
    for code in &cipher_codes {
        match all_descriptors.iter().find(|d| d.short.to_lowercase() == *code || d.name.to_lowercase() == *code) {
            Some(descriptor) => ciphers.push((descriptor.factory)()),
            None => {
                return Err(bad_request(
                    ErrorCode::UnknownCipher,
                    format!("unknown cipher code: {code}"),
                ));
            }
        }
    }

    if ciphers.is_empty() {
        ciphers.push((all_descriptors[0].factory)()); // Fallback to ordinal
    }

    let cipher_names: Vec<String> = ciphers.iter().map(|c| c.name().to_string()).collect();

    // Generate matrix pairs
    let pairs = generate_matrix_pairs(inputs.len());

    // Build pipeline with PhaseRelation steps
    let mut pipeline = Pipeline::new();
    for (left_idx, right_idx) in pairs {
        let step = Step::new(0, 0, Operation::PhaseRelation)
            .with_metadata(StepMetadata::PhaseRelation {
                left_index: left_idx,
                right_index: right_idx,
            });
        pipeline.add_step(step);
    }

    // Execute pipeline
    let mut ctx = FlowContext::new(FlowFlags {
        verbose: false,
        ciphers: cipher_names,
        total: false,
    });

    let _result_set = pipeline.run(&mut ctx, &inputs, &ciphers);

    // Convert PhaseRelationResult to API response format
    let relations: Vec<PhaseRelationItem> = ctx
        .phase_results
        .iter()
        .map(|r| PhaseRelationItem {
            left_input: r.left_input.clone(),
            right_input: r.right_input.clone(),
            left_value: r.left_value,
            right_value: r.right_value,
            left_compartment: r.left_compartment,
            right_compartment: r.right_compartment,
            phase: r.phase,
            cipher: r.cipher.clone(),
        })
        .collect();

    Ok(Json(PhaseResponse { relations }))
}

async fn rtap_handler(
    Query(params): Query<RtapParams>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // RTAP prompts exist in German and English only; anything else is rejected.
    let prompt_lang = resolve_prompt_lang(params.lang.as_deref())?;
    let prompts = kern::core::load_rtap_prompts_lang(prompt_lang)
        .expect("resolve_prompt_lang guarantees prompts exist");

    let part_str = params.part.as_deref().unwrap_or("1");

    if part_str.eq_ignore_ascii_case("both") {
        let mut results = Vec::new();

        for part_num in [1u8, 2u8] {
            match kern::core::get_rtap_prompt(part_num, &prompts) {
                Some(prompt) => {
                    results.push(RtapResponse {
                        lang: prompt_lang.code(),
                        prompt: prompt.to_string(),
                        part: part_num,
                    });
                }
                None => {
                    return Err(server_error(
                        ErrorCode::RtapPromptMissing,
                        format!("RTAP prompt {part_num} not found"),
                    ));
                }
            }
        }

        let response = RtapBothResponse {
            lang: prompt_lang.code(),
            prompts: results,
        };
        Ok(Json(serde_json::to_value(response).unwrap()))
    } else {
        let part_num = part_str.parse::<u8>().map_err(|_| {
            bad_request(
                ErrorCode::InvalidRtapPart,
                format!("invalid part number: {part_str}. must be 1, 2, or 'both'"),
            )
        })?;

        if part_num != 1 && part_num != 2 {
            return Err(bad_request(
                ErrorCode::InvalidRtapPart,
                format!("invalid part number: {part_num}. must be 1 or 2"),
            ));
        }

        match kern::core::get_rtap_prompt(part_num, &prompts) {
            Some(prompt) => {
                let response = RtapResponse {
                    lang: prompt_lang.code(),
                    prompt: prompt.to_string(),
                    part: part_num,
                };
                Ok(Json(serde_json::to_value(response).unwrap()))
            }
            None => Err(server_error(
                ErrorCode::RtapPromptMissing,
                format!("RTAP prompt {part_num} not found in configuration"),
            )),
        }
    }
}

// ============================================================================
// Main Server Setup
// ============================================================================

#[tokio::main]
async fn main() {
    let state = AppState {
        maps: Arc::new(load_all_bedeutungen()),
    };

    let app = Router::new()
        .route("/", get(root_handler))
        .route("/help", get(help_handler))
        .route("/version", get(version_handler))
        .route("/reduce", get(reduce_handler))
        .route("/lookup", get(lookup_multi_handler))
        .route("/lookup/:number", get(lookup_handler))
        .route("/date", get(date_handler))
        .route("/spektra", get(spektra_handler))
        .route("/phase", get(phase_handler))
        .route("/rtap", get(rtap_handler))
        .route("/index", get(index_handler))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!(
        "KERN Server v{} listening on http://{} (languages: {})",
        VERSION,
        addr,
        Lang::supported()
    );
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
