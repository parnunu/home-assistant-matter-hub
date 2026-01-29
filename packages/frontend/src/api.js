const API_BASE = "/api";

async function request(path, options = {}) {
  const response = await fetch(`${API_BASE}${path}`, {
    headers: {
      "Content-Type": "application/json",
    },
    ...options,
  });

  if (!response.ok) {
    const message = await response.text();
    throw new Error(message || "Request failed");
  }

  if (response.status === 204) {
    return undefined;
  }

  return response.json();
}

export async function getBridges() {
  return request("/bridges");
}

export async function getBridge(bridgeId) {
  return request(`/bridges/${bridgeId}`);
}

export async function getBridgeDevices(bridgeId) {
  return request(`/bridges/${bridgeId}/devices`);
}

export async function createBridge(payload) {
  return request("/bridges", {
    method: "POST",
    body: JSON.stringify(payload),
  });
}

export async function updateBridge(payload) {
  return request(`/bridges/${payload.id}`, {
    method: "PUT",
    body: JSON.stringify(payload),
  });
}

export async function deleteBridge(id) {
  return request(`/bridges/${id}`, { method: "DELETE" });
}

export async function factoryReset(id) {
  return request(`/bridges/${id}/actions/factory-reset`);
}
