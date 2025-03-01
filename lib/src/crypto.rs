use std::str::FromStr;

use error_stack::{Result, ResultExt};
use ethers::{
    abi::Address,
    types::{RecoveryMessage, Signature, H160},
};
use serde::Serialize;

use crate::error::Error;

pub fn recover_address(message: &str, signature: &str) -> Result<Address, Error> {
    let signature = Signature::from_str(signature).change_context(Error::InvalidSignature)?;

    signature
        .recover(RecoveryMessage::Data(message.as_bytes().to_vec()))
        .change_context(Error::InvalidSignature)
}

pub fn verify_address<T>(address: &H160, message: &T, signature: &str) -> Result<bool, Error>
where
    T: Serialize,
{
    let message = serde_json::to_string(&message).change_context(Error::SerdeSerialize)?;

    let recovered_address = recover_address(&message, signature)?;
    Ok(recovered_address.eq(address))
}
