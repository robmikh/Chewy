use std::fs::*;
use std::path::PathBuf;
use std::process::*;

fn main() -> std::io::Result<()> {
    println!("cargo:rerun-if-changed=src/component.idl");
    let _ = std::fs::remove_file("src/bindings.rs");
    let _ = std::fs::remove_file("component.winmd");

    let windows_sdk_dir = std::env::var("WindowsSdkDir").expect("Failed to find Windows SDK path!");
    let target_windows_sdk = "10.0.20348.0";

    let metadata_dir = {
        let mut path = PathBuf::from(&windows_sdk_dir);
        path.push("UnionMetadata");
        path.push(target_windows_sdk);
        path
    };

    let windows_path = {
        let mut path = metadata_dir.clone();
        path.push("Windows.winmd");
        path
    };

    let output_winmd = {
        let out_dir = std::env::var("OUT_DIR").expect("Failed to get output directory!");
        let mut path = PathBuf::from(out_dir);
        path.push("component.winmd");
        path
    };

    Command::new("midlrt.exe")
        .arg("/winrt")
        .arg("/nomidl")
        .arg("/h")
        .arg("nul")
        .arg("/metadata_dir")
        .arg(metadata_dir)
        .arg("/reference")
        .arg(&windows_path)
        .arg("/winmd")
        .arg(&output_winmd)
        .arg("src/component.idl")
        .status()?;

    let files = vec![
        windows_metadata::reader::File::new(windows_path.to_str().unwrap())?,
        windows_metadata::reader::File::new(output_winmd.to_str().unwrap())?,
    ];

    write(
        "src/bindings.rs",
        windows_bindgen::component("Component", &files),
    )?;

    Command::new("rustfmt").arg("src/bindings.rs").status()?;
    Ok(())
}
