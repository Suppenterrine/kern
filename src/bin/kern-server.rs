use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::get,
};
use chrono::{Duration, Local};
use kern::core::{Bedeutung, load_bedeutungen, lookup, parse_range, reduce_number_verbose};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, net::SocketAddr, path::Path as StdPath, sync::Arc};

#[derive(Clone)]
struct AppState {
    map: Arc<HashMap<u32, Bedeutung>>,
}

#[derive(Deserialize)]
struct ReduceParams {
    input: String,
}

#[derive(Serialize)]
struct ReduceResponse {
    values: Vec<u32>, // jede Eingabe mit Wert
    total: u32,       // Gesamtsumme reduziert
}

async fn reduce_handler(Query(params): Query<ReduceParams>) -> Json<ReduceResponse> {
    if params.input.trim().is_empty() {
        return Json(ReduceResponse { values: vec![], total: 0 });
    }

    // Eingaben per Komma trennen
    let inputs: Vec<&str> = params.input.split(',').map(|s| s.trim()).collect();

    let mut results = Vec::new();
    for word in inputs {
        results.push(reduce_number_verbose(word, false));
    }

    // Gesamtsumme berechnen
    let total = reduce_number_verbose(&results.iter().sum::<u32>().to_string(), false);

    Json(ReduceResponse { values: results, total })
}

#[derive(Serialize)]
struct LookupResponse {
    number: u32,
    meaning: String,
}

async fn lookup_handler(
    Path(number): Path<u32>,
    State(state): State<AppState>,
) -> Json<LookupResponse> {
    let meaning = lookup(number, &state.map).to_string();
    Json(LookupResponse { number, meaning })
}

#[derive(Deserialize)]
struct DateParams {
    range: String,
}

#[derive(Serialize)]
struct DateItem {
    offset: i32,
    date: String,
    value: u32,
    meaning: String,
}

#[derive(Serialize)]
struct DateResponse {
    dates: Vec<DateItem>,
}

async fn date_handler(
    Query(params): Query<DateParams>,
    State(state): State<AppState>,
) -> Result<Json<DateResponse>, axum::http::StatusCode> {
    let offsets = parse_range(&params.range).map_err(|_| axum::http::StatusCode::BAD_REQUEST)?;
    let today = Local::now().date_naive();
    let mut dates = Vec::new();
    for off in offsets {
        let date = today + Duration::days(off as i64);
        let date_str = date.format("%d.%m.%Y").to_string();
        let num = reduce_number_verbose(&date.format("%d%m%Y").to_string(), false);
        let meaning = lookup(num, &state.map).to_string();
        dates.push(DateItem {
            offset: off,
            date: date_str,
            value: num,
            meaning,
        });
    }
    Ok(Json(DateResponse { dates }))
}

#[tokio::main]
async fn main() {
    let map = load_bedeutungen(StdPath::new("bedeutungen.yaml"));
    let state = AppState { map: Arc::new(map) };

    let app = Router::new()
        .route("/reduce", get(reduce_handler))
        .route("/lookup/:number", get(lookup_handler))
        .route("/date", get(date_handler))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Listening on http://{addr}");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
