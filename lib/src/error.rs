use ethers::types::{Address, Bytes, U256};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("CLI command is unknown")]
    UnknownCLICommand,

    #[error("Ticket invalid fee")]
    TicketServiceInvalidFee,
    
    #[error("Stream error")]
    Stream,

    #[error("Failed to start chain")]
    ChainStart,

    #[error("Configuration file not found: {0}")]
    ConfigNotFound(String),

    #[error("Invalid configuration file format")]
    ConfigInvalid,

    #[error("Store error")]
    Store,

    #[error("Store transaction failed")]
    StoreTransactionFailed,

    #[error("Store insert failed")]
    StoreInsertFailed,

    #[error("Store update failed")]
    StoreUpdateFailed,

    #[error("Store is not available")]
    StoreNotAvailable,

    #[error("Failed to migrate store")]
    StoreMigration,

    #[error("Failed to create store collection")]
    FailedCreateStoreCollection,

    #[error("Failed to create store index")]
    FailedCreateStoreIndex,

    #[error("Entity already exists")]
    AlreadyExists,

    #[error("Failed to parse event log")]
    FailedToParseEventLog,

    #[error("Undefined log parameter")]
    UndefinedLogParam,

    #[error("Error during contract query")]
    ContractQuery,

    #[error("Contract doesn't have event")]
    ContractEventNotExist,

    #[error("Error occurred while processing event")]
    EventsRunner,

    #[error("GraphQL mutation failed")]
    GraphQLMutation,

    #[error("Invalid signature")]
    InvalidSignature,

    #[error("Failed to serialize provided object")]
    SerdeSerialize,

    #[error("Failed to deserialize provided object")]
    SerdeDeserialize,

    #[error("Redis Error")]
    Redis,

    #[error("Failed to connect to redis")]
    RedisConnect,

    #[error("AWS S3 error")]
    AWSS3,

    #[error("Resource not found")]
    NotFound,

    #[error("Tenderly request failed")]
    TenderlyRequestFailed,

    #[error("Tenderly invalid response")]
    TenderlyInvalidResponse,

    #[error("Discord error")]
    Discord,

    #[error("Reqwest error")]
    Reqwest,

    #[error("Invalid private key")]
    InvalidPrivateKey,

    #[error("Failed to decode event")]
    EventDecodeFailed,

    #[error("Event doesn't have signature")]
    TransformNoSignature,

    #[error("Event doesn't have block number")]
    TransformNoBlockNumber,

    #[error("Event does't have related transformer")]
    TransformUnknownSignature,

    #[error("Event doesn't have transaction hash")]
    TransformNoTransactionHash,

    #[error("Event doesn't have log index")]
    TransformNoLogIndex,

    #[error("Chain provider state is not synced")]
    ChainStateNotSynced,

    #[error("Contract error")]
    Contract,

    #[error("Chain event has already been processed")]
    ChainEventAlreadyProcessed,

    #[error("Unknown error")]
    Unknown,

    #[error("Twitter invalid state")]
    TwitterInvalidState,
}

pub struct ContractCallError {
    pub from: Address,
    pub to: Address,
    pub data: Bytes,
    pub value: U256,
    pub gas: Option<U256>,
    pub gas_price: Option<U256>,
    pub message: Option<String>,
}
