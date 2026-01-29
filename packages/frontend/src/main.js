import "./styles.css";
import { renderApp } from "./app.js";

const root = document.querySelector("#root");
if (!root) {
  throw new Error("Root element not found");
}

const appRoot = document.createElement("div");
appRoot.className = "app";
root.append(appRoot);

const render = () => {
  renderApp(appRoot).catch((error) => {
    console.error("Failed to render", error);
    appRoot.innerHTML = "";
    const errorEl = document.createElement("div");
    errorEl.className = "state-error";
    errorEl.textContent = "Something went wrong while rendering the app.";
    appRoot.append(errorEl);
  });
};

window.addEventListener("hashchange", render);
render();
