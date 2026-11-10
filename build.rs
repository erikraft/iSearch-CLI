use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-env-changed=ISEARCH_WINDOWS_ICON");

    if env::var("CARGO_CFG_TARGET_OS").ok().as_deref() != Some("windows") {
        return;
    }

    let Some(icon_value) = env::var_os("ISEARCH_WINDOWS_ICON") else {
        return;
    };

    let icon_path = PathBuf::from(icon_value)
        .canonicalize()
        .expect("ISEARCH_WINDOWS_ICON must point to an existing .ico file");
    println!("cargo:rerun-if-changed={}", icon_path.display());

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR is set by Cargo"));
    let rc_path = out_dir.join("isearch-icon.rc");
    let res_path = out_dir.join("isearch-icon.res");

    let escaped_icon_path = icon_path
        .to_string_lossy()
        .replace('\\', "\\\\")
        .replace('"', "\\\"");
    let resource = format!("1 ICON DISCARDABLE \"{}\"\n", escaped_icon_path);
    fs::write(&rc_path, resource).expect("failed to write Windows resource script");

    let output = Command::new("rc.exe")
        .arg("/nologo")
        .arg(format!("/fo{}", res_path.display()))
        .arg(&rc_path)
        .output()
        .expect("Windows SDK resource compiler rc.exe was not found on PATH");

    if !output.status.success() {
        panic!(
            "rc.exe failed to compile the native CMD icon resource:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // Pass the compiled Windows resource to the MSVC linker so the icon is
    // embedded in the executable itself (not only in its installer shortcut).
    println!("cargo:rustc-link-arg-bins={}", res_path.display());
}
