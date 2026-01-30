const apiBase = "/api/matter";

const state = {
  include: [],
  exclude: [],
  bridges: [],
  operations: [],
  devices: [],
  pairing: null,
  currentBridge: null,
  editInclude: [],
  editExclude: [],
  view: "list",
  runtime: {},
  lastBridgesJson: "",
  lastOpsJson: "",
  lastRuntimeJson: "",
  opsVisible: false,
};

const el = (id) => document.getElementById(id);

const filterTypes = [
  "pattern",
  "domain",
  "platform",
  "entity_category",
  "label",
  "area",
  "entity_id",
  "device_id",
];

const formatDate = (value) => {
  if (!value) return "-";
  if (Array.isArray(value)) {
    const [year, ordinal, hour, minute, second, nanosecond = 0, offsetHour = 0, offsetMinute = 0, offsetSecond = 0] = value;
    if (!year || !ordinal) return value.join(",");
    const offsetMs = ((offsetHour || 0) * 3600 + (offsetMinute || 0) * 60 + (offsetSecond || 0)) * 1000;
    const baseMs = Date.UTC(year, 0, 1, 0, 0, 0, 0);
    const dayMs = (ordinal - 1) * 24 * 60 * 60 * 1000;
    const timeMs = ((hour || 0) * 3600 + (minute || 0) * 60 + (second || 0)) * 1000;
    const ms = baseMs + dayMs + timeMs + Math.floor((nanosecond || 0) / 1e6) - offsetMs;
    const date = new Date(ms);
    return Number.isNaN(date.getTime()) ? value.join(",") : date.toLocaleString();
  }
  const date = new Date(value);
  return Number.isNaN(date.getTime()) ? value : date.toLocaleString();
};

const showToast = (message, tone = "info") => {
  const toast = el("toast");
  toast.textContent = message;
  toast.className = `toast show ${tone}`;
  clearTimeout(showToast._timer);
  showToast._timer = setTimeout(() => {
    toast.className = "toast";
  }, 2800);
};

const setStatus = (ok, message) => {
  const pill = el("statusPill");
  pill.textContent = message;
  pill.classList.remove("ok", "warn", "bad");
  if (ok === true) pill.classList.add("ok");
  if (ok === false) pill.classList.add("bad");
};

const setPills = (container, list, onRemove) => {
  container.innerHTML = "";
  list.forEach((item, index) => {
    const pill = document.createElement("div");
    pill.className = "pill";
    const label = document.createElement("span");
    label.textContent = `${item.type}:${item.value}`;
    const btn = document.createElement("button");
    btn.type = "button";
    btn.textContent = "x";
    btn.onclick = () => onRemove(index);
    pill.appendChild(label);
    pill.appendChild(btn);
    container.appendChild(pill);
  });
};

const formatFilterList = (list) => {
  if (!list || !list.length) return "<span class=\"muted\">None</span>";
  return list
    .map((item) => `<span class="pill pill-static">${item.type}:${item.value}</span>`)
    .join(" ");
};

const fetchJson = async (url, options = {}) => {
  const res = await fetch(url, {
    headers: { "Content-Type": "application/json" },
    ...options,
  });
  if (!res.ok) {
    const text = await res.text();
    throw new Error(text || res.statusText);
  }
  if (res.status === 204) return null;
  const text = await res.text();
  if (!text) return null;
  try {
    return JSON.parse(text);
  } catch {
    return text;
  }
};

const loadBridges = async () => {
  const bridges = await fetchJson(`${apiBase}/bridges`);
  state.bridges = bridges;
};

const loadOperations = async () => {
  const ops = await fetchJson(`${apiBase}/operations`);
  state.operations = ops;
};

const loadRuntime = async () => {
  const runtime = await fetchJson(`${apiBase}/bridges/runtime`);
  const map = {};
  (runtime || []).forEach((entry) => {
    map[entry.bridge_id] = entry.state;
  });
  state.runtime = map;
};

const loadBridge = async (id) => {
  const bridge = await fetchJson(`${apiBase}/bridges/${id}`);
  state.currentBridge = bridge;
};

const loadDevices = async (id) => {
  const devices = await fetchJson(`${apiBase}/bridges/${id}/devices`);
  state.devices = devices || [];
};

const loadPairing = async (id) => {
  try {
    const pairing = await fetchJson(`${apiBase}/bridges/${id}/pairing`);
    state.pairing = pairing;
  } catch (err) {
    state.pairing = null;
  }
};

