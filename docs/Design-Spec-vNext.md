# Design Spec — HAMH vNext (Configurable Matter Bridge)

## 1) Goals
- Provide a robust Matter bridge that exposes Home Assistant entities to Matter controllers.
- Support multiple configurable bridges with independent filters, ports, and commissioning.
- Ensure stable delete/restart behavior and no backend crashes.
- Keep add-on installation and Supervisor integration fully functional.
- Improve observability: persistent logs, diagnostics, and clear error states.

## 2) Non‑Goals
- No reliance on cloud services or external skills.
- No requirement to keep backward compatibility with existing UI/CLI behavior if it reduces robustness.
- Not aiming to be a general Matter controller (bridge only).

## 3) Key Requirements
- **Configurable Bridges**: Each bridge has its own config, filters, and status.
- **Independent Lifecycle**: Create/Start/Stop/Refresh/Delete is isolated per bridge.
- **Stable Deletion**: Delete never crashes backend; always idempotent.
- **Home Assistant Add‑on**: Uses Supervisor token and ingress; runs in container.
- **Local Dev**: PC mode for dev connected to remote HA.
- **Operations Queue**: Persisted lifecycle operations with retries and clear status.
- **Admin UI Logs**: Provide in‑app log downloads for persistent log files.

## 4) High‑Level Architecture
- **API Layer** (Express or Fastify)
  - `/api/matter/...` endpoints for bridges and devices.
  - No legacy alias `/api/bridges` (enforced prefix).
- **Core Bridge Engine**
  - BridgeService: lifecycle orchestration.
  - BridgeRuntime: per‑bridge runtime with Matter node, endpoints, and registries.
  - BridgeStorage: persists config and runtime metadata.
- **Integrations**
  - HomeAssistantClient: websocket for state and services.
  - EntityMapper: maps HA entities to Matter device types.
- **Frontend**
  - Single‑page UI for bridge list/details/create/edit and device status.

## 5) Data Model
### BridgeConfig
- `id`, `name`, `port`, `filter`, `featureFlags`, `commissioning`, `createdAt`, `updatedAt`

### BridgeFilter
- `include`: array of `{ type: "domain" | "entity_id" | "area" | "label" | "device_id", value }`
- `exclude`: same schema

### BridgeRuntimeState
- `status`: `stopped | starting | running | stopping | deleting | error | queued`
- `lastError`, `lastStart`, `lastStop`, `operationId`

### BridgeOperation
- `operationId`, `bridgeId`, `type`, `status`, `queuedAt`, `startedAt`, `finishedAt`, `error`
- `type`: `create | start | stop | refresh | delete | factory-reset | update`
- `status`: `queued | running | completed | failed | cancelled`

### BridgeDevice
- `entity_id`, `device_type`, `endpoint_id`, `capabilities`, `reachable`

## 6) REST API (v1)
Base: `/api/matter`

- `GET /bridges` → list bridges
- `POST /bridges` → create bridge
- `GET /bridges/:id` → bridge details
- `PUT /bridges/:id` → update bridge
- `DELETE /bridges/:id` → delete bridge
- `POST /bridges/:id/actions/start` → start
- `POST /bridges/:id/actions/stop` → stop
- `POST /bridges/:id/actions/refresh` → refresh endpoints
- `POST /bridges/:id/actions/factory-reset` → factory reset
- `GET /bridges/:id/devices` → list devices for a bridge
- `GET /operations` → list operations (latest first)
- `GET /operations/:id` → operation details
- `GET /health` → health + build info

### API Behavior Rules
- Deletion is **idempotent**: deleting a missing bridge returns `204`.
- Mutating operations return `202` with `operationId` when queued/async.
- Consistent JSON errors: `{ code, message, details }`.

## 7) Bridge Lifecycle
### Create
- Validate config → persist → create runtime entry.

### Start
- Acquire bridge lock → build Matter node + aggregator → subscribe to HA entities → ready.

### Refresh
- Diff HA registry → update endpoints with minimal rebuild.

### Stop
- Graceful stop: unsubscribe HA → stop Matter node → mark stopped.

### Delete
- Mark `deleting` → stop → remove storage → remove runtime entry.
- Never crash on missing storage; return success.

## 8) Concurrency & Robustness
- Per‑bridge mutex (async lock) so only one lifecycle action runs at once.
- **Operations queue** persists lifecycle actions to storage.
- On crash/restart, incomplete operations resume or roll back safely.
- System‑wide shutdown stops bridges in parallel with timeout and error aggregation.

## 9) Endpoint Mapping
- Mapping table from HA domains to Matter devices:
  - `light` → `DimmableLight` / `ColorTemperatureLight` / `ExtendedColorLight`
  - `switch` → `OnOffSwitch`
  - `fan`, `cover`, `sensor`, etc. as defined in mapping spec
- Behaviors:
  - `HomeAssistantEntityBehavior` adapter for HA state/actions
  - Cluster behaviors aligned with Matter spec
- **Sync policy**
  - Initial full sync on start
  - Incremental updates on HA state changes
  - Periodic full reconciliation every N minutes

## 10) Frontend UX
- Bridge list with status badges: `running`, `stopped`, `deleting`, `error`, `queued`.
- Delete flows:
  - Shows “Deleting…” and disables actions.
  - Polls operation state; shows error if failed.
- Detailed view:
  - Commissioning info, bridge settings, mapped entities summary.
- Operations view:
  - Recent operations with status and timestamps.

## 11) Observability & Logs
- Persistent logs under `${HAMH_STORAGE_LOCATION}/logs`:
  - `backend.log`
  - `bridge-delete.log`
  - `backend-crash.log`
- Structured logging (JSON optional).
- Correlation ID for API requests.
- Health endpoint with version and uptime.

## 12) Admin UI: Log Downloads
- Provide a small admin page with:
  - Download links for persistent log files.
  - Basic metadata (size, last modified).
- **Rationale**: Supervisor add‑on logs show console output, but not always file‑based
  logs in storage. In‑app downloads make support easier without SSH.

## 13) Home Assistant Add‑on Integration
- Add‑on runs with `host_network` for mDNS.
- Uses Supervisor token by default.
- Supports ingress by default.
- Add‑on config options:
  - `app_log_level`, `mdns_interface`, `disable_log_colors`
- Add‑ons repo points to `ghcr.io/parnunu/home-assistant-matter-hub-addon`.

## 14) Local PC Mode
- `pnpm run dev:pc` for backend + frontend
- Backend listens on `8482`, frontend on `5173`
- `.env` supports remote HA URL/token
- Local storage defaults to `~/.hamh-development`

## 15) Testing Strategy
- Unit tests: entity mapping, API handlers, storage logic
- Integration tests: fake HA websocket, refresh/diff behavior
- E2E: create/start/delete bridge and verify stability

## 16) Migration Plan
- Detect existing storage format from main branch.
- Migrate configs to new schema.
- If migration fails, keep backup and start empty.

## 17) Open Questions (Resolved)
- Legacy API alias `/api/bridges`? **No**.
- Persisted operations queue? **Yes**.
- Admin UI log downloads? **Yes** (Supervisor logs do not cover file logs).
