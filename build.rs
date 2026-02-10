use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=src/updater.gresource.xml");
    println!("cargo:rerun-if-changed=src/window.ui");

    let status = Command::new("glib-compile-resources")
        .args([
            "--target=src/updater-new.gresource",
            "--sourcedir=src",
            "src/updater.gresource.xml",
        ])
        .status()
        .expect("Failed to execute glib-compile-resources");

    if !status.success() {
        panic!("glib-compile-resources failed with status {}", status);
    }
}
