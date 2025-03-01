use async_graphql::{Context, Guard};

use crate::objects::GQLJWTData;

pub struct AuthGuard {}

impl Default for AuthGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl AuthGuard {
    pub fn new() -> Self {
        AuthGuard {}
    }
}

impl Guard for AuthGuard {
    async fn check(&self, ctx: &Context<'_>) -> async_graphql::Result<()> {
        let claims = ctx
            .data_opt::<GQLJWTData>()
            .and_then(|rd| rd.claims.as_ref());

        if claims.is_none() {
            return Err("Unauthorized request".into());
        }

        Ok(())
    }
}
