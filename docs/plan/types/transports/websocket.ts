import type { StringMap } from "../shared";

export interface WebSocketObject {
  acceptProxyProtocol?: boolean;
  path?: string;
  host?: string;
  headers?: StringMap;
  heartbeatPeriod?: number;
}
