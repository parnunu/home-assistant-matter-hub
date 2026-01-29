use async_trait::async_trait;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum HaError {
    #[error("not implemented")]
    NotImplemented,
}

#[derive(Debug, Clone)]
pub struct HomeAssistantClient {
    pub url: String,
    pub token: String,
}

impl HomeAssistantClient {
    pub fn new(url: String, token: String) -> Self {
        Self { url, token }
    }

    pub async fn connect(&self) -> Result<(), HaError> {
        Err(HaError::NotImplemented)
    }
}

#[async_trait]
pub trait HomeAssistantAdapter: Send + Sync {
    async fn connect(&self) -> Result<(), HaError>;
    async fn subscribe_entities(&self, bridge_id: Uuid) -> Result<(), HaError>;
}

#[derive(Debug, Clone)]
pub struct HassAdapter {
    pub client: HomeAssistantClient,
}

#[async_trait]
impl HomeAssistantAdapter for HassAdapter {
    async fn connect(&self) -> Result<(), HaError> {
        self.client.connect().await
    }

    async fn subscribe_entities(&self, _bridge_id: Uuid) -> Result<(), HaError> {
        Err(HaError::NotImplemented)
    }
}
