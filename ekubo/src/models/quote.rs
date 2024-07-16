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

/// Quote : The suggested route(s) to get the best price
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct Quote {
    /// An input amount
    #[serde(rename = "specifiedAmount")]
        #[serde(deserialize_with = "crate::helpers::deserialize_felt_from_string")]
    pub specified_amount: starknet_core::types::Felt,
    /// The calculated amount for the quote
    #[serde(rename = "amount")]
    #[serde(deserialize_with = "crate::helpers::deserialize_felt_from_string")]
    pub amount: starknet_core::types::Felt,
    /// The list of pool keys through which to swap
    #[serde(rename = "route")]
    pub route: Vec<models::RouteNode>,
}

impl Quote {
    /// The suggested route(s) to get the best price
    pub fn new(specified_amount: starknet_core::types::Felt, amount: starknet_core::types::Felt, route: Vec<models::RouteNode>) -> Quote {
        Quote {
            specified_amount,
            amount,
            route,
        }
    }
}

