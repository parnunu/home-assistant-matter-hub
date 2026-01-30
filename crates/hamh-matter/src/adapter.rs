use std::cell::Cell;
use std::collections::HashMap;
use std::net::{SocketAddr, UdpSocket};
use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use hamh_core::models::{BridgeConfig, BridgeDevice, PairingInfo};
use rs_matter::dm::clusters::{desc, on_off};
use rs_matter::dm::clusters::decl::{bridged_device_basic_information, on_off as on_off_cluster};
use rs_matter::dm::clusters::net_comm::NetworkType;
use rs_matter::dm::clusters::on_off::{
    EffectVariantEnum, NoLevelControl, OnOffHandler, OnOffHooks, StartUpOnOffEnum,
};
use rs_matter::dm::devices::test::{TEST_DEV_ATT, TEST_DEV_COMM, TEST_DEV_DET};
use rs_matter::dm::devices::{DEV_TYPE_AGGREGATOR, DEV_TYPE_BRIDGED_NODE, DEV_TYPE_ON_OFF_LIGHT};
use rs_matter::dm::endpoints;
use rs_matter::dm::subscriptions::DefaultSubscriptions;
use rs_matter::dm::{
    Async as DmAsync, AsyncHandler, AsyncMetadata, Cluster, DataModel, Dataver, Endpoint,
    HandlerContext, InvokeContext, Node, ReadContext, WriteContext,
};
use rs_matter::error::{Error as MatterLibError, ErrorCode};
use rs_matter::pairing::qr::{no_optional_data, CommFlowType, Qr, QrPayload, QrTextRenderer, QrTextType};
use rs_matter::pairing::DiscoveryCapabilities;
use rs_matter::persist::{Psm, NO_NETWORKS};
use rs_matter::respond::DefaultResponder;
use rs_matter::sc::pake::MAX_COMM_WINDOW_TIMEOUT_SECS;
use rs_matter::utils::storage::pooled::PooledBuffers;
use rs_matter::{BasicCommData, Matter};
use tokio::sync::mpsc;
use uuid::Uuid;

use async_io::Async as AsyncIo;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use futures::stream::{FuturesUnordered, StreamExt};
use rs_matter::tlv::{Nullable, TLVBuilderParent, Utf8StrBuilder};
use rs_matter::with;
use rs_matter::dm::clusters::desc::ClusterHandler as _;

use crate::{EntityState, MatterError};

const AGGREGATOR_ENDPOINT_ID: u16 = 1;
const PERSIST_DIR_NAME: &str = "matter";

pub fn pairing_info(bridge_id: Uuid) -> Result<PairingInfo, MatterError> {
    let device_name = Box::leak(format!("HAMH {}", bridge_id).into_boxed_str());
    let serial_no: &'static str = Box::leak(format!("hamh-{}", bridge_id).into_boxed_str());
    let unique_id = serial_no;

    let dev_det = rs_matter::dm::clusters::basic_info::BasicInfoConfig {
        device_name,
        serial_no,
        unique_id,
        ..TEST_DEV_DET
    };

    let dev_comm = BasicCommData {
        password: env_u32("HAMH_MATTER_PASSCODE", TEST_DEV_COMM.password),
        discriminator: env_u16("HAMH_MATTER_DISCRIMINATOR", TEST_DEV_COMM.discriminator),
    };

    let payload: QrPayload<'_, rs_matter::pairing::qr::NoOptionalData> =
        QrPayload::new_from_basic_info(
        DiscoveryCapabilities::IP,
        CommFlowType::Standard,
        dev_comm,
        &dev_det,
        no_optional_data as _,
    );

    let mut text_buf = vec![0u8; 2048];
    let (qr_text, _) = payload
        .as_str(&mut text_buf)
        .map_err(|err| MatterError::Runtime(err.to_string()))?;

    let mut tmp_buf = vec![0u8; 2048];
    let mut out_buf = vec![0u8; 4096];
    let qr = Qr::compute(qr_text, &mut tmp_buf, &mut out_buf)
        .map_err(|err| MatterError::Runtime(err.to_string()))?;

    let renderer = QrTextRenderer::Unicode(qr);
    let mut render_buf = vec![0u8; 8192];
    let (qr_unicode, _) = renderer
        .render(2, false, &mut render_buf)
        .map_err(|err| MatterError::Runtime(err.to_string()))?;

    Ok(PairingInfo {
        qr_text: qr_text.to_string(),
        qr_unicode: qr_unicode.to_string(),
        manual_code: dev_comm.compute_pretty_pairing_code().to_string(),
        discriminator: dev_comm.discriminator,
    })
}

