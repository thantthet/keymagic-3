#!/usr/bin/env python3
"""
Generate updates.json from installer files in the output directory.
This script reads installer files and creates the updates.json file with
proper version information, file sizes, and SHA256 hashes.
"""

import json
import os
import hashlib
import re
from datetime import datetime, timezone
from pathlib import Path

# Configuration
INSTALLER_DIR = Path(__file__).parent.parent / "keymagic-windows" / "installer" / "output"
OUTPUT_FILE = Path(__file__).parent.parent / "updates.json"
GITHUB_REPO = "thantthet/keymagic-3"

# Pattern to extract version and architecture from filename
# Expected format: KeyMagic3-Setup-{version}-{arch}.exe
FILENAME_PATTERN = re.compile(r"KeyMagic3-Setup-(\d+\.\d+\.\d+(?:-\w+)?)-(\w+)\.exe")

def calculate_sha256(filepath):
    """Calculate SHA256 hash of a file."""
    sha256_hash = hashlib.sha256()
    with open(filepath, "rb") as f:
        for byte_block in iter(lambda: f.read(4096), b""):
            sha256_hash.update(byte_block)
    return sha256_hash.hexdigest()

def get_file_size(filepath):
    """Get file size in bytes."""
    return os.path.getsize(filepath)

def parse_installer_filename(filename):
    """Extract version and architecture from installer filename."""
    match = FILENAME_PATTERN.match(filename)
    if not match:
        return None, None
    return match.group(1), match.group(2)

def normalize_arch(arch):
    """Normalize architecture names to match our JSON schema."""
    arch_map = {
        "x64": "x86_64",
        "arm64": "aarch64",
        "x86": "x86",
    }
    return arch_map.get(arch.lower(), arch)

def generate_updates_json():
    """Generate updates.json from installer files."""
    
    if not INSTALLER_DIR.exists():
        print(f"Error: Installer directory not found: {INSTALLER_DIR}")
        return False
    
    # Find all installer files
    installers = list(INSTALLER_DIR.glob("KeyMagic3-Setup-*.exe"))
    
    if not installers:
        print(f"No installer files found in {INSTALLER_DIR}")
        return False
    
    # Group installers by version
    versions = {}
    
    for installer in installers:
        version, arch = parse_installer_filename(installer.name)
        if not version or not arch:
            print(f"Warning: Could not parse filename: {installer.name}")
            continue
        
        arch = normalize_arch(arch)
        
        if version not in versions:
            versions[version] = {}
        
        print(f"Processing: {installer.name}")
        
        # Calculate file info
        file_size = get_file_size(installer)
        sha256 = calculate_sha256(installer)
        
        versions[version][arch] = {
            "version": version,
            "releaseDate": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
            "minimumSystemVersion": "10.0.17763",  # Windows 10 1809
            "url": f"https://github.com/{GITHUB_REPO}/releases/download/v{version}/{installer.name}",
            "signature": "",  # TODO: Add signature support
            "size": file_size,
            "sha256": sha256
        }
    
    if not versions:
        print("Error: No valid installer files found")
        return False
    
    # Get the latest version
    latest_version = sorted(versions.keys(), key=lambda v: [int(x) for x in v.split('-')[0].split('.')])[-1]
    
    # Create the full JSON structure
    updates_data = {
        "name": "KeyMagic",
        "platforms": {
            "windows": versions[latest_version]
        },
        "releaseNotes": {
            latest_version: {
                "en": f"### KeyMagic {latest_version}\n\n- Please add release notes here\n"
            }
        }
    }
    
    # Add placeholder entries for other platforms
    updates_data["platforms"]["macos"] = {
        "x86_64": {
            "version": latest_version,
            "releaseDate": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
            "minimumSystemVersion": "10.15",
            "url": f"https://github.com/{GITHUB_REPO}/releases/download/v{latest_version}/KeyMagic3-{latest_version}-x64.dmg",
            "signature": "",
            "size": 0,
            "sha256": ""
        },
        "aarch64": {
            "version": latest_version,
            "releaseDate": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
            "minimumSystemVersion": "11.0",
            "url": f"https://github.com/{GITHUB_REPO}/releases/download/v{latest_version}/KeyMagic3-{latest_version}-arm64.dmg",
            "signature": "",
            "size": 0,
            "sha256": ""
        }
    }
    
    updates_data["platforms"]["linux"] = {
        "x86_64": {
            "version": latest_version,
            "releaseDate": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
            "minimumSystemVersion": "",
            "url": f"https://github.com/{GITHUB_REPO}/releases/download/v{latest_version}/keymagic3-{latest_version}-amd64.deb",
            "signature": "",
            "size": 0,
            "sha256": ""
        }
    }
    
    # Write the JSON file
    with open(OUTPUT_FILE, 'w', encoding='utf-8') as f:
        json.dump(updates_data, f, indent=2, ensure_ascii=False)
    
    print(f"\nSuccessfully generated {OUTPUT_FILE}")
    print(f"Latest version: {latest_version}")
    print("\nFile details:")
    for arch, info in versions[latest_version].items():
        print(f"  {arch}: {info['size']:,} bytes, SHA256: {info['sha256'][:16]}...")
    
    return True

def main():
    """Main entry point."""
    print("Generating updates.json from installer files...")
    
    if generate_updates_json():
        print("\nNext steps:")
        print("1. Review the generated updates.json file")
        print("2. Add appropriate release notes")
        print("3. Deploy to GitHub Pages using ./scripts/deploy-updates.sh")
    else:
        exit(1)

if __name__ == "__main__":
    main()