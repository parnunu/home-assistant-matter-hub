use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use hamh-core::models::{BridgeConfig, BridgeOperation};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serde error: {0}")]
    Serde(#[from] serde_json::Error),
}

#[derive(Debug, Clone)]
pub struct FileStorage {
    root: PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
struct StorageState {
    bridges: Vec<BridgeConfig>,
    operations: Vec<BridgeOperation>,
}

impl FileStorage {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn load(&self) -> Result<StorageState, StorageError> {
        let path = self.root.join("storage.json");
        if !path.exists() {
            return Ok(StorageState {
                bridges: Vec::new(),
                operations: Vec::new(),
            });
        }
        let data = fs::read_to_string(path)?;
        Ok(serde_json::from_str(&data)?)
    }

    pub fn save(&self, state: &StorageState) -> Result<(), StorageError> {
        fs::create_dir_all(&self.root)?;
        let tmp_path = self.root.join("storage.json.tmp");
        let final_path = self.root.join("storage.json");
        let mut file = fs::File::create(&tmp_path)?;
        let payload = serde_json::to_vec_pretty(state)?;
        file.write_all(&payload)?;
        file.sync_all()?;
        fs::rename(tmp_path, final_path)?;
        Ok(())
    }

    pub fn list_bridges(&self) -> Result<Vec<BridgeConfig>, StorageError> {
        Ok(self.load()?.bridges)
    }

    pub fn list_operations(&self) -> Result<Vec<BridgeOperation>, StorageError> {
        Ok(self.load()?.operations)
    }

    pub fn upsert_bridge(&self, bridge: BridgeConfig) -> Result<(), StorageError> {
        let mut state = self.load()?;
        state.bridges.retain(|b| b.id != bridge.id);
        state.bridges.push(bridge);
        self.save(&state)
    }

    pub fn add_operation(&self, op: BridgeOperation) -> Result<(), StorageError> {
        let mut state = self.load()?;
        state.operations.insert(0, op);
        self.save(&state)
    }
}
