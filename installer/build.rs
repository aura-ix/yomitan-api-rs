use std::process::Command;
use std::env;

// see https://zameermanji.com/blog/2021/6/17/embedding-a-rust-binary-in-another-rust-binary/
fn main() {
    let profile = env::var("PROFILE").unwrap();
    let mut args = vec!["build", "-p", "yomitan-api-rs-client"];
    if profile == "release" {
        args.push("--release");
    }
    
    let alt_target_dir = format!("{}/client-target", env::var("OUT_DIR").unwrap());
    
    let status = Command::new("cargo")
        .args(&args)
        .env("CARGO_TARGET_DIR", &alt_target_dir)
        .current_dir("..")
        .status()
        .expect("Failed to build client");
    
    if !status.success() {
        panic!("Client build failed");
    }
    
    let client_path = format!("{}/{}/yomitan-api-rs-client", alt_target_dir, profile);
    println!("cargo:rustc-env=CLIENT_BINARY_PATH={}", client_path);
    println!("cargo:rerun-if-changed=../client/src");
}