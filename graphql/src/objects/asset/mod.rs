pub mod types;

use self::types::{AssetType};
use async_graphql::{Context, Object, Subscription};
use futures::{Stream, StreamExt};
use service::asset::store::AssetStore;
use service::services::ServiceProvider;
use service::{prelude::StoreService};
use tracing::warn;


#[derive(Default)]
pub struct AssetQuery;

#[Object]
impl AssetQuery {
    /// Get list of all supported assets
    async fn assets(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<AssetType>> {
        let services = ctx.data_unchecked::<ServiceProvider>();
        let store_service = services.get_service_unchecked::<StoreService>().await;

        let pool = store_service.read();
        let assets = AssetStore::find_all(pool).await.map_err(|e| {
            warn!("Failed to assets: {e:?}");
            async_graphql::Error::from("Internal error")
        })?;

        Ok(assets.into_iter().map(Into::into).collect())
    }
}

#[derive(Default)]
pub struct AssetSubscription;

// #[Subscription]
// impl AssetSubscription {
//     async fn asset_created(
//         &self,
//         ctx: &Context<'_>,
//     ) -> async_graphql::Result<impl Stream<Item = AssetType>> {
//         let services = ctx.data_unchecked::<ServiceProvider>();
//         let broadcast_service = services.get_service_unchecked::<BroadcastService>().await;

//         Ok(broadcast_service.subscribe().await.filter_map(|event| {
//             futures::future::ready(match event {
//                 Event::AssetCreated(event) => Some(event.into()),
//                 _ => None,
//             })
//         }))
//     }

//     async fn asset_updated(
//         &self,
//         ctx: &Context<'_>,
//     ) -> async_graphql::Result<impl Stream<Item = AssetType>> {
//         let services = ctx.data_unchecked::<ServiceProvider>();
//         let broadcast_service = services.get_service_unchecked::<BroadcastService>().await;

//         Ok(broadcast_service.subscribe().await.filter_map(|event| {
//             futures::future::ready(match event {
//                 Event::AssetUpdated(event) => Some(event.into()),
//                 _ => None,
//             })
//         }))
//     }

//     async fn asset_pair_price(
//         &self,
//         ctx: &Context<'_>,
//         input: AssetPairInput,
//     ) -> async_graphql::Result<impl Stream<Item = AssetPairPriceType>> {
//         let services = ctx.data_unchecked::<ServiceProvider>();
//         let broadcast_service = services.get_service_unchecked::<BroadcastService>().await;
//         let store_service = services.get_service_unchecked::<StoreService>().await;

//         Ok(broadcast_service
//             .subscribe()
//             .await
//             .filter_map(move |event| {
//                 let input = input.clone();
//                 let store_service = store_service.clone();

//                 async move {
//                     match event.clone() {
//                         // TODO: We are not checking that price has changed, so should add
//                         // this check.
//                         Event::AssetUpdated(event) => {
//                             if event.chain != input.chain.to_string()
//                                 || (event.symbol != input.first_asset
//                                     && event.symbol != input.second_asset)
//                             {
//                                 return None;
//                             }

//                             let (first_asset, second_asset) = if event.symbol == input.first_asset {
//                                 let first_asset = event;
//                                 let pool = store_service.read();
//                                 let second_asset = AssetStore::try_find_by_symbol_and_chain(
//                                     pool,
//                                     &input.second_asset,
//                                     &input.chain.to_string(),
//                                 )
//                                 .await
//                                 .ok()??;

//                                 Some((first_asset, second_asset))
//                             } else {
//                                 let second_asset = event;
//                                 let pool = store_service.read();
//                                 let first_asset = AssetStore::try_find_by_symbol_and_chain(
//                                     pool,
//                                     &input.first_asset,
//                                     &input.chain.to_string(),
//                                 )
//                                 .await
//                                 .ok()??;

//                                 Some((first_asset, second_asset))
//                             }?;

//                             Some(AssetPairPriceType::new(first_asset, second_asset))
//                         }
//                         _ => None,
//                     }
//                 }
//             }))
//     }

//     async fn asset_linked(
//         &self,
//         ctx: &Context<'_>,
//     ) -> async_graphql::Result<impl Stream<Item = AssetLinkType>> {
//         let services = ctx.data_unchecked::<ServiceProvider>();
//         let broadcast_service = services.get_service_unchecked::<BroadcastService>().await;

//         Ok(broadcast_service.subscribe().await.filter_map(|event| {
//             futures::future::ready(match event {
//                 Event::AssetLinked(event) => Some(event.into()),
//                 _ => None,
//             })
//         }))
//     }
// }
