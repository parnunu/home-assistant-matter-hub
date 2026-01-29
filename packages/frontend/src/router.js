export function getRoute() {
  const rawHash = window.location.hash.replace(/^#/, "");
  const path = rawHash || "/bridges";
  const segments = path.split("/").filter(Boolean);

  if (segments[0] !== "bridges") {
    return { section: "unknown", view: "not-found" };
  }

  if (segments.length === 1) {
    return { section: "bridges", view: "bridge-list" };
  }

  if (segments[1] === "new") {
    return { section: "bridges", view: "bridge-create" };
  }

  if (segments.length === 2) {
    return { section: "bridges", view: "bridge-details", bridgeId: segments[1] };
  }

  if (segments.length === 3 && segments[2] === "edit") {
    return { section: "bridges", view: "bridge-edit", bridgeId: segments[1] };
  }

  return { section: "unknown", view: "not-found" };
}
