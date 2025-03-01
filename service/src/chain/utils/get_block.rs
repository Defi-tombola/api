use crate::chain::ChainClient;
use crate::config::service::ChainConfig;
use error_stack::{Report, Result, ResultExt};
use ethers::providers::Middleware;
use ethers::types::{U256, U64};
use lib::error::Error;
use std::cmp;
use std::collections::HashMap;
use std::sync::{Arc, OnceLock, RwLock};
use tracing::{debug, info};

static BLOCK_NUMBER_TREE: OnceLock<BlockNumberTree> = OnceLock::new();

struct BlockNumberTree {
    tree: Arc<RwLock<HashMap<String, BlockNumberNodeRef>>>,
}

impl BlockNumberTree {
    pub fn new() -> Self {
        BlockNumberTree {
            tree: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn add(&self, key: &str, block_number: U64, timestamp: U256) {
        let node = {
            let tree = self.tree.read().unwrap();
            tree.get(key).cloned()
        };

        match node {
            Some(node) => {
                node.write().unwrap().add(block_number, timestamp);
            }
            None => {
                self.tree.write().unwrap().insert(
                    key.to_string(),
                    Arc::new(RwLock::new(BlockNumberNode::new(block_number, timestamp))),
                );
            }
        };
    }

    pub fn find(&self, key: &str, timestamp: U256) -> Option<BlockNumberResult> {
        self.tree
            .read()
            .unwrap()
            .get(key)
            .map(|node| node.read().unwrap().find(timestamp))
    }
}

struct BlockNumberNode {
    value: BlockNumberNodeValue,
    left: Option<BlockNumberNodeRef>,
    right: Option<BlockNumberNodeRef>,
}

type BlockNumberNodeRef = Arc<RwLock<BlockNumberNode>>;

#[derive(Clone)]
struct BlockNumberNodeValue {
    timestamp: U256,
    block_number: Vec<U64>,
}

enum BlockNumberResult {
    Less(BlockNumberNodeValue),
    Greater(BlockNumberNodeValue),
    Exact(BlockNumberNodeValue),
}

impl BlockNumberNode {
    pub fn new(block_number: U64, timestamp: U256) -> Self {
        BlockNumberNode {
            value: BlockNumberNodeValue {
                timestamp,
                block_number: vec![block_number],
            },
            left: None,
            right: None,
        }
    }

    pub fn add(&mut self, block_number: U64, timestamp: U256) {
        match self.value.timestamp.cmp(&timestamp) {
            cmp::Ordering::Less => match self.left.as_ref() {
                Some(left) => {
                    left.write().unwrap().add(block_number, timestamp);
                }
                None => {
                    self.left = Some(Arc::new(RwLock::new(BlockNumberNode::new(
                        block_number,
                        timestamp,
                    ))));
                }
            },
            cmp::Ordering::Greater => match self.right.as_ref() {
                Some(right) => {
                    right.write().unwrap().add(block_number, timestamp);
                }
                None => {
                    self.right = Some(Arc::new(RwLock::new(BlockNumberNode::new(
                        block_number,
                        timestamp,
                    ))));
                }
            },
            cmp::Ordering::Equal => {
                self.value.block_number.push(block_number);
            }
        }
    }

    pub fn find(&self, timestamp: U256) -> BlockNumberResult {
        match self.value.timestamp.cmp(&timestamp) {
            cmp::Ordering::Less => match &self.left {
                Some(left) => left.read().unwrap().find(timestamp),
                None => BlockNumberResult::Less(self.value.clone()),
            },
            cmp::Ordering::Greater => match &self.right {
                Some(right) => right.read().unwrap().find(timestamp),
                None => BlockNumberResult::Greater(self.value.clone()),
            },
            cmp::Ordering::Equal => BlockNumberResult::Exact(self.value.clone()),
        }
    }
}

pub async fn get_timestamp_by_block(block: U64, chain: Arc<ChainClient>) -> Result<u64, Error> {
    let block = chain
        .get_block(block)
        .await
        .change_context(Error::Unknown)?;

    let Some(block) = block else {
        return Err(Report::from(Error::Unknown).attach_printable("Failed to get block"));
    };

    Ok(block.timestamp.as_u64())
}
/// Estimate block number by provided timestamp.
/// This function uses binary search algorithm to estimate block number by provided timestamp.
/// If max number of cycles will be reached, function will return the most closest block number.
pub async fn get_block_by_timestamp(
    chain: Arc<ChainClient>,
    config: &ChainConfig,
    timestamp: U256,
) -> Result<Option<U64>, Error> {
    let tree = BLOCK_NUMBER_TREE.get_or_init(|| BlockNumberTree::new());

    // Try to find block number by timestamp in the tree
    if let Some(BlockNumberResult::Exact(result)) = tree.find(&config.name, timestamp) {
        return Ok(Some(result.block_number.last().cloned().unwrap()));
    }

    // Use tree of block numbers to find closest block number to provided timestamp
    let (mut min_block_number, mut max_block_number) = match tree.find(&config.name, timestamp) {
        Some(BlockNumberResult::Less(item)) => {
            let latest_block_number: U64 = chain
                .get_block_number()
                .await
                .change_context(Error::Unknown)?;

            (
                item.block_number.first().cloned().unwrap(),
                latest_block_number,
            )
        }
        Some(BlockNumberResult::Greater(item)) => (
            U64::from(config.block_number),
            item.block_number.first().cloned().unwrap(),
        ),
        None => {
            let latest_block_number: U64 = chain
                .get_block_number()
                .await
                .change_context(Error::Unknown)?;

            (U64::from(config.block_number), latest_block_number)
        }
        // We've handled BlockNumberResult::Exact case above, and will return block number if
        // exact match will be found.
        _ => {
            unreachable!()
        }
    };

    let mut block_number = max_block_number;

    // Max number of tries until we give up on finding block by timestamp
    let mut tries: u8 = 30;

    loop {
        // Return the most closest result if we reached max number of tries
        if tries == 0 {
            break;
        }

        // Get delta between min and max block range, make sure there is at
        // least 2 block in between, otherwise it does not make sense to continue
        let block_number_delta = max_block_number - min_block_number;
        if block_number_delta <= U64::one() {
            break;
        }

        tries -= 1;

        block_number = block_number_delta / 2 + min_block_number;

        debug!("{tries}. min: {min_block_number}; max: {max_block_number}; block_number: {block_number}; chain: {chain}", chain = config.name);

        let block = chain
            .get_block(block_number)
            .await
            .change_context(Error::Unknown)?;

        if let Some(block) = block {
            // Cache mapping between block_number and timestamp for further use
            tree.add(&config.name, block_number, block.timestamp);

            match block.timestamp.cmp(&timestamp) {
                cmp::Ordering::Equal => {
                    break;
                }
                cmp::Ordering::Less => {
                    min_block_number = block_number;
                }
                cmp::Ordering::Greater => {
                    max_block_number = block_number;
                }
            }
        }
    }

    info!(
        "Searching for block number and timestamp {timestamp} took {tries} tries, result: {block_number}",
        tries = 30 - tries,
    );

    Ok(Some(block_number))
}
