import type { Address, Network, StringMap } from "../shared";
import type {
  VlessClientObject,
  VmessClientObject,
  TrojanClientObject,
  HysteriaClientObject,
  ShadowsocksClientObject,
} from "./clients";
import type { HttpAccountObject, FallbackObject } from "./common";

export interface InboundSettingsDokodemo {
  address?: Address;
  port?: number;
  network?: Network;
  followRedirect?: boolean;
  userLevel?: number;
}

export interface InboundSettingsHttp {
  accounts?: HttpAccountObject[];
  allowTransparent?: boolean;
  userLevel?: number;
}

export interface InboundSettingsHysteria {
  version: 2;
  clients?: HysteriaClientObject[];
}

export interface InboundSettingsShadowsocks {
  network?: Network;
  method?: string;
  password?: string;
  level?: number;
  email?: string;
  clients?: ShadowsocksClientObject[];
}

export interface InboundSettingsSocks {
  auth?: "noauth" | "password";
  accounts?: HttpAccountObject[];
  udp?: boolean;
  ip?: Address;
  userLevel?: number;
}

export interface InboundSettingsTrojan {
  clients: TrojanClientObject[];
  fallbacks?: FallbackObject[];
}

export interface InboundSettingsTun {
  name?: string;
  MTU?: number;
  mtu?: number;
  UserLevel?: number;
  userLevel?: number;
}

export interface InboundSettingsTunnel {
  address?: Address;
  port?: number;
  portMap?: StringMap;
  network?: Network;
  followRedirect?: boolean;
  userLevel?: number;
}

export interface InboundSettingsVless {
  clients: VlessClientObject[];
  decryption: "none";
  fallbacks?: FallbackObject[];
}

export interface InboundSettingsVmess {
  clients: VmessClientObject[];
  default?: {
    level?: number;
  };
}

export interface WireguardInboundPeerObject {
  publicKey: string;
  allowedIPs?: string[];
}

export interface InboundSettingsWireguard {
  secretKey: string;
  mtu?: number;
  peers: WireguardInboundPeerObject[];
}
