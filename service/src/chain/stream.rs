use ethers::types::{Block, Log, H256};

// TODO: Deprecated

#[derive(Clone, Debug)]
pub struct StreamEvent {
    pub chain: String,
    pub block: Block<H256>,
    pub log: Log,
}

impl StreamEvent {
    /// Returns a mock of StreamEvent
    pub fn mock() -> Self {
        Self {
            chain: "Optimism".to_string(),
            block: Block {
                ..Default::default()
            },
            log: Log {
                ..Default::default()
            },
        }
    }
}