const latestOpByBridge = () => {
  const sorted = [...state.operations].sort((a, b) => {
    const aTime = new Date(a.queued_at || 0).getTime();
    const bTime = new Date(b.queued_at || 0).getTime();
    return bTime - aTime;
  });
  const map = new Map();
  sorted.forEach((op) => {
    if (!map.has(op.bridge_id)) {
      map.set(op.bridge_id, op);
    }
  });
  return map;
};

const badgeForStatus = (op) => {
  if (!op) return { label: "idle", tone: "" };
  const status = op.status || "unknown";
  const tone = status === "Completed" ? "ok" : status === "Failed" ? "bad" : "warn";
  return { label: `${op.op_type} - ${status}`.toLowerCase(), tone };
};

const runtimeBadge = (bridgeId) => {
  const runtime = state.runtime[bridgeId];
  if (!runtime) return null;
  const status = (runtime.status || "unknown").toLowerCase();
  let tone = "warn";
  if (status === "running") tone = "ok";
  if (status === "stopped" || status === "error") tone = "bad";
  return { label: status, tone };
};

const nextAvailablePort = () => {
  const base = 5540;
  if (!state.bridges.length) return base;
  const ports = state.bridges.map((b) => b.port).filter((p) => Number.isFinite(p));
  const max = ports.length ? Math.max(...ports) : base - 1;
  const candidate = Math.min(max + 1, 65535);
  return candidate >= 1024 ? candidate : base;
};

const domainIcon = (domain) => {
  switch (domain) {
    case "light":
      return "L";
    case "switch":
      return "S";
    case "fan":
      return "F";
    case "cover":
      return "C";
    default:
      return domain.slice(0, 1).toUpperCase();
  }
};

const deviceLabel = (device) => device.display_name || device.entity_id;

const renderListView = () => {
  el("viewList").classList.remove("hidden");
  el("viewCreate").classList.add("hidden");
  el("viewDetails").classList.add("hidden");
  el("viewEdit").classList.add("hidden");

  const list = el("bridgeList");
  const empty = el("bridgeEmpty");
  list.innerHTML = "";
  if (!state.bridges.length) {
    empty.style.display = "block";
    return;
  }
  empty.style.display = "none";
  const opsMap = latestOpByBridge();

  state.bridges.forEach((bridge) => {
    const card = document.createElement("div");
    card.className = "bridge-card";

    const header = document.createElement("div");
    header.className = "bridge-header";

    const title = document.createElement("div");
    title.className = "bridge-title";
    title.innerHTML = `<strong>${bridge.name}</strong><span class="muted">Port ${bridge.port}</span>`;

    const op = opsMap.get(bridge.id);
    const runtime = runtimeBadge(bridge.id);
    const badgeMeta = runtime || badgeForStatus(op);
    const badge = document.createElement("span");
    badge.className = `badge ${badgeMeta.tone}`;
    badge.textContent = badgeMeta.label;

    header.appendChild(title);
    header.appendChild(badge);
    card.appendChild(header);

    const meta = document.createElement("div");
    meta.className = "bridge-meta";
    const activity =
      op && op.status && op.status.toLowerCase() === "running"
        ? `Activity: ${op.op_type.toLowerCase()}`
        : null;
    meta.innerHTML = `
      <span>ID: ${bridge.id}</span>
      <span>Updated: ${formatDate(bridge.updated_at)}</span>
      ${runtime ? `<span>State: ${runtime.label}</span>` : ""}
      ${activity ? `<span>${activity}</span>` : ""}
    `;
    card.appendChild(meta);

    const actions = document.createElement("div");
    actions.className = "bridge-actions";

    const viewBtn = document.createElement("button");
    viewBtn.className = "btn btn-ghost";
    viewBtn.textContent = "View details";
    viewBtn.onclick = () => navigateTo(`#bridge/${bridge.id}`);
    actions.appendChild(viewBtn);

    card.appendChild(actions);
    list.appendChild(card);
  });
};

