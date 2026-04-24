import type { LogLevel, MaskAddress } from "../shared";

export interface LogObject {
  access?: string;
  error?: string;
  loglevel?: LogLevel;
  dnsLog?: boolean;
  maskAddress?: MaskAddress;
}