#[derive(Debug, Clone)]
pub struct MatterBridgeHandle {
    pub id: Uuid,
    pub port: u16,
    cmd_tx: mpsc::UnboundedSender<BridgeCommand>,
}

#[derive(Debug)]
enum BridgeCommand {
    UpdateDevices(Vec<BridgeDevice>),
    UpdateStates(Vec<EntityState>),
    FactoryReset,
    Shutdown,
}

#[async_trait]
pub trait MatterAdapter: Send + Sync {
    async fn start_bridge(
        &self,
        bridge: &BridgeConfig,
        devices: &[BridgeDevice],
    ) -> Result<MatterBridgeHandle, MatterError>;
    async fn stop_bridge(&self, handle: &MatterBridgeHandle) -> Result<(), MatterError>;
    async fn refresh_bridge(
        &self,
        handle: &MatterBridgeHandle,
        devices: &[BridgeDevice],
    ) -> Result<(), MatterError>;
    async fn factory_reset(&self, handle: &MatterBridgeHandle) -> Result<(), MatterError>;
    async fn apply_entity_states(
        &self,
        handle: &MatterBridgeHandle,
        updates: &[EntityState],
    ) -> Result<(), MatterError>;
}

#[derive(Debug, Default, Clone)]
pub struct RsMatterAdapter;

#[async_trait]
impl MatterAdapter for RsMatterAdapter {
    async fn start_bridge(
        &self,
        bridge: &BridgeConfig,
        devices: &[BridgeDevice],
    ) -> Result<MatterBridgeHandle, MatterError> {
        let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();
        let bridge_id = bridge.id;
        let port = bridge.port;
        let devices = devices.to_vec();

        let thread = std::thread::Builder::new()
            .name(format!("hamh-matter-{}", bridge_id))
            .stack_size(2 * 1024 * 1024)
            .spawn(move || {
                let runtime = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("matter runtime");
                runtime.block_on(async move {
                    if let Err(err) = run_matter_runtime(bridge_id, port, devices, cmd_rx).await {
                        tracing::error!("Matter runtime stopped: {err}");
                    }
                });
            })
            .map_err(|err| MatterError::Io(err.to_string()))?;

        drop(thread);

        Ok(MatterBridgeHandle {
            id: bridge_id,
            port,
            cmd_tx,
        })
    }

    async fn stop_bridge(&self, handle: &MatterBridgeHandle) -> Result<(), MatterError> {
        handle
            .cmd_tx
            .send(BridgeCommand::Shutdown)
            .map_err(|_| MatterError::Runtime("shutdown channel closed".into()))
    }

    async fn refresh_bridge(
        &self,
        handle: &MatterBridgeHandle,
        devices: &[BridgeDevice],
    ) -> Result<(), MatterError> {
        handle
            .cmd_tx
            .send(BridgeCommand::UpdateDevices(devices.to_vec()))
            .map_err(|_| MatterError::Runtime("refresh channel closed".into()))
    }

    async fn factory_reset(&self, handle: &MatterBridgeHandle) -> Result<(), MatterError> {
        handle
            .cmd_tx
            .send(BridgeCommand::FactoryReset)
            .map_err(|_| MatterError::Runtime("factory reset channel closed".into()))
    }

