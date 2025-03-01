use crate::events::*;
use crate::stream::{ChainEvent, ChainEventKind, Subscription, Transformer};
use chrono::Utc;
use error_stack::{Report, Result, ResultExt};
use ethers::abi::RawLog;
use ethers::prelude::EthEvent;
use lib::error::Error;
use tracing::info;

use super::subscription::ChainSubscription;

/// Implement [`Transformer`] for [`ChainSubscription`]
pub(crate) struct EventTransformer;

impl Transformer<ChainSubscription> for EventTransformer {
    fn transform(input: <ChainSubscription as Subscription>::Item) -> Result<ChainEvent, Error> {
        let Some(signature) = input.topics.first() else {
            return Err(Report::new(Error::TransformNoSignature));
        };
        
        // Match signature to event
        let kind = match signature {
            value if LotteryOpened::signature().eq(value) => ChainEventKind::LotteryOpened(
                LotteryOpened::decode_log(&input.clone().into())
                    .change_context(Error::EventDecodeFailed)?,
            ),
            value if LotteryClosed::signature().eq(value) => ChainEventKind::LotteryClosed(
                LotteryClosed::decode_log(&input.clone().into())
                    .change_context(Error::EventDecodeFailed)?,
            ),
            value if TicketBought::signature().eq(value) => ChainEventKind::TicketBought(
                TicketBought::decode_log(&input.clone().into())
                    .change_context(Error::EventDecodeFailed)?,
            ),
            value if WinnerPaid::signature().eq(value) => ChainEventKind::WinnerPaid(
                WinnerPaid::decode_log(&input.clone().into())
                    .change_context(Error::EventDecodeFailed)?,
            ),
            value if LotteryNumberGenerated::signature().eq(value) => {
                ChainEventKind::LotteryNumberGenerated(
                    LotteryNumberGenerated::decode_log(&input.clone().into())
                        .change_context(Error::EventDecodeFailed)?,
                )
            }
            value if LotteryCanceled::signature().eq(value) => ChainEventKind::LotteryCanceled(
                LotteryCanceled::decode_log(&input.clone().into())
                    .change_context(Error::EventDecodeFailed)?,
            ),
            value if FeeCollected::signature().eq(value) => ChainEventKind::FeeCollected(
                FeeCollected::decode_log(&input.clone().into())
                    .change_context(Error::EventDecodeFailed)?,
            ),
            _ => return Err(Report::new(Error::TransformUnknownSignature)),
        };

        // Validate required fields
        let Some(block_number) = input.block_number else {
            return Err(Report::new(Error::TransformNoBlockNumber));
        };

        let Some(transaction_hash) = input.transaction_hash else {
            return Err(Report::new(Error::TransformNoTransactionHash));
        };

        let Some(log_index) = input.log_index else {
            return Err(Report::new(Error::TransformNoLogIndex));
        };

        Ok(ChainEvent {
            block_number,
            log_index,
            transaction_hash,
            src_address: input.address,
            dst_address: input.address,
            kind,
            triggered_at: Utc::now(), // This value will be overriden on stream
        })
    }
}