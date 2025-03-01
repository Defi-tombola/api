use ethers::types::{Address, H256};

pub trait ToHexString {
    fn to_hex_string(&self) -> String;
}

impl ToHexString for Address {
    fn to_hex_string(&self) -> String {
        format!("{:#x}", self)
    }
}

impl ToHexString for H256 {
    fn to_hex_string(&self) -> String {
        format!("{:#x}", self)
    }
}