    async fn apply_entity_states(
        &self,
        handle: &MatterBridgeHandle,
        updates: &[EntityState],
    ) -> Result<(), MatterError> {
        handle
            .cmd_tx
            .send(BridgeCommand::UpdateStates(updates.to_vec()))
            .map_err(|_| MatterError::Runtime("update channel closed".into()))
    }
}

#[derive(Debug)]
struct BridgeMetadata {
    endpoints: Vec<Endpoint<'static>>,
}

struct BridgeMetadataGuard<'a> {
    meta: &'a BridgeMetadata,
}

impl rs_matter::dm::MetadataGuard for BridgeMetadataGuard<'_> {
    fn node(&self) -> Node<'_> {
        Node {
            id: 0,
            endpoints: &self.meta.endpoints,
        }
    }
}

impl AsyncMetadata for BridgeMetadata {
    type MetadataGuard<'a> = BridgeMetadataGuard<'a> where Self: 'a;

    async fn lock(&self) -> Self::MetadataGuard<'_> {
        BridgeMetadataGuard { meta: self }
    }
}

struct BridgeHandler {
    aggregator_desc: desc::HandlerAdaptor<desc::DescHandler<'static>>,
    descriptors: HashMap<u16, desc::HandlerAdaptor<desc::DescHandler<'static>>>,
    bridged_info: HashMap<u16, bridged_device_basic_information::HandlerAdaptor<BridgedHandler>>,
    on_off: HashMap<u16, OnOffHandler<'static, BridgeOnOffHooks, NoLevelControl>>,
}

#[derive(Clone)]
struct BridgeHandlerRef {
    inner: Arc<BridgeHandler>,
}

impl BridgeHandlerRef {
    fn new(inner: Arc<BridgeHandler>) -> Self {
        Self { inner }
    }
}

impl AsyncHandler for BridgeHandlerRef {
    fn read_awaits(&self, ctx: impl ReadContext) -> bool {
        self.inner.read_awaits(ctx)
    }

    fn write_awaits(&self, ctx: impl WriteContext) -> bool {
        self.inner.write_awaits(ctx)
    }

    fn invoke_awaits(&self, ctx: impl InvokeContext) -> bool {
        self.inner.invoke_awaits(ctx)
    }

    fn read(
        &self,
        ctx: impl ReadContext,
        reply: impl rs_matter::dm::ReadReply,
    ) -> impl std::future::Future<Output = Result<(), MatterLibError>> {
        self.inner.read(ctx, reply)
    }

    fn write(
        &self,
        ctx: impl WriteContext,
    ) -> impl std::future::Future<Output = Result<(), MatterLibError>> {
        self.inner.write(ctx)
    }

    fn invoke(
        &self,
        ctx: impl InvokeContext,
        reply: impl rs_matter::dm::InvokeReply,
    ) -> impl std::future::Future<Output = Result<(), MatterLibError>> {
        self.inner.invoke(ctx, reply)
    }

    fn run(
        &self,
        ctx: impl HandlerContext,
    ) -> impl std::future::Future<Output = Result<(), MatterLibError>> {
        self.inner.run(ctx)
    }
}

