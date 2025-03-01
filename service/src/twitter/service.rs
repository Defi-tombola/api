use std::sync::Arc;

use crate::account::store::AccountStore;
use crate::account::AccountService;
use crate::{
    account::types::UpdateAccount, cache::service::CacheService, config::service::ConfigService,
    prelude::ServiceProvider, services::ServiceFactory, store::service::StoreService,
};
use chrono::Utc;
use entity::account::AccountModel;
use error_stack::{Report, Result, ResultExt};
use lib::error::Error;
use redis_macros::FromRedisValue;
use serde::{Deserialize, Serialize};
use serenity::async_trait;
use twitter_v2::{
    authorization::{Oauth2Client, Scope},
    oauth2::{AuthorizationCode, CsrfToken, PkceCodeChallenge, PkceCodeVerifier},
    TwitterApi,
};

// Expire after 5 minutes
const TWITTER_AUTH_EXPIRY: usize = 300;
const TWITTER_REDIS_KEY: &str = "twitter";

pub struct TwitterService {
    config: Arc<ConfigService>,
    cache: Arc<CacheService>,
    store: Arc<StoreService>,
    account: Arc<AccountService>,
}

#[derive(Serialize, Deserialize, FromRedisValue)]
pub struct TwitterAuth {
    pub state: CsrfToken,
    pub verifier: PkceCodeVerifier,
    pub redirect_url: String,
}

impl TwitterService {
    pub fn new(
        config: Arc<ConfigService>,
        cache: Arc<CacheService>,
        store: Arc<StoreService>,
        account: Arc<AccountService>,
    ) -> Self {
        Self {
            config,
            cache,
            store,
            account,
        }
    }

    /// Get the auth url for a given account id where twitter will be assigned to
    /// when user will be redirected back to the redirect url.
    pub async fn get_auth_url(
        &self,
        account_id: String,
        redirect_url: String,
    ) -> Result<String, Error> {
        let client = Oauth2Client::new(
            self.config.twitter.client_id.clone(),
            self.config.twitter.client_secret.clone(),
            redirect_url.parse().unwrap(),
        );

        let (challenge, verifier) = PkceCodeChallenge::new_random_sha256();
        let (url, state) = client.auth_url(
            challenge,
            [Scope::TweetRead, Scope::UsersRead, Scope::OfflineAccess],
        );

        let mut connection = self.cache.get_connection().await?;
        let auth = TwitterAuth {
            state,
            verifier,
            redirect_url,
        };

        redis::cmd("SETEX")
            .arg(format!("{TWITTER_REDIS_KEY}:{account_id}"))
            .arg(TWITTER_AUTH_EXPIRY)
            .arg(serde_json::to_string(&auth).unwrap())
            .query_async::<_, String>(&mut connection)
            .await
            .unwrap();

        Ok(url.to_string())
    }

    /// Link Twitter account to the account id provided.
    pub async fn link_to_account(
        &self,
        account_id: String,
        state: String,
        code: String,
    ) -> Result<AccountModel, Error> {
        let mut connection = self.cache.get_connection().await?;

        let auth = redis::cmd("GET")
            .arg(format!("{TWITTER_REDIS_KEY}:{account_id}"))
            .query_async::<_, TwitterAuth>(&mut connection)
            .await
            .change_context(Error::Redis)?;

        let (client, code, verifier) = {
            let state = CsrfToken::new(state);
            let code = AuthorizationCode::new(code);
            let verifier = auth.verifier;

            if state.secret() != auth.state.secret() {
                return Err(
                    Report::from(Error::TwitterInvalidState).attach_printable("Invalid state")
                );
            }

            let client = Oauth2Client::new(
                self.config.twitter.client_id.clone(),
                self.config.twitter.client_secret.clone(),
                auth.redirect_url.parse().unwrap(),
            );

            (client, code, verifier)
        };

        let token = client
            .request_token(code, verifier)
            .await
            .change_context(Error::Unknown)?;

        let api = TwitterApi::new(token);
        let me = api
            .get_users_me()
            .send()
            .await
            .change_context(Error::Unknown)?
            .into_data()
            .ok_or(Report::from(Error::Unknown).attach_printable("No user found"))?;

        let pool = self.store.read();
        let account = AccountStore::try_find_by_address(pool, account_id)
            .await?
            .ok_or(Report::new(Error::Unknown).attach_printable("Account not found"))?;

        let mut db_tx = self.store.begin_transaction().await?;
        let account = self
            .account
            .update(
                account.id,
                UpdateAccount {
                    twitter: Some(me.username),
                    updated_at: Utc::now(),
                    ..Default::default()
                },
                &mut db_tx,
            )
            .await;

        self.store.commit_transaction(db_tx).await?;

        account
    }
}

#[async_trait]
impl ServiceFactory for TwitterService {
    async fn factory(services: ServiceProvider) -> Result<Self, Error> {
        let config = services.get_service_unchecked::<ConfigService>().await;
        let cache = services.get_service_unchecked::<CacheService>().await;
        let store = services.get_service_unchecked::<StoreService>().await;
        let account = services.get_service_unchecked::<AccountService>().await;

        Ok(Self::new(config, cache, store, account))
    }
}
