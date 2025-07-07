fn main() {
    // Compile Slint files
    let config = slint_build::CompilerConfiguration::new();
    slint_build::compile_with_config("ui/main_window.slint", config).unwrap();
    
    // Windows resources (icon, manifest)
    #[cfg(target_os = "windows")]
    {
        let mut res = winres::WindowsResource::new();
        
        // Try to set icon if it exists
        let icon_path = "../resources/icons/keymagic.ico";
        if std::path::Path::new(icon_path).exists() {
            res.set_icon(icon_path);
            println!("cargo:rerun-if-changed={}", icon_path);
        } else {
            println!("cargo:warning=Icon not found at: {}", icon_path);
        }
        
        // Set application manifest
        res.set_manifest(r#"
<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
    <assemblyIdentity
        version="1.0.0.0"
        processorArchitecture="*"
        name="KeyMagic.ConfigurationManager"
        type="win32"
    />
    <description>KeyMagic Configuration Manager</description>
    <dependency>
        <dependentAssembly>
            <assemblyIdentity
                type="win32"
                name="Microsoft.Windows.Common-Controls"
                version="6.0.0.0"
                processorArchitecture="*"
                publicKeyToken="6595b64144ccf1df"
                language="*"
            />
        </dependentAssembly>
    </dependency>
    <application xmlns="urn:schemas-microsoft-com:asm.v3">
        <windowsSettings>
            <dpiAware xmlns="http://schemas.microsoft.com/SMI/2005/WindowsSettings">true/PM</dpiAware>
            <dpiAwareness xmlns="http://schemas.microsoft.com/SMI/2016/WindowsSettings">PerMonitorV2</dpiAwareness>
        </windowsSettings>
    </application>
</assembly>
        "#);
        
        res.compile().unwrap();
    }
}