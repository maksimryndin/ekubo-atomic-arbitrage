use color_eyre::eyre::Result;
use std::env;
use std::fs::{self, OpenOptions};
use std::io::{self, prelude::*};
use std::process::Command;

fn replace(path: &str, old: &str, new: &str) -> io::Result<()> {
    let contents = fs::read_to_string(path)?;
    let new_contents = contents.replace(old, new);
    let mut file = OpenOptions::new().write(true).truncate(true).open(path)?;
    file.write_all(new_contents.as_bytes())?;
    Ok(())
}

const SPEC_NAME: &str = "openapi.yml";

fn main() -> Result<()> {
    if std::env::var("EKUBO_API_REBUILD").is_ok() {
        let package_version = env!("CARGO_PKG_VERSION");
        let package_name = env!("CARGO_PKG_NAME");
        let current_dir = env!("CARGO_MANIFEST_DIR");
        // https://github.com/OpenAPITools/openapi-generator/blob/master/docs/generators/rust.md
        Command::new("docker")
            .arg("run")
            .arg("--rm")
            .arg("-v")
            .arg(&format!("{current_dir}:/local"))
            .arg("openapitools/openapi-generator-cli")
            .arg("generate")
            .arg("-i")
            .arg(&format!("/local/{SPEC_NAME}"))
            .arg("-g")
            .arg("rust")
            .arg("-o")
            .arg("/local")
            .arg("--additional-properties")
            .arg("library=reqwest")
            .arg("--additional-properties")
            .arg(&format!("packageName={package_name}"))
            .arg("--additional-properties")
            .arg(&format!("packageVersion={package_version}"))
            .arg("--additional-properties")
            .arg("preferUnsignedInt=true")
            .arg("--additional-properties")
            .arg("supportMiddleware=true")
            .arg("--additional-properties")
            .arg("avoidBoxedModels=true")
            .status()?;

        // sudo chown -R ${USER}:${USER} ekubo/src/apis/ ekubo/src/models/
        replace(
            "src/models/quote.rs",
            "amount: String",
            "amount: starknet_core::types::Felt",
        )?;
        replace(
            "src/models/quote.rs",
            r#"#[serde(rename = "specifiedAmount")]"#,
            r#"#[serde(rename = "specifiedAmount")]
        #[serde(deserialize_with = "crate::helpers::deserialize_felt_from_string")]"#,
        )?;
        replace(
            "src/models/quote.rs",
            r#"#[serde(rename = "amount")]"#,
            r#"#[serde(rename = "amount")]
    #[serde(deserialize_with = "crate::helpers::deserialize_felt_from_string")]"#,
        )?;

        replace(
            "src/models/quotes.rs",
            "total: String",
            "total: starknet_core::types::Felt",
        )?;
        replace(
            "src/models/quotes.rs",
            r#"#[serde(rename = "total")]"#,
            r#"#[serde(rename = "total")]
    #[serde(deserialize_with = "crate::helpers::deserialize_felt_from_string")]"#,
        )?;

        replace(
            "src/models/route_node.rs",
            "sqrt_ratio_limit: String",
            "sqrt_ratio_limit: starknet_core::types::Felt",
        )?;
        replace(
            "src/models/route_node.rs",
            r#"#[serde(rename = "sqrt_ratio_limit")]"#,
            r#"#[serde(rename = "sqrt_ratio_limit")]
    #[serde(deserialize_with = "crate::helpers::deserialize_felt_from_string")]"#,
        )?;
        replace(
            "src/models/pool_key.rs",
            "token0: String",
            "token0: starknet_core::types::Felt",
        )?;
        replace(
            "src/models/pool_key.rs",
            r#"#[serde(rename = "token0")]"#,
            r#"#[serde(rename = "token0")]
    #[serde(deserialize_with = "crate::helpers::deserialize_felt_from_string")]"#,
        )?;
        replace(
            "src/models/pool_key.rs",
            "token1: String",
            "token1: starknet_core::types::Felt",
        )?;
        replace(
            "src/models/pool_key.rs",
            r#"#[serde(rename = "token1")]"#,
            r#"#[serde(rename = "token1")]
    #[serde(deserialize_with = "crate::helpers::deserialize_felt_from_string")]"#,
        )?;
        replace(
            "src/models/pool_key.rs",
            "fee: String",
            "fee: starknet_core::types::Felt",
        )?;
        replace(
            "src/models/pool_key.rs",
            r#"#[serde(rename = "fee")]"#,
            r#"#[serde(rename = "fee")]
    #[serde(deserialize_with = "crate::helpers::deserialize_felt_from_string")]"#,
        )?;
        replace(
            "src/models/pool_key.rs",
            "extension: String",
            "extension: starknet_core::types::Felt",
        )?;
        replace(
            "src/models/pool_key.rs",
            r#"#[serde(rename = "extension")]"#,
            r#"#[serde(rename = "extension")]
    #[serde(deserialize_with = "crate::helpers::deserialize_felt_from_string")]"#,
        )?;

        println!("cargo:rerun-if-changed=Cargo.toml");
        println!("cargo:rerun-if-changed=build.rs");
        println!("cargo:rerun-if-changed=.openapi-generator-ignore");
        println!("cargo:rerun-if-changed={current_dir}/{SPEC_NAME}");
    }
    Ok(())
}
