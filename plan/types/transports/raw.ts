import type { StringArrayMap } from "../shared";

export interface HttpRequestObject {
  version?: string;
  method?: string;
  path?: string[];
  headers?: StringArrayMap;
}

export interface HttpResponseObject {
  version?: string;
  status?: string;
  reason?: string;
  headers?: StringArrayMap;
}

export interface NoneHeaderObject {
  type: "none";
}

export interface HttpHeaderObject {
  type: "http";
  request?: HttpRequestObject;
  response?: HttpResponseObject;
}

export interface RawObject {
  acceptProxyProtocol?: boolean;
  header?: NoneHeaderObject | HttpHeaderObject;
}
