pub mod inputs;
pub mod types;

use self::inputs::{LoginSignatureInput, UpdateAccountInput};
use self::types::{AccountType, AuthType};
use async_graphql::{Context, Object, Subscription};
use chrono::Utc;
use futures::{Stream, StreamExt};
use lib::crypto::recover_address;
use service::account::store::AccountStore;
use service::account::types::{CreateAccount, UpdateAccount};
use service::account::AccountService;
use service::{
    prelude::StoreService,
    services::ServiceProvider,
};
use tracing::warn;

use crate::guards::auth::AuthGuard;
use crate::helpers::jwt::JWT;
use crate::objects::GQLJWTData;

#[derive(Default)]
pub struct AccountQuery;

#[Object]
impl AccountQuery {
    /// Get account graph by address
    async fn account(
        &self,
        ctx: &Context<'_>,
        address: String,
    ) -> async_graphql::Result<AccountType> {
        let services = ctx.data_unchecked::<ServiceProvider>();
        let store_service = services.get_service_unchecked::<StoreService>().await;

        let pool = store_service.read();
        let account = AccountStore::try_find_by_address(pool, address)
            .await
            .map_err(|e| {
                warn!("Failed to get account: {e:?}");
                async_graphql::Error::new("Unable to fetch account with provided address")
            })?
            .ok_or(async_graphql::Error::new(
                "Account with provided address not found",
            ))?;

        Ok(account.into())
    }
}

#[derive(Default)]
pub struct AccountMutation;

#[Object]
impl AccountMutation {
    /// Exchange signature to token that can be used to authorize user
    pub async fn login_with_signature(
        &self,
        ctx: &Context<'_>,
        input: LoginSignatureInput,
    ) -> async_graphql::Result<AuthType> {
        let services = ctx.data_unchecked::<ServiceProvider>();
        let store = services.get_service_unchecked::<StoreService>().await;

        // Recover address of the signer
        let recovered_address =
            recover_address(&input.timestamp, &input.signature).map_err(|_| {
                async_graphql::Error::new("Failed to recover address from provided signature")
            })?;
        let recovered_address = format!("{recovered_address:?}");

        // Try to get signer, if user is new, it won't exist yet
        let account = {
            let pool = store.read();
            let account = AccountStore::try_find_by_address(pool, recovered_address.clone())
                .await
                .map_err(|e| {
                    warn!("{e:?}");
                    async_graphql::Error::from("Internal error")
                })?;

            if account.is_none() {
                let service = services.get_service_unchecked::<AccountService>().await;

                let mut db_tx = store.begin_transaction().await.map_err(|e| {
                    warn!("Failed to start transaction: {e:?}");
                    async_graphql::Error::new("Internal error")
                })?;
                
                let create_account_dto = CreateAccount {
                    address: recovered_address.clone(),
                    created_at: Utc::now(),
                };

                let account = service
                    .create_if_no_exists(create_account_dto, &mut db_tx)
                    .await?;

                store.commit_transaction(db_tx).await.map_err(|e| {
                    warn!("Failed to commit transaction: {e:?}");
                    async_graphql::Error::new("Internal error")
                })?;

                Some(account)
            } else {
                account
            }
        };

        let jwt = ctx.data_unchecked::<JWT>();
        Ok(AuthType {
            access_token: jwt.encode(recovered_address.clone(), None)?,
            address: recovered_address,
            account,
        })
    }

    /// Update current account with provided data
    #[graphql(guard = "AuthGuard::new()")]
    async fn update_account(
        &self,
        ctx: &Context<'_>,
        input: UpdateAccountInput,
    ) -> async_graphql::Result<AccountType, async_graphql::Error> {
        let services = ctx.data_unchecked::<ServiceProvider>();
        let store_service = services.get_service_unchecked::<StoreService>().await;

        let claims = ctx
            .data_opt::<GQLJWTData>()
            .and_then(|rd| rd.claims.as_ref())
            .ok_or(async_graphql::Error::from("Not authorized"))?;

        let pool = store_service.read();
        let account = AccountStore::try_find_by_address(pool, claims.sub.clone())
            .await
            .map_err(|_| async_graphql::Error::from("Internal error"))?
            .ok_or(async_graphql::Error::from("Account not found"))?;

        let dto = UpdateAccount {
            name: input.name,
            twitter: input.twitter,
            updated_at: Utc::now(),
            ..Default::default()
        };

        let account_service = services.get_service_unchecked::<AccountService>().await;
        let mut db_tx = store_service.begin_transaction().await.unwrap();

        let account = account_service
            .update(account.id, dto, &mut db_tx)
            .await
            .map_err(|_| async_graphql::Error::new("Failed to update account"))?;

        store_service.commit_transaction(db_tx).await.unwrap();

        Ok(account.into())
    }
}

#[derive(Default)]
pub struct AccountSubscription;

// #[Subscription]
// impl AccountSubscription {
//     async fn account_created(
//         &self,
//         ctx: &Context<'_>,
//     ) -> async_graphql::Result<impl Stream<Item = AccountType>> {
//         let services = ctx.data_unchecked::<ServiceProvider>();
//         let broadcast = services.get_service_unchecked::<BroadcastService>().await;

//         Ok(broadcast.subscribe().await.filter_map(|event| {
//             futures::future::ready(match event {
//                 Event::AccountCreated(event) => Some(event.into()),
//                 _ => None,
//             })
//         }))
//     }

//     async fn account_updated(
//         &self,
//         ctx: &Context<'_>,
//     ) -> async_graphql::Result<impl Stream<Item = AccountType>> {
//         let services = ctx.data_unchecked::<ServiceProvider>();
//         let broadcast = services.get_service_unchecked::<BroadcastService>().await;

//         Ok(broadcast.subscribe().await.filter_map(|event| {
//             futures::future::ready(match event {
//                 Event::AccountUpdated(event) => Some(event.into()),
//                 _ => None,
//             })
//         }))
//     }
// }
