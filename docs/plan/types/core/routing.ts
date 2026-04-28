import type { DurationString } from "../shared";

export interface RoutingWebhookObject {
  url: string;
  deduplication?: number;
  headers?: Record<string, string>;
}

export interface RuleObject {
  domain?: string[];
  ip?: string[];
  port?: number | string;
  sourcePort?: number | string;
  localPort?: number | string;
  network?: "tcp" | "udp" | "tcp,udp";
  sourceIP?: string[];
  localIP?: string[];
  user?: string[];
  inboundTag?: string[];
  protocol?: Array<"http" | "tls" | "quic" | "bittorrent">;
  attrs?: Record<string, string>;
  process?: string[];
  vlessRoute?: number | string;
  outboundTag?: string;
  balancerTag?: string;
  ruleTag?: string;
  webhook?: RoutingWebhookObject;
}

export interface RoutingCostObject {
  regexp?: boolean;
  match?: string;
  value?: number;
}

export interface RoutingStrategySettingsObject {
  expected?: number;
  maxRTT?: DurationString;
  tolerance?: number;
  baselines?: DurationString[];
  costs?: RoutingCostObject[];
}

export interface RoutingStrategyObject {
  type?: "random" | "roundRobin" | "leastPing" | "leastLoad";
  settings?: RoutingStrategySettingsObject;
}

export interface BalancerObject {
  tag: string;
  selector: string[];
  fallbackTag?: string;
  strategy?: RoutingStrategyObject;
}

export interface RoutingObject {
  domainStrategy?: "AsIs" | "IPIfNonMatch" | "IPOnDemand";
  rules?: RuleObject[];
  balancers?: BalancerObject[];
}
