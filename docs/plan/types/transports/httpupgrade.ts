import type { StringMap } from "../shared";

export interface HttpUpgradeObject {
  acceptProxyProtocol?: boolean;
  path?: string;
  host?: string;
  headers?: StringMap;
}
