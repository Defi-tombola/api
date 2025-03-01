use bigdecimal::BigDecimal;
use serde::{Deserialize, Deserializer};

/// Parse a hex string as a BigDecimal
pub fn deserialize_hex_string_to_bigdecimal<'de, D>(deserializer: D) -> Result<BigDecimal, D::Error>
where
    D: Deserializer<'de>,
{
    let hex_string = String::deserialize(deserializer)?;
    let bytes = hex::decode(hex_string).map_err(serde::de::Error::custom)?;
    let decimal = BigDecimal::parse_bytes(&bytes, 10).ok_or(serde::de::Error::custom(
        "Failed to parse hex string as decimal",
    ))?;

    Ok(decimal)
}
