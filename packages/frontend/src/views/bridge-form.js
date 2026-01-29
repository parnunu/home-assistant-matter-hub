import { createBridge, getBridge, updateBridge } from "../api.js";
import { button, el, fieldGroup } from "../dom.js";

const MATCHER_TYPES = [
  "pattern",
  "domain",
  "platform",
  "label",
  "area",
  "entity_category",
];

export async function renderBridgeForm(mode, bridgeId) {
  const container = el("section", { className: "page" });
  const header = el("div", { className: "page-header" });
  const backButton = button("Back to bridges", "ghost");
  backButton.addEventListener("click", () => {
    window.location.hash = "#/bridges";
  });
  header.append(
    el("div", {
      children: [
        el("p", { className: "eyebrow", text: "Configuration" }),
        el("h1", { text: mode === "create" ? "Create bridge" : "Edit bridge" }),
        el("p", {
          className: "muted",
          text:
            "Define which Home Assistant entities the bridge should expose to Matter controllers.",
        }),
      ],
    }),
    backButton,
  );
  container.append(header);

  const card = el("article", { className: "card" });
  const form = el("form", { className: "form" });
  card.append(form);
  container.append(card);

  const nameInput = el("input", { attrs: { type: "text", required: "true" } });
  const portInput = el("input", {
    attrs: { type: "number", min: "1", required: "true" },
  });
  const countryInput = el("input", {
    attrs: { type: "text", placeholder: "US, DE, JP..." },
  });

  const includeList = el("div", { className: "filter-list" });
  const excludeList = el("div", { className: "filter-list" });

  const includeAdd = button("Add include rule", "ghost");
  includeAdd.addEventListener("click", () =>
    includeList.append(createFilterRow()),
  );
  const excludeAdd = button("Add exclude rule", "ghost");
  excludeAdd.addEventListener("click", () =>
    excludeList.append(createFilterRow()),
  );

  const includeSection = buildFilterSection(
    "Include rules",
    "Only entities matching these rules will be shared.",
    includeList,
    includeAdd,
  );
  const excludeSection = buildFilterSection(
    "Exclude rules",
    "Entities matching these rules will be removed from the bridge.",
    excludeList,
    excludeAdd,
  );

  const coverFlag = el("input", { attrs: { type: "checkbox" } });
  const hiddenFlag = el("input", { attrs: { type: "checkbox" } });

  form.append(
    fieldGroup("Bridge name", nameInput),
    fieldGroup("Port", portInput),
    fieldGroup("Country code (optional)", countryInput),
    includeSection,
    excludeSection,
    buildFlagSection(coverFlag, hiddenFlag),
  );

  const actions = el("div", { className: "form-actions" });
  const submitButton = el("button", {
    text: mode === "create" ? "Create bridge" : "Save changes",
    attrs: { type: "submit" },
  });
  submitButton.className = "primary";
  actions.append(submitButton);
  form.append(actions);

  if (mode === "edit" && bridgeId) {
    const bridge = await getBridge(bridgeId);
    hydrateForm(bridge, {
      nameInput,
      portInput,
      countryInput,
      includeList,
      excludeList,
      coverFlag,
      hiddenFlag,
    });
  } else {
    includeList.append(createFilterRow());
  }

  form.addEventListener("submit", async (event) => {
    event.preventDefault();
    submitButton.setAttribute("disabled", "true");
    try {
      const payload = buildPayload({
        id: bridgeId,
        nameInput,
        portInput,
        countryInput,
        includeList,
        excludeList,
        coverFlag,
        hiddenFlag,
      });
      if (mode === "create") {
        await createBridge(payload);
      } else {
        await updateBridge(payload);
      }
      window.location.hash = "#/bridges";
    } catch (error) {
      alert(`Failed to save bridge: ${String(error)}`);
    } finally {
      submitButton.removeAttribute("disabled");
    }
  });

  return container;
}

