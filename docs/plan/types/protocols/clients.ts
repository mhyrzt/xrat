import type { ReverseTagObject } from "./common";

export interface VlessClientObject {
  id: string;
  level?: number;
  email?: string;
  flow?: string;
  reverse?: ReverseTagObject;
}

export interface VmessClientObject {
  id: string;
  level?: number;
  email?: string;
}

export interface TrojanClientObject {
  password: string;
  email?: string;
  level?: number;
}

export interface HysteriaClientObject {
  auth: string;
  level?: number;
  email?: string;
}

export interface ShadowsocksClientObject {
  password: string;
  method?: string;
  level?: number;
  email?: string;
}
