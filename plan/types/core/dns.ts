import type { QueryStrategy } from "../shared";

export type DnsHostValue = string | string[];

export interface DnsServerObject {
  address: string;
  port?: number;
  domains?: string[];
  expectedIPs?: string[];
  unexpectedIPs?: string[];
  skipFallback?: boolean;
  timeoutMs?: number;
  tag?: string;
  clientIP?: string;
  queryStrategy?: QueryStrategy;
  disableCache?: boolean;
  serveStale?: boolean;
  serveExpiredTTL?: number;
  finalQuery?: boolean;
}

export interface DnsObject {
  hosts?: Record<string, DnsHostValue>;
  servers?: Array<string | DnsServerObject>;
  clientIp?: string;
  queryStrategy?: QueryStrategy;
  disableCache?: boolean;
  disableFallback?: boolean;
  disableFallbackIfMatch?: boolean;
  enableParallelQuery?: boolean;
  useSystemHosts?: boolean;
  serveStale?: boolean;
  serveExpiredTTL?: number;
  tag?: string;
}
