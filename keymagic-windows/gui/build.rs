fn main() {
    if cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();
        // Only set icon if it exists
        if std::path::Path::new("resources/keymagic.ico").exists() {
            res.set_icon("resources/keymagic.ico");
        }
        res.set_manifest_file("resources/app.manifest");
        if let Err(e) = res.compile() {
            eprintln!("Warning: Failed to compile resources: {}", e);
            // Continue build even if resource compilation fails
        }
    }
}