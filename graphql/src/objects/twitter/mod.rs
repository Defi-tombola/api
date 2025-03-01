use async_graphql::{Context, Object};
use service::services::ServiceProvider;
use service::twitter::service::TwitterService;
use tracing::warn;

use crate::guards::auth::AuthGuard;
use crate::objects::account::inputs::LinkTwitterInput;
use crate::objects::account::types::AccountType;
use crate::objects::GQLJWTData;

#[derive(Default)]
pub struct TwitterQuery;

#[Object]
impl TwitterQuery {
    #[graphql(guard = "AuthGuard::new()")]
    async fn twitter_auth_url(
        &self,
        ctx: &Context<'_>,
        redirect_uri: String,
    ) -> async_graphql::Result<String> {
        let services = ctx.data_unchecked::<ServiceProvider>();
        let claims = ctx
            .data_opt::<GQLJWTData>()
            .and_then(|rd| rd.claims.as_ref())
            .ok_or(async_graphql::Error::from("Not authorized"))?;

        let twitter_service = services.get_service_unchecked::<TwitterService>().await;
        twitter_service
            .get_auth_url(claims.sub.clone(), redirect_uri)
            .await
            .map_err(|e| {
                warn!("Failed to get twitter auth url: {e:?}");
                async_graphql::Error::new("Internal error")
            })
    }
}

#[derive(Default)]
pub struct TwitterMutation;

#[Object]
impl TwitterMutation {
    #[graphql(guard = "AuthGuard::new()")]
    async fn link_twitter_to_account(
        &self,
        ctx: &Context<'_>,
        input: LinkTwitterInput,
    ) -> async_graphql::Result<AccountType, async_graphql::Error> {
        let services = ctx.data_unchecked::<ServiceProvider>();
        let claims = ctx
            .data_opt::<GQLJWTData>()
            .and_then(|rd| rd.claims.as_ref())
            .ok_or(async_graphql::Error::from("Not authorized"))?;

        let twitter_service = services.get_service_unchecked::<TwitterService>().await;

        let account = twitter_service
            .link_to_account(claims.sub.clone(), input.state, input.code)
            .await
            .map_err(|e| {
                warn!("{:?}", e);
                async_graphql::Error::new("Failed to link twitter account")
            })?;

        Ok(account.into())
    }
}
