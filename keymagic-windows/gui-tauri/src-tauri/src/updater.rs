use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use semver::Version;

const GITHUB_API_URL: &str = "https://api.github.com/repos/thantthet/keymagic/releases/latest";
const USER_AGENT: &str = "KeyMagic-Updater/0.1.0";

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
struct GithubRelease {
    tag_name: String,
    name: Option<String>,
    body: Option<String>,
    published_at: Option<String>,
    assets: Vec<GithubAsset>,
}

#[derive(Debug, Deserialize)]
struct GithubAsset {
    name: String,
    browser_download_url: String,
}

pub fn check_for_updates() -> Result<UpdateInfo> {
    // Get current version from cargo
    let current_version_str = env!("CARGO_PKG_VERSION");
    let current_version = Version::parse(current_version_str)?;
    
    // Create a blocking runtime for the HTTP request
    let runtime = tokio::runtime::Runtime::new()?;
    let github_release = runtime.block_on(fetch_latest_release())?;
    
    // Parse version from tag (remove 'v' prefix if present)
    let latest_version_str = github_release.tag_name.trim_start_matches('v');
    let latest_version = Version::parse(latest_version_str)?;
    
    let update_available = latest_version > current_version;
    
    // Find the Windows installer asset
    let download_url = github_release.assets.iter()
        .find(|asset| {
            let name = asset.name.to_lowercase();
            name.ends_with(".exe") || name.ends_with(".msi")
        })
        .map(|asset| asset.browser_download_url.clone());
    
    Ok(UpdateInfo {
        current_version: current_version_str.to_string(),
        latest_version: latest_version_str.to_string(),
        update_available,
        download_url,
        release_notes: github_release.body,
        published_at: github_release.published_at,
    })
}

async fn fetch_latest_release() -> Result<GithubRelease> {
    let client = reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .build()?;
    
    let response = client
        .get(GITHUB_API_URL)
        .send()
        .await?;
    
    if !response.status().is_success() {
        return Err(anyhow!("Failed to fetch release info: {}", response.status()));
    }
    
    let release = response.json::<GithubRelease>().await?;
    Ok(release)
}

pub async fn check_for_updates_async() -> Result<UpdateInfo> {
    // Get current version from cargo
    let current_version_str = env!("CARGO_PKG_VERSION");
    let current_version = Version::parse(current_version_str)?;
    
    let github_release = fetch_latest_release().await?;
    
    // Parse version from tag (remove 'v' prefix if present)
    let latest_version_str = github_release.tag_name.trim_start_matches('v');
    let latest_version = Version::parse(latest_version_str)?;
    
    let update_available = latest_version > current_version;
    
    // Find the Windows installer asset
    let download_url = github_release.assets.iter()
        .find(|asset| {
            let name = asset.name.to_lowercase();
            name.ends_with(".exe") || name.ends_with(".msi")
        })
        .map(|asset| asset.browser_download_url.clone());
    
    Ok(UpdateInfo {
        current_version: current_version_str.to_string(),
        latest_version: latest_version_str.to_string(),
        update_available,
        download_url,
        release_notes: github_release.body,
        published_at: github_release.published_at,
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