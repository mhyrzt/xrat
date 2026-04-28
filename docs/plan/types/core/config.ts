import type { InboundObject } from "../protocols/inbounds";
import type { OutboundObject } from "../protocols/outbounds";
import type { TransportObject } from "../transports";
import type { VersionObject, StatsObject, ReverseObject, FakeDnsObject, MetricsObject, ObservatoryObject, BurstObservatoryObject } from "./features";
import type { LogObject } from "./log";
import type { ApiObject } from "./api";
import type { DnsObject } from "./dns";
import type { RoutingObject } from "./routing";
import type { PolicyObject } from "./policy";

export interface XrayConfig {
  version?: VersionObject;
  log?: LogObject;
  api?: ApiObject;
  dns?: DnsObject;
  routing?: RoutingObject;
  policy?: PolicyObject;
  inbounds?: InboundObject[];
  outbounds?: OutboundObject[];
  transport?: TransportObject;
  stats?: StatsObject;
  reverse?: ReverseObject;
  fakedns?: FakeDnsObject;
  metrics?: MetricsObject;
  observatory?: ObservatoryObject;
  burstObservatory?: BurstObservatoryObject;
}
