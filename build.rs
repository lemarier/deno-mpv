use std::env;
use std::path::Path;
use std::path::PathBuf;

#[cfg(all(feature = "build_libmpv", target_os = "windows"))]
fn main() {
    let src_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let mut lib_path = Path::new(&src_dir).join("MPV_SOURCE");
    if env::var("CARGO_CFG_TARGET_POINTER_WIDTH").unwrap() == "64" {
        lib_path = Path::new(&lib_path).join("64");
    } else {
        lib_path = Path::new(&lib_path).join("32");
    }
    println!("cargo:rustc-link-search={}", lib_path.display());
}

#[cfg(all(feature = "build_libmpv", not(target_os = "windows")))]
fn main() {}