const renderDeviceTree = () => {
  const deviceList = el("deviceList");
  const devicesEmpty = el("devicesEmpty");
  deviceList.innerHTML = "";

  if (!state.devices.length) {
    devicesEmpty.style.display = "block";
    return;
  }

  devicesEmpty.style.display = "none";

  const root = document.createElement("details");
  root.className = "tree-node";
  root.open = true;
  root.innerHTML = `<summary><span class="tree-label">Endpoints</span></summary>`;

  const bridgeNode = document.createElement("details");
  bridgeNode.className = "tree-node";
  bridgeNode.open = true;
  bridgeNode.innerHTML = `<summary><span class="tree-label">${state.currentBridge?.name || "Bridge"}</span></summary>`;

  const aggNode = document.createElement("details");
  aggNode.className = "tree-node";
  aggNode.open = true;
  aggNode.innerHTML = `<summary><span class="tree-label">aggregator</span></summary>`;

  const list = document.createElement("div");
  list.className = "tree-leaf-list";

  state.devices.forEach((device) => {
    const domain = device.entity_id.split(".")[0] || "unknown";
    const leaf = document.createElement("div");
    leaf.className = "tree-leaf";

    const main = document.createElement("div");
    main.className = "device-main";
    main.innerHTML = `
      <div class="device-icon">${domainIcon(domain)}</div>
      <div>
        <strong>${deviceLabel(device)}</strong>
        <div class="muted">${device.entity_id}</div>
      </div>
    `;

    const actions = document.createElement("div");
    actions.className = "device-actions";

    if (domain === "light" || domain === "switch") {
      const onBtn = document.createElement("button");
      onBtn.className = "btn btn-ghost";
      onBtn.textContent = "On";
      onBtn.onclick = () => handleDeviceAction(device.entity_id, "on", onBtn);
      actions.appendChild(onBtn);

      const offBtn = document.createElement("button");
      offBtn.className = "btn btn-ghost";
      offBtn.textContent = "Off";
      offBtn.onclick = () => handleDeviceAction(device.entity_id, "off", offBtn);
      actions.appendChild(offBtn);
    }

    if (domain === "light") {
      const colorInput = document.createElement("input");
      colorInput.type = "color";
      colorInput.className = "color-input";

      const colorBtn = document.createElement("button");
      colorBtn.className = "btn btn-ghost";
      colorBtn.textContent = "Set color";
      colorBtn.onclick = () => handleDeviceColor(device.entity_id, colorInput.value, colorBtn);

      actions.appendChild(colorInput);
      actions.appendChild(colorBtn);
    }

    leaf.appendChild(main);
    leaf.appendChild(actions);
    list.appendChild(leaf);
  });

  aggNode.appendChild(list);
  bridgeNode.appendChild(aggNode);
  root.appendChild(bridgeNode);
  deviceList.appendChild(root);
};

const renderDetailsView = () => {
  el("viewList").classList.add("hidden");
  el("viewCreate").classList.add("hidden");
  el("viewDetails").classList.remove("hidden");
  el("viewEdit").classList.add("hidden");

  const bridge = state.currentBridge;
  if (!bridge) return;

  const opsMap = latestOpByBridge();
  const op = opsMap.get(bridge.id);
  const runtime = runtimeBadge(bridge.id);
  const badgeMeta = runtime || badgeForStatus(op);

  const detailsCard = el("detailsCard");
  detailsCard.innerHTML = `
    <div class="bridge-header">
      <div class="bridge-title"><strong>${bridge.name}</strong><span class="muted">Port ${bridge.port}</span></div>
      <span class="badge ${badgeMeta.tone}">${badgeMeta.label}</span>
    </div>
    <div class="bridge-meta">
      <span>ID: ${bridge.id}</span>
      <span>Updated: ${formatDate(bridge.updated_at)}</span>
      ${runtime ? `<span>State: ${runtime.label}</span>` : ""}
    </div>
    <div class="bridge-actions">
      <button data-action="start" class="btn btn-ghost">Start</button>
      <button data-action="stop" class="btn btn-ghost">Stop</button>
      <button data-action="refresh" class="btn btn-ghost">Refresh</button>
      <button data-action="factory-reset" class="btn btn-ghost">Factory reset</button>
      <button data-action="delete" class="btn btn-danger">Delete</button>
    </div>
  `;

  detailsCard.querySelectorAll("button[data-action]").forEach((button) => {
    button.onclick = () => handleBridgeAction(bridge.id, button.dataset.action, button);
  });

  const configSummary = el("configSummary");
  const includeHtml = formatFilterList(bridge.filter?.include);
  const excludeHtml = formatFilterList(bridge.filter?.exclude);
  const coverFlag = bridge.feature_flags?.cover_do_not_invert_percentage ? "Enabled" : "Disabled";
  configSummary.innerHTML = `
    <div class="config-row">
      <div class="config-label">Include</div>
      <div class="config-value">${includeHtml}</div>
    </div>
    <div class="config-row">
      <div class="config-label">Exclude</div>
      <div class="config-value">${excludeHtml}</div>
    </div>
    <div class="config-row">
      <div class="config-label">Cover invert</div>
      <div class="config-value">${coverFlag}</div>
    </div>
  `;

  renderDeviceTree();

  const pairingEmpty = el("pairingEmpty");
  const pairingCard = el("pairingCard");
  if (!state.pairing) {
    pairingEmpty.style.display = "block";
    pairingCard.classList.add("hidden");
  } else {
    pairingEmpty.style.display = "none";
    pairingCard.classList.remove("hidden");
    el("pairingQrText").textContent = state.pairing.qr_text;
    el("pairingManual").textContent = state.pairing.manual_code;
    el("pairingQr").textContent = state.pairing.qr_unicode;
  }
};

