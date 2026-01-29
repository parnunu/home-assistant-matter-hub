use hamh_core::models::{BridgeOperation, OperationStatus, OperationType};
use hamh-storage::{FileStorage, StorageError};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug)]
pub struct OperationQueue {
    storage: FileStorage,
}

impl OperationQueue {
    pub fn new(storage: FileStorage) -> Self {
        Self { storage }
    }

    pub fn enqueue(&self, bridge_id: Uuid, op_type: OperationType) -> Result<BridgeOperation, StorageError> {
        let op = BridgeOperation {
            operation_id: Uuid::new_v4(),
            bridge_id,
            op_type,
            status: OperationStatus::Queued,
            queued_at: OffsetDateTime::now_utc(),
            started_at: None,
            finished_at: None,
            error: None,
        };
        self.storage.add_operation(op.clone())?;
        Ok(op)
    }
}
