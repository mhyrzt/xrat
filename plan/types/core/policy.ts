export interface LevelPolicyObject {
  handshake?: number;
  connIdle?: number;
  uplinkOnly?: number;
  downlinkOnly?: number;
  statsUserUplink?: boolean;
  statsUserDownlink?: boolean;
  statsUserOnline?: boolean;
  bufferSize?: number;
}

export interface SystemPolicyObject {
  statsInboundUplink?: boolean;
  statsInboundDownlink?: boolean;
  statsOutboundUplink?: boolean;
  statsOutboundDownlink?: boolean;
}

export interface PolicyObject {
  levels?: Record<string, LevelPolicyObject>;
  system?: SystemPolicyObject;
}
