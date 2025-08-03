#!/usr/bin/env python3
"""
Generate updates.json from the latest GitHub release.
This script fetches release information from GitHub and creates the updates.json file with
proper version information, file sizes, and SHA256 hashes.
"""

import json
import os
import hashlib
import re
import requests
from datetime import datetime, timezone
from pathlib import Path

# Configuration
OUTPUT_FILE = Path(__file__).parent.parent / "updates.json"
GITHUB_REPO = "thantthet/keymagic-3"
GITHUB_API_URL = f"https://api.github.com/repos/{GITHUB_REPO}/releases/latest"

# Pattern to extract version and architecture from filename
# Expected formats:
# - Windows: KeyMagic3-Setup-{version}.exe or KeyMagic3-Setup-{version}-{arch}.exe
# - macOS: KeyMagic3-{version}.dmg or KeyMagic3-{version}-{arch}.dmg  
# - Linux: keymagic3_{version}_{arch}.deb
# - Linux RPM: keymagic3-{version}-1.{arch}.rpm
WINDOWS_PATTERN = re.compile(r"KeyMagic3-Setup-(\d+\.\d+\.\d+(?:-\w+)?)(?:-(\w+))?\.exe")
MACOS_PATTERN = re.compile(r"KeyMagic3-(\d+\.\d+\.\d+(?:-\w+)?)(?:-(\w+))?\.dmg")
LINUX_DEB_PATTERN = re.compile(r"keymagic3_(\d+\.\d+\.\d+(?:-\w+)?)_(\w+)\.deb")
LINUX_RPM_PATTERN = re.compile(r"keymagic3-(\d+\.\d+\.\d+(?:-\w+)?)-\d+\.(\w+)\.rpm")

def fetch_github_release():
    """Fetch the latest release information from GitHub."""
    try:
        response = requests.get(GITHUB_API_URL)
        response.raise_for_status()
        return response.json()
    except requests.RequestException as e:
        print(f"Error fetching GitHub release: {e}")
        return None

def parse_asset_info(asset, platform):
    """Parse asset information based on platform."""
    filename = asset['name']
    
    if platform == 'windows':
        match = WINDOWS_PATTERN.match(filename)
        if match:
            # Default to universal for Windows if no arch specified
            return match.group(1), match.group(2) or 'universal'
    elif platform == 'macos':
        match = MACOS_PATTERN.match(filename)
        if match:
            # Default to universal binary if no arch specified
            return match.group(1), match.group(2) or 'universal'
    elif platform == 'linux':
        # Try DEB pattern first
        match = LINUX_DEB_PATTERN.match(filename)
        if match:
            return match.group(1), match.group(2)
        # Try RPM pattern
        match = LINUX_RPM_PATTERN.match(filename)
        if match:
            return match.group(1), match.group(2)
    
    return None, None

def normalize_arch(arch):
    """Normalize architecture names to match our JSON schema."""
    arch_map = {
        "x64": "x86_64",
        "amd64": "x86_64",
        "x86_64": "x86_64",
        "arm64": "aarch64",
        "aarch64": "aarch64",
        "x86": "x86",
        "universal": "universal",  # macOS universal binary
    }
    return arch_map.get(arch.lower(), arch)

def extract_sha256_from_digest(digest_string):
    """Extract SHA256 hash from GitHub asset digest field."""
    if digest_string and digest_string.startswith('sha256:'):
        return digest_string.replace('sha256:', '')
    return None

