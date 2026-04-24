import type { Address, Int32Range } from "../shared";

export interface PacketSpecObject {
  delay?: number;
  rand?: number | string;
  randRange?: string;
  type?: "array" | "str" | "hex" | "base64" | string;
  packet?: unknown[];
}

export interface TcpHeaderCustomObject {
  clients?: PacketSpecObject[][];
  servers?: PacketSpecObject[][];
  errors?: PacketSpecObject[][];
}

export interface FinalMaskFragmentObject {
  packets?: string;
  length?: Int32Range;
  delay?: Int32Range;
  maxSplit?: Int32Range;
}

export interface SudokuObject {
  password?: string;
  ascii?: string;
  customTable?: string;
  customTables?: string[];
  paddingMin?: number;
  paddingMax?: number;
}

export interface TcpFinalMaskLayerObject {
  type: "header-custom" | "fragment" | "sudoku";
  settings?: TcpHeaderCustomObject | FinalMaskFragmentObject | SudokuObject;
}

export interface UdpHeaderCustomObject {
  client?: PacketSpecObject[];
  server?: PacketSpecObject[];
}

export interface DnsLikeHeaderObject {
  domain?: string;
}

export interface PasswordObject {
  password?: string;
}

export interface XicmpObject {
  listenIp?: Address;
  id?: number;
}

export interface UdpNoisePacketObject {
  rand?: string | number;
  randRange?: string;
  type?: "array" | "str" | "hex" | "base64" | string;
  packet?: unknown[];
  delay?: Int32Range;
}

export interface UdpNoiseObject {
  reset?: number;
  noise?: UdpNoisePacketObject[];
}

export interface UdpFinalMaskLayerObject {
  type:
    | "header-custom"
    | "header-dns"
    | "header-dtls"
    | "header-srtp"
    | "header-utp"
    | "header-wechat"
    | "header-wireguard"
    | "mkcp-original"
    | "mkcp-aes128gcm"
    | "noise"
    | "salamander"
    | "sudoku"
    | "xdns"
    | "xicmp";
  settings?:
    | UdpHeaderCustomObject
    | DnsLikeHeaderObject
    | PasswordObject
    | UdpNoiseObject
    | SudokuObject
    | XicmpObject;
}

export interface UdpHopObject {
  ports?: string;
  interval?: string | number;
}

export interface QuicParamsObject {
  congestion?: "reno" | "bbr" | "brutal" | "force-brutal";
  debug?: boolean;
  brutalUp?: string | number;
  brutalDown?: string | number;
  udpHop?: UdpHopObject;
  initStreamReceiveWindow?: number;
  maxStreamReceiveWindow?: number;
  initConnectionReceiveWindow?: number;
  maxConnectionReceiveWindow?: number;
  maxIdleTimeout?: number;
  keepAlivePeriod?: number;
  disablePathMTUDiscovery?: boolean;
  maxIncomingStreams?: number;
}

export interface FinalMaskObject {
  tcp?: TcpFinalMaskLayerObject[];
  udp?: UdpFinalMaskLayerObject[];
  quicParams?: QuicParamsObject;
}