function buildFilterSection(title, description, list, addButton) {
  const section = el("div", { className: "filter-section" });
  section.append(
    el("div", {
      className: "section-heading",
      children: [
        el("h2", { text: title }),
        el("p", { className: "muted", text: description }),
      ],
    }),
    list,
    addButton,
  );
  return section;
}

function buildFlagSection(coverFlag, hiddenFlag) {
  const section = el("div", { className: "filter-section" });
  section.append(
    el("div", {
      className: "section-heading",
      children: [
        el("h2", { text: "Feature flags" }),
        el("p", {
          className: "muted",
          text: "Fine-tune how the bridge maps entities to Matter devices.",
        }),
      ],
    }),
  );
  section.append(
    renderCheckbox(
      coverFlag,
      "Do not invert cover percentages",
      "Match Home Assistant cover percentages (non-Matter compliant).",
    ),
    renderCheckbox(
      hiddenFlag,
      "Include hidden entities",
      "Expose entities that are hidden in Home Assistant.",
    ),
  );
  return section;
}

function renderCheckbox(input, title, description) {
  const wrapper = el("label", { className: "checkbox" });
  const text = el("div", { className: "checkbox-text" });
  text.append(
    el("span", { text: title }),
    el("small", { className: "muted", text: description }),
  );
  wrapper.append(input, text);
  return wrapper;
}

function createFilterRow(matcher) {
  const row = el("div", { className: "filter-row" });
  row.dataset.row = "true";
  const select = el("select");
  for (const type of MATCHER_TYPES) {
    const option = el("option", { text: type });
    option.value = type;
    select.append(option);
  }
  const input = el("input", {
    attrs: { type: "text", placeholder: "Pattern, domain, label..." },
  });
  const remove = button("Remove", "ghost");
  remove.addEventListener("click", () => row.remove());

  if (matcher) {
    select.value = matcher.type;
    input.value = matcher.value;
  }

  row.append(select, input, remove);
  return row;
}

function hydrateForm(bridge, elements) {
  elements.nameInput.value = bridge.name;
  elements.portInput.value = String(bridge.port);
  elements.countryInput.value = bridge.countryCode ?? "";
  elements.coverFlag.checked = Boolean(
    bridge.featureFlags?.coverDoNotInvertPercentage,
  );
  elements.hiddenFlag.checked = Boolean(
    bridge.featureFlags?.includeHiddenEntities,
  );

  for (const matcher of bridge.filter.include) {
    elements.includeList.append(createFilterRow(matcher));
  }
  if (bridge.filter.include.length === 0) {
    elements.includeList.append(createFilterRow());
  }
  for (const matcher of bridge.filter.exclude) {
    elements.excludeList.append(createFilterRow(matcher));
  }
}

function buildPayload(elements) {
  const include = extractMatchers(elements.includeList);
  const exclude = extractMatchers(elements.excludeList);

  const payload = {
    name: elements.nameInput.value.trim(),
    port: Number(elements.portInput.value),
    filter: { include, exclude },
    countryCode: elements.countryInput.value.trim() || undefined,
    featureFlags: buildFeatureFlags(
      elements.coverFlag.checked,
      elements.hiddenFlag.checked,
    ),
  };

  if (elements.id) {
    return { ...payload, id: elements.id };
  }
  return payload;
}

function buildFeatureFlags(cover, hidden) {
  const flags = {};
  if (cover) {
    flags.coverDoNotInvertPercentage = true;
  }
  if (hidden) {
    flags.includeHiddenEntities = true;
  }
  return Object.keys(flags).length ? flags : undefined;
}

function extractMatchers(list) {
  const rows = list.querySelectorAll("[data-row]");
  const matchers = [];
  for (const row of Array.from(rows)) {
    const select = row.querySelector("select");
    const input = row.querySelector("input");
    if (!select || !input) {
      continue;
    }
    const value = input.value.trim();
    if (!value) {
      continue;
    }
    matchers.push({ type: select.value, value });
  }
  return matchers;
}
