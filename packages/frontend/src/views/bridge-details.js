import { getBridge, getBridgeDevices } from "../api.js";
import { button, el } from "../dom.js";

export async function renderBridgeDetails(bridgeId) {
  const container = el("section", { className: "page" });
  const header = el("div", { className: "page-header" });
  const backButton = button("Back to bridges", "ghost");
  backButton.addEventListener("click", () => {
    window.location.hash = "#/bridges";
  });
  header.append(
    el("div", {
      children: [
        el("p", { className: "eyebrow", text: "Bridge profile" }),
        el("h1", { text: "Bridge details" }),
        el("p", {
          className: "muted",
          text: "Review commissioning, filters, and endpoint data.",
        }),
      ],
    }),
    backButton,
  );
  container.append(header);

  const state = el("div", { className: "state", text: "Loading..." });
  container.append(state);

  try {
    const bridge = await getBridge(bridgeId);
    const devices = await getBridgeDevices(bridgeId);
    state.replaceWith(renderBridgeDetailsCard(bridge, devices));
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    state.className = "state-error";
    state.textContent = `Failed to load bridge: ${message}`;
  }

  return container;
}

function renderBridgeDetailsCard(bridge, devices) {
  const card = el("article", { className: "card detail-card" });
  const title = el("div", { className: "card-title" });
  title.append(
    el("span", { text: bridge.name }),
    el("span", {
      className: `status status-${bridge.status}`,
      text: bridge.status,
    }),
  );

  const meta = el("div", { className: "detail-grid" });
  meta.append(
    renderDetail("Bridge ID", bridge.id),
    renderDetail("Port", String(bridge.port)),
    renderDetail("Devices", String(bridge.deviceCount ?? 0)),
    renderDetail("Vendor", bridge.basicInformation.vendorName),
    renderDetail("Product", bridge.basicInformation.productLabel),
    renderDetail("Country", bridge.countryCode || "Not set"),
  );

  const commissioning = renderSection("Commissioning", [
    bridge.commissioning
      ? el("div", {
          className: "detail-grid",
          children: [
            renderDetail(
              "Commissioned",
              bridge.commissioning.isCommissioned ? "Yes" : "No",
            ),
            renderDetail("Passcode", String(bridge.commissioning.passcode)),
            renderDetail(
              "Manual code",
              bridge.commissioning.manualPairingCode,
            ),
            renderDetail("QR code", bridge.commissioning.qrPairingCode),
            renderDetail(
              "Fabrics",
              String(bridge.commissioning.fabrics.length),
            ),
          ],
        })
      : el("p", {
          className: "muted",
          text: "Bridge has not been commissioned yet.",
        }),
  ]);

  const filterSection = renderSection("Filters", [
    renderFilterList("Include", bridge.filter.include),
    renderFilterList("Exclude", bridge.filter.exclude),
  ]);

  const devicesSection = renderSection("Devices", [
    el("pre", { className: "code-block", text: JSON.stringify(devices, null, 2) }),
  ]);

  card.append(title, meta, commissioning, filterSection, devicesSection);
  return card;
}

function renderSection(title, content) {
  const section = el("div", { className: "detail-section" });
  section.append(el("h2", { text: title }), ...content);
  return section;
}

function renderFilterList(title, filters) {
  const wrapper = el("div", { className: "filter-preview" });
  wrapper.append(el("h3", { text: title }));
  if (!filters.length) {
    wrapper.append(el("p", { className: "muted", text: "None" }));
    return wrapper;
  }
  const list = el("ul", { className: "pill-list" });
  for (const filter of filters) {
    list.append(
      el("li", {
        className: "pill",
        text: `${filter.type}: ${filter.value}`,
      }),
    );
  }
  wrapper.append(list);
  return wrapper;
}

function renderDetail(label, value) {
  const row = el("div", { className: "detail-row" });
  row.append(el("span", { className: "muted", text: label }), el("span", { text: value }));
  return row;
}
