use opentelemetry::KeyValue;
use opentelemetry_sdk::Resource;

use crate::config::ConfigService;

/// Creates a new opentelemetry resource with the given service name and namespace.
/// @
/// # Arguments
///
/// * `service_name` - The name of the service.
/// * `service_namespace` - The namespace of the service. The environment the instance is running in.
///
/// Example:
/// ```rust
/// let resource_indexer = get_otlp_resource("indexer".to_string(), "stage".to_string());
/// let resource_graphql = get_otlp_resource("graphql".to_string(), "stage".to_string());
/// ```
pub(crate) fn get_otlp_resource(service_name: String, service_namespace: String) -> Resource {
    // We may use semantic convenction from opentelemetry on the key side instead of raw str.
    let service_attr = KeyValue::new("service.name", service_name.clone());
    let service_namespace_attr = KeyValue::new("service.namespace", service_namespace.clone());

    Resource::new(vec![service_attr, service_namespace_attr])
}

#[derive(Debug, Clone)]
pub struct TelemetryParams {
    pub service_name: String,
    pub service_namespace: String,
    pub log_level: tracing::Level,
    pub grcp_endpoint: String,
    pub http_endpoint: String,
}

impl TelemetryParams {
    pub fn new(config_service: ConfigService, service_name: String, log_level: Option<tracing::Level>) -> Self {
        Self {
            service_name,
            service_namespace: config_service.environment.name.clone(),
            log_level: log_level.unwrap_or(tracing::Level::INFO),
            grcp_endpoint: config_service.environment.otlp_grcp_endpoint.clone(),
            http_endpoint: config_service.environment.otlp_http_endpoint.clone(),
        }
    }
}