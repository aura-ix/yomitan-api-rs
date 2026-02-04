use std::process::Command;
use std::env;
use std::path::{Path, PathBuf};

// see https://zameermanji.com/blog/2021/6/17/embedding-a-rust-binary-in-another-rust-binary/
fn main() {
    let profile = env::var("PROFILE").unwrap();

    let mut args = vec!["build", "-p", "yomitan-api-rs-client"];
    if profile == "release" {
        args.push("--release");
    }

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let alt_target_dir = out_dir.join("client-target");

    let status = Command::new("cargo")
        .args(&args)
        .env("CARGO_TARGET_DIR", &alt_target_dir)
        .current_dir(Path::new(".."))
        .status()
        .expect("Failed to build client");

    if !status.success() {
        panic!("Client build failed");
    }

    let mut _client_path = alt_target_dir.join(&profile).join("yomitan-api-rs-client");

    #[cfg(windows)]
    {
        _client_path.set_extension("exe");
    }

    println!(
        "cargo:rustc-env=CLIENT_BINARY_PATH={}",
        _client_path.display()
    );
    println!("cargo:rerun-if-changed=../client/src");
}