const renderCreateView = () => {
  el("viewList").classList.add("hidden");
  el("viewCreate").classList.remove("hidden");
  el("viewDetails").classList.add("hidden");
  el("viewEdit").classList.add("hidden");
  const portInput = el("bridgePort");
  if (portInput && (!portInput.value || Number(portInput.value) === 5540)) {
    portInput.value = nextAvailablePort();
  }
};

const renderEditView = () => {
  el("viewList").classList.add("hidden");
  el("viewCreate").classList.add("hidden");
  el("viewDetails").classList.add("hidden");
  el("viewEdit").classList.remove("hidden");

  const bridge = state.currentBridge;
  if (!bridge) return;

  el("editName").value = bridge.name;
  el("editPort").value = bridge.port;
  el("editCoverInvert").checked = bridge.feature_flags?.cover_do_not_invert_percentage || false;
  state.editInclude = bridge.filter?.include ? [...bridge.filter.include] : [];
  state.editExclude = bridge.filter?.exclude ? [...bridge.filter.exclude] : [];
  renderEditFilters();
};

const renderOps = () => {
  const table = el("opsTable").querySelector("tbody");
  const empty = el("opsEmpty");
  table.innerHTML = "";
  if (!state.operations.length) {
    empty.style.display = "block";
    return;
  }
  empty.style.display = "none";

  state.operations
    .slice()
    .sort((a, b) => new Date(b.queued_at || 0) - new Date(a.queued_at || 0))
    .slice(0, 50)
    .forEach((op) => {
      const row = document.createElement("tr");
      row.innerHTML = `
        <td><strong>${op.bridge_id}</strong></td>
        <td>${op.op_type}</td>
        <td>${op.status}</td>
        <td>${formatDate(op.queued_at)}</td>
        <td><small>${op.error || "-"}</small></td>
      `;
      table.appendChild(row);
    });
};

const setOpsVisibility = (visible) => {
  state.opsVisible = visible;
  const panel = el("viewOps");
  const body = el("opsBody");
  const toggle = el("toggleOps");
  if (visible) {
    panel.classList.remove("collapsed");
    body.classList.remove("hidden");
    toggle.textContent = "Hide activity";
  } else {
    panel.classList.add("collapsed");
    body.classList.add("hidden");
    toggle.textContent = "Show activity";
  }
  localStorage.setItem("hamh_ops_visible", visible ? "1" : "0");
};

const refreshList = async () => {
  await Promise.all([loadBridges(), loadOperations(), loadRuntime()]);
  const bridgesJson = JSON.stringify(state.bridges);
  const opsJson = JSON.stringify(state.operations);
  const runtimeJson = JSON.stringify(state.runtime);
  if (
    bridgesJson !== state.lastBridgesJson ||
    opsJson !== state.lastOpsJson ||
    runtimeJson !== state.lastRuntimeJson
  ) {
    state.lastBridgesJson = bridgesJson;
    state.lastOpsJson = opsJson;
    state.lastRuntimeJson = runtimeJson;
    renderListView();
    renderOps();
  }
};

const refreshDetails = async (id) => {
  await Promise.all([
    loadBridge(id),
    loadDevices(id),
    loadPairing(id),
    loadOperations(),
    loadRuntime(),
  ]);
  renderDetailsView();
  renderOps();
};

const refreshAll = async () => {
  try {
    if (state.view === "list") {
      await refreshList();
    } else if (state.view === "detail" && state.currentBridge) {
      await refreshDetails(state.currentBridge.id);
    }
    setStatus(true, "Backend online");
  } catch (err) {
    setStatus(false, "Backend unavailable");
    showToast(`Failed to refresh: ${err.message}`, "bad");
  }
};

