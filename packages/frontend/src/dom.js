export function el(tag, options = {}) {
  const element = document.createElement(tag);
  if (options.className) {
    element.className = options.className;
  }
  if (options.text) {
    element.textContent = options.text;
  }
  if (options.attrs) {
    for (const [key, value] of Object.entries(options.attrs)) {
      element.setAttribute(key, value);
    }
  }
  if (options.children) {
    for (const child of options.children) {
      if (child == null) {
        continue;
      }
      element.append(child);
    }
  }
  return element;
}

export function button(label, className) {
  const element = el("button", { text: label });
  if (className) {
    element.className = className;
  }
  element.type = "button";
  return element;
}

export function fieldGroup(labelText, input) {
  const label = el("label", { className: "field" });
  const span = el("span", { className: "field-label", text: labelText });
  label.append(span, input);
  return label;
}
