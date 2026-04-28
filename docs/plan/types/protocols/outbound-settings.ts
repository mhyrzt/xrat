import type { Address, DomainStrategy, Network, StringMap } from "../shared";

export interface FragmentObject {
  packets?: string;
  length?: string;
  interval?: string;
}

export interface NoiseObject {
  type?: "rand" | "str" | "hex" | "base64" | string;
  packet?: string;
  delay?: string;
}

export interface WireguardPeerObject {
  endpoint: string;
  publicKey: string;
  preSharedKey?: string;
  keepAlive?: number;
  allowedIPs?: string[];
}

export interface OutboundSettingsBlackhole {
  response?: {
    type?: "http" | "none";
  };
}

export interface OutboundSettingsDns {
  network?: Extract<Network, "tcp" | "udp">;
  address?: Address;
  port?: number;
  userLevel?: number;
  nonIPQuery?: "drop" | "skip" | "reject" | string;
  blockTypes?: number[];
}

export interface OutboundSettingsFreedom {
  domainStrategy?: DomainStrategy;
  redirect?: string;
  userLevel?: number;
  fragment?: FragmentObject;
  noises?: NoiseObject[];
  proxyProtocol?: number;
  ipsBlocked?: string[];
}

export interface OutboundSettingsHttp {
  address: Address;
  port: number;
  user?: string;
  pass?: string;
  level?: number;
  email?: string;
  headers?: StringMap;
}

export interface OutboundSettingsHysteria {
  version: 2;
  address: Address;
  port: number;
}

export interface OutboundSettingsLoopback {
  inboundTag: string;
}

export interface OutboundSettingsShadowsocks {
  email?: string;
  address: Address;
  port: number;
  method: string;
  password: string;
  uot?: boolean;
  UoTVersion?: 1 | 2;
  level?: number;
}

export interface OutboundSettingsSocks {
  address: Address;
  port: number;
  user?: string;
  pass?: string;
  level?: number;
  email?: string;
}

export interface OutboundSettingsTrojan {
  address: Address;
  port: number;
  password: string;
  email?: string;
  level?: number;
}

export interface OutboundSettingsVless {
  address: Address;
  port: number;
  id: string;
  encryption: "none";
  flow?: string;
  level?: number;
  reverse?: {
    tag?: string;
  };
}

export interface OutboundSettingsVmess {
  address: Address;
  port: number;
  id: string;
  security?: "aes-128-gcm" | "chacha20-poly1305" | "auto" | "none" | "zero";
  level?: number;
  experiments?: string;
}

export interface OutboundSettingsWireguard {
  secretKey: string;
  address?: string[];
  noKernelTun?: boolean;
  mtu?: number;
  reserved?: number[];
  workers?: number;
  peers: WireguardPeerObject[];
  domainStrategy?: Extract<DomainStrategy, "ForceIPv6v4" | "ForceIPv6" | "ForceIPv4v6" | "ForceIPv4" | "ForceIP">;
}
