import type { Address, DomainStrategy } from "../shared";
import type { StreamSettingsObject } from "../transports";
import type {
  OutboundSettingsBlackhole,
  OutboundSettingsDns,
  OutboundSettingsFreedom,
  OutboundSettingsHttp,
  OutboundSettingsHysteria,
  OutboundSettingsLoopback,
  OutboundSettingsShadowsocks,
  OutboundSettingsSocks,
  OutboundSettingsTrojan,
  OutboundSettingsVless,
  OutboundSettingsVmess,
  OutboundSettingsWireguard,
} from "./outbound-settings";

export interface MuxObject {
  enabled?: boolean;
  concurrency?: number;
  xudpConcurrency?: number;
  xudpProxyUDP443?: "reject" | "allow" | "skip";
}

export interface ProxySettingsObject {
  tag: string;
  transportLayer?: boolean;
}

interface BaseOutboundObject {
  sendThrough?: Address;
  tag?: string;
  streamSettings?: StreamSettingsObject;
  proxySettings?: ProxySettingsObject;
  mux?: MuxObject;
  targetStrategy?: DomainStrategy;
}

export interface BlackholeOutboundObject extends BaseOutboundObject {
  protocol: "blackhole";
  settings?: OutboundSettingsBlackhole;
}

export interface DnsOutboundObject extends BaseOutboundObject {
  protocol: "dns";
  settings?: OutboundSettingsDns;
}

export interface FreedomOutboundObject extends BaseOutboundObject {
  protocol: "freedom";
  settings?: OutboundSettingsFreedom;
}

export interface HttpOutboundObject extends BaseOutboundObject {
  protocol: "http";
  settings: OutboundSettingsHttp;
}

export interface HysteriaOutboundObject extends BaseOutboundObject {
  protocol: "hysteria";
  settings: OutboundSettingsHysteria;
}

export interface LoopbackOutboundObject extends BaseOutboundObject {
  protocol: "loopback";
  settings: OutboundSettingsLoopback;
}

export interface ShadowsocksOutboundObject extends BaseOutboundObject {
  protocol: "shadowsocks";
  settings: OutboundSettingsShadowsocks;
}

export interface SocksOutboundObject extends BaseOutboundObject {
  protocol: "socks";
  settings: OutboundSettingsSocks;
}

export interface TrojanOutboundObject extends BaseOutboundObject {
  protocol: "trojan";
  settings: OutboundSettingsTrojan;
}

export interface VlessOutboundObject extends BaseOutboundObject {
  protocol: "vless";
  settings: OutboundSettingsVless;
}

export interface VmessOutboundObject extends BaseOutboundObject {
  protocol: "vmess";
  settings: OutboundSettingsVmess;
}

export interface WireguardOutboundObject extends BaseOutboundObject {
  protocol: "wireguard";
  settings: OutboundSettingsWireguard;
  streamSettings?: never;
}

export type OutboundObject =
  | BlackholeOutboundObject
  | DnsOutboundObject
  | FreedomOutboundObject
  | HttpOutboundObject
  | HysteriaOutboundObject
  | LoopbackOutboundObject
  | ShadowsocksOutboundObject
  | SocksOutboundObject
  | TrojanOutboundObject
  | VlessOutboundObject
  | VmessOutboundObject
  | WireguardOutboundObject;
