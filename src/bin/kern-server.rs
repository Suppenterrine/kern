use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
};
use chrono::{Duration, Local};
use kern::core::{
    Bedeutung, Cipher, FlowContext, FlowFlags, KernResult, Operation, Pipeline, Step,
    descriptors, load_bedeutungen, lookup, parse_range, reduce_number_steps, reduce_number_verbose,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, net::SocketAddr, sync::Arc};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const PKG_NAME: &str = env!("CARGO_PKG_NAME");

#[derive(Clone)]
struct AppState {
    map: Arc<HashMap<u32, Bedeutung>>,
}

// ============================================================================
// Root Endpoint - API Overview
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
    endpoints: Vec<EndpointInfo>,
    examples: HashMap<&'static str, &'static str>,
}

async fn root_handler() -> Json<ApiOverview> {
    let mut examples = HashMap::new();
    examples.insert("reduce_simple", "/reduce?input=Wickfeld");
    examples.insert("reduce_debug", "/reduce?input=Wickfeld&debug=true");
    examples.insert("reduce_multi", "/reduce?input=Test,Love,Life");
    examples.insert("reduce_cipher", "/reduce?input=Test&cipher=ord,py");
    examples.insert("reduce_all_ciphers", "/reduce?input=Test&cipher=all");
    examples.insert("lookup_single", "/lookup/7?parts=full");
    examples.insert("lookup_multi", "/lookup?numbers=1,7,11&parts=full");
    examples.insert("date_range", "/date?range=0..7&debug=true");
    examples.insert("spektra", "/spektra?word=Love");

    Json(ApiOverview {
        name: "KERN API",
        version: VERSION,
        endpoints: vec![
            EndpointInfo {
                path: "/",
                method: "GET",
                description: "API overview and documentation",
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
                description: "Get meaning of a single number",
            },
            EndpointInfo {
                path: "/lookup",
                method: "GET",
                description: "Get meanings of multiple numbers",
            },
            EndpointInfo {
                path: "/date",
                method: "GET",
                description: "Analyze date range with numerology",
            },
            EndpointInfo {
                path: "/spektra",
                method: "GET",
                description: "Multi-cipher spectral analysis for LLM prompt generation",
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
    #[serde(rename = "onlyTotal", default)]
    only_total: bool,
    cipher: Option<String>, // NEW: comma-separated cipher codes or "all"
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
    #[serde(skip_serializing_if = "Option::is_none")]
    items: Option<Vec<ReduceItem>>,
    total: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    total_chain: Option<Vec<String>>,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

async fn reduce_handler(
    Query(params): Query<ReduceParams>,
) -> Result<Json<ReduceResponse>, (StatusCode, Json<ErrorResponse>)> {
    let input = params
        .input
        .as_ref()
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "input parameter missing".into(),
                }),
            )
        })?;

    // Parse inputs (comma-separated)
    let inputs: Vec<&str> = input
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    if inputs.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "no valid inputs provided".into(),
            }),
        ));
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
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "cipher parameter provided but no valid cipher codes found".into(),
            }),
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
                    return Err((
                        StatusCode::BAD_REQUEST,
                        Json(ErrorResponse {
                            error: format!("unknown cipher code: {}", code),
                        }),
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

            if !params.only_total {
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
            if !params.only_total {
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

    // Calculate total
    let sum: u32 = results.iter().sum();
    let (total, total_chain) = if params.debug {
        let (val, chain) = reduce_number_steps(&sum.to_string());
        (val, Some(chain))
    } else {
        (reduce_number_verbose(&sum.to_string(), false), None)
    };

    let response = ReduceResponse {
        items: if params.only_total { None } else { Some(items) },
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
    meaning: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    positive: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    negative: Option<String>,
}

#[derive(Deserialize)]
struct LookupPartsParam {
    parts: Option<String>, // "pos", "neg", "both", "full" (also supports legacy: "light", "shadow")
}

async fn lookup_handler(
    Path(number): Path<u32>,
    Query(param): Query<LookupPartsParam>,
    State(state): State<AppState>,
) -> Json<LookupResponse> {
    let meaning = lookup(number, &state.map).to_string();
    let entry = state.map.get(&number);
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
    Json(LookupResponse {
        number,
        meaning,
        positive,
        negative,
    })
}

#[derive(Deserialize)]
struct LookupParams {
    numbers: String,
    parts: Option<String>, // optional: "pos", "neg", "both", "full" (also supports legacy: "light", "shadow")
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
    items: Vec<LookupItem>,
}

async fn lookup_multi_handler(
    Query(params): Query<LookupParams>,
    State(state): State<AppState>,
) -> Json<LookupListResponse> {
    let mut items = Vec::new();
    let sel = params.parts.as_deref();

    // Support both new and legacy parameter names
    let want_positive = matches!(sel, Some("pos") | Some("light") | Some("both") | Some("full"));
    let want_negative = matches!(sel, Some("neg") | Some("shadow") | Some("both") | Some("full"));

    for part in params.numbers.split(',') {
        let s = part.trim();
        if s.is_empty() {
            continue;
        }
        if let Ok(n) = s.parse::<u32>() {
            let meaning = lookup(n, &state.map).to_string();
            let entry = state.map.get(&n);
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
    Json(LookupListResponse { items })
}

// ============================================================================
// Date Endpoint
// ============================================================================

#[derive(Deserialize)]
struct DateParams {
    range: String,
    #[serde(default)]
    debug: bool,
}

#[derive(Serialize)]
struct DateItem {
    offset: i32,
    date: String,
    value: u32,
    meaning: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    chain: Option<Vec<String>>,
}

#[derive(Serialize)]
struct DateResponse {
    dates: Vec<DateItem>,
}

async fn date_handler(
    Query(params): Query<DateParams>,
    State(state): State<AppState>,
) -> Result<Json<DateResponse>, (StatusCode, Json<ErrorResponse>)> {
    let offsets = parse_range(&params.range)
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(ErrorResponse { error: e })))?;
    let today = Local::now().date_naive();
    let mut dates = Vec::new();
    for off in offsets {
        let date = today + Duration::days(off as i64);
        let date_str = date.format("%d.%m.%Y").to_string();
        let raw = date.format("%d%m%Y").to_string();
        let (num, chain) = reduce_number_steps(&raw);
        let meaning = lookup(num, &state.map).to_string();
        dates.push(DateItem {
            offset: off,
            date: date_str,
            value: num,
            meaning,
            chain: if params.debug { Some(chain) } else { None },
        });
    }
    Ok(Json(DateResponse { dates }))
}

// ============================================================================
// Spektra Endpoint
// ============================================================================

#[derive(Deserialize)]
struct SpektraParams {
    word: Option<String>,
}

#[derive(Serialize)]
struct SpektraResponse {
    prompt: String,
}

async fn spektra_handler(
    Query(params): Query<SpektraParams>,
    State(state): State<AppState>,
) -> Result<Json<SpektraResponse>, (StatusCode, Json<ErrorResponse>)> {
    let word = params
        .word
        .as_ref()
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "word parameter missing".into(),
                }),
            )
        })?;

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

    let _result_set = pipeline.run(&mut ctx, &[word.clone()], &spektra_ciphers);

    // Collect results from memory (all reduce operations)
    let reduce_results: Vec<KernResult> = ctx
        .memory
        .iter()
        .filter(|res| matches!(res.step.operation, Operation::Reduce))
        .cloned()
        .collect();

    // Build spektra prompt
    match kern::core::spektra::build_spektra_prompt(word, &reduce_results, &state.map) {
        Ok(prompt) => Ok(Json(SpektraResponse { prompt })),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Error building spektra prompt: {}", e),
            }),
        )),
    }
}

// ============================================================================
// Main Server Setup
// ============================================================================

#[tokio::main]
async fn main() {
    let map = load_bedeutungen();
    let state = AppState { map: Arc::new(map) };

    let app = Router::new()
        .route("/", get(root_handler))
        .route("/version", get(version_handler))
        .route("/reduce", get(reduce_handler))
        .route("/lookup", get(lookup_multi_handler))
        .route("/lookup/:number", get(lookup_handler))
        .route("/date", get(date_handler))
        .route("/spektra", get(spektra_handler))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("KERN Server v{} listening on http://{}", VERSION, addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
