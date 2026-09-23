use std::process::Command;

fn git_short_hash() -> String {
    if let Some(hash) = std::env::var("PULSAR_GIT_HASH")
        .ok()
        .map(|hash| hash.trim().to_string())
        .filter(|hash| !hash.is_empty())
    {
        return hash;
    }

    Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|hash| hash.trim().to_string())
        .filter(|hash| !hash.is_empty())
        .unwrap_or_else(|| "unknown".to_string())
}

fn main() {
    println!("cargo::rustc-check-cfg=cfg(coverage)");

    println!("cargo:rustc-env=PULSAR_GIT_HASH={}", git_short_hash());
    println!(
        "cargo:rustc-env=PULSAR_BUILD_TIME={}",
        chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ")
    );
    println!("cargo:rerun-if-env-changed=PULSAR_GIT_HASH");
    println!("cargo:rerun-if-changed=../.git/HEAD");
    println!("cargo:rerun-if-changed=../.git/refs");

    tauri_build::build()
}
