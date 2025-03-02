use crate::routes::{graphql_handler, graphql_playground, graphql_ws_handler};
use crate::schema::ServiceSchema;
use crate::{helpers::jwt::JWT, schema, schema::GQLGlobalData};
use axum::response::{Html, IntoResponse};
use axum::{routing::get, serve, Router};
use axum_prometheus::metrics_exporter_prometheus::PrometheusBuilder;
use axum_prometheus::PrometheusMetricLayer;
use axum_server::tls_rustls::RustlsConfig;
use error_stack::{Result, ResultExt};
use lib::error::Error;
use service::{config::service::ConfigService, services::ServiceProvider};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::Path;
use tower_http::compression::CompressionLayer;
use tower_http::cors;
use tower_http::trace::TraceLayer;

#[derive(Clone)]
pub struct AppState {
    pub services: ServiceProvider,
    pub schema: ServiceSchema,
    pub jwt: JWT,
}

pub struct Server {
    services: ServiceProvider,
}

impl Server {
    pub fn new(services: ServiceProvider) -> Self {
        Self { services }
    }

    fn router(&self, app_state: AppState) -> Router {
        let cors_layer = cors::CorsLayer::new()
            .allow_origin(cors::Any)
            .allow_headers(cors::Any)
            .allow_methods(cors::Any);

        // let metrics_exporter_address =
        //     SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 4317);

        // let metric_layer = PrometheusMetricLayer::new();
        // // This is the default if you use `PrometheusMetricLayer::pair`.
        // let metric_handle = PrometheusBuilder::new()
        //     .with_http_listener(metrics_exporter_address)
        //     .install_recorder()
        //     .unwrap();

        // let metrics_router =
        //     Router::new().route("/", get(|| async move { metric_handle.render() }));

        Router::new()
            .route("/", get(graphql_playground).post(graphql_handler))
            .route("/health", get(health))
            .route("/ws", get(graphql_ws_handler))
            .with_state(app_state)
            // .nest("/metrics", metrics_router) // TODO: do not expose metrics for everyone
            // .layer(metric_layer)
            .layer(cors_layer)
            .layer(CompressionLayer::new().no_br())
            .layer(TraceLayer::new_for_http())
    }

    pub async fn start(self) -> Result<(), Error> {
        let config = self.services.get_service_unchecked::<ConfigService>().await;

        // Build axum app state
        let app_state = {
            let jwt = JWT::new_from_pem(
                config.jwt.private_key.as_bytes(),
                config.jwt.public_key.as_bytes(),
            )
            .expect("Failed to init JWT");

            let schema = schema::new(GQLGlobalData {
                services: self.services.clone(),
                jwt: jwt.clone(),
            })
            .await;

            AppState {
                services: self.services.clone(),
                schema,
                jwt,
            }
        };

        let app = self.router(app_state);

        let address = config.graphql.listen.clone();
        
        if config.environment.name == "prod" {
            let listener = std::net::TcpListener::bind(address)
                .change_context(Error::Unknown)?;
            
            let certs_folder = Path::new("./self_signed_certs");
            let rustls_config = RustlsConfig::from_pem_file(
                certs_folder
                    .join("cert.pem"),
                certs_folder
                    .join("key.pem"),
            )
            .await
            .unwrap();

            axum_server::from_tcp_rustls(listener, rustls_config)
                .serve(app.into_make_service())
                .await
                .change_context(Error::Unknown)?;
        } else {
            let listener = tokio::net::TcpListener::bind(address)
                        .await
                        .change_context(Error::Unknown)?;
            
            serve(listener, app.into_make_service())
                        .await
                        .change_context(Error::Unknown)?;
        }
        
        Ok(())
        
    }
}

async fn health() -> impl IntoResponse {
    Html("OK")
}
