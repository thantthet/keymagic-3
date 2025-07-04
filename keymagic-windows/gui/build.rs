fn main() {
    if cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();
        
        // Set the main resource file which includes icons and manifest
        res.set_resource_file("resources/app.rc");
        
        // Compile the resources
        if let Err(e) = res.compile() {
            eprintln!("Warning: Failed to compile resources: {}", e);
            // Continue build even if resource compilation fails
        }
    }
}