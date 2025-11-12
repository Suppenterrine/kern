use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
};
use chrono::{Duration, Local};
use kern::core::{
    Bedeutung, load_bedeutungen, lookup, parse_range, reduce_number_steps, reduce_number_verbose,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, net::SocketAddr, sync::Arc};

#[derive(Clone)]
struct AppState {
    map: Arc<HashMap<u32, Bedeutung>>,
}

#[derive(Deserialize)]
struct ReduceParams {
    input: Option<String>,
    #[serde(default)]
    debug: bool,
    #[serde(default)]
    length: bool,
    #[serde(rename = "onlyTotal", default)]
    only_total: bool,
}

#[derive(Serialize)]
struct ReduceItem {
    value: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    length: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    chain: Option<Vec<String>>,
}

#[derive(Serialize)]
struct ReduceResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    items: Option<Vec<ReduceItem>>, // jede Eingabe mit Wert
    total: u32, // Gesamtsumme reduziert
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

    // Eingaben per Komma trennen
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

    let mut results = Vec::new();
    let mut items = Vec::new();

    for word in &inputs {
        let (value, chain) = reduce_number_steps(word);
        results.push(value);
        if !params.only_total {
            items.push(ReduceItem {
                value,
                length: if params.length {
                    Some(word.chars().count())
                } else {
                    None
                },
                chain: if params.debug { Some(chain) } else { None },
            });
        }
    }

    // Gesamtsumme berechnen
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

#[tokio::main]
async fn main() {
    let map = load_bedeutungen();
    let state = AppState { map: Arc::new(map) };

    let app = Router::new()
        .route("/reduce", get(reduce_handler))
        .route("/lookup", get(lookup_multi_handler))
        .route("/lookup/:number", get(lookup_handler))
        .route("/date", get(date_handler))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("Listening on http://{addr}");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
