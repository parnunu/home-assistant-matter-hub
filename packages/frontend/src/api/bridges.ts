import type {
  BridgeDataWithMetadata,
  CreateBridgeRequest,
  UpdateBridgeRequest,
} from "@home-assistant-matter-hub/common";

export async function fetchBridges() {
  const res = await fetch(`api/matter/bridges?_s=${Date.now()}`);
  const json = await res.json();
  return json as BridgeDataWithMetadata[];
}

export async function createBridge(req: CreateBridgeRequest) {
  return fetch("api/matter/bridges", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(req),
  }).then((res) => res.json() as Promise<BridgeDataWithMetadata>);
}

export async function updateBridge(req: UpdateBridgeRequest) {
  return fetch(`api/matter/bridges/${req.id}`, {
    method: "PUT",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(req),
  }).then((res) => res.json() as Promise<BridgeDataWithMetadata>);
}

export async function deleteBridge(bridgeId: string) {
  const res = await fetch(`api/matter/bridges/${bridgeId}`, {
    method: "DELETE",
  });
  if (!res.ok) {
    throw new Error(`Failed to delete bridge (${res.status})`);
  }
}

export async function resetBridge(bridgeId: string) {
  return await fetch(`api/matter/bridges/${bridgeId}/actions/factory-reset`, {
    method: "GET",
  }).then((res) => res.json() as Promise<BridgeDataWithMetadata>);
}
