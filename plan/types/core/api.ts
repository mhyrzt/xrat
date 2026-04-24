export type ApiServiceName =
  | "HandlerService"
  | "LoggerService"
  | "StatsService"
  | "RoutingService"
  | "ReflectionService";

export interface ApiObject {
  tag: string;
  listen?: string;
  services: ApiServiceName[];
}
