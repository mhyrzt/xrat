import type { Address, DomainStrategy, Int32Range } from "../shared";

export interface TLSCertificateObject {
  usage?: "encipherment" | "verify" | "issue";
  certificateFile?: string;
  keyFile?: string;
  certificate?: string[];
  key?: string[];
  ocspStapling?: number;
  oneTimeLoading?: boolean;
  buildChain?: boolean;
}

export interface LimitFallbackObject {
  afterBytes?: number;
  bytesPerSec?: number;
  burstBytesPerSec?: number;
}

export interface HappyEyeballsObject {
  tryDelayMs?: number;
  prioritizeIPv6?: boolean;
  interleave?: number;
  maxConcurrentTry?: number;
}

export interface CustomSockoptObject {
  system?: "linux" | "windows" | "darwin";
  type?: "int" | "str";
  level?: string | number;
  opt?: string | number;
  value?: string | number | boolean;
}

export interface SockoptObject {
  mark?: number;
  tcpMaxSeg?: number;
  tcpFastOpen?: boolean | number;
  tproxy?: "off" | "redirect" | "tproxy";
  domainStrategy?: DomainStrategy;
  happyEyeballs?: HappyEyeballsObject;
  dialerProxy?: string;
  acceptProxyProtocol?: boolean;
  tcpKeepAliveInterval?: number;
  tcpKeepAliveIdle?: number;
  tcpUserTimeout?: number;
  tcpCongestion?: string;
  tcpcongestion?: string;
  interface?: string;
  v6only?: boolean;
  V6Only?: boolean;
  tcpWindowClamp?: number;
  tcpMptcp?: boolean;
  tcpNoDelay?: boolean;
  addressPortStrategy?:
    | "none"
    | "SrvPortOnly"
    | "SrvAddressOnly"
    | "SrvPortAndAddress"
    | "TxtPortOnly"
    | "TxtAddressOnly"
    | "TxtPortAndAddress";
  customSockopt?: CustomSockoptObject[];
}

export interface TLSObject {
  serverName?: string;
  verifyPeerCertByName?: string;
  rejectUnknownSni?: boolean;
  allowInsecure?: boolean;
  alpn?: string[];
  minVersion?: string;
  maxVersion?: string;
  cipherSuites?: string;
  certificates?: TLSCertificateObject[];
  disableSystemRoot?: boolean;
  enableSessionResumption?: boolean;
  fingerprint?: "chrome" | "firefox" | "safari" | "ios" | "android" | "edge" | "360" | "qq" | string;
  pinnedPeerCertSha256?: string;
  curvePreferences?: string[];
  masterKeyLog?: string;
  echServerKeys?: string;
  echConfigList?: string;
  echForceQuery?: "none" | "half" | "full";
  echSockopt?: SockoptObject;
}

export interface RealityObject {
  show?: boolean;
  target?: string;
  xver?: number;
  serverNames?: string[];
  privateKey?: string;
  minClientVer?: string;
  maxClientVer?: string;
  maxTimeDiff?: number;
  shortIds?: string[];
  mldsa65Seed?: string;
  limitFallbackUpload?: LimitFallbackObject;
  limitFallbackDownload?: LimitFallbackObject;
  fingerprint?: string;
  serverName?: string;
  password?: string;
  shortId?: string;
  mldsa65Verify?: string;
  spiderX?: string;
}
