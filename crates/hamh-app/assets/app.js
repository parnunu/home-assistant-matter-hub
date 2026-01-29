const apiBase = "/api/matter";

const state = {
  include: [],
  exclude: [],
  bridges: [],
  operations: [],
};

const el = (id) => document.getElementById(id);

const formatDate = (value) => {
  if (!value) return "-";
  const date = new Date(value);
  return Number.isNaN(date.getTime()) ? value : date.toLocaleString();
};

const setPills = (container, list, onRemove) => {
  container.innerHTML = "";
  list.forEach((item, index) => {
    const pill = document.createElement("div");
    pill.className = "pill";
    pill.textContent = `${item.type}:${item.value}`;
    const btn = document.createElement("button");
    btn.textContent = "×";
    btn.onclick = () => onRemove(index);
    pill.appendChild(btn);
    container.appendChild(pill);
  });
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
  return res.json();
};

const loadBridges = async () => {
  const bridges = await fetchJson(`${apiBase}/bridges`);
  state.bridges = bridges;
};

const loadOperations = async () => {
  const ops = await fetchJson(`${apiBase}/operations`);
  state.operations = ops;
};

const latestOpByBridge = () => {
  const map = new Map();
  state.operations.forEach((op) => {
    if (!map.has(op.bridge_id)) {
      map.set(op.bridge_id, op);
    }
  });
  return map;
};

const renderBridges = () => {
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
    header.innerHTML = `<div><strong>${bridge.name}</strong><div class="muted">Port ${bridge.port}</div></div>`;

    const op = opsMap.get(bridge.id);
    const badge = document.createElement("span");
    badge.className = "badge";
    badge.textContent = op ? `${op.op_type} · ${op.status}` : "idle";
    header.appendChild(badge);
    card.appendChild(header);

    const details = document.createElement("div");
    details.className = "muted";
    details.textContent = `Updated: ${formatDate(bridge.updated_at)}`;
    card.appendChild(details);

    const actions = document.createElement("div");
    actions.className = "bridge-actions";

    const actionBtn = (label, action) => {
      const button = document.createElement("button");
      button.className = "btn ghost";
      button.textContent = label;
      button.onclick = async () => {
        await fetchJson(`${apiBase}/bridges/${bridge.id}/actions/${action}`, {
          method: "POST",
        });
        await refreshAll();
      };
      return button;
    };

    actions.appendChild(actionBtn("Start", "start"));
    actions.appendChild(actionBtn("Stop", "stop"));
    actions.appendChild(actionBtn("Refresh", "refresh"));
    actions.appendChild(actionBtn("Factory Reset", "factory-reset"));

    const deleteBtn = document.createElement("button");
    deleteBtn.className = "btn";
    deleteBtn.textContent = "Delete";
    deleteBtn.onclick = async () => {
      await fetchJson(`${apiBase}/bridges/${bridge.id}`, { method: "DELETE" });
      await refreshAll();
    };
    actions.appendChild(deleteBtn);

    const devicesBtn = document.createElement("button");
    devicesBtn.className = "btn ghost";
    devicesBtn.textContent = "Devices";
    devicesBtn.onclick = async () => {
      const devices = await fetchJson(`${apiBase}/bridges/${bridge.id}/devices`);
      const lines = devices.map((d) => `${d.entity_id} → ${d.device_type}`);
      alert(lines.length ? lines.join("\n") : "No devices mapped yet.");
    };
    actions.appendChild(devicesBtn);

    card.appendChild(actions);

    list.appendChild(card);
  });
};

const renderOperations = () => {
  const table = el("opsTable").querySelector("tbody");
  const empty = el("opsEmpty");
  table.innerHTML = "";
  if (!state.operations.length) {
    empty.style.display = "block";
    return;
  }
  empty.style.display = "none";
  state.operations.forEach((op) => {
    const row = document.createElement("tr");
    row.innerHTML = `
      <td>${op.bridge_id}</td>
      <td>${op.op_type}</td>
      <td>${op.status}</td>
      <td>${formatDate(op.queued_at)}</td>
      <td>${op.error || "-"}</td>
    `;
    table.appendChild(row);
  });
};

const refreshAll = async () => {
  await Promise.all([loadBridges(), loadOperations()]);
  renderBridges();
  renderOperations();
};

const bindEvents = () => {
  el("addInclude").onclick = () => addFilter("include");
  el("addExclude").onclick = () => addFilter("exclude");
  el("refreshAll").onclick = refreshAll;

  el("createForm").onsubmit = async (event) => {
    event.preventDefault();
    const name = el("bridgeName").value.trim();
    const port = Number(el("bridgePort").value);
    const coverFlag = el("coverInvert").checked;
    if (!name) return;

    const payload = {
      id: "00000000-0000-0000-0000-000000000000",
      name,
      port,
      filter: {
        include: state.include,
        exclude: state.exclude,
      },
      feature_flags: {
        cover_do_not_invert_percentage: coverFlag,
      },
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    };

    await fetchJson(`${apiBase}/bridges`, {
      method: "POST",
      body: JSON.stringify(payload),
    });

    state.include = [];
    state.exclude = [];
    renderFilters();
    el("createForm").reset();
    await refreshAll();
  };
};

bindEvents();
renderFilters();
refreshAll();
setInterval(refreshAll, 8000);
