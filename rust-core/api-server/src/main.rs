use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use tower_http::cors::CorsLayer;
use tracing::info;
use tracing_subscriber;

// ===== EXISTING IMPORTS =====
use correlator::{Correlator, Incident};
use detector::{Detector, DetectionThresholds};
use graph::{EventGraph, GraphSnapshot};

// ===== EVENT STRUCT (NEW) =====
#[derive(Deserialize, Debug)]
struct Event {
    event_type: String,
    entity: HashMap<String, String>,
    metadata: HashMap<String, String>,
}

// ===== APP STATE =====
#[derive(Clone)]
struct AppState {
    graph: Arc<RwLock<EventGraph>>,
    baseline_store: Arc<baseline::BaselineStore>,
    detector: Arc<Detector>,
    correlator: Arc<Correlator>,
    incidents: Arc<RwLock<Vec<Incident>>>,
}

// ===== QUERY STRUCT =====
#[derive(Deserialize)]
struct IncidentQuery {
    page: Option<usize>,
    limit: Option<usize>,
    severity: Option<String>,
}

// ===== HEALTH =====
#[derive(Serialize)]
struct HealthResponse {
    status: String,
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
    })
}

// ===== 🚨 NEW: INGEST EVENT =====
async fn ingest_event(
    State(state): State<AppState>,
    Json(event): Json<Event>,
) -> StatusCode {
    info!("📥 Received event: {:?}", event);

    // Simple detection logic (brute force simulation)
    if event.event_type == "login_failed" {
        let mut incidents = state.incidents.write().unwrap();

    let incident = Incident {
    id: uuid::Uuid::new_v4().to_string(),
    timestamp: chrono::Utc::now(),
    severity: "HIGH".to_string(),

    chain: vec!["login_failed".to_string()],
    entities: vec![
        event.entity.get("value").cloned().unwrap_or_default()
    ],

    sigma_score: 0.9,
    z_score: 2.5,

    ioc_match: None,

    actions: Some(vec!["alert".to_string()]),
    base_signal: 1.0,
    cvss: 7.5,

    summary: Some("Multiple failed login attempts detected".to_string()),
    };

        incidents.push(incident);

        info!("🚨 Incident created!");
    }

    StatusCode::OK
}

// ===== INCIDENT APIs =====
async fn get_incidents(
    State(state): State<AppState>,
    Query(query): Query<IncidentQuery>,
) -> Json<Vec<Incident>> {
    let limit = query.limit.unwrap_or(50);
    let incidents = state.incidents.read().unwrap();

    let mut filtered: Vec<&Incident> = incidents
        .iter()
        .filter(|i| {
            query.severity
                .as_ref()
                .map_or(true, |s| i.severity.to_uppercase() == s.to_uppercase())
        })
        .collect();

    filtered.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    let result: Vec<Incident> = filtered.into_iter().take(limit).cloned().collect();

    Json(result)
}

async fn get_incident(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Incident>, StatusCode> {
    let incidents = state.incidents.read().unwrap();

    incidents
        .iter()
        .find(|i| i.id == id)
        .cloned()
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

// ===== GRAPH =====
async fn get_graph_snapshot(State(state): State<AppState>) -> Json<GraphSnapshot> {
    let graph = state.graph.read().unwrap();
    Json(graph.get_snapshot())
}

// ===== FEEDBACK =====
#[derive(Deserialize, Debug)]
struct FeedbackRequest {
    incident_id: String,
    action: String,
    entity: Option<String>,
    tactic: Option<String>,
    new_threshold: Option<f32>,
}

async fn post_feedback(
    Json(_feedback): Json<FeedbackRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    Ok(Json(serde_json::json!({
        "status": "ok"
    })))
}

// ===== STATE INIT =====
fn create_app_state() -> AppState {
    let baseline_store = Arc::new(baseline::BaselineStore::new("baselines.db"));
    let graph = Arc::new(RwLock::new(EventGraph::new()));

    let thresholds = DetectionThresholds::default();
    let detector = Arc::new(Detector::new(baseline_store.clone()).with_thresholds(thresholds));

    let correlator = Arc::new(Correlator::new(graph.clone()));
    let incidents = Arc::new(RwLock::new(Vec::new()));

    AppState {
        graph,
        baseline_store,
        detector,
        correlator,
        incidents,
    }
}

// ===== MAIN =====
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    info!("🚀 Starting SOC API Server on 0.0.0.0:8080");

    let state = create_app_state();

    let cors = CorsLayer::permissive();

    let app = Router::new()
        .route("/health", get(health))
        .route("/events", post(ingest_event)) // ✅ FIXED
        .route("/incidents", get(get_incidents))
        .route("/incidents/:id", get(get_incident))
        .route("/graph/snapshot", get(get_graph_snapshot))
        .route("/feedback", post(post_feedback))
        .layer(cors)
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    axum::serve(listener, app).await.unwrap();
}