mod db;

use anyhow::{Context, Result};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;

use db::{Aircraft, ApiDb, BoundingBox, Position};

#[derive(Parser, Debug)]
#[command(about = "HTTP API serving aircraft positions from adsb-decoder's database")]
struct Args {
    /// Path to the SQLite database written by adsb-decoder
    #[arg(long, default_value = "adsb.db")]
    db: String,

    /// Address to bind. Defaults to all interfaces so the Pi is reachable.
    #[arg(long, default_value = "0.0.0.0:8080")]
    bind: String,

    /// Default age window, in seconds, for /api/aircraft
    #[arg(long, default_value_t = 1800)]
    max_age_secs: i64,

    /// Positions outside this box are treated as bad CPR solves and dropped.
    /// Format: lat_min,lat_max,lon_min,lon_max (Ottawa: 44.0,46.5,-77.5,-74.0).
    /// Defaults to worldwide.
    #[arg(long)]
    bbox: Option<BoundingBox>,
}

struct AppState {
    db: Mutex<ApiDb>,
    default_max_age_secs: i64,
}

/// Any handler failure becomes a 500 with a JSON body.
struct ApiError(anyhow::Error);

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        eprintln!("request failed: {:#}", self.0);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorBody {
                error: self.0.to_string(),
            }),
        )
            .into_response()
    }
}

impl<E: Into<anyhow::Error>> From<E> for ApiError {
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

#[derive(Deserialize)]
struct AircraftQuery {
    /// Overrides the server's default window for this request.
    max_age_secs: Option<i64>,
}

#[derive(Serialize)]
struct AircraftResponse {
    count: usize,
    aircraft: Vec<Aircraft>,
}

#[derive(Deserialize)]
struct TrailQuery {
    limit: Option<usize>,
}

#[derive(Serialize)]
struct TrailResponse {
    icao: String,
    count: usize,
    positions: Vec<Position>,
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    aircraft_known: i64,
}

async fn health(State(state): State<Arc<AppState>>) -> Result<Json<HealthResponse>, ApiError> {
    let aircraft_known = state.db.lock().await.ping()?;
    Ok(Json(HealthResponse {
        status: "ok",
        aircraft_known,
    }))
}

async fn list_aircraft(
    State(state): State<Arc<AppState>>,
    Query(q): Query<AircraftQuery>,
) -> Result<Json<AircraftResponse>, ApiError> {
    let max_age = q.max_age_secs.unwrap_or(state.default_max_age_secs);
    let aircraft = state.db.lock().await.load_aircraft(max_age)?;

    Ok(Json(AircraftResponse {
        count: aircraft.len(),
        aircraft,
    }))
}

async fn aircraft_trail(
    State(state): State<Arc<AppState>>,
    Path(icao): Path<String>,
    Query(q): Query<TrailQuery>,
) -> Result<Json<TrailResponse>, ApiError> {
    // adsb-decoder stores ICAO addresses as uppercase hex.
    let icao = icao.to_uppercase();
    let limit = q.limit.unwrap_or(200).min(10_000);
    let positions = state.db.lock().await.load_trail(&icao, limit)?;

    Ok(Json(TrailResponse {
        icao,
        count: positions.len(),
        positions,
    }))
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let bbox = args.bbox.unwrap_or_default();
    let db = ApiDb::open(&args.db, bbox)
        .with_context(|| format!("run adsb-decoder first to create {}", args.db))?;

    let state = Arc::new(AppState {
        db: Mutex::new(db),
        default_max_age_secs: args.max_age_secs,
    });

    let app = Router::new()
        .route("/healthz", get(health))
        .route("/api/aircraft", get(list_aircraft))
        .route("/api/aircraft/:icao/trail", get(aircraft_trail))
        // The map front-end is served from somewhere else during the talk.
        .layer(CorsLayer::permissive())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&args.bind)
        .await
        .with_context(|| format!("binding {}", args.bind))?;

    println!("adsb-api listening on http://{}", listener.local_addr()?);
    println!("  GET /healthz");
    println!("  GET /api/aircraft?max_age_secs=1800");
    println!("  GET /api/aircraft/:icao/trail?limit=200");

    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await
        .context("serving")?;

    Ok(())
}
