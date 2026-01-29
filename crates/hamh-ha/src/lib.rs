use thiserror::Error;

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
