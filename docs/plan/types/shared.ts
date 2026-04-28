export type Address = string;
export type CIDR = string;
export type DomainMatcher = string;
export type DurationString = string;
export type PortValue = number | string;
export type Int32Range = number | string;
export type StringMap = Record<string, string>;
export type StringArrayMap = Record<string, string[]>;

export type Network = "tcp" | "udp" | "tcp,udp";
export type StreamNetwork =
  | "raw"
  | "xhttp"
  | "kcp"
  | "grpc"
  | "ws"
  | "httpupgrade"
  | "hysteria";
export type Security = "none" | "tls" | "reality";
export type LogLevel = "debug" | "info" | "warning" | "error" | "none";
export type MaskAddress = "" | "quarter" | "half" | "full";
export type QueryStrategy = "UseIP" | "UseIPv4" | "UseIPv6" | "UseSystem";
export type DomainStrategy =
  | "AsIs"
  | "UseIP"
  | "UseIPv4"
  | "UseIPv6"
  | "UseIPv4v6"
  | "UseIPv6v4"
  | "ForceIP"
  | "ForceIPv4"
  | "ForceIPv6"
  | "ForceIPv4v6"
  | "ForceIPv6v4";
