use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum HaError {
    #[error("not implemented")]
    NotImplemented,
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("invalid header")]
    InvalidHeader,
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
        let client = reqwest::Client::new();
        let url = format!("{}/api/", self.url.trim_end_matches('/'));
        let res = client.get(url).headers(self.auth_headers()?).send().await?;
        if res.status().is_success() {
            Ok(())
        } else {
            Err(HaError::NotImplemented)
        }
    }

    pub async fn get_states(&self) -> Result<Vec<HaEntityState>, HaError> {
        let client = reqwest::Client::new();
        let url = format!("{}/api/states", self.url.trim_end_matches('/'));
        let res = client.get(url).headers(self.auth_headers()?).send().await?;
        let states = res.json::<Vec<HaEntityState>>().await?;
        Ok(states)
    }

    fn auth_headers(&self) -> Result<HeaderMap, HaError> {
        let mut headers = HeaderMap::new();
        let value = HeaderValue::from_str(&format!("Bearer {}", self.token))
            .map_err(|_| HaError::InvalidHeader)?;
        headers.insert(AUTHORIZATION, value);
        Ok(headers)
    }
}

#[async_trait]
pub trait HomeAssistantAdapter: Send + Sync {
    async fn connect(&self) -> Result<(), HaError>;
    async fn subscribe_entities(&self, bridge_id: Uuid) -> Result<(), HaError>;
    async fn list_entities(&self) -> Result<Vec<HaEntityState>, HaError>;
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

    async fn list_entities(&self) -> Result<Vec<HaEntityState>, HaError> {
        self.client.get_states().await
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HaEntityState {
    pub entity_id: String,
    pub state: String,
    pub attributes: Value,
}
