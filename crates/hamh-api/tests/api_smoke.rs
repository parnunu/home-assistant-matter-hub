use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use hamh_api::build_router;
use hamh_core::models::{BridgeConfig, BridgeFilter, FeatureFlags, OperationType};
use hamh_storage::FileStorage;
use tempfile::TempDir;
use tower::util::ServiceExt;
use uuid::Uuid;

fn new_app(temp_dir: &TempDir) -> axum::Router {
    let storage = FileStorage::new(temp_dir.path());
    build_router(storage, None)
}

fn bridge_payload(name: &str) -> BridgeConfig {
    BridgeConfig {
        id: Uuid::nil(),
        name: name.to_string(),
        port: 5540,
        filter: BridgeFilter::default(),
        feature_flags: FeatureFlags {
            cover_do_not_invert_percentage: false,
        },
        created_at: time::OffsetDateTime::now_utc(),
        updated_at: time::OffsetDateTime::now_utc(),
    }
}

#[tokio::test]
async fn create_bridge_and_list() {
    let temp_dir = TempDir::new().expect("temp dir");
    let app = new_app(&temp_dir);

    let payload = bridge_payload("Test Bridge");
    let req = Request::builder()
        .method("POST")
        .uri("/api/matter/bridges")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&payload).unwrap()))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let created: BridgeConfig = serde_json::from_slice(&body).unwrap();
    assert_ne!(created.id, Uuid::nil());
    assert_eq!(created.name, "Test Bridge");

    let list_req = Request::builder()
        .uri("/api/matter/bridges")
        .body(Body::empty())
        .unwrap();
    let list_res = app.oneshot(list_req).await.unwrap();
    assert_eq!(list_res.status(), StatusCode::OK);
    let list_body = to_bytes(list_res.into_body(), usize::MAX).await.unwrap();
    let list: Vec<BridgeConfig> = serde_json::from_slice(&list_body).unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].id, created.id);
}

#[tokio::test]
async fn delete_bridge_enqueues_operation() {
    let temp_dir = TempDir::new().expect("temp dir");
    let app = new_app(&temp_dir);

    let payload = bridge_payload("Delete Me");
    let create_req = Request::builder()
        .method("POST")
        .uri("/api/matter/bridges")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&payload).unwrap()))
        .unwrap();
    let create_res = app.clone().oneshot(create_req).await.unwrap();
    let body = to_bytes(create_res.into_body(), usize::MAX).await.unwrap();
    let created: BridgeConfig = serde_json::from_slice(&body).unwrap();

    let delete_req = Request::builder()
        .method("DELETE")
        .uri(format!("/api/matter/bridges/{}", created.id))
        .body(Body::empty())
        .unwrap();
    let delete_res = app.clone().oneshot(delete_req).await.unwrap();
    assert_eq!(delete_res.status(), StatusCode::NO_CONTENT);

    let ops_req = Request::builder()
        .uri("/api/matter/operations")
        .body(Body::empty())
        .unwrap();
    let ops_res = app.oneshot(ops_req).await.unwrap();
    assert_eq!(ops_res.status(), StatusCode::OK);
    let ops_body = to_bytes(ops_res.into_body(), usize::MAX).await.unwrap();
    let ops: Vec<hamh_core::models::BridgeOperation> =
        serde_json::from_slice(&ops_body).unwrap();

    assert!(!ops.is_empty());
    assert_eq!(ops[0].bridge_id, created.id);
    assert!(matches!(ops[0].op_type, OperationType::Delete));
}
