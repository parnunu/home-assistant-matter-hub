use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, post, put},
    Json, Router,
};
use hamh-core::models::{BridgeConfig, BridgeOperation, OperationType};
use hamh_ops::OperationQueue;
use hamh_storage::{FileStorage, StorageError};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    storage: FileStorage,
    ops: OperationQueue,
}

pub fn build_router(storage: FileStorage) -> Router {
    let ops = OperationQueue::new(storage.clone());
    let state = Arc::new(AppState { storage, ops });

    Router::new()
        .route("/api/matter/bridges", get(list_bridges).post(create_bridge))
        .route(
            "/api/matter/bridges/:id",
            get(get_bridge).put(update_bridge).delete(delete_bridge),
        )
        .route("/api/matter/operations", get(list_operations))
        .route("/api/matter/health", get(health))
        .with_state(state)
}

async fn health() -> StatusCode {
    StatusCode::OK
}

async fn list_bridges(State(state): State<Arc<AppState>>) -> Result<Json<Vec<BridgeConfig>>, ApiError> {
    let bridges = state.storage.list_bridges()?;
    Ok(Json(bridges))
}

async fn list_operations(State(state): State<Arc<AppState>>) -> Result<Json<Vec<BridgeOperation>>, ApiError> {
    let ops = state.storage.list_operations()?;
    Ok(Json(ops))
}

async fn get_bridge(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<BridgeConfig>, ApiError> {
    let bridges = state.storage.list_bridges()?;
    bridges
        .into_iter()
        .find(|b| b.id == id)
        .map(Json)
        .ok_or(ApiError::NotFound)
}

async fn create_bridge(
    State(state): State<Arc<AppState>>,
    Json(mut payload): Json<BridgeConfig>,
) -> Result<Json<BridgeConfig>, ApiError> {
    let now = OffsetDateTime::now_utc();
    payload.id = Uuid::new_v4();
    payload.created_at = now;
    payload.updated_at = now;
    state.storage.upsert_bridge(payload.clone())?;
    let _ = state.ops.enqueue(payload.id, OperationType::Create)?;
    Ok(Json(payload))
}

async fn update_bridge(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    Json(mut payload): Json<BridgeConfig>,
) -> Result<Json<BridgeConfig>, ApiError> {
    if id != payload.id {
        return Err(ApiError::BadRequest("id mismatch".into()));
    }
    payload.updated_at = OffsetDateTime::now_utc();
    state.storage.upsert_bridge(payload.clone())?;
    let _ = state.ops.enqueue(payload.id, OperationType::Update)?;
    Ok(Json(payload))
}

async fn delete_bridge(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
) -> Result<StatusCode, ApiError> {
    let _ = state.ops.enqueue(id, OperationType::Delete)?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug)]
pub enum ApiError {
    Storage(StorageError),
    NotFound,
    BadRequest(String),
}

impl From<StorageError> for ApiError {
    fn from(value: StorageError) -> Self {
        ApiError::Storage(value)
    }
}

impl axum::response::IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        match self {
            ApiError::NotFound => (StatusCode::NOT_FOUND, "Not Found").into_response(),
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg).into_response(),
            ApiError::Storage(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("storage error: {err}"),
            )
                .into_response(),
        }
    }
}
