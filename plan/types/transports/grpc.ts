export interface GRPCObject {
  authority?: string;
  serviceName?: string;
  user_agent?: string;
  multiMode?: boolean;
  idle_timeout?: number;
  health_check_timeout?: number;
  permit_without_stream?: boolean;
  initial_windows_size?: number;
}
