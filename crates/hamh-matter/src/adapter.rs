use async_trait::async_trait;
use hamh_core::models::BridgeConfig;
use tokio::net::{TcpListener, UdpSocket};
use tokio::sync::{oneshot, Mutex};
use uuid::Uuid;

use crate::MatterError;

#[derive(Debug, Clone)]
pub struct MatterBridgeHandle {
    pub id: Uuid,
    pub port: u16,
    shutdown: std::sync::Arc<Mutex<Option<oneshot::Sender<()>>>>,
}

#[async_trait]
pub trait MatterAdapter: Send + Sync {
    async fn start_bridge(&self, bridge: &BridgeConfig) -> Result<MatterBridgeHandle, MatterError>;
    async fn stop_bridge(&self, handle: &MatterBridgeHandle) -> Result<(), MatterError>;
    async fn refresh_bridge(&self, handle: &MatterBridgeHandle) -> Result<(), MatterError>;
    async fn factory_reset(&self, handle: &MatterBridgeHandle) -> Result<(), MatterError>;
}

#[derive(Debug, Default, Clone)]
pub struct RsMatterAdapter;

#[async_trait]
impl MatterAdapter for RsMatterAdapter {
    async fn start_bridge(&self, bridge: &BridgeConfig) -> Result<MatterBridgeHandle, MatterError> {
        let port = bridge.port;
        let tcp = TcpListener::bind(("0.0.0.0", port))
            .await
            .map_err(|_| MatterError::PortInUse(port))?;
        let udp = UdpSocket::bind(("0.0.0.0", port))
            .await
            .map_err(|err| MatterError::Io(err.to_string()))?;
        let (tx, mut rx) = oneshot::channel::<()>();
        let shutdown = std::sync::Arc::new(Mutex::new(Some(tx)));

        tokio::spawn(async move {
            let mut buf = [0u8; 64];
            loop {
                tokio::select! {
                    _ = &mut rx => {
                        break;
                    }
                    res = tcp.accept() => {
                        if let Ok((socket, _)) = res {
                            let _ = socket.try_write(b"");
                        }
                    }
                    res = udp.recv_from(&mut buf) => {
                        if res.is_err() {
                            // ignore
                        }
                    }
                }
            }
        });

        Ok(MatterBridgeHandle {
            id: bridge.id,
            port,
            shutdown,
        })
    }

    async fn stop_bridge(&self, handle: &MatterBridgeHandle) -> Result<(), MatterError> {
        let mut guard = handle.shutdown.lock().await;
        if let Some(tx) = guard.take() {
            let _ = tx.send(());
        }
        Ok(())
    }

    async fn refresh_bridge(&self, handle: &MatterBridgeHandle) -> Result<(), MatterError> {
        let _ = handle;
        Ok(())
    }

    async fn factory_reset(&self, handle: &MatterBridgeHandle) -> Result<(), MatterError> {
        let _ = handle;
        Ok(())
    }
}
