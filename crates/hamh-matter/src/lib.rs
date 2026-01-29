mod adapter;

pub use adapter::{MatterAdapter, MatterBridgeHandle, RsMatterAdapter};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MatterError {
    #[error("bridge port {0} already in use")]
    PortInUse(u16),
    #[error("io error: {0}")]
    Io(String),
    #[error("not implemented")]
    NotImplemented,
}
