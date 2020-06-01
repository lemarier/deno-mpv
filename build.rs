use std::env;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

#[cfg(all(feature = "build_libmpv", target_os = "windows"))]
fn main() {
    let mut src_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    src_dir = Path::new(&src_dir).join("mpv_source");
    fs::create_dir_all(&src_dir);

    let root_url = "https://github.com/lemarier/libmpv/releases/download/v0.0.7";

    let archive_dir = Path::new(&src_dir).join("archive.zip");
    let mut archive_file = fs::File::create(archive_dir).unwrap();
    let mut download_url = [root_url, "/win32.zip"].join("");
    let mut arch_type = "X86";

    if env::var("CARGO_CFG_TARGET_POINTER_WIDTH").unwrap() == "64" {
        download_url = [root_url, "/win32-x64.zip"].join("");
        arch_type = "X64";
    }

    reqwest::get(&download_url)
        .unwrap()
        .copy_to(&mut archive_file);

    let mut main_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let mut out_dir = format!("{}\\mpv_source", main_dir);
    let archive_file = format!("{}\\archive.zip", out_dir);

    Command::new("7z")
        .args(&[
            "x".to_string(),
            archive_file,
            ["-o".to_string(), out_dir].join(""),
        ])
        .output()
        .expect("failed to unzip archive");

    out_dir = format!("{}\\mpv_source", main_dir);
    Command::new("dir")
        .args(&[out_dir])
        .output()
        .expect("failed to dir");

    out_dir = format!("{}\\mpv_source", main_dir);
    println!("cargo:rustc-link-search={}", out_dir);
}

#[cfg(all(feature = "build_libmpv", target_os = "linux"))]
fn main() {}

#[cfg(all(feature = "build_libmpv", target_os = "macos"))]
fn main() {}

#[cfg(all(not(feature = "build_libmpv")))]
fn main() {}
