use std::env;
use std::path::PathBuf;

fn main() {
    if cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();
        
        // Get the path to the icon file relative to the build script location
        let mut icon_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
        icon_path.push("..");
        icon_path.push("resources");
        icon_path.push("icons");
        icon_path.push("keymagic.ico");
        
        println!("cargo:warning=Icon path: {}", icon_path.display());
        
        // Set the executable icon explicitly
        res.set_icon(icon_path.to_str().unwrap());
        
        // Set manifest directly
        let mut manifest_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
        manifest_path.push("resources");
        manifest_path.push("app.manifest");
        res.set_manifest_file(manifest_path.to_str().unwrap());
        
        // Compile the resources
        res.compile().expect("Failed to compile Windows resources");
    }
}