impl BridgeHandler {
    fn new(matter: &Matter<'_>, devices: &[BridgeDevice]) -> (Arc<Self>, BridgeMetadata) {
        let mut endpoints = Vec::with_capacity(devices.len() + 2);
        endpoints.push(endpoints::root_endpoint(NetworkType::Ethernet));
        endpoints.push(Endpoint {
            id: AGGREGATOR_ENDPOINT_ID,
            device_types: rs_matter::devices!(DEV_TYPE_AGGREGATOR),
            clusters: rs_matter::clusters!(desc::DescHandler::CLUSTER),
        });

        let mut descriptors = HashMap::new();
        let mut bridged_info = HashMap::new();
        let mut on_off = HashMap::new();

        for device in devices {
            endpoints.push(Endpoint {
                id: device.endpoint_id,
                device_types: endpoint_device_types(device),
                clusters: rs_matter::clusters!(
                    desc::DescHandler::CLUSTER,
                    BridgedHandler::CLUSTER,
                    BridgeOnOffHooks::CLUSTER
                ),
            });

            descriptors.insert(
                device.endpoint_id,
                desc::DescHandler::new(Dataver::new_rand(matter.rand())).adapt(),
            );

            bridged_info.insert(
                device.endpoint_id,
                BridgedHandler::new(
                    Dataver::new_rand(matter.rand()),
                    device.entity_id.clone(),
                    device.reachable,
                )
                .adapt(),
            );

            on_off.insert(
                device.endpoint_id,
                OnOffHandler::new_standalone(
                    Dataver::new_rand(matter.rand()),
                    device.endpoint_id,
                    BridgeOnOffHooks::new(device.entity_id.clone()),
                ),
            );
        }

        let handler = BridgeHandler {
            aggregator_desc: desc::DescHandler::new_aggregator(Dataver::new_rand(matter.rand())).adapt(),
            descriptors,
            bridged_info,
            on_off,
        };

        (
            Arc::new(handler),
            BridgeMetadata {
                endpoints,
            },
        )
    }

    fn on_off_handler(&self, endpoint_id: u16) -> Option<&OnOffHandler<'static, BridgeOnOffHooks, NoLevelControl>> {
        self.on_off.get(&endpoint_id)
    }
}

impl AsyncHandler for BridgeHandler {
    fn read_awaits(&self, _ctx: impl ReadContext) -> bool {
        true
    }

    fn write_awaits(&self, _ctx: impl WriteContext) -> bool {
        true
    }

    fn invoke_awaits(&self, _ctx: impl InvokeContext) -> bool {
        true
    }

    async fn read(
        &self,
        ctx: impl ReadContext,
        reply: impl rs_matter::dm::ReadReply,
    ) -> Result<(), MatterLibError> {
        let endpoint_id = ctx.attr().endpoint_id;
        let cluster_id = ctx.attr().cluster_id;

        if cluster_id == desc::DescHandler::CLUSTER.id {
            if endpoint_id == AGGREGATOR_ENDPOINT_ID {
                return DmAsync(&self.aggregator_desc).read(ctx, reply).await;
            }
            if let Some(desc) = self.descriptors.get(&endpoint_id) {
                return DmAsync(desc).read(ctx, reply).await;
            }
        }

        if cluster_id == BridgedHandler::CLUSTER.id {
            if let Some(bridged) = self.bridged_info.get(&endpoint_id) {
                return DmAsync(bridged).read(ctx, reply).await;
            }
        }

        if cluster_id == BridgeOnOffHooks::CLUSTER.id {
            if let Some(handler) = self.on_off.get(&endpoint_id) {
                return on_off::HandlerAsyncAdaptor(handler).read(ctx, reply).await;
            }
        }

        Err(ErrorCode::AttributeNotFound.into())
    }

    async fn write(&self, ctx: impl WriteContext) -> Result<(), MatterLibError> {
        let endpoint_id = ctx.attr().endpoint_id;
        let cluster_id = ctx.attr().cluster_id;

        if cluster_id == BridgeOnOffHooks::CLUSTER.id {
            if let Some(handler) = self.on_off.get(&endpoint_id) {
                return on_off::HandlerAsyncAdaptor(handler).write(ctx).await;
            }
        }

        Err(ErrorCode::AttributeNotFound.into())
    }

    async fn invoke(
        &self,
        ctx: impl InvokeContext,
        reply: impl rs_matter::dm::InvokeReply,
    ) -> Result<(), MatterLibError> {
        let endpoint_id = ctx.cmd().endpoint_id;
        let cluster_id = ctx.cmd().cluster_id;

        if cluster_id == BridgeOnOffHooks::CLUSTER.id {
            if let Some(handler) = self.on_off.get(&endpoint_id) {
                return on_off::HandlerAsyncAdaptor(handler).invoke(ctx, reply).await;
            }
        }

        if cluster_id == BridgedHandler::CLUSTER.id {
            if let Some(bridged) = self.bridged_info.get(&endpoint_id) {
                return DmAsync(bridged).invoke(ctx, reply).await;
            }
        }

        Err(ErrorCode::CommandNotFound.into())
    }

