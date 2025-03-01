use super::account::AccountType;
use async_graphql::Object;
use entity::prelude::AccountModel;

pub struct AuthType {
    pub access_token: String,
    // TODO: Implement refresh token
    // pub refresh_token: String,
    pub address: String,
    pub account: Option<AccountModel>,
}

#[Object]
impl AuthType {
    async fn access_token(&self) -> &str {
        &self.access_token
    }

    async fn address(&self) -> &str {
        &self.address
    }

    async fn account(&self) -> Option<AccountType> {
        self.account.as_ref().map(|i| i.clone().into())
    }
}
