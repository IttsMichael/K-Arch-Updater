use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=src/updater.gresource.xml");
    println!("cargo:rerun-if-changed=src/window.ui");

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let status = Command::new("glib-compile-resources")
        .args([
            &format!("--target={}/updater-new.gresource", out_dir),
            "--sourcedir=src",
            "src/updater.gresource.xml",
        ])
        .status()
        .expect("Failed to execute glib-compile-resources");

    if !status.success() {
        panic!("glib-compile-resources failed with status {}", status);
    }
}
