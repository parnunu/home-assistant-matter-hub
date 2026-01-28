import {
  BridgeStatus,
  type UpdateBridgeRequest,
} from "@home-assistant-matter-hub/common";
import type { Environment, Logger } from "@matter/general";
import { StorageService } from "@matter/main";
import fs from "node:fs";
import path from "node:path";
import type { LoggerService } from "../../core/app/logger.js";
import { BridgeServerNode } from "../../matter/endpoints/bridge-server-node.js";
import type {
  BridgeDataProvider,
  BridgeServerStatus,
} from "./bridge-data-provider.js";
import type { BridgeEndpointManager } from "./bridge-endpoint-manager.js";
import { appendDebugLog } from "../../utils/logging/file-log.js";

export class Bridge {
  private readonly env: Environment;
  private readonly log: Logger;
  readonly server: BridgeServerNode;

  private status: BridgeServerStatus = {
    code: BridgeStatus.Stopped,
    reason: undefined,
  };

  get id() {
    return this.dataProvider.id;
  }

  get data() {
    return this.dataProvider.withMetadata(
      this.status,
      this.server,
      this.aggregator.parts.size,
    );
  }

  get aggregator() {
    return this.endpointManager.root;
  }

  constructor(
    env: Environment,
    logger: LoggerService,
    private readonly dataProvider: BridgeDataProvider,
    private readonly endpointManager: BridgeEndpointManager,
  ) {
    this.env = env;
    this.log = logger.get(`Bridge / ${dataProvider.id}`);
    this.server = new BridgeServerNode(
      env,
      this.dataProvider,
      this.endpointManager.root,
    );
  }

  async initialize(): Promise<void> {
    await this.server.construction.ready.then();
    await this.refreshDevices();
  }
  async dispose(): Promise<void> {
    await this.stop();
  }

  async refreshDevices() {
    await this.endpointManager.refreshDevices();
  }

  async start() {
    if (this.status.code === BridgeStatus.Running) {
      return;
    }
    try {
      this.endpointManager.setStopping(false);
      this.status = {
        code: BridgeStatus.Starting,
        reason: "The bridge is starting... Please wait.",
      };
      await this.refreshDevices();
      this.endpointManager.startObserving();
      await this.server.start();
      this.status = { code: BridgeStatus.Running };
    } catch (e) {
      const reason = "Failed to start bridge due to error:";
      this.log.error(reason, e);
      await this.stop(BridgeStatus.Failed, `${reason}\n${e?.toString()}`);
    }
  }

  async stop(
    code: BridgeStatus = BridgeStatus.Stopped,
    reason = "Manually stopped",
  ) {
    if (
      this.status.code === BridgeStatus.Stopped ||
      this.status.code === BridgeStatus.Failed
    ) {
      return;
    }
    this.endpointManager.setStopping(true);
    await this.endpointManager.waitForIdle();
    this.endpointManager.stopObserving();
    await this.server.cancel();
    this.status = { code, reason };
  }

  async update(update: UpdateBridgeRequest) {
    try {
      this.endpointManager.setStopping(false);
      this.dataProvider.update(update);
      await this.refreshDevices();
    } catch (e) {
      const reason = "Failed to update bridge due to error:";
      this.log.error(reason, e);
      await this.stop(BridgeStatus.Failed, `${reason}\n${e?.toString()}`);
    }
  }

  async factoryReset() {
    if (this.status.code !== BridgeStatus.Running) {
      return;
    }
    await this.server.factoryReset();
    this.status = { code: BridgeStatus.Stopped };
    await this.start();
  }

  async delete() {
    this.endpointManager.setStopping(true);
    await this.endpointManager.waitForIdle();
    try {
      const storageService = this.env.get(StorageService);
      if (storageService.location) {
        const bridgeStoragePath = path.join(storageService.location, this.id);
        fs.rmSync(bridgeStoragePath, { recursive: true, force: true });
        appendDebugLog("bridge-delete.log", [
          `[Bridge] Removed Matter storage at ${bridgeStoragePath}`,
        ]);
      }
    } catch (e) {
      appendDebugLog("bridge-delete.log", [
        `[Bridge] Failed to remove Matter storage for ${this.id}: ${String(e)}`,
      ]);
      throw e;
    }
  }
}
