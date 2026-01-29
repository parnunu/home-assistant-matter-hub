use std::collections::HashMap;
use std::env;
use std::net::SocketAddr;

use hamh_api::build_router;
use hamh_core::device_map::map_descriptor_to_device_type;
use hamh_core::filter::matches_filter;
use hamh_core::models::{
    BridgeConfig, BridgeDevice, BridgeOperation, OperationStatus, OperationType,
};
use hamh_ha::{HassAdapter, HomeAssistantAdapter, HomeAssistantClient};
use hamh_matter::{MatterAdapter, RsMatterAdapter};
use hamh_storage::FileStorage;
use time::OffsetDateTime;
use tracing_subscriber::EnvFilter;
use tower_http::services::{ServeDir, ServeFile};
use std::path::PathBuf;

fn env_or(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}

async fn build_bridge_devices(
    bridge: &BridgeConfig,
    ha: &impl HomeAssistantAdapter,
) -> Result<Vec<BridgeDevice>, String> {
    let descriptors = ha
        .list_entity_descriptors()
        .await
        .map_err(|err| err.to_string())?;
    let mut devices = Vec::new();

    for descriptor in descriptors {
        if !matches_filter(&bridge.filter, &descriptor) {
            continue;
        }
        if let Some(device_type) = map_descriptor_to_device_type(&descriptor) {
            devices.push(BridgeDevice {
                entity_id: descriptor.entity_id,
                device_type,
                endpoint_id: 0,
                capabilities: Vec::new(),
                reachable: true,
            });
        }
    }

    if devices
        .iter()
        .any(|device| device.device_type == "RoboticVacuumCleaner")
    {
        devices.retain(|device| device.device_type == "RoboticVacuumCleaner");
    }

    for (idx, device) in devices.iter_mut().enumerate() {
        device.endpoint_id = (idx + 1) as u16;
    }

    Ok(devices)
}

fn resolve_bridge(storage: &FileStorage, id: uuid::Uuid) -> Result<BridgeConfig, String> {
    match storage.get_bridge(id).map_err(|err| err.to_string())? {
        Some(bridge) => Ok(bridge),
        None => Err("bridge not found".to_string()),
    }
}

async fn process_operation(
    op: &BridgeOperation,
    storage: &FileStorage,
    ha: &impl HomeAssistantAdapter,
    matter: &impl MatterAdapter,
    handles: &mut HashMap<uuid::Uuid, hamh_matter::MatterBridgeHandle>,
) -> Result<(), String> {
    match op.op_type {
        OperationType::Create | OperationType::Update => {
            let bridge = resolve_bridge(storage, op.bridge_id)?;
            let devices = build_bridge_devices(&bridge, ha).await?;
            storage
                .set_bridge_devices(bridge.id, devices)
                .map_err(|err| err.to_string())?;
            Ok(())
        }
        OperationType::Start => {
            let bridge = resolve_bridge(storage, op.bridge_id)?;
            let devices = build_bridge_devices(&bridge, ha).await?;
            storage
                .set_bridge_devices(bridge.id, devices)
                .map_err(|err| err.to_string())?;
            let handle = matter
                .start_bridge(&bridge)
                .await
                .map_err(|err| err.to_string())?;
            handles.insert(op.bridge_id, handle);
            Ok(())
        }
        OperationType::Stop => {
            if let Some(handle) = handles.remove(&op.bridge_id) {
                matter
                    .stop_bridge(&handle)
                    .await
                    .map_err(|err| err.to_string())
            } else {
                Err("bridge not running".to_string())
            }
        }
        OperationType::Refresh => {
            let bridge = resolve_bridge(storage, op.bridge_id)?;
            let devices = build_bridge_devices(&bridge, ha).await?;
            storage
                .set_bridge_devices(bridge.id, devices)
                .map_err(|err| err.to_string())?;
            if let Some(handle) = handles.get(&op.bridge_id) {
                matter
                    .refresh_bridge(handle)
                    .await
                    .map_err(|err| err.to_string())?;
            }
            Ok(())
        }
        OperationType::FactoryReset => {
            if let Some(handle) = handles.get(&op.bridge_id) {
                matter
                    .factory_reset(handle)
                    .await
                    .map_err(|err| err.to_string())?;
                Ok(())
            } else {
                Err("bridge not running".to_string())
            }
        }
        OperationType::Delete => {
            if let Some(handle) = handles.remove(&op.bridge_id) {
                let _ = matter.stop_bridge(&handle).await;
            }
            storage
                .delete_bridge(op.bridge_id)
                .map_err(|err| err.to_string())?;
            Ok(())
        }
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let port: u16 = env_or("HAMH_API_PORT", "8482")
        .parse()
        .unwrap_or(8482);
    let storage_root = env_or("HAMH_STORAGE_LOCATION", ".hamh-storage");
    let ha_url = env_or("HAMH_HOME_ASSISTANT_URL", "http://localhost:8123/");
    let ha_token = env_or("HAMH_HOME_ASSISTANT_ACCESS_TOKEN", "");

    let storage = FileStorage::new(storage_root.clone());
    let api = build_router(storage.clone());

    let assets_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");
    let index_file = assets_root.join("index.html");
    let static_service = ServeDir::new(assets_root)
        .not_found_service(ServeFile::new(index_file));
    let app = api.fallback_service(static_service);

    // Initialize adapters (stubbed for now).
    let ha_client = HomeAssistantClient::new(ha_url, ha_token);
    let ha_adapter = HassAdapter { client: ha_client };
    let matter_adapter = RsMatterAdapter::default();

    // Operation runner (simple loop for now).
    let runner_storage = storage.clone();
    let runner_ha = ha_adapter.clone();
    let runner_matter = matter_adapter.clone();
    let _ = tokio::spawn(async move {
        let mut handles: HashMap<uuid::Uuid, hamh_matter::MatterBridgeHandle> = HashMap::new();
        let mut ha_logged = false;
        loop {
            if let Err(err) = runner_ha.connect().await {
                tracing::debug!("HA connect failed: {err}");
            } else if !ha_logged {
                match runner_ha.list_entity_descriptors().await {
                    Ok(entities) => {
                        tracing::info!(
                            "Home Assistant reachable. Entity descriptors: {}",
                            entities.len()
                        );
                        ha_logged = true;
                    }
                    Err(err) => tracing::debug!("HA list_entities failed: {err}"),
                }
            }

            let next_op = match runner_storage.next_queued_operation() {
                Ok(op) => op,
                Err(err) => {
                    tracing::warn!("Failed to load queued operation: {err}");
                    None
                }
            };

            if let Some(mut op) = next_op {
                op.status = OperationStatus::Running;
                op.started_at = Some(OffsetDateTime::now_utc());
                if let Err(err) = runner_storage.update_operation(op.clone()) {
                    tracing::warn!("Failed to mark op running: {err}");
                }

                let result = process_operation(
                    &op,
                    &runner_storage,
                    &runner_ha,
                    &runner_matter,
                    &mut handles,
                )
                .await;

                op.finished_at = Some(OffsetDateTime::now_utc());
                match result {
                    Ok(_) => op.status = OperationStatus::Completed,
                    Err(err) => {
                        op.status = OperationStatus::Failed;
                        op.error = Some(err);
                    }
                }

                if let Err(err) = runner_storage.update_operation(op) {
                    tracing::warn!("Failed to update op status: {err}");
                }
            }

            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }
    });

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("Starting API server on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("bind failed");
    axum::serve(listener, app).await.expect("serve failed");
}
