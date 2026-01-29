use async_trait::async_trait;
use hamh_core::filter::EntityDescriptor;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use reqwest::StatusCode;
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

    pub async fn get_entity_registry(&self) -> Result<Vec<HaEntityRegistryEntry>, HaError> {
        let client = reqwest::Client::new();
        let url = format!("{}/api/config/entity_registry", self.url.trim_end_matches('/'));
        let res = client.get(url).headers(self.auth_headers()?).send().await?;
        if res.status() == StatusCode::NOT_FOUND {
            return Ok(Vec::new());
        }
        Ok(res.json::<Vec<HaEntityRegistryEntry>>().await?)
    }

    pub async fn get_area_registry(&self) -> Result<Vec<HaAreaRegistryEntry>, HaError> {
        let client = reqwest::Client::new();
        let url = format!("{}/api/config/area_registry", self.url.trim_end_matches('/'));
        let res = client.get(url).headers(self.auth_headers()?).send().await?;
        if res.status() == StatusCode::NOT_FOUND {
            return Ok(Vec::new());
        }
        Ok(res.json::<Vec<HaAreaRegistryEntry>>().await?)
    }

    pub async fn get_label_registry(&self) -> Result<Vec<HaLabelRegistryEntry>, HaError> {
        let client = reqwest::Client::new();
        let url = format!("{}/api/config/label_registry", self.url.trim_end_matches('/'));
        let res = client.get(url).headers(self.auth_headers()?).send().await?;
        if res.status() == StatusCode::NOT_FOUND {
            return Ok(Vec::new());
        }
        Ok(res.json::<Vec<HaLabelRegistryEntry>>().await?)
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
    async fn list_entity_descriptors(&self) -> Result<Vec<EntityDescriptor>, HaError>;
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

    async fn list_entity_descriptors(&self) -> Result<Vec<EntityDescriptor>, HaError> {
        let states = self.client.get_states().await?;
        let registry = self.client.get_entity_registry().await?;
        let areas = self.client.get_area_registry().await?;
        let labels = self.client.get_label_registry().await?;

        let mut registry_map = std::collections::HashMap::new();
        for entry in registry {
            registry_map.insert(entry.entity_id.clone(), entry);
        }

        let mut area_map = std::collections::HashMap::new();
        for area in areas {
            area_map.insert(area.area_id.clone(), slugify(&area.name));
        }

        let mut label_map = std::collections::HashMap::new();
        for label in labels {
            label_map.insert(label.label_id.clone(), slugify(&label.name));
        }

        let mut descriptors = Vec::new();
        for state in states {
            let domain = state
                .entity_id
                .split('.')
                .next()
                .unwrap_or("")
                .to_string();
            let reg = registry_map.get(&state.entity_id);
            let platform = reg.and_then(|r| r.platform.clone());
            let entity_category = reg.and_then(|r| r.entity_category.clone());
            let device_id = reg.and_then(|r| r.device_id.clone());
            let area = reg
                .and_then(|r| r.area_id.clone())
                .and_then(|id| area_map.get(&id).cloned());
            let labels = reg
                .and_then(|r| r.labels.clone())
                .unwrap_or_default()
                .into_iter()
                .filter_map(|id| label_map.get(&id).cloned())
                .collect::<Vec<_>>();

            descriptors.push(EntityDescriptor {
                entity_id: state.entity_id,
                domain,
                platform,
                entity_category,
                area,
                labels,
                device_id,
                attributes: state.attributes,
            });
        }

        Ok(descriptors)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HaEntityState {
    pub entity_id: String,
    pub state: String,
    pub attributes: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HaEntityRegistryEntry {
    #[serde(default)]
    pub entity_id: String,
    #[serde(default)]
    pub platform: Option<String>,
    #[serde(default)]
    pub entity_category: Option<String>,
    #[serde(default)]
    pub area_id: Option<String>,
    #[serde(default)]
    pub labels: Option<Vec<String>>,
    #[serde(default)]
    pub device_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HaAreaRegistryEntry {
    #[serde(default)]
    pub area_id: String,
    #[serde(default)]
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HaLabelRegistryEntry {
    #[serde(default)]
    pub label_id: String,
    #[serde(default)]
    pub name: String,
}

fn slugify(input: &str) -> String {
    let mut raw = String::new();
    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() {
            raw.push(ch.to_ascii_lowercase());
        } else {
            raw.push('_');
        }
    }
    let mut out = String::new();
    let mut prev_underscore = false;
    for ch in raw.chars() {
        if ch == '_' {
            if !prev_underscore {
                out.push('_');
                prev_underscore = true;
            }
        } else {
            out.push(ch);
            prev_underscore = false;
        }
    }
    out.trim_matches('_').to_string()
}
