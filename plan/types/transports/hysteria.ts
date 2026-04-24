import type { StringMap } from "../shared";

export interface HysteriaMasqObject {
  type?: "file" | "proxy" | "string";
  dir?: string;
  url?: string;
  rewriteHost?: boolean;
  insecure?: boolean;
  content?: string;
  headers?: StringMap;
  statusCode?: number;
}

export interface HysteriaObject {
  version?: 2;
  auth?: string;
  udpIdleTimeout?: number;
  masquerade?: HysteriaMasqObject;
}
