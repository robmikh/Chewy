use std::path::PathBuf;
use std::process::*;

fn main() -> std::io::Result<()> {
    println!("cargo:rerun-if-changed=src/Chewy.idl");
    let _ = std::fs::remove_file("src/bindings.rs");
    let _ = std::fs::remove_file("metadata/Chewy.winmd");

    let windows_sdk_dir = std::env::var("WindowsSdkDir").expect("Failed to find Windows SDK path!");
    let windows_sdk_version =
        std::env::var("WindowsSDKVersion").expect("Failed to find Windows SDK version!");

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
        if !path.exists() {
            std::fs::create_dir(&path).unwrap();
        }
        path.push("Chewy.winmd");
        path
    };

    // For whatever reason, midlrt doesn't like trailing slashes
    let metadata_dir = {
        let mut metadata_dir = format!("{}", metadata_dir.display());
        if metadata_dir.ends_with("\\") {
            metadata_dir.remove(metadata_dir.len() - 1);
        }
        metadata_dir
    };

    let status = Command::new("midlrt.exe")
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

    if !status.success() {
        panic!("Failed to generate metadata!")
    }

    windows_bindgen::bindgen([
        "--in",
        output_winmd.to_str().unwrap(),
        windows_path.to_str().unwrap(),
        "--out",
        "src/bindings.rs",
        "--filter",
        "Chewy",
        "--config",
        "implement",
    ])
    .unwrap();

    Command::new("rustfmt").arg("src/bindings.rs").status()?;
    Ok(())
}
