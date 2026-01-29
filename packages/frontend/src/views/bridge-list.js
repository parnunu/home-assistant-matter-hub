import { deleteBridge, factoryReset, getBridges } from "../api.js";
import { button, el } from "../dom.js";

export async function renderBridgeList() {
  const container = el("section", { className: "page" });
  const header = el("div", { className: "page-header" });
  const titleBlock = el("div", {
    children: [
      el("p", { className: "eyebrow", text: "Control center" }),
      el("h1", { text: "Bridges" }),
      el("p", {
        className: "muted",
        text: "Manage Matter bridges, filters, and commissioning status in one place.",
      }),
    ],
  });

  const headerActions = el("div", { className: "header-actions" });
  const refreshButton = button("Refresh", "ghost");
  const createButton = button("Create bridge", "primary");
  refreshButton.addEventListener("click", () => refresh(grid, summary));
  createButton.addEventListener("click", () => {
    window.location.hash = "#/bridges/new";
  });
  headerActions.append(refreshButton, createButton);

  header.append(titleBlock, headerActions);
  container.append(header);

  const summary = el("div", { className: "summary" });
  const grid = el("div", { className: "card-grid" });
  const loading = el("div", { className: "state", text: "Loading bridges..." });
  grid.append(loading);

  container.append(summary, grid);

  await refresh(grid, summary);

  return container;
}

function renderBridgeCard(bridge, refresh) {
  const card = el("article", { className: "card" });
  const title = el("div", { className: "card-title" });
  const statusBadge = el("span", {
    className: `status status-${bridge.status}`,
    text: bridge.status,
  });
  title.append(el("span", { text: bridge.name }), statusBadge);

  const body = el("div", { className: "card-body" });
  body.append(
    renderMetaRow("Port", String(bridge.port)),
    renderMetaRow("Devices", String(bridge.deviceCount ?? 0)),
    renderMetaRow(
      "Commissioned",
      bridge.commissioning?.isCommissioned ? "Yes" : "No",
    ),
  );
  if (bridge.statusReason) {
    body.append(el("p", { className: "muted", text: bridge.statusReason }));
  }

  const actions = el("div", { className: "card-actions" });
  const viewButton = button("Details");
  viewButton.addEventListener("click", () => {
    window.location.hash = `#/bridges/${bridge.id}`;
  });

  const editButton = button("Edit");
  editButton.addEventListener("click", () => {
    window.location.hash = `#/bridges/${bridge.id}/edit`;
  });

  const resetButton = button("Factory reset", "ghost");
  resetButton.addEventListener("click", async () => {
    resetButton.setAttribute("disabled", "true");
    try {
      await factoryReset(bridge.id);
      refresh();
    } catch (error) {
      alert(`Failed to reset: ${String(error)}`);
    } finally {
      resetButton.removeAttribute("disabled");
    }
  });

  const deleteButton = button("Delete", "danger");
  deleteButton.addEventListener("click", async () => {
    if (!confirm(`Delete ${bridge.name}? This cannot be undone.`)) {
      return;
    }
    deleteButton.setAttribute("disabled", "true");
    try {
      await deleteBridge(bridge.id);
      refresh();
    } catch (error) {
      alert(`Failed to delete: ${String(error)}`);
    } finally {
      deleteButton.removeAttribute("disabled");
    }
  });

  actions.append(viewButton, editButton, resetButton, deleteButton);

  card.append(title, body, actions);
  return card;
}

function renderMetaRow(label, value) {
  const row = el("div", { className: "meta-row" });
  row.append(el("span", { className: "muted", text: label }), el("span", { text: value }));
  return row;
}

function renderSummary(bridges) {
  const running = bridges.filter((bridge) => bridge.status === "running").length;
  const commissioned = bridges.filter(
    (bridge) => bridge.commissioning?.isCommissioned,
  ).length;
  return el("div", {
    className: "summary-grid",
    children: [
      summaryCard("Total bridges", String(bridges.length)),
      summaryCard("Running", String(running)),
      summaryCard("Commissioned", String(commissioned)),
    ],
  });
}

function summaryCard(label, value) {
  const card = el("div", { className: "summary-card" });
  card.append(el("span", { className: "muted", text: label }), el("strong", { text: value }));
  return card;
}

async function refresh(grid, summary) {
  grid.innerHTML = "";
  summary.innerHTML = "";
  grid.append(el("div", { className: "state", text: "Refreshing..." }));
  try {
    const bridges = await getBridges();
    summary.append(renderSummary(bridges));
    grid.innerHTML = "";
    if (bridges.length === 0) {
      grid.append(
        el("div", {
          className: "state",
          text: "No bridges yet. Create one to start sharing devices.",
        }),
      );
      return;
    }
    for (const bridge of bridges) {
      grid.append(renderBridgeCard(bridge, () => refresh(grid, summary)));
    }
  } catch (error) {
    grid.innerHTML = "";
    const message = error instanceof Error ? error.message : String(error);
    grid.append(
      el("div", {
        className: "state-error",
        text: `Failed to load bridges: ${message}`,
      }),
    );
  }
}
