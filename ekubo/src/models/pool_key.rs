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
pub struct PoolKey {
    /// Address of token0
    #[serde(rename = "token0")]
    pub token0: String,
    /// Address of token1
    #[serde(rename = "token1")]
    pub token1: String,
    /// Address of fee
    #[serde(rename = "fee")]
    pub fee: String,
    #[serde(rename = "tick_spacing")]
    pub tick_spacing: i32,
    /// extension id
    #[serde(rename = "extension")]
    pub extension: String,
}

impl PoolKey {
    pub fn new(
        token0: String,
        token1: String,
        fee: String,
        tick_spacing: i32,
        extension: String,
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