    fn run(&self, ctx: impl HandlerContext) -> impl std::future::Future<Output = Result<(), MatterLibError>> {
        async move {
            let adaptors: Vec<_> = self
                .on_off
                .values()
                .map(on_off::HandlerAsyncAdaptor)
                .collect();
            let mut tasks = FuturesUnordered::new();

            for adaptor in &adaptors {
                tasks.push(adaptor.run(&ctx));
            }

            if tasks.is_empty() {
                futures::future::pending::<()>().await;
                return Ok(());
            }

            while let Some(result) = tasks.next().await {
                if let Err(err) = result {
                    return Err(err);
                }
            }

            Ok(())
        }
    }
}

#[derive(Debug)]
struct BridgedHandler {
    dataver: Dataver,
    unique_id: String,
    reachable: Cell<bool>,
}

impl BridgedHandler {
    const CLUSTER: Cluster<'static> = bridged_device_basic_information::FULL_CLUSTER
        .with_features(0)
        .with_attrs(with!(required))
        .with_cmds(with!());

    fn new(dataver: Dataver, unique_id: String, reachable: bool) -> Self {
        Self {
            dataver,
            unique_id,
            reachable: Cell::new(reachable),
        }
    }

    fn adapt(self) -> bridged_device_basic_information::HandlerAdaptor<Self> {
        bridged_device_basic_information::HandlerAdaptor(self)
    }
}

impl bridged_device_basic_information::ClusterHandler for BridgedHandler {
    const CLUSTER: Cluster<'static> = BridgedHandler::CLUSTER;

    fn dataver(&self) -> u32 {
        self.dataver.get()
    }

    fn dataver_changed(&self) {
        self.dataver.changed();
    }

    fn reachable(&self, _ctx: impl ReadContext) -> Result<bool, MatterLibError> {
        Ok(self.reachable.get())
    }

    fn unique_id<P: TLVBuilderParent>(
        &self,
        _ctx: impl ReadContext,
        builder: Utf8StrBuilder<P>,
    ) -> Result<P, MatterLibError> {
        builder.set(self.unique_id.as_str())
    }

    fn handle_keep_active(
        &self,
        _ctx: impl InvokeContext,
        _request: bridged_device_basic_information::KeepActiveRequest<'_>,
    ) -> Result<(), MatterLibError> {
        Ok(())
    }
}

#[derive(Debug)]
struct BridgeOnOffHooks {
    entity_id: String,
    on_off: Cell<bool>,
    start_up_on_off: Cell<Option<StartUpOnOffEnum>>,
}

impl BridgeOnOffHooks {
    fn new(entity_id: String) -> Self {
        Self {
            entity_id,
            on_off: Cell::new(false),
            start_up_on_off: Cell::new(None),
        }
    }
}

impl OnOffHooks for BridgeOnOffHooks {
    const CLUSTER: Cluster<'static> = on_off_cluster::FULL_CLUSTER
        .with_revision(6)
        .with_attrs(with!(required; on_off_cluster::AttributeId::OnOff))
        .with_cmds(with!(
            on_off_cluster::CommandId::Off
                | on_off_cluster::CommandId::On
                | on_off_cluster::CommandId::Toggle
        ));

    fn on_off(&self) -> bool {
        self.on_off.get()
    }

    fn set_on_off(&self, on: bool) {
        self.on_off.set(on);
        tracing::debug!(
            entity_id = %self.entity_id,
            state = on,
            "Matter on/off state updated"
        );
    }

    fn start_up_on_off(&self) -> Nullable<StartUpOnOffEnum> {
        match self.start_up_on_off.get() {
            Some(value) => Nullable::some(value),
            None => Nullable::none(),
        }
    }

