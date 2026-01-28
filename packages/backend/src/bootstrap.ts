import { config } from "@matter/nodejs/config";
import { appendDebugLog } from "./utils/logging/file-log.js";

config.trapProcessSignals = true;
config.setProcessExitCodeOnError = true;
config.loadConfigFile = false;
config.loadProcessArgv = false;
config.loadProcessEnv = false;

process.on("uncaughtException", (err) => {
  appendDebugLog("backend-crash.log", [
    `[uncaughtException] ${err?.stack ?? String(err)}`,
  ]);
});

process.on("unhandledRejection", (reason) => {
  appendDebugLog("backend-crash.log", [
    `[unhandledRejection] ${String(reason)}`,
  ]);
});

process.on("warning", (warning) => {
  appendDebugLog("backend-crash.log", [
    `[warning] ${warning?.stack ?? String(warning)}`,
  ]);
});
