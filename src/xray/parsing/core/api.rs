use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ApiServiceName {
    HandlerService,
    LoggerService,
    StatsService,
    RoutingService,
    ReflectionService,
    ObservatoryService,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiObject {
    pub tag: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub listen: Option<String>,
    pub services: Vec<ApiServiceName>,
}