const handleBridgeAction = async (id, action, button) => {
  if (action === "delete") {
    if (!confirm("Delete this bridge?")) return;
  }
  button.disabled = true;
  try {
    if (action === "delete") {
      await fetchJson(`${apiBase}/bridges/${id}`, { method: "DELETE" });
    } else {
      await fetchJson(`${apiBase}/bridges/${id}/actions/${action}`, { method: "POST" });
    }
    showToast(`${action} queued`);
    await refreshDetails(id);
  } catch (err) {
    showToast(`Failed: ${err.message}`, "bad");
  } finally {
    button.disabled = false;
  }
};

const handleDeviceAction = async (entityId, action, button) => {
  if (!state.currentBridge) return;
  button.disabled = true;
  try {
    await fetchJson(
      `${apiBase}/bridges/${state.currentBridge.id}/devices/${encodeURIComponent(entityId)}/actions/${action}`,
      { method: "POST" }
    );
    showToast(`${entityId} ${action}`);
  } catch (err) {
    showToast(`Failed: ${err.message}`, "bad");
  } finally {
    button.disabled = false;
  }
};

const handleDeviceColor = async (entityId, hex, button) => {
  if (!state.currentBridge) return;
  const rgb = [
    parseInt(hex.slice(1, 3), 16),
    parseInt(hex.slice(3, 5), 16),
    parseInt(hex.slice(5, 7), 16),
  ];
  button.disabled = true;
  try {
    await fetchJson(
      `${apiBase}/bridges/${state.currentBridge.id}/devices/${encodeURIComponent(entityId)}/actions/color`,
      {
        method: "POST",
        body: JSON.stringify({ rgb }),
      }
    );
    showToast("Color updated");
  } catch (err) {
    showToast(`Failed: ${err.message}`, "bad");
  } finally {
    button.disabled = false;
  }
};

const addFilter = (kind) => {
  const type = el(`${kind}Type`).value;
  const value = el(`${kind}Value`).value.trim();
  if (!value) return;
  state[kind].push({ type, value });
  el(`${kind}Value`).value = "";
  renderFilters();
};

const renderFilters = () => {
  setPills(el("includeList"), state.include, (index) => {
    state.include.splice(index, 1);
    renderFilters();
  });
  setPills(el("excludeList"), state.exclude, (index) => {
    state.exclude.splice(index, 1);
    renderFilters();
  });
};

const addEditFilter = (kind) => {
  const type = el(`edit${kind}Type`).value;
  const value = el(`edit${kind}Value`).value.trim();
  if (!value) return;
  const target = kind === "Include" ? state.editInclude : state.editExclude;
  target.push({ type, value });
  el(`edit${kind}Value`).value = "";
  renderEditFilters();
};

const renderEditFilters = () => {
  const includeContainer = el("editIncludeList");
  const excludeContainer = el("editExcludeList");
  includeContainer.innerHTML = "";
  excludeContainer.innerHTML = "";

  const renderRows = (container, list, onRemove, onUpdate) => {
    if (!list.length) {
      const empty = document.createElement("div");
      empty.className = "filter-empty";
      empty.textContent = "No filters configured.";
      container.appendChild(empty);
      return;
    }

    list.forEach((item, index) => {
      const row = document.createElement("div");
      row.className = "filter-edit-row";

      const select = document.createElement("select");
      filterTypes.forEach((type) => {
        const option = document.createElement("option");
        option.value = type;
        option.textContent = type;
        if (type === item.type) option.selected = true;
        select.appendChild(option);
      });
      select.onchange = () => onUpdate(index, { ...item, type: select.value });

      const input = document.createElement("input");
      input.type = "text";
      input.value = item.value || "";
      input.placeholder = "value";
      input.oninput = () => onUpdate(index, { ...item, value: input.value.trim() });

      const remove = document.createElement("button");
      remove.type = "button";
      remove.className = "btn btn-ghost";
      remove.textContent = "Remove";
      remove.onclick = () => onRemove(index);

      row.appendChild(select);
      row.appendChild(input);
      row.appendChild(remove);
      container.appendChild(row);
    });
  };

  renderRows(
    includeContainer,
    state.editInclude,
    (index) => {
      state.editInclude.splice(index, 1);
      renderEditFilters();
    },
    (index, next) => {
      state.editInclude[index] = next;
    }
  );

  renderRows(
    excludeContainer,
    state.editExclude,
    (index) => {
      state.editExclude.splice(index, 1);
      renderEditFilters();
    },
    (index, next) => {
      state.editExclude[index] = next;
    }
  );
};

const navigateTo = (hash) => {
  if (window.location.hash !== hash) {
    window.location.hash = hash;
  } else {
    onRouteChange();
  }
};

