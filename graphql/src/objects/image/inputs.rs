use async_graphql::{Enum, InputObject};
use serde::Serialize;

#[derive(Clone, Debug, Serialize, InputObject)]
#[serde(rename_all = "camelCase")]
pub struct ImageUploadInput {
    /// Entity identifier, may have different meaning for different entity type:
    /// Vault   - UID (VaultId)
    /// Account - Address of account
    #[graphql(validator(min_length = 20, max_length = 100))]
    pub entity_id: String,

    /// Entity type
    pub entity_type: EntityType,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug, Serialize)]
pub enum EntityType {
    Account,
    Vault,
    Cover,
}
