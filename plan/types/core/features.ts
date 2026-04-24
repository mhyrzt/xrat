import type { DurationString } from "../shared";

export interface VersionObject {
  min?: string;
  max?: string;
}

export type StatsObject = Record<string, never>;

export interface BridgeObject {
  tag: string;
  domain: string;
}

export interface PortalObject {
  tag: string;
  domain: string;
}

export interface ReverseObject {
  bridges?: BridgeObject[];
  portals?: PortalObject[];
}

export interface FakeDnsPoolObject {
  ipPool: string;
  poolSize?: number;
}

export type FakeDnsObject = FakeDnsPoolObject | FakeDnsPoolObject[];

export interface MetricsObject {
  tag: string;
  listen: string;
}

export interface ObservatoryObject {
  subjectSelector: string[];
  probeUrl?: string;
  probeInterval?: DurationString;
  enableConcurrency?: boolean;
}

export interface PingConfigObject {
  destination?: string;
  connectivity?: string;
  interval?: DurationString;
  sampling?: number;
  timeout?: DurationString;
  httpMethod?: string;
}

export interface BurstObservatoryObject {
  subjectSelector: string[];
  pingConfig: PingConfigObject;
}
