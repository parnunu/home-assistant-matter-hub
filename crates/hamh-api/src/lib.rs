use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use hamh_core::models::{
    BridgeConfig, BridgeDevice, BridgeFilter, BridgeOperation, BridgeRuntimeEntry,
    BridgeRuntimeState, FeatureFlags, OperationType, PairingInfo,
};
use hamh_ha::HomeAssistantClient;
use hamh_matter::pairing_info;
use serde::Deserialize;
use hamh_ops::OperationQueue;
use hamh_storage::{FileStorage, StorageError};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    storage: FileStorage,
    ops: OperationQueue,
    ha: Option<HomeAssistantClient>,
}

pub fn build_router(storage: FileStorage, ha: Option<HomeAssistantClient>) -> Router {
    let ops = OperationQueue::new(storage.clone());
    let state = Arc::new(AppState { storage, ops, ha });

    Router::new()
        .route("/api/matter/bridges", get(list_bridges).post(create_bridge))
        .route(
            "/api/matter/bridges/:id",
            get(get_bridge).put(update_bridge).delete(delete_bridge),
        )
        .route(
            "/api/matter/bridges/:id/actions/start",
            post(start_bridge),
        )
        .route(
            "/api/matter/bridges/:id/actions/stop",
            post(stop_bridge),
        )
        .route(
            "/api/matter/bridges/:id/actions/refresh",
            post(refresh_bridge),
        )
        .route(
            "/api/matter/bridges/:id/actions/factory-reset",
            post(factory_reset_bridge),
        )
        .route("/api/matter/bridges/:id/devices", get(list_devices))
        .route(
            "/api/matter/bridges/:id/devices/:entity_id/actions/on",
            post(device_on),
        )
        .route(
            "/api/matter/bridges/:id/devices/:entity_id/actions/off",
            post(device_off),
        )
        .route(
            "/api/matter/bridges/:id/devices/:entity_id/actions/color",
            post(device_color),
        )
        .route("/api/matter/bridges/:id/runtime", get(get_runtime))
        .route("/api/matter/bridges/runtime", get(list_runtime))
        .route("/api/matter/bridges/:id/pairing", get(get_pairing))
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
    Json(payload): Json<CreateBridgePayload>,
) -> Result<Json<BridgeConfig>, ApiError> {
    let now = OffsetDateTime::now_utc();
    let bridge = BridgeConfig {
        id: Uuid::new_v4(),
        name: payload.name,
        port: payload.port,
        filter: payload.filter.unwrap_or_default(),
        feature_flags: payload.feature_flags.unwrap_or_default(),
        created_at: now,
        updated_at: now,
    };
    state.storage.upsert_bridge(bridge.clone())?;
    let _ = state.ops.enqueue(bridge.id, OperationType::Create)?;
    Ok(Json(bridge))
}

async fn update_bridge(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<UpdateBridgePayload>,
) -> Result<Json<BridgeConfig>, ApiError> {
    let mut bridge = state
        .storage
        .get_bridge(id)?
        .ok_or(ApiError::NotFound)?;
    bridge.name = payload.name;
    bridge.port = payload.port;
    bridge.filter = payload.filter.unwrap_or_default();
    bridge.feature_flags = payload.feature_flags.unwrap_or_default();
    bridge.updated_at = OffsetDateTime::now_utc();
    state.storage.upsert_bridge(bridge.clone())?;
    let _ = state.ops.enqueue(bridge.id, OperationType::Update)?;
    Ok(Json(bridge))
}

