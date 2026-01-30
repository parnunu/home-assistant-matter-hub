mod adapter;

pub use adapter::{pairing_info, MatterAdapter, MatterBridgeHandle, RsMatterAdapter};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MatterError {
    #[error("bridge port {0} already in use")]
    PortInUse(u16),
    #[error("io error: {0}")]
    Io(String),
    #[error("runtime error: {0}")]
    Runtime(String),
    #[error("not implemented")]
    NotImplemented,
}

#[derive(Debug, Clone)]
pub struct EntityState {
    pub entity_id: String,
    pub on: bool,
}
