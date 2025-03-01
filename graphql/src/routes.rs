use async_graphql::{http::ALL_WEBSOCKET_PROTOCOLS, Data, ServerError};
use async_graphql_axum::{GraphQLProtocol, GraphQLRequest, GraphQLResponse, GraphQLWebSocket};
use axum::{
    extract::{State, WebSocketUpgrade},
    http::HeaderMap,
    response::{Html, IntoResponse, Response},
};
use serde::Deserialize;
use service::config::service::ConfigService;
use tracing::{debug, warn};

use crate::ide::altair::AltairGraphQL;
use crate::objects::GQLJWTData;
use crate::server::AppState;

pub async fn graphql_ws_handler(
    State(state): State<AppState>,
    protocol: GraphQLProtocol,
    websocket: WebSocketUpgrade,
) -> Response {
    debug!("GraphQL WS handler");
    websocket
        .protocols(ALL_WEBSOCKET_PROTOCOLS)
        .on_upgrade(move |stream| {
            GraphQLWebSocket::new(stream, state.schema.clone(), protocol)
                .on_connection_init(|value| async move {
                    #[derive(Deserialize)]
                    #[serde(rename_all = "PascalCase")]
                    struct Payload {
                        authorization: String,
                    }

                    let claims = {
                        if let Ok(payload) = serde_json::from_value::<Payload>(value) {
                            let token = extract_token_from_str(&payload.authorization);
                            if token.is_none() {
                                return Err(async_graphql::Error::new("Token is invalid"));
                            }
                            let token = token.unwrap();

                            match state.jwt.decode(token) {
                                Ok(token) => Ok(Some(token.claims)),
                                Err(err) => match *err.kind() {
                                    jsonwebtoken::errors::ErrorKind::InvalidToken => {
                                        Err(async_graphql::Error::new("Token is invalid"))
                                    }
                                    jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                                        Err(async_graphql::Error::new("Token has expired"))
                                    }
                                    _ => {
                                        warn!("Token validation error: {}", err);
                                        Err(async_graphql::Error::new(
                                            "Unable to validate auth token",
                                        ))
                                    }
                                },
                            }?
                        } else {
                            None
                        }
                    };

                    let mut context = Data::default();
                    context.insert(GQLJWTData { claims });

                    Ok(context)
                })
                .serve()
        })
}

pub async fn graphql_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    gql_req: GraphQLRequest,
) -> GraphQLResponse {
    let mut request = gql_req.into_inner();

    if let Some(token) = get_auth_token_from_headers(&headers) {
        let err_msg_response = |msg: &str| -> GraphQLResponse {
            async_graphql::Response::from_errors(vec![ServerError::new(msg, None)]).into()
        };

        match state.jwt.decode(token) {
            Ok(token) => {
                request = request.data(GQLJWTData {
                    claims: Some(token.claims),
                })
            }
            Err(err) => match *err.kind() {
                jsonwebtoken::errors::ErrorKind::InvalidToken => {
                    return err_msg_response("Token is invalid");
                }
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                    return err_msg_response("Token has expired");
                }
                _ => {
                    warn!("Token validation error: {}", err);
                    return err_msg_response("Unable to validate auth token");
                }
            },
        }
    }

    state.schema.execute(request).await.into()
}

pub async fn graphql_playground(State(state): State<AppState>) -> impl IntoResponse {
    let config = state
        .services
        .get_service_unchecked::<ConfigService>()
        .await;

    Html(
        AltairGraphQL::build()
            .endpoint(&config.graphql.endpoint)
            .subscription_endpoint(&config.graphql.subscription_endpoint)
            .title("Valio GQL Explorer")
            .finish(),
    )
}

pub async fn health() -> impl IntoResponse {
    Html("OK")
}

fn extract_token_from_str(value: &str) -> Option<String> {
    value.split_once(' ').map(|s| s.1).map(|s| s.to_owned())
}

fn get_auth_token_from_headers(headers: &HeaderMap) -> Option<String> {
    headers
        .get("Authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(extract_token_from_str)
}
