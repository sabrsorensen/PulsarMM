fn main() {
    println!("cargo::rustc-check-cfg=cfg(coverage)");
    tauri_build::build()
}
