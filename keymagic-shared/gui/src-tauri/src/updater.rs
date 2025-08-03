use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use semver::Version;
use std::collections::HashMap;

const UPDATE_JSON_URL: &str = "https://thantthet.github.io/keymagic-3/updates.json";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateInfo {
    pub current_version: String,
    pub latest_version: String,
    pub update_available: bool,
    pub download_url: Option<String>,
    pub release_notes: Option<String>,
    pub published_at: Option<String>,
}

#[derive(Debug, Deserialize)]
struct UpdateManifest {
    #[allow(dead_code)]
    name: String,
    platforms: HashMap<String, HashMap<String, PlatformRelease>>,
    #[serde(rename = "releaseNotes")]
    release_notes: HashMap<String, HashMap<String, String>>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum PlatformRelease {
    // Windows/macOS format
    Direct {
        version: String,
        #[serde(rename = "releaseDate")]
        release_date: Option<String>,
        #[serde(rename = "minimumSystemVersion")]
        #[allow(dead_code)]
        minimum_system_version: Option<String>,
        url: String,
        #[allow(dead_code)]
        signature: Option<String>,
        #[allow(dead_code)]
        size: Option<u64>,
        #[allow(dead_code)]
        sha256: Option<String>,
    },
    // Linux format with packages
    Linux {
        version: String,
        #[serde(rename = "releaseDate")]
        release_date: Option<String>,
        #[serde(rename = "minimumSystemVersion")]
        #[allow(dead_code)]
        minimum_system_version: Option<String>,
        packages: HashMap<String, PackageInfo>,
    }
}

#[derive(Debug, Deserialize)]
struct PackageInfo {
    url: String,
    #[allow(dead_code)]
    size: Option<u64>,
    #[allow(dead_code)]
    sha256: Option<String>,
}

fn get_platform_info() -> (&'static str, &'static str) {
    #[cfg(target_os = "windows")]
    let os = "windows";
    #[cfg(target_os = "macos")]
    let os = "macos";
    #[cfg(target_os = "linux")]
    let os = "linux";
    
    #[cfg(target_arch = "x86_64")]
    let arch = "x86_64";
    #[cfg(target_arch = "aarch64")]
    let arch = "aarch64";
    #[cfg(all(target_arch = "arm", target_pointer_width = "32"))]
    let arch = "armv7";
    #[cfg(target_arch = "x86")]
    let arch = "x86";
    
    (os, arch)
}


#[cfg(target_os = "linux")]
fn determine_linux_package_type() -> &'static str {
    // Check if running on a Debian-based system
    if std::path::Path::new("/etc/debian_version").exists() {
        return "deb";
    }
    
    // Check if running on a Red Hat-based system
    if std::path::Path::new("/etc/redhat-release").exists() || 
       std::path::Path::new("/etc/fedora-release").exists() ||
       std::path::Path::new("/etc/centos-release").exists() {
        return "rpm";
    }
    
    // Check for dpkg command (Debian/Ubuntu)
    if std::process::Command::new("which")
        .arg("dpkg")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false) {
        return "deb";
    }
    
    // Check for rpm command (Red Hat/Fedora)
    if std::process::Command::new("which")
        .arg("rpm")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false) {
        return "rpm";
    }
    
    // Default to deb for unknown systems
    "deb"
}

#[cfg(not(target_os = "linux"))]
fn determine_linux_package_type() -> &'static str {
    // This function should never be called on non-Linux systems
    unreachable!("determine_linux_package_type called on non-Linux system")
}

async fn fetch_update_manifest() -> Result<UpdateManifest> {
    // Build user agent with current version and platform info
    let (os, arch) = get_platform_info();
    let user_agent = format!(
        "KeyMagic/{} ({}/{})",
        env!("CARGO_PKG_VERSION"),
        os,
        arch
    );
    
    let client = reqwest::Client::builder()
        .user_agent(user_agent)
        .build()?;
    
    let response = client
        .get(UPDATE_JSON_URL)
        .send()
        .await?;
    
    if !response.status().is_success() {
        return Err(anyhow!("Failed to fetch update manifest: {}", response.status()));
    }
    
    let manifest = response.json::<UpdateManifest>().await?;
    Ok(manifest)
}

pub async fn check_for_updates_async() -> Result<UpdateInfo> {
    // Get current version from cargo
    let current_version_str = env!("CARGO_PKG_VERSION");
    let current_version = Version::parse(current_version_str)?;
    
    let update_manifest = fetch_update_manifest().await?;
    
    // Get platform-specific release info
    let (os, arch) = get_platform_info();
    
    let platform_releases = update_manifest.platforms
        .get(os)
        .ok_or_else(|| anyhow!("No releases found for {}", os))?;
    
    let release_info = platform_releases
        .get(arch)
        .ok_or_else(|| anyhow!("No releases found for {} {}", os, arch))?;
    
    // Extract version, URL, and release date based on platform format
    let (version, download_url, release_date) = match release_info {
        PlatformRelease::Direct { version, url, release_date, .. } => {
            (version.clone(), url.clone(), release_date.clone())
        },
        PlatformRelease::Linux { version, packages, release_date, .. } => {
            // For Linux, determine which package format to use
            let package_type = determine_linux_package_type();
            let package_info = packages
                .get(package_type)
                .ok_or_else(|| anyhow!("No {} package found for Linux", package_type))?;
            (version.clone(), package_info.url.clone(), release_date.clone())
        }
    };
    
    let latest_version = Version::parse(&version)?;
    let update_available = latest_version > current_version;
    
    // Get release notes for the latest version
    let release_notes = update_manifest.release_notes
        .get(&version)
        .and_then(|notes| notes.get("en"))
        .cloned();
    
    Ok(UpdateInfo {
        current_version: current_version_str.to_string(),
        latest_version: version,
        update_available,
        download_url: Some(download_url),
        release_notes,
        published_at: release_date,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_version_comparison() {
        let v1 = Version::parse("0.1.0").unwrap();
        let v2 = Version::parse("0.2.0").unwrap();
        assert!(v2 > v1);
        
        let v3 = Version::parse("1.0.0").unwrap();
        let v4 = Version::parse("1.0.0-beta").unwrap();
        assert!(v3 > v4);
    }
}