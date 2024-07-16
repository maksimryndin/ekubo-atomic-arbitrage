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

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct RouteNode {
    #[serde(rename = "pool_key")]
    pub pool_key: models::PoolKey,
    /// a limit on how far the price can move as part of the swap. Note this must always be specified, and must be between the maximum and minimum sqrt ratio. See also https://docs.ekubo.org/integration-guides/reference/reading-pool-price
    #[serde(rename = "sqrt_ratio_limit")]
    #[serde(deserialize_with = "crate::helpers::deserialize_felt_from_string")]
    pub sqrt_ratio_limit: starknet_core::types::Felt,
    /// A suggested skip_ahead value for gas optimizing the trade. It is an optimization parameter for large swaps across many uninitialized ticks to reduce the number of swap iterations that must be performed.
    #[serde(rename = "skip_ahead")]
    #[serde(deserialize_with = "crate::helpers::deserialize_felt_from_string")]
    pub skip_ahead: starknet_core::types::Felt,
}

impl RouteNode {
    pub fn new(pool_key: models::PoolKey, sqrt_ratio_limit: starknet_core::types::Felt, skip_ahead: starknet_core::types::Felt) -> RouteNode {
        RouteNode {
            pool_key,
            sqrt_ratio_limit,
            skip_ahead,
        }
    }
}