const onRouteChange = async () => {
  const hash = window.location.hash || "#bridges";
  if (hash.startsWith("#create")) {
    state.view = "create";
    await refreshList();
    renderCreateView();
    return;
  }
  if (hash.startsWith("#bridge/")) {
    const parts = hash.replace("#bridge/", "").split("/");
    const id = parts[0];
    if (parts[1] === "edit") {
      state.view = "edit";
      await loadBridge(id);
      renderEditView();
      return;
    }
    state.view = "detail";
    await refreshDetails(id);
    return;
  }
  state.view = "list";
  await refreshList();
};

const bindEvents = () => {
  el("refreshAll").onclick = refreshAll;
  el("toggleOps").onclick = () => setOpsVisibility(!state.opsVisible);
  el("goCreate").onclick = () => navigateTo("#create");
  el("backToListFromCreate").onclick = () => navigateTo("#bridges");
  el("backToListFromDetails").onclick = () => navigateTo("#bridges");
  el("editBridge").onclick = () => {
    if (state.currentBridge) navigateTo(`#bridge/${state.currentBridge.id}/edit`);
  };
  el("backToDetails").onclick = () => {
    if (state.currentBridge) navigateTo(`#bridge/${state.currentBridge.id}`);
  };
  el("refreshDevices").onclick = async () => {
    if (!state.currentBridge) return;
    await refreshDetails(state.currentBridge.id);
    showToast("Devices refreshed");
  };
  el("copyQrText").onclick = async () => {
    if (!state.pairing) return;
    await navigator.clipboard.writeText(state.pairing.qr_text);
    showToast("QR text copied");
  };
  el("copyManual").onclick = async () => {
    if (!state.pairing) return;
    await navigator.clipboard.writeText(state.pairing.manual_code);
    showToast("Manual code copied");
  };

  el("addInclude").onclick = () => addFilter("include");
  el("addExclude").onclick = () => addFilter("exclude");
  el("clearFilters").onclick = () => {
    state.include = [];
    state.exclude = [];
    renderFilters();
    showToast("Filters cleared");
  };

  el("editAddInclude").onclick = () => addEditFilter("Include");
  el("editAddExclude").onclick = () => addEditFilter("Exclude");

  el("createForm").onsubmit = async (event) => {
    event.preventDefault();
    const name = el("bridgeName").value.trim();
    const port = Number(el("bridgePort").value);
    const coverFlag = el("coverInvert").checked;
    if (!name) {
      showToast("Name is required", "bad");
      return;
    }

    const payload = {
      name,
      port,
      filter: {
        include: state.include,
        exclude: state.exclude,
      },
      feature_flags: {
        cover_do_not_invert_percentage: coverFlag,
      },
    };

    try {
      const created = await fetchJson(`${apiBase}/bridges`, {
        method: "POST",
        body: JSON.stringify(payload),
      });
      if (created && created.id) {
        await fetchJson(`${apiBase}/bridges/${created.id}/actions/refresh`, {
          method: "POST",
        });
      }
      showToast("Bridge created and refresh queued");
      state.include = [];
      state.exclude = [];
      renderFilters();
      el("createForm").reset();
      if (created && created.id) {
        navigateTo(`#bridge/${created.id}`);
      } else {
        navigateTo("#bridges");
      }
    } catch (err) {
      showToast(`Create failed: ${err.message}`, "bad");
    }
  };

  el("editForm").onsubmit = async (event) => {
    event.preventDefault();
    const bridge = state.currentBridge;
    if (!bridge) return;

    const payload = {
      name: el("editName").value.trim(),
      port: Number(el("editPort").value),
      filter: {
        include: state.editInclude,
        exclude: state.editExclude,
      },
      feature_flags: {
        cover_do_not_invert_percentage: el("editCoverInvert").checked,
      },
    };

    try {
      await fetchJson(`${apiBase}/bridges/${bridge.id}`, {
        method: "PUT",
        body: JSON.stringify(payload),
      });
      showToast("Bridge updated");
      navigateTo(`#bridge/${bridge.id}`);
    } catch (err) {
      showToast(`Update failed: ${err.message}`, "bad");
    }
  };

  window.addEventListener("hashchange", onRouteChange);
};

bindEvents();
renderFilters();
renderEditFilters();
setOpsVisibility(localStorage.getItem("hamh_ops_visible") === "1");
onRouteChange();

setInterval(() => {
  if (document.hidden) return;
  refreshAll();
}, 15000);
