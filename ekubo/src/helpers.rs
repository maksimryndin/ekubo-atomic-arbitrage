use serde::de::Error;
use serde::{Deserialize, Deserializer};
use starknet_core::types::Felt;

pub fn deserialize_felt_from_string<'de, D>(deserializer: D) -> Result<Felt, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s.starts_with("0x") {
        Felt::from_hex(&s).map_err(Error::custom)
    } else {
        Felt::from_dec_str(&s).map_err(Error::custom)
    }
}
