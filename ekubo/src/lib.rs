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
        amount: &str,
        token: &str,
        other_token: &str,
    ) -> Result<models::QuoteResponse> {
        Ok(apis::default_api::quote_amount_token_other_token_get(
            &self.configuration,
            amount,
            token,
            other_token,
        )
        .await?)
    }
}