    fn set_start_up_on_off(
        &self,
        value: Nullable<StartUpOnOffEnum>,
    ) -> Result<(), MatterLibError> {
        self.start_up_on_off.set(value.into_option());
        Ok(())
    }

    async fn handle_off_with_effect(&self, _effect: EffectVariantEnum) {}
}

async fn run_matter_runtime(
    bridge_id: Uuid,
    port: u16,
    devices: Vec<BridgeDevice>,
    mut cmd_rx: mpsc::UnboundedReceiver<BridgeCommand>,
) -> Result<(), MatterError> {
    let device_name = Box::leak(format!("HAMH {}", bridge_id).into_boxed_str());
    let serial_no: &'static str = Box::leak(format!("hamh-{}", bridge_id).into_boxed_str());
    let unique_id = serial_no;

    let dev_det = rs_matter::dm::clusters::basic_info::BasicInfoConfig {
        device_name,
        serial_no,
        unique_id,
        ..TEST_DEV_DET
    };

    let dev_comm = BasicCommData {
        password: env_u32("HAMH_MATTER_PASSCODE", TEST_DEV_COMM.password),
        discriminator: env_u16("HAMH_MATTER_DISCRIMINATOR", TEST_DEV_COMM.discriminator),
    };

    let matter = Matter::new_default(&dev_det, dev_comm, &TEST_DEV_ATT, port);
    matter
        .initialize_transport_buffers()
        .map_err(|err| MatterError::Runtime(err.to_string()))?;

    let buffers = PooledBuffers::<10, NoopRawMutex, _>::new(0);
    let subscriptions = DefaultSubscriptions::new();
    let (handler, metadata) = BridgeHandler::new(&matter, &devices);
    let handler_ref = BridgeHandlerRef::new(handler.clone());

    let dm = DataModel::new(&matter, &buffers, &subscriptions, (metadata, handler_ref));
    let responder = DefaultResponder::new(&dm);

    let mut respond = std::pin::pin!(responder.run::<4, 4>());
    let mut dm_job = std::pin::pin!(dm.run());

    let socket_addr = SocketAddr::from(([0, 0, 0, 0], port));
    let socket = AsyncIo::<UdpSocket>::bind(socket_addr)
        .map_err(|err| map_io_error(err, port))?;

    let mut mdns = std::pin::pin!(run_mdns(&matter));
    let mut transport = std::pin::pin!(matter.run(&socket, &socket));

    let mut psm: Psm<4096> = Psm::new();
    let psm_path = persist_path(bridge_id);
    if let Some(parent) = psm_path.parent() {
        if let Err(err) = std::fs::create_dir_all(parent) {
            tracing::warn!("Failed to create persist dir: {err}");
        }
    }

    if let Err(err) = psm.load(&psm_path, &matter, NO_NETWORKS) {
        tracing::warn!("Failed to load Matter storage: {err}");
    }

    if !matter.is_commissioned() {
        matter
            .print_standard_qr_text(DiscoveryCapabilities::IP)
            .map_err(|err| MatterError::Runtime(err.to_string()))?;
        matter
            .print_standard_qr_code(QrTextType::Unicode, DiscoveryCapabilities::IP)
            .map_err(|err| MatterError::Runtime(err.to_string()))?;
        matter
            .open_basic_comm_window(MAX_COMM_WINDOW_TIMEOUT_SECS)
            .map_err(|err| MatterError::Runtime(err.to_string()))?;
    }

    let mut persist = std::pin::pin!(psm.run(&psm_path, &matter, NO_NETWORKS));

    let mut entity_to_endpoint = build_entity_map(&devices);
    let mut mdns_running = true;
    let mut persist_running = true;

    loop {
        tokio::select! {
            res = &mut transport => {
                return res.map_err(|err: MatterLibError| MatterError::Runtime(err.to_string()));
            }
            res = &mut mdns, if mdns_running => {
                if let Err(err) = res {
                    tracing::warn!("mDNS stopped: {err}");
                }
                mdns_running = false;
            }
            res = &mut persist, if persist_running => {
                if let Err(err) = res {
                    tracing::warn!("Matter persistence stopped: {err}");
                }
                persist_running = false;
            }
            res = &mut respond => {
                return res.map_err(|err: MatterLibError| MatterError::Runtime(err.to_string()));
            }
            res = &mut dm_job => {
                return res.map_err(|err: MatterLibError| MatterError::Runtime(err.to_string()));
            }
            cmd = cmd_rx.recv() => {
                match cmd {
                    Some(BridgeCommand::Shutdown) => break,
                    Some(BridgeCommand::FactoryReset) => {
                        matter.reset_persist(true);
                        let _ = std::fs::remove_file(&psm_path);
                        if let Err(err) = matter.open_basic_comm_window(MAX_COMM_WINDOW_TIMEOUT_SECS) {
                            tracing::warn!("Failed to open comm window after reset: {err}");
                        }
                    }
                    Some(BridgeCommand::UpdateDevices(devices)) => {
                        entity_to_endpoint = build_entity_map(&devices);
                        tracing::info!(
                            "Bridge refresh requested; device list updated (restart required for endpoint changes)."
                        );
                    }
                    Some(BridgeCommand::UpdateStates(states)) => {
                        for update in states {
                            if let Some(endpoint_id) = entity_to_endpoint.get(&update.entity_id) {
                                if let Some(handler) = handler.on_off_handler(*endpoint_id) {
                                    handler.set_on_off(update.on);
                                }
                            }
                        }
                    }
                    None => break,
                }
            }
        }
    }

    Ok(())
}

