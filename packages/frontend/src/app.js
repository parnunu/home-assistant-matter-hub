import { getRoute } from "./router.js";
import { renderBridgeDetails } from "./views/bridge-details.js";
import { renderBridgeForm } from "./views/bridge-form.js";
import { renderBridgeList } from "./views/bridge-list.js";

const NAV_LINKS = [
  { href: "#/bridges", label: "Bridges" },
  {
    href: "https://t0bst4r.github.io/home-assistant-matter-hub",
    label: "Docs",
    external: true,
  },
  {
    href: "https://github.com/t0bst4r/home-assistant-matter-hub",
    label: "GitHub",
    external: true,
  },
];

export async function renderApp(root) {
  root.innerHTML = "";
  const route = getRoute();
  root.append(buildHeader(route));
  const main = document.createElement("main");
  main.className = "main";
  const content = await renderRoute(route);
  main.append(content);
  root.append(main);
}

function buildHeader(route) {
  const header = document.createElement("header");
  header.className = "header";
  const brand = document.createElement("div");
  brand.className = "brand";
  brand.innerHTML = `
    <img src="/hamh-logo.svg" alt="Home Assistant Matter Hub" />
    <div>
      <span class="brand-title">Matter Hub</span>
      <span class="brand-subtitle">Lightweight management console</span>
    </div>
  `;

  const nav = document.createElement("nav");
  nav.className = "nav";
  for (const link of NAV_LINKS) {
    const anchor = document.createElement("a");
    anchor.textContent = link.label;
    anchor.href = link.href;
    if (link.external) {
      anchor.target = "_blank";
      anchor.rel = "noreferrer";
    }
    if (!link.external && route.section === "bridges") {
      anchor.classList.add("active");
    }
    nav.append(anchor);
  }

  header.append(brand, nav);
  return header;
}

async function renderRoute(route) {
  switch (route.view) {
    case "bridge-list":
      return renderBridgeList();
    case "bridge-details":
      return renderBridgeDetails(route.bridgeId);
    case "bridge-create":
      return renderBridgeForm("create");
    case "bridge-edit":
      return renderBridgeForm("edit", route.bridgeId);
    default: {
      const container = document.createElement("div");
      container.className = "state-error";
      container.textContent = "Page not found.";
      return container;
    }
  }
}