async fn delete_bridge(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
) -> Result<StatusCode, ApiError> {
    let _ = state.ops.enqueue(id, OperationType::Delete)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn stop_bridge(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
) -> Result<StatusCode, ApiError> {
    let _ = state.ops.enqueue(id, OperationType::Stop)?;
    Ok(StatusCode::ACCEPTED)
}

async fn refresh_bridge(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
) -> Result<StatusCode, ApiError> {
    let _ = state.ops.enqueue(id, OperationType::Refresh)?;
    Ok(StatusCode::ACCEPTED)
}

async fn factory_reset_bridge(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
) -> Result<StatusCode, ApiError> {
    let _ = state.ops.enqueue(id, OperationType::FactoryReset)?;
    Ok(StatusCode::ACCEPTED)
}

async fn start_bridge(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
) -> Result<StatusCode, ApiError> {
    let _ = state.ops.enqueue(id, OperationType::Start)?;
    Ok(StatusCode::ACCEPTED)
}

async fn list_devices(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<BridgeDevice>>, ApiError> {
    let devices = state.storage.list_bridge_devices(id)?;
    Ok(Json(devices))
}

async fn list_runtime(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<BridgeRuntimeEntry>>, ApiError> {
    let runtime = state.storage.list_bridge_runtime()?;
    Ok(Json(runtime))
}

async fn get_runtime(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<BridgeRuntimeState>, ApiError> {
    let runtime = state.storage.get_bridge_runtime(id)?;
    runtime.map(Json).ok_or(ApiError::NotFound)
}

#[derive(Debug, Deserialize)]
struct ColorPayload {
    rgb: [u8; 3],
}

async fn device_on(
    Path((id, entity_id)): Path<(Uuid, String)>,
    State(state): State<Arc<AppState>>,
) -> Result<StatusCode, ApiError> {
    call_device_service(&state, id, &entity_id, "turn_on", None).await?;
    Ok(StatusCode::ACCEPTED)
}

async fn device_off(
    Path((id, entity_id)): Path<(Uuid, String)>,
    State(state): State<Arc<AppState>>,
) -> Result<StatusCode, ApiError> {
    call_device_service(&state, id, &entity_id, "turn_off", None).await?;
    Ok(StatusCode::ACCEPTED)
}

async fn device_color(
    Path((id, entity_id)): Path<(Uuid, String)>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ColorPayload>,
) -> Result<StatusCode, ApiError> {
    call_device_service(
        &state,
        id,
        &entity_id,
        "turn_on",
        Some(serde_json::json!({ "rgb_color": payload.rgb })),
    )
    .await?;
    Ok(StatusCode::ACCEPTED)
}

async fn call_device_service(
    state: &AppState,
    bridge_id: Uuid,
    entity_id: &str,
    service: &str,
    extra: Option<serde_json::Value>,
) -> Result<(), ApiError> {
    let devices = state.storage.list_bridge_devices(bridge_id)?;
    if !devices.iter().any(|d| d.entity_id == entity_id) {
        return Err(ApiError::NotFound);
    }
    let ha = state
        .ha
        .as_ref()
        .ok_or_else(|| ApiError::Runtime("home assistant not configured".into()))?;
    let domain = entity_id
        .split('.')
        .next()
        .ok_or_else(|| ApiError::BadRequest("invalid entity_id".into()))?;
    let mut payload = serde_json::json!({ "entity_id": entity_id });
    if let Some(extra) = extra {
        if let Some(obj) = payload.as_object_mut() {
            if let Some(extra_obj) = extra.as_object() {
                for (k, v) in extra_obj {
                    obj.insert(k.clone(), v.clone());
                }
            }
        }
    }
    ha.call_service(domain, service, payload)
        .await
        .map_err(|err| ApiError::Runtime(err.to_string()))
}

async fn get_pairing(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<PairingInfo>, ApiError> {
    state
        .storage
        .get_bridge(id)?
        .ok_or(ApiError::NotFound)?;
    let info = pairing_info(id).map_err(|err| ApiError::Runtime(err.to_string()))?;
    Ok(Json(info))
}

#[derive(Debug)]
pub enum ApiError {
    Storage(StorageError),
    NotFound,
    BadRequest(String),
    Runtime(String),
}

#[derive(Debug, Deserialize)]
struct CreateBridgePayload {
    name: String,
    port: u16,
    #[serde(default)]
    filter: Option<BridgeFilter>,
    #[serde(default)]
    feature_flags: Option<FeatureFlags>,
}

#[derive(Debug, Deserialize)]
struct UpdateBridgePayload {
    name: String,
    port: u16,
    #[serde(default)]
    filter: Option<BridgeFilter>,
    #[serde(default)]
    feature_flags: Option<FeatureFlags>,
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
            ApiError::Runtime(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("runtime error: {err}"),
            )
                .into_response(),
        }
    }
}
