mod adapter;

pub use adapter::{MatterAdapter, MatterBridgeHandle, RsMatterAdapter};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MatterError {
    #[error("not implemented")]
    NotImplemented,
}
