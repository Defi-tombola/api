// mod inputs;

// use self::inputs::SimulateTransactionInput;
// use async_graphql::{Context, Object, Subscription};
// use futures::Stream;
// use service::tenderly::types::SimulationType;
// use service::{
//     discord::internal::DiscordInternalBot,
//     services::ServiceProvider,
//     tenderly::{service::TenderlyService, types::SimulateTransactionDTO},
// };
// use tracing::warn;

// #[derive(Default)]
// pub struct SystemQuery;

// #[Object]
// impl SystemQuery {
//     async fn version(&self) -> String {
//         env!("CARGO_PKG_VERSION").to_string()
//     }
// }

// #[derive(Default)]
// pub struct SystemMutation;

// #[Object]
// impl SystemMutation {
//     /// Using Tenderly generate link for transaction simulation
//     async fn simulate_transaction(
//         &self,
//         ctx: &Context<'_>,
//         input: SimulateTransactionInput,
//     ) -> async_graphql::Result<String, async_graphql::Error> {
//         let services = ctx.data_unchecked::<ServiceProvider>();

//         let tenderly_service = services.get_service_unchecked::<TenderlyService>().await;

//         let simulation = tenderly_service
//             .simulate(&SimulateTransactionDTO {
//                 save: true,
//                 save_if_fails: true,
//                 simulation_type: SimulationType::Full,
//                 from: input.from,
//                 to: input.to,
//                 input: input.input,
//                 value: Some(input.value),
//                 gas: Some(input.gas),
//                 gas_price: Some(input.gas_price),
//                 network_id: input.network_id,
//             })
//             .await
//             .map_err(|e| {
//                 warn!("{:?}", e);
//                 async_graphql::Error::new("Failed to analyze transaction")
//             })?;

//         let discord_service = services.get_service_unchecked::<DiscordInternalBot>().await;

//         let link = format!(
//             "https://dashboard.tenderly.co/project/{project}/simulator/{id}",
//             project = simulation.project_id,
//             id = simulation.id
//         );

//         discord_service.simulation(&simulation).await
//             .map_err(|_| async_graphql::Error::new(format!("Simulation has been executed, but failed to send message to Discord. Here is the simulation link {link}")))?;

//         Ok(link)
//     }
// }

// #[derive(Default)]
// pub struct SystemSubscription;

// #[Subscription]
// impl SystemSubscription {
//     async fn ping(&self, _ctx: &Context<'_>) -> async_graphql::Result<impl Stream<Item = String>> {
//         Ok(futures::stream::once(async { "PONG".to_string() }))
//     }
// }
