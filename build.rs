use std::fs::*;
use std::path::PathBuf;
use std::process::*;

fn main() -> std::io::Result<()> {
    println!("cargo:rerun-if-changed=src/Chewy.idl");
    let _ = std::fs::remove_file("src/bindings.rs");
    let _ = std::fs::remove_file("Chewy.winmd");

    let windows_sdk_dir = std::env::var("WindowsSdkDir").expect("Failed to find Windows SDK path!");
    let windows_sdk_version = std::env::var("WindowsSDKVersion").expect("Failed to find Windows SDK version!");

    let metadata_dir = {
        let mut path = PathBuf::from(&windows_sdk_dir);
        path.push("UnionMetadata");
        path.push(windows_sdk_version);
        path
    };

    let windows_path = {
        let mut path = metadata_dir.clone();
        path.push("Windows.winmd");
        path
    };

    let output_winmd = {
        let mut path = PathBuf::from("metadata");
        path.push("Chewy.winmd");
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
        .arg("src/Chewy.idl")
        .status()?;

    let files = vec![
        windows_metadata::reader::File::new(windows_path.to_str().unwrap())?,
        windows_metadata::reader::File::new(output_winmd.to_str().unwrap())?,
    ];

    write(
        "src/bindings.rs",
        windows_bindgen::component("Chewy", &files),
    )?;

    Command::new("rustfmt").arg("src/bindings.rs").status()?;
    Ok(())
}
