use crate::validators::timestamp::TimestampValidator;
use async_graphql::InputObject;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, InputObject)]
pub struct LoginSignatureInput {
    /// Timestamp used for the signature
    #[graphql(validator(custom = "TimestampValidator::new(30, 120)"))]
    pub timestamp: String,

    /// Signature for provided timestamp
    #[graphql(validator(min_length = 130, max_length = 130))]
    pub signature: String,
}

#[derive(Clone, Debug, Serialize, InputObject)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAccountInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[graphql(validator(min_length = 1, max_length = 50))]
    pub name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[graphql(validator(min_length = 1, max_length = 50))]
    pub slug: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[graphql(validator(min_length = 4, max_length = 50))]
    pub twitter: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[graphql(validator(min_length = 0, max_length = 120))]
    pub bio: Option<String>,
}

#[derive(Clone, Debug, Serialize, InputObject)]
pub struct LinkTwitterInput {
    pub code: String,
    pub state: String,
}