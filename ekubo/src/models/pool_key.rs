/*
 * Ekubo API Client
 *
 * Сlient for Ekubo AMM DEX.
 *
 * The version of the OpenAPI document: 0.1.0
 *
 * Generated by: https://openapi-generator.tech
 */

use crate::models;
use serde::{Deserialize, Serialize};

/// PoolKey : The composite key identifier for a pool in Ekubo
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct PoolKey {
    /// Address of token0
    #[serde(rename = "token0")]
    #[serde(deserialize_with = "crate::helpers::deserialize_felt_from_string")]
    pub token0: starknet_core::types::Felt,
    /// Address of token1
    #[serde(rename = "token1")]
    #[serde(deserialize_with = "crate::helpers::deserialize_felt_from_string")]
    pub token1: starknet_core::types::Felt,
    /// Address of fee
    #[serde(rename = "fee")]
    #[serde(deserialize_with = "crate::helpers::deserialize_felt_from_string")]
    pub fee: starknet_core::types::Felt,
    #[serde(rename = "tick_spacing")]
    pub tick_spacing: i32,
    /// extension id
    #[serde(rename = "extension")]
    #[serde(deserialize_with = "crate::helpers::deserialize_felt_from_string")]
    pub extension: starknet_core::types::Felt,
}

impl PoolKey {
    /// The composite key identifier for a pool in Ekubo
    pub fn new(
        token0: starknet_core::types::Felt,
        token1: starknet_core::types::Felt,
        fee: starknet_core::types::Felt,
        tick_spacing: i32,
        extension: starknet_core::types::Felt,
    ) -> PoolKey {
        PoolKey {
            token0,
            token1,
            fee,
            tick_spacing,
            extension,
        }
    }
}
