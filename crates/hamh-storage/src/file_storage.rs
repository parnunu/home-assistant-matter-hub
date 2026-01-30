use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

use hamh_core::models::{
    BridgeConfig, BridgeDevice, BridgeOperation, BridgeRuntimeEntry, BridgeRuntimeState,
    OperationStatus,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

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
    #[serde(default)]
    bridges: Vec<BridgeConfig>,
    #[serde(default)]
    operations: Vec<BridgeOperation>,
    #[serde(default)]
    devices: BTreeMap<String, Vec<BridgeDevice>>,
    #[serde(default)]
    runtime: BTreeMap<String, BridgeRuntimeState>,
}

impl Default for StorageState {
    fn default() -> Self {
        Self {
            bridges: Vec::new(),
            operations: Vec::new(),
            devices: BTreeMap::new(),
            runtime: BTreeMap::new(),
        }
    }
}

impl FileStorage {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    fn load(&self) -> Result<StorageState, StorageError> {
        let path = self.root.join("storage.json");
        if !path.exists() {
            return Ok(StorageState::default());
        }
        let data = fs::read_to_string(path)?;
        Ok(serde_json::from_str(&data)?)
    }

    fn save(&self, state: &StorageState) -> Result<(), StorageError> {
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

    pub fn get_bridge(&self, id: Uuid) -> Result<Option<BridgeConfig>, StorageError> {
        Ok(self.load()?.bridges.into_iter().find(|b| b.id == id))
    }

    pub fn list_operations(&self) -> Result<Vec<BridgeOperation>, StorageError> {
        Ok(self.load()?.operations)
    }

    pub fn next_queued_operation(&self) -> Result<Option<BridgeOperation>, StorageError> {
        let state = self.load()?;
        Ok(state
            .operations
            .into_iter()
            .find(|op| op.status == OperationStatus::Queued))
    }

    pub fn upsert_bridge(&self, bridge: BridgeConfig) -> Result<(), StorageError> {
        let mut state = self.load()?;
        state.bridges.retain(|b| b.id != bridge.id);
        state.bridges.push(bridge);
        self.save(&state)
    }

    pub fn delete_bridge(&self, id: Uuid) -> Result<(), StorageError> {
        let mut state = self.load()?;
        state.bridges.retain(|b| b.id != id);
        state.devices.remove(&id.to_string());
        state.runtime.remove(&id.to_string());
        self.save(&state)
    }

    pub fn list_bridge_devices(&self, id: Uuid) -> Result<Vec<BridgeDevice>, StorageError> {
        let state = self.load()?;
        Ok(state
            .devices
            .get(&id.to_string())
            .cloned()
            .unwrap_or_default())
    }

    pub fn set_bridge_devices(
        &self,
        id: Uuid,
        devices: Vec<BridgeDevice>,
    ) -> Result<(), StorageError> {
        let mut state = self.load()?;
        state.devices.insert(id.to_string(), devices);
        self.save(&state)
    }

    pub fn delete_bridge_devices(&self, id: Uuid) -> Result<(), StorageError> {
        let mut state = self.load()?;
        state.devices.remove(&id.to_string());
        self.save(&state)
    }

    pub fn list_bridge_runtime(&self) -> Result<Vec<BridgeRuntimeEntry>, StorageError> {
        let state = self.load()?;
        let mut out = Vec::new();
        for (key, value) in state.runtime {
            if let Ok(bridge_id) = Uuid::parse_str(&key) {
                out.push(BridgeRuntimeEntry {
                    bridge_id,
                    state: value,
                });
            }
        }
        Ok(out)
    }

    pub fn get_bridge_runtime(
        &self,
        id: Uuid,
    ) -> Result<Option<BridgeRuntimeState>, StorageError> {
        let state = self.load()?;
        Ok(state.runtime.get(&id.to_string()).cloned())
    }

    pub fn set_bridge_runtime(
        &self,
        id: Uuid,
        runtime: BridgeRuntimeState,
    ) -> Result<(), StorageError> {
        let mut state = self.load()?;
        state.runtime.insert(id.to_string(), runtime);
        self.save(&state)
    }

    pub fn delete_bridge_runtime(&self, id: Uuid) -> Result<(), StorageError> {
        let mut state = self.load()?;
        state.runtime.remove(&id.to_string());
        self.save(&state)
    }

    pub fn add_operation(&self, op: BridgeOperation) -> Result<(), StorageError> {
        let mut state = self.load()?;
        state.operations.insert(0, op);
        self.save(&state)
    }

    pub fn update_operation(&self, op: BridgeOperation) -> Result<(), StorageError> {
        let mut state = self.load()?;
        state.operations.retain(|existing| existing.operation_id != op.operation_id);
        state.operations.insert(0, op);
        self.save(&state)
    }
}
