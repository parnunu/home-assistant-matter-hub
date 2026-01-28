import type { Logger } from "@matter/general";
import type { Endpoint } from "@matter/main";
import { Service } from "../../core/ioc/service.js";
import { AggregatorEndpoint } from "../../matter/endpoints/aggregator-endpoint.js";
import type { EntityEndpoint } from "../../matter/endpoints/entity-endpoint.js";
import { LegacyEndpoint } from "../../matter/endpoints/legacy/legacy-endpoint.js";
import { InvalidDeviceError } from "../../utils/errors/invalid-device-error.js";
import { subscribeEntities } from "../home-assistant/api/subscribe-entities.js";
import type { HomeAssistantClient } from "../home-assistant/home-assistant-client.js";
import type { HomeAssistantStates } from "../home-assistant/home-assistant-registry.js";
import type { BridgeRegistry } from "./bridge-registry.js";

export class BridgeEndpointManager extends Service {
  readonly root: Endpoint;
  private entityIds: string[] = [];
  private unsubscribe?: () => void;
  private isStopping = false;
  private inFlight: Promise<void> | null = null;

  constructor(
    private readonly client: HomeAssistantClient,
    private readonly registry: BridgeRegistry,
    private readonly log: Logger,
  ) {
    super("BridgeEndpointManager");
    this.root = new AggregatorEndpoint("aggregator");
  }

  override async dispose(): Promise<void> {
    this.stopObserving();
  }

  async startObserving() {
    this.stopObserving();

    if (this.isStopping) {
      return;
    }

    if (!this.entityIds.length) {
      return;
    }

    this.unsubscribe = subscribeEntities(
      this.client.connection,
      (e) => this.updateStates(e),
      this.entityIds,
    );
  }

  stopObserving() {
    this.unsubscribe?.();
    this.unsubscribe = undefined;
  }

  setStopping(isStopping: boolean) {
    this.isStopping = isStopping;
    if (isStopping) {
      this.stopObserving();
    }
  }

  private async runExclusive(task: () => Promise<void>) {
    if (this.inFlight) {
      await this.inFlight;
    }
    this.inFlight = (async () => {
      await task();
    })();
    try {
      await this.inFlight;
    } finally {
      if (this.inFlight) {
        this.inFlight = null;
      }
    }
  }

  async waitForIdle() {
    if (this.inFlight) {
      await this.inFlight;
    }
  }

  async refreshDevices() {
    if (this.isStopping) {
      return;
    }
    await this.runExclusive(async () => {
      if (this.isStopping) {
        return;
      }
      this.registry.refresh();

      const endpoints = this.root.parts.map((p) => p as EntityEndpoint);
      this.entityIds = this.registry.entityIds;

      const existingEndpoints: EntityEndpoint[] = [];
      for (const endpoint of endpoints) {
        if (!this.entityIds.includes(endpoint.entityId)) {
          await endpoint.delete();
        } else {
          existingEndpoints.push(endpoint);
        }
      }

      for (const entityId of this.entityIds) {
        if (this.isStopping) {
          break;
        }
        let endpoint = existingEndpoints.find((e) => e.entityId === entityId);
        if (!endpoint) {
          try {
            endpoint = await LegacyEndpoint.create(this.registry, entityId);
          } catch (e) {
            if (e instanceof InvalidDeviceError) {
              this.log.warn(
                `Invalid device detected. Entity: ${entityId} Reason: ${(e as Error).message}`,
              );
              continue;
            } else {
              this.log.error(
                `Failed to create device ${entityId}. Error: ${e?.toString()}`,
              );
              throw e;
            }
          }

          if (endpoint && !this.isStopping) {
            await this.root.add(endpoint);
          }
        }
      }

      if (this.unsubscribe) {
        this.startObserving();
      }
    });
  }

  async updateStates(states: HomeAssistantStates) {
    if (this.isStopping) {
      return;
    }
    await this.runExclusive(async () => {
      if (this.isStopping) {
        return;
      }
      const endpoints = this.root.parts.map((p) => p as EntityEndpoint);
      for (const endpoint of endpoints) {
        if (this.isStopping) {
          break;
        }
        await endpoint.updateStates(states);
      }
    });
  }
}
