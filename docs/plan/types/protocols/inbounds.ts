import type { Address, PortValue } from "../shared";
import type { StreamSettingsObject } from "../transports";
import type {
  InboundSettingsDokodemo,
  InboundSettingsHttp,
  InboundSettingsHysteria,
  InboundSettingsShadowsocks,
  InboundSettingsSocks,
  InboundSettingsTrojan,
  InboundSettingsTun,
  InboundSettingsTunnel,
  InboundSettingsVless,
  InboundSettingsVmess,
  InboundSettingsWireguard,
} from "./inbound-settings";

export interface SniffingObject {
  enabled?: boolean;
  destOverride?: Array<"http" | "tls" | "quic" | "fakedns">;
  metadataOnly?: boolean;
  domainsExcluded?: string[];
  ipsExcluded?: string[];
  routeOnly?: boolean;
}

interface BaseInboundObject {
  listen?: Address;
  port: PortValue;
  streamSettings?: StreamSettingsObject;
  tag?: string;
  sniffing?: SniffingObject;
}

export interface DokodemoInboundObject extends BaseInboundObject {
  protocol: "dokodemo-door";
  settings: InboundSettingsDokodemo;
}

export interface HttpInboundObject extends BaseInboundObject {
  protocol: "http";
  settings: InboundSettingsHttp;
}

export interface HysteriaInboundObject extends BaseInboundObject {
  protocol: "hysteria";
  settings: InboundSettingsHysteria;
}

export interface ShadowsocksInboundObject extends BaseInboundObject {
  protocol: "shadowsocks";
  settings: InboundSettingsShadowsocks;
}

export interface SocksInboundObject extends BaseInboundObject {
  protocol: "socks";
  settings: InboundSettingsSocks;
}

export interface TrojanInboundObject extends BaseInboundObject {
  protocol: "trojan";
  settings: InboundSettingsTrojan;
}

export interface TunInboundObject extends BaseInboundObject {
  protocol: "tun";
  settings: InboundSettingsTun;
}

export interface TunnelInboundObject extends BaseInboundObject {
  protocol: "tunnel";
  settings: InboundSettingsTunnel;
}

export interface VlessInboundObject extends BaseInboundObject {
  protocol: "vless";
  settings: InboundSettingsVless;
}

export interface VmessInboundObject extends BaseInboundObject {
  protocol: "vmess";
  settings: InboundSettingsVmess;
}

export interface WireguardInboundObject extends BaseInboundObject {
  protocol: "wireguard";
  settings: InboundSettingsWireguard;
}

export type InboundObject =
  | DokodemoInboundObject
  | HttpInboundObject
  | HysteriaInboundObject
  | ShadowsocksInboundObject
  | SocksInboundObject
  | TrojanInboundObject
  | TunInboundObject
  | TunnelInboundObject
  | VlessInboundObject
  | VmessInboundObject
  | WireguardInboundObject;
