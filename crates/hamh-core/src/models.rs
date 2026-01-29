use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeConfig {
    pub id: Uuid,
    pub name: String,
    pub port: u16,
    pub filter: BridgeFilter,
    pub feature_flags: FeatureFlags,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BridgeFilter {
    pub include: Vec<EntityFilter>,
    pub exclude: Vec<EntityFilter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityFilter {
    #[serde(rename = "type")]
    pub kind: FilterKind,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilterKind {
    Pattern,
    Domain,
    Platform,
    EntityId,
    EntityCategory,
    Area,
    Label,
    DeviceId,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FeatureFlags {
    pub cover_do_not_invert_percentage: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeRuntimeState {
    pub status: BridgeStatus,
    pub last_error: Option<String>,
    pub operation_id: Option<Uuid>,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BridgeStatus {
    Stopped,
    Starting,
    Running,
    Stopping,
    Deleting,
    Error,
    Queued,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeOperation {
    pub operation_id: Uuid,
    pub bridge_id: Uuid,
    pub op_type: OperationType,
    pub status: OperationStatus,
    pub queued_at: OffsetDateTime,
    pub started_at: Option<OffsetDateTime>,
    pub finished_at: Option<OffsetDateTime>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationType {
    Create,
    Start,
    Stop,
    Refresh,
    Delete,
    FactoryReset,
    Update,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum OperationStatus {
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeDevice {
    pub entity_id: String,
    pub device_type: String,
    pub endpoint_id: u16,
    pub capabilities: Vec<String>,
    pub reachable: bool,
}
