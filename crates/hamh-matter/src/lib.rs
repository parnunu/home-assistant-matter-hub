use thiserror::Error;

#[derive(Debug, Error)]
pub enum MatterError {
    #[error("not implemented")]
    NotImplemented,
}

#[derive(Debug, Clone)]
pub struct MatterBridge {
    pub port: u16,
}

impl MatterBridge {
    pub fn new(port: u16) -> Self {
        Self { port }
    }

    pub async fn start(&self) -> Result<(), MatterError> {
        Err(MatterError::NotImplemented)
    }
}
