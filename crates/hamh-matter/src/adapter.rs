use async_trait::async_trait;
use hamh_core::models::BridgeConfig;
use uuid::Uuid;

use crate::MatterError;

#[derive(Debug, Clone)]
pub struct MatterBridgeHandle {
    pub id: Uuid,
    pub port: u16,
}

#[async_trait]
pub trait MatterAdapter: Send + Sync {
    async fn start_bridge(&self, bridge: &BridgeConfig) -> Result<MatterBridgeHandle, MatterError>;
    async fn stop_bridge(&self, handle: &MatterBridgeHandle) -> Result<(), MatterError>;
    async fn refresh_bridge(&self, handle: &MatterBridgeHandle) -> Result<(), MatterError>;
    async fn factory_reset(&self, handle: &MatterBridgeHandle) -> Result<(), MatterError>;
}

#[derive(Debug, Default, Clone)]
pub struct RsMatterAdapter;

#[async_trait]
impl MatterAdapter for RsMatterAdapter {
    async fn start_bridge(&self, bridge: &BridgeConfig) -> Result<MatterBridgeHandle, MatterError> {
        let _ = bridge;
        Err(MatterError::NotImplemented)
    }

    async fn stop_bridge(&self, handle: &MatterBridgeHandle) -> Result<(), MatterError> {
        let _ = handle;
        Err(MatterError::NotImplemented)
    }

    async fn refresh_bridge(&self, handle: &MatterBridgeHandle) -> Result<(), MatterError> {
        let _ = handle;
        Err(MatterError::NotImplemented)
    }

    async fn factory_reset(&self, handle: &MatterBridgeHandle) -> Result<(), MatterError> {
        let _ = handle;
        Err(MatterError::NotImplemented)
    }
}
