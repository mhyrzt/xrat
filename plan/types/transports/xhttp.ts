export interface XHttpSettingsObject {
  path?: string;
  host?: string;
  mode?: string;
  extra?: Record<string, unknown>;
  [key: string]: unknown;
}
