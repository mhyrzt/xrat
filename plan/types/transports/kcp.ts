export interface KcpObject {
  mtu?: number;
  tti?: number;
  uplinkCapacity?: number;
  downlinkCapacity?: number;
  congestion?: boolean;
  readBufferSize?: number;
  writeBufferSize?: number;
}
