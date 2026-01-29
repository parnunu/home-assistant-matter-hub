use std::env;
use std::net::SocketAddr;

use hamh_api::build_router;
use hamh_ha::{HassAdapter, HomeAssistantAdapter, HomeAssistantClient};
use hamh_matter::{MatterAdapter, RsMatterAdapter};
use hamh_storage::FileStorage;
use hamh_core::models::{OperationStatus, OperationType};
use time::OffsetDateTime;
use tracing_subscriber::EnvFilter;

fn env_or(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
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
    let app = build_router(storage.clone());

    // Initialize adapters (stubbed for now).
    let ha_client = HomeAssistantClient::new(ha_url, ha_token);
    let ha_adapter = HassAdapter { client: ha_client };
    let matter_adapter = RsMatterAdapter::default();

    // Operation runner (simple loop for now).
    let runner_storage = storage.clone();
    let runner_ha = ha_adapter.clone();
    let runner_matter = matter_adapter.clone();
    let _ = tokio::spawn(async move {
        loop {
            if let Err(err) = runner_ha.connect().await {
                tracing::debug!("HA connect not implemented: {err}");
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

                let result: Result<(), hamh_matter::MatterError> = match op.op_type {
                    OperationType::Start => match runner_storage.get_bridge(op.bridge_id) {
                        Ok(Some(bridge)) => runner_matter.start_bridge(&bridge).await.map(|_| ()),
                        Ok(None) => Err(hamh_matter::MatterError::NotImplemented),
                        Err(_) => Err(hamh_matter::MatterError::NotImplemented),
                    },
                    _ => Ok(()),
                };

                op.finished_at = Some(OffsetDateTime::now_utc());
                match result {
                    Ok(_) => op.status = OperationStatus::Completed,
                    Err(err) => {
                        op.status = OperationStatus::Failed;
                        op.error = Some(format!("{err}"));
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
