import type { Security, StreamNetwork } from "../shared";
import type { TLSObject, RealityObject, SockoptObject } from "./security";
import type { RawObject } from "./raw";
import type { KcpObject } from "./kcp";
import type { GRPCObject } from "./grpc";
import type { WebSocketObject } from "./websocket";
import type { HttpUpgradeObject } from "./httpupgrade";
import type { HysteriaObject } from "./hysteria";
import type { XHttpSettingsObject } from "./xhttp";
import type { FinalMaskObject } from "./finalmask";

export * from "./security";
export * from "./raw";
export * from "./kcp";
export * from "./grpc";
export * from "./websocket";
export * from "./httpupgrade";
export * from "./hysteria";
export * from "./xhttp";
export * from "./finalmask";

export interface StreamSettingsObject {
  network?: StreamNetwork;
  security?: Security;
  tlsSettings?: TLSObject;
  realitySettings?: RealityObject;
  rawSettings?: RawObject;
  xhttpSettings?: XHttpSettingsObject | Record<string, unknown>;
  kcpSettings?: KcpObject;
  grpcSettings?: GRPCObject;
  wsSettings?: WebSocketObject;
  httpupgradeSettings?: HttpUpgradeObject;
  hysteriaSettings?: HysteriaObject;
  sockopt?: SockoptObject;
  finalmask?: FinalMaskObject;
}

export type TransportObject = StreamSettingsObject;
