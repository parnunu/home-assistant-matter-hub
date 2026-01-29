use std::env;
use std::net::SocketAddr;

use hamh_api::build_router;
use hamh_storage::FileStorage;
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

    let storage = FileStorage::new(storage_root);
    let app = build_router(storage);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("Starting API server on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("bind failed");
    axum::serve(listener, app).await.expect("serve failed");
}
