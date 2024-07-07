#[allow(unused_imports)]
#[allow(clippy::ref_patterns)]
#[allow(clippy::missing_inline_in_public_items)]
#[allow(clippy::pattern_type_mismatch)]
#[allow(clippy::missing_trait_methods)]
#[allow(clippy::unimplemented)]
#[allow(clippy::absolute_paths)]
#[allow(clippy::wildcard_enum_match_arm)]
#[allow(clippy::exhaustive_enums)]
#[allow(clippy::shadow_unrelated)]
#[allow(clippy::std_instead_of_alloc)]
#[allow(clippy::shadow_unrelated)]
#[allow(clippy::error_impl_error)]
pub mod apis;
#[allow(unused_imports)]
#[allow(clippy::ref_patterns)]
#[allow(clippy::missing_inline_in_public_items)]
#[allow(clippy::pattern_type_mismatch)]
#[allow(clippy::missing_trait_methods)]
#[allow(clippy::unimplemented)]
#[allow(clippy::absolute_paths)]
#[allow(clippy::wildcard_enum_match_arm)]
#[allow(clippy::exhaustive_enums)]
#[allow(clippy::std_instead_of_alloc)]
#[allow(clippy::shadow_unrelated)]
#[allow(clippy::error_impl_error)]
pub mod models;

use color_eyre::eyre::Result;

pub struct Client {
    configuration: apis::configuration::Configuration,
}

impl Client {
    #[inline]
    pub fn new(base_path: String, user_agent: String) -> Self {
        let configuration = apis::configuration::Configuration {
            base_path,
            user_agent: Some(user_agent),
            client: reqwest_middleware::ClientBuilder::new(reqwest::Client::new()).build(),
            basic_auth: None,
            oauth_access_token: None,
            bearer_access_token: None,
            api_key: None,
        };
        Self { configuration }
    }

    #[inline]
    pub async fn quote(
        &self,
        amount: u128,
        token: &str,
        other_token: &str,
    ) -> Result<models::Quote> {
        match apis::default_api::quote_amount_token_other_token_get(
            &self.configuration,
            &amount.to_string(),
            token,
            other_token,
            None,
            None,
        )
        .await?
        {
            models::QuoteResponse::Quote(q) => Ok(q),
            _ => panic!("assert: quote returns a single Quote"),
        }
    }

    #[inline]
    pub async fn quotes(
        &self,
        amount: u128,
        token: &str,
        other_token: &str,
        max_splits: u8,
        max_hops: u8,
    ) -> Result<models::Quotes> {
        match apis::default_api::quote_amount_token_other_token_get(
            &self.configuration,
            &amount.to_string(),
            token,
            other_token,
            Some(max_splits.into()),
            Some(max_hops.into()),
        )
        .await?
        {
            models::QuoteResponse::Quotes(q) => Ok(q),
            _ => panic!("assert: quote returns a few Quotes when params are provided"),
        }
    }
}
