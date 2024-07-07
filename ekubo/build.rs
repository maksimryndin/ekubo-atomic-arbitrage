use color_eyre::eyre::Result;
use std::env;
use std::process::Command;

const SPEC_NAME: &str = "openapi.yml";

fn main() -> Result<()> {
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

    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=.openapi-generator-ignore");
    println!("cargo:rerun-if-changed={current_dir}/{SPEC_NAME}");

    Ok(())
}
