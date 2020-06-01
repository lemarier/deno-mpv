use std::env;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use tempfile::tempdir;

#[cfg(all(feature = "build_libmpv", target_os = "windows"))]
fn main() {
    let src_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let mut lib_path = Path::new(&src_dir).join("mpv_source");
    if env::var("CARGO_CFG_TARGET_POINTER_WIDTH").unwrap() == "64" {
        lib_path = Path::new(&lib_path).join("win32-x64");
    } else {
        lib_path = Path::new(&lib_path).join("win32");
    }
    println!("cargo:rustc-link-search={}", lib_path.display());
}

#[cfg(all(feature = "build_libmpv", target_os = "linux"))]
fn main() {}

#[cfg(all(feature = "build_libmpv", target_os = "macos"))]
fn main() {}

#[cfg(all(not(feature = "build_libmpv")))]
fn main() {}
