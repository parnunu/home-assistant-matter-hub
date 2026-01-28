import fs from "node:fs";
import os from "node:os";
import path from "node:path";

function resolveStorageLocation() {
  const homedir = os.homedir();
  const storageLocation = process.env.HAMH_STORAGE_LOCATION;
  return storageLocation
    ? path.resolve(storageLocation.replace(/^~\//, `${homedir}/`))
    : path.join(homedir, ".home-assistant-matter-hub");
}

export function appendDebugLog(fileName: string, lines: string[]) {
  try {
    const base = resolveStorageLocation();
    const dir = path.join(base, "logs");
    fs.mkdirSync(dir, { recursive: true });
    const filePath = path.join(dir, fileName);
    const payload =
      lines.map((line) => `${new Date().toISOString()} ${line}`).join("\n") +
      "\n";
    fs.appendFileSync(filePath, payload, "utf8");
  } catch {
    // Best-effort logging only; avoid crashing the app.
  }
}