fn build_entity_map(devices: &[BridgeDevice]) -> HashMap<String, u16> {
    let mut map = HashMap::new();
    for device in devices {
        map.insert(device.entity_id.clone(), device.endpoint_id);
    }
    map
}

fn endpoint_device_types(_device: &BridgeDevice) -> &'static [rs_matter::dm::DeviceType] {
    rs_matter::devices!(DEV_TYPE_ON_OFF_LIGHT, DEV_TYPE_BRIDGED_NODE)
}

fn persist_path(bridge_id: Uuid) -> PathBuf {
    let root = std::env::var("HAMH_STORAGE_LOCATION").unwrap_or_else(|_| ".hamh-storage".to_string());
    PathBuf::from(root)
        .join(PERSIST_DIR_NAME)
        .join(format!("bridge-{}.psm", bridge_id))
}

fn map_io_error(err: std::io::Error, port: u16) -> MatterError {
    if err.kind() == std::io::ErrorKind::AddrInUse {
        MatterError::PortInUse(port)
    } else {
        MatterError::Io(err.to_string())
    }
}

fn env_u32(key: &str, default: u32) -> u32 {
    std::env::var(key)
        .ok()
        .and_then(|value| value.parse::<u32>().ok())
        .unwrap_or(default)
}

fn env_u16(key: &str, default: u16) -> u16 {
    std::env::var(key)
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(default)
}