def generate_updates_json():
    """Generate updates.json from GitHub release information."""
    
    print("Fetching latest release from GitHub...")
    release = fetch_github_release()
    
    if not release:
        print("Error: Could not fetch release information from GitHub")
        return False
    
    version = release['tag_name'].lstrip('v')
    release_date = release['published_at']
    release_body = release.get('body', '')
    
    print(f"Found release: v{version}")
    
    # Initialize platform data structure
    platforms = {
        "windows": {},
        "macos": {},
        "linux": {}
    }
    
    # Process each asset
    for asset in release['assets']:
        filename = asset['name']
        download_url = asset['browser_download_url']
        file_size = asset['size']
        digest = asset.get('digest', '')
        
        # Determine platform and parse info
        platform = None
        if filename.endswith('.exe'):
            platform = 'windows'
        elif filename.endswith('.dmg'):
            platform = 'macos'
        elif filename.endswith('.deb') or filename.endswith('.rpm'):
            platform = 'linux'
        else:
            continue
        
        asset_version, arch = parse_asset_info(asset, platform)
        if not asset_version or not arch:
            print(f"Warning: Could not parse asset: {filename}")
            continue
        
        arch = normalize_arch(arch)
        
        print(f"Processing: {filename} ({platform}/{arch})")
        
        # Extract SHA256 from digest field
        sha256 = extract_sha256_from_digest(digest)
        if sha256:
            print(f"  SHA256: {sha256[:16]}...")
        else:
            print(f"  Warning: No SHA256 digest for {filename}")
            sha256 = ""
        
        # Set minimum system version based on platform
        min_version = ""
        if platform == 'windows':
            min_version = "10.0.17763"  # Windows 10 1809
        elif platform == 'macos':
            min_version = "11.0" if arch == "aarch64" else "10.15"
        
        # For universal binaries, populate both architectures
        if arch == 'universal':
            if platform == 'macos':
                for mac_arch in ['x86_64', 'aarch64']:
                    platforms[platform][mac_arch] = {
                        "version": version,
                        "releaseDate": release_date,
                        "minimumSystemVersion": "11.0" if mac_arch == "aarch64" else "10.15",
                        "url": download_url,
                        "signature": "",  # TODO: Add signature support
                        "size": file_size,
                        "sha256": sha256
                    }
            elif platform == 'windows':
                for win_arch in ['x86_64', 'aarch64']:
                    platforms[platform][win_arch] = {
                        "version": version,
                        "releaseDate": release_date,
                        "minimumSystemVersion": "10.0.17763",  # Windows 10 1809
                        "url": download_url,
                        "signature": "",  # TODO: Add signature support
                        "size": file_size,
                        "sha256": sha256
                    }
        else:
            if platform == 'linux':
                # For Linux, we need to handle the nested packages structure
                if arch not in platforms[platform]:
                    platforms[platform][arch] = {
                        "version": version,
                        "releaseDate": release_date,
                        "minimumSystemVersion": "",
                        "packages": {}
                    }
                
                # Determine package type from filename
                package_type = None
                if filename.endswith('.deb'):
                    package_type = 'deb'
                elif filename.endswith('.rpm'):
                    package_type = 'rpm'
                
                if package_type:
                    platforms[platform][arch]["packages"][package_type] = {
                        "url": download_url,
                        "size": file_size,
                        "sha256": sha256
                    }
            else:
                platforms[platform][arch] = {
                    "version": version,
                    "releaseDate": release_date,
                    "minimumSystemVersion": min_version,
                    "url": download_url,
                    "signature": "",  # TODO: Add signature support
                    "size": file_size,
                    "sha256": sha256
                }
    
    # Create the full JSON structure
    updates_data = {
        "name": "KeyMagic",
        "platforms": platforms,
        "releaseNotes": {}
    }
    
    # Parse release notes from GitHub release body
    if release_body:
        # Try to extract release notes format, or use the entire body
        updates_data["releaseNotes"][version] = {
            "en": release_body
        }
    else:
        updates_data["releaseNotes"][version] = {
            "en": f"### KeyMagic {version}\n\n- See GitHub release for details\n"
        }
    
    # Fill in missing platforms with placeholders if needed
    for platform in ['windows', 'macos', 'linux']:
        if not platforms[platform]:
            print(f"\nWarning: No assets found for {platform} platform")
            # Add placeholder entries
            if platform == 'windows':
                platforms[platform] = {
                    "x86_64": {
                        "version": version,
                        "releaseDate": release_date,
                        "minimumSystemVersion": "10.0.17763",
                        "url": f"https://github.com/{GITHUB_REPO}/releases/download/v{version}/KeyMagic3-Setup-{version}-x64.exe",
                        "signature": "",
                        "size": 0,
                        "sha256": ""
                    },
                    "aarch64": {
                        "version": version,
                        "releaseDate": release_date,
                        "minimumSystemVersion": "10.0.17763",
                        "url": f"https://github.com/{GITHUB_REPO}/releases/download/v{version}/KeyMagic3-Setup-{version}-arm64.exe",
                        "signature": "",
                        "size": 0,
                        "sha256": ""
                    }
                }
            elif platform == 'macos':
                platforms[platform] = {
                    "x86_64": {
                        "version": version,
                        "releaseDate": release_date,
                        "minimumSystemVersion": "10.15",
                        "url": f"https://github.com/{GITHUB_REPO}/releases/download/v{version}/KeyMagic3-{version}-x64.dmg",
                        "signature": "",
                        "size": 0,
                        "sha256": ""
                    },
                    "aarch64": {
                        "version": version,
                        "releaseDate": release_date,
                        "minimumSystemVersion": "11.0",
                        "url": f"https://github.com/{GITHUB_REPO}/releases/download/v{version}/KeyMagic3-{version}-arm64.dmg",
                        "signature": "",
                        "size": 0,
                        "sha256": ""
                    }
                }
            elif platform == 'linux':
                platforms[platform] = {
                    "x86_64": {
                        "version": version,
                        "releaseDate": release_date,
                        "minimumSystemVersion": "",
                        "packages": {
                            "deb": {
                                "url": f"https://github.com/{GITHUB_REPO}/releases/download/v{version}/keymagic3_{version}_amd64.deb",
                                "size": 0,
                                "sha256": ""
                            },
                            "rpm": {
                                "url": f"https://github.com/{GITHUB_REPO}/releases/download/v{version}/keymagic3-{version}-1.x86_64.rpm",
                                "size": 0,
                                "sha256": ""
                            }
                        }
                    },
                    "aarch64": {
                        "version": version,
                        "releaseDate": release_date,
                        "minimumSystemVersion": "",
                        "packages": {
                            "deb": {
                                "url": f"https://github.com/{GITHUB_REPO}/releases/download/v{version}/keymagic3_{version}_arm64.deb",
                                "size": 0,
                                "sha256": ""
                            },
                            "rpm": {
                                "url": f"https://github.com/{GITHUB_REPO}/releases/download/v{version}/keymagic3-{version}-1.aarch64.rpm",
                                "size": 0,
                                "sha256": ""
                            }
                        }
                    }
                }
    
    # Write the JSON file
    with open(OUTPUT_FILE, 'w', encoding='utf-8') as f:
        json.dump(updates_data, f, indent=2, ensure_ascii=False)
    
    print(f"\nSuccessfully generated {OUTPUT_FILE}")
    print(f"Latest version: {version}")
    print("\nFile details:")
    for platform_name, platform_data in platforms.items():
        if platform_data:
            print(f"\n{platform_name.capitalize()}:")
            for arch, info in platform_data.items():
                if platform_name == 'linux' and 'packages' in info:
                    # Linux has nested packages structure
                    for pkg_type, pkg_info in info['packages'].items():
                        if pkg_info.get('size', 0) > 0:
                            sha_display = pkg_info['sha256'][:16] if pkg_info.get('sha256') else 'N/A'
                            print(f"  {arch}/{pkg_type}: {pkg_info['size']:,} bytes, SHA256: {sha_display}...")
                else:
                    # Windows/macOS have direct structure
                    if info.get('size', 0) > 0:
                        sha_display = info['sha256'][:16] if info.get('sha256') else 'N/A'
                        print(f"  {arch}: {info['size']:,} bytes, SHA256: {sha_display}...")
    
    return True

def main():
    """Main entry point."""
    print("Generating updates.json from GitHub release...")
    
    if generate_updates_json():
        print("\nNext steps:")
        print("1. Review the generated updates.json file")
        print("2. Deploy to GitHub Pages using ./scripts/deploy-updates.sh")
    else:
        exit(1)

if __name__ == "__main__":
    main()