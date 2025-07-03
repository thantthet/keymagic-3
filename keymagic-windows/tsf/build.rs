use std::env;
use std::path::PathBuf;

fn main() {
    // Only build on Windows
    if env::var("CARGO_CFG_TARGET_OS").unwrap() != "windows" {
        return;
    }

    // Get the manifest directory
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let src_dir = PathBuf::from(&manifest_dir).join("src");

    // Compile C++ sources
    let mut build = cc::Build::new();
    build.cpp(true)
        .file(src_dir.join("Globals.cpp"))
        .file(src_dir.join("KeyMagicTextService.cpp"))
        .file(src_dir.join("DllMain.cpp"))
        .include(PathBuf::from(&manifest_dir).join("include"))
        .flag("/std:c++17");
    
    // Add debug flag in debug builds
    if env::var("PROFILE").unwrap_or_default() == "debug" {
        build.define("_DEBUG", None);
    }
    
    build.compile("keymagic_tsf");

    // Link Windows libraries
    println!("cargo:rustc-link-lib=user32");
    println!("cargo:rustc-link-lib=ole32");
    println!("cargo:rustc-link-lib=oleaut32");
    println!("cargo:rustc-link-lib=uuid");
    println!("cargo:rustc-link-lib=advapi32");

    // Use the module definition file
    println!("cargo:rustc-cdylib-link-arg=/DEF:{}", src_dir.join("keymagic.def").display());
}