#[cfg(unix)]
async fn run_mdns(matter: &Matter<'_>) -> Result<(), MatterLibError> {
    use std::net::Ipv6Addr;

    use rs_matter::transport::network::mdns::builtin::{BuiltinMdnsResponder, Host};
    use rs_matter::transport::network::mdns::{
        MDNS_IPV4_BROADCAST_ADDR, MDNS_IPV6_BROADCAST_ADDR, MDNS_SOCKET_DEFAULT_BIND_ADDR,
    };
    use socket2::{Domain, Protocol, Socket, Type};

    use nix::{net::if_::InterfaceFlags, sys::socket::SockaddrIn6};

    fn initialize_network() -> Result<(Ipv4Addr, Ipv6Addr, u32), MatterLibError> {
        let interfaces = || {
            nix::ifaddrs::getifaddrs().unwrap().filter(|ia| {
                ia.flags
                    .contains(InterfaceFlags::IFF_UP | InterfaceFlags::IFF_BROADCAST)
                    && !ia
                        .flags
                        .intersects(InterfaceFlags::IFF_LOOPBACK | InterfaceFlags::IFF_POINTOPOINT)
            })
        };

        let (iname, ip, ipv6) = interfaces()
            .filter_map(|ia| {
                ia.address
                    .and_then(|addr| addr.as_sockaddr_in6().map(SockaddrIn6::ip))
                    .map(|ipv6| (ia.interface_name, ipv6))
            })
            .filter_map(|(iname, ipv6)| {
                interfaces()
                    .filter(|ia2| ia2.interface_name == iname)
                    .find_map(|ia2| {
                        ia2.address
                            .and_then(|addr| addr.as_sockaddr_in().map(|addr| addr.ip().into()))
                            .map(|ip: Ipv4Addr| (iname.clone(), ip, ipv6))
                    })
            })
            .next()
            .ok_or_else(|| {
                tracing::error!("Cannot find network interface suitable for mDNS broadcasting");
                ErrorCode::StdIoError.into()
            })?;

        tracing::info!("mDNS using interface {iname} with {ip}/{ipv6}");

        Ok((ip, ipv6, 0))
    }

    let (ipv4_addr, ipv6_addr, interface) = initialize_network()?;

    let mut socket = Socket::new(Domain::IPV6, Type::DGRAM, Some(Protocol::UDP))?;
    socket.set_reuse_address(true)?;
    socket.set_only_v6(false)?;
    socket.bind(&MDNS_SOCKET_DEFAULT_BIND_ADDR.into())?;
    let socket = AsyncIo::<UdpSocket>::new_nonblocking(socket.into())?;

    socket
        .get_ref()
        .join_multicast_v6(&MDNS_IPV6_BROADCAST_ADDR, interface)?;
    socket
        .get_ref()
        .join_multicast_v4(&MDNS_IPV4_BROADCAST_ADDR, &ipv4_addr)?;

    let host = Host {
        id: 0,
        hostname: "hamh",
        ip: ipv4_addr,
        ipv6: ipv6_addr,
    };

    BuiltinMdnsResponder::new(matter)
        .run(&socket, &socket, &host, Some(ipv4_addr), Some(interface))
        .await
}

#[cfg(not(unix))]
async fn run_mdns(matter: &Matter<'_>) -> Result<(), MatterLibError> {
    use rs_matter::transport::network::mdns::zeroconf::ZeroconfMdnsResponder;

    ensure_bonjour_path();
    ZeroconfMdnsResponder::new(matter).run().await?;
    Ok(())
}

#[cfg(not(unix))]
fn ensure_bonjour_path() {
    use std::path::PathBuf;

    fn push_path(dir: &PathBuf) {
        let dir = dir.to_string_lossy().to_string();
        let current = std::env::var("PATH").unwrap_or_default();
        if !current.split(';').any(|p| p.eq_ignore_ascii_case(&dir)) {
            std::env::set_var("PATH", format!("{dir};{current}"));
            tracing::info!("mDNS: added Bonjour SDK path to PATH ({dir})");
        }
    }

    let sdk_dir = std::env::var("HAMH_BONJOUR_SDK").ok().map(PathBuf::from);
    let candidates = [
        sdk_dir,
        Some(PathBuf::from("C:\\Program Files\\Bonjour SDK\\lib\\x64")),
        Some(PathBuf::from("C:\\Program Files (x86)\\Bonjour SDK\\lib\\x64")),
        Some(PathBuf::from("C:\\Program Files\\Bonjour SDK\\lib")),
        Some(PathBuf::from("C:\\Program Files (x86)\\Bonjour SDK\\lib")),
    ];

    for dir in candidates.iter().flatten() {
        if dir.join("dns_sd.dll").exists() {
            push_path(dir);
            return;
        }
    }

    tracing::warn!(
        "mDNS: Bonjour SDK dns_sd.dll not found. Install Bonjour SDK and/or set HAMH_BONJOUR_SDK to its lib path."
    );
}
