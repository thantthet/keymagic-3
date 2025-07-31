#!/usr/bin/env python3
"""
Cross-platform version update script for KeyMagic 3.
Updates version numbers across all components for Windows, macOS, and Linux/IBus.
"""

import argparse
import os
import re
import json
import sys
from pathlib import Path


def get_project_root():
    """Get the project root directory."""
    script_dir = Path(__file__).parent
    return script_dir.parent


def read_version_from_file(version_file):
    """Read version from version.txt file."""
    try:
        with open(version_file, 'r') as f:
            return f.read().strip()
    except FileNotFoundError:
        return None


def validate_version(version):
    """Validate semantic version format."""
    pattern = r'^\d+\.\d+\.\d+(-.*)?$'
    return bool(re.match(pattern, version))


def update_file(file_path, update_func, description):
    """Update a file with the given update function."""
    if not os.path.exists(file_path):
        print(f"⚠️  File not found: {file_path}")
        return False
    
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        new_content = update_func(content)
        
        if content != new_content:
            with open(file_path, 'w', encoding='utf-8') as f:
                f.write(new_content)
            print(f"✅ Updated: {description}")
            return True
        else:
            print(f"ℹ️  No changes needed: {description}")
            return False
    except Exception as e:
        print(f"❌ Error updating {description}: {e}")
        return False


def update_cargo_toml(content, version):
    """Update version in Cargo.toml files."""
    lines = content.split('\n')
    in_package_section = False
    package_version_updated = False
    
    for i, line in enumerate(lines):
        # Check if we're entering the [package] section
        if line.strip() == '[package]':
            in_package_section = True
        # Check if we're leaving the package section
        elif line.strip().startswith('[') and line.strip() != '[package]':
            in_package_section = False
        # Update version in package section
        elif in_package_section and not package_version_updated and line.strip().startswith('version'):
            lines[i] = f'version = "{version}"'
            package_version_updated = True
    
    return '\n'.join(lines)


def update_workspace_cargo_toml(content, version):
    """Update version in workspace Cargo.toml."""
    lines = content.split('\n')
    in_workspace_package = False
    
    for i, line in enumerate(lines):
        # Check if we're entering the [workspace.package] section
        if line.strip() == '[workspace.package]':
            in_workspace_package = True
        # Check if we're leaving the section
        elif line.strip().startswith('[') and in_workspace_package:
            in_workspace_package = False
        # Update version in workspace.package section
        elif in_workspace_package and line.strip().startswith('version'):
            lines[i] = f'version = "{version}"'
    
    return '\n'.join(lines)


def update_json_file(content, version):
    """Update version in JSON files."""
    try:
        data = json.loads(content)
        data['version'] = version
        return json.dumps(data, indent=2) + '\n'
    except:
        # Fallback to regex if JSON parsing fails
        return re.sub(r'"version"\s*:\s*"[^"]*"', f'"version": "{version}"', content)


def update_rc_file(content, version):
    """Update version in Windows RC files."""
    # Convert version to file version format (X.Y.Z to X,Y,Z,0)
    parts = version.split('-')[0].split('.')
    file_version = f"{parts[0]},{parts[1]},{parts[2]},0"
    dot_file_version = f"{parts[0]}.{parts[1]}.{parts[2]}.0"
    
    # Update FILEVERSION
    content = re.sub(r'FILEVERSION\s+\d+,\d+,\d+,\d+', f'FILEVERSION     {file_version}', content)
    # Update PRODUCTVERSION
    content = re.sub(r'PRODUCTVERSION\s+\d+,\d+,\d+,\d+', f'PRODUCTVERSION  {file_version}', content)
    # Update FileVersion string
    content = re.sub(r'VALUE\s+"FileVersion",\s*"[^"]*"', f'VALUE "FileVersion",      "{dot_file_version}"', content)
    # Update ProductVersion string
    content = re.sub(r'VALUE\s+"ProductVersion",\s*"[^"]*"', f'VALUE "ProductVersion",   "{dot_file_version}"', content)
    
    return content


def update_iss_file(content, version):
    """Update version in Inno Setup files."""
    return re.sub(r'#define\s+MyAppVersion\s+"[^"]*"', f'#define MyAppVersion "{version}"', content)


def update_plist_file(content, version):
    """Update version in macOS Info.plist files."""
    # Update CFBundleShortVersionString
    content = re.sub(
        r'(<key>CFBundleShortVersionString</key>\s*<string>)[^<]*(</string>)',
        f'\\g<1>{version}\\g<2>',
        content
    )
    # Update CFBundleVersion (use major.minor.patch without suffix)
    base_version = version.split('-')[0]
    content = re.sub(
        r'(<key>CFBundleVersion</key>\s*<string>)[^<]*(</string>)',
        f'\\g<1>{base_version}\\g<2>',
        content
    )
    return content


def update_desktop_file(content, version):
    """Update version in .desktop files."""
    # Update Version field
    return re.sub(r'^Version=.*$', f'Version={version}', content, flags=re.MULTILINE)


def update_spec_in_file(content, version):
    """Update version in .spec.in files."""
    # Update the Version: line directly
    return re.sub(r'^Version:\s+.*$', f'Version:        {version}', content, flags=re.MULTILINE)


def main():
    parser = argparse.ArgumentParser(description='Update version across all KeyMagic components')
    parser.add_argument('version', nargs='?', help='New version (e.g., 1.2.3 or 1.2.3-beta.1)')
    parser.add_argument('--dry-run', action='store_true', help='Show what would be changed without modifying files')
    
    args = parser.parse_args()
    
    project_root = get_project_root()
    version_file = project_root / 'version.txt'
    
    # Get version from argument or version.txt
    if args.version:
        new_version = args.version
    else:
        new_version = read_version_from_file(version_file)
        if not new_version:
            print("❌ No version specified and version.txt not found")
            sys.exit(1)
    
    # Validate version format
    if not validate_version(new_version):
        print(f"❌ Invalid version format: {new_version}")
        print("   Expected format: X.Y.Z or X.Y.Z-suffix")
        sys.exit(1)
    
    print(f"🔄 Updating version to: {new_version}")
    if args.dry_run:
        print("   (DRY RUN - no files will be modified)")
    print()
    
    # List of files to update
    updates = [
        # Workspace Cargo.toml
        (project_root / 'Cargo.toml',
         lambda c: update_workspace_cargo_toml(c, new_version),
         'Workspace Cargo.toml'),
        
        # GUI Cargo.toml
        (project_root / 'keymagic-shared' / 'gui' / 'src-tauri' / 'Cargo.toml',
         lambda c: update_cargo_toml(c, new_version),
         'GUI Cargo.toml'),
        
        # GUI package.json
        (project_root / 'keymagic-shared' / 'gui' / 'package.json',
         lambda c: update_json_file(c, new_version),
         'GUI package.json'),
        
        # Tauri config
        (project_root / 'keymagic-shared' / 'gui' / 'src-tauri' / 'tauri.conf.json',
         lambda c: update_json_file(c, new_version),
         'Tauri configuration'),
        
        # Windows TSF RC file
        (project_root / 'keymagic-windows' / 'tsf' / 'src' / 'KeyMagicTSF.rc',
         lambda c: update_rc_file(c, new_version),
         'Windows TSF resource file'),
        
        # Windows installer file
        (project_root / 'keymagic-windows' / 'installer' / 'setup.iss',
         lambda c: update_iss_file(c, new_version),
         'Windows installer'),
        
        # macOS Info.plist
        (project_root / 'keymagic-macos' / 'Info.plist',
         lambda c: update_plist_file(c, new_version),
         'macOS Info.plist'),
        
        # Linux/IBus desktop file
        (project_root / 'keymagic-ibus' / 'data' / 'keymagic3.desktop',
         lambda c: update_desktop_file(c, new_version),
         'IBus desktop file'),
        
        # Linux packaging files (these use placeholders)
        (project_root / 'keymagic-ibus' / 'packaging' / 'keymagic3.spec.in',
         lambda c: update_spec_in_file(c, new_version),
         'RPM spec template'),
    ]
    
    # Perform updates
    success_count = 0
    for file_path, update_func, description in updates:
        if args.dry_run:
            if file_path.exists():
                print(f"Would update: {description}")
            else:
                print(f"Would skip (not found): {description}")
        else:
            if update_file(file_path, update_func, description):
                success_count += 1
    
    # Update version.txt
    if not args.dry_run:
        try:
            with open(version_file, 'w') as f:
                f.write(new_version)
            print(f"✅ Updated: version.txt")
        except Exception as e:
            print(f"❌ Error updating version.txt: {e}")
    
    print()
    if args.dry_run:
        print("✨ Dry run complete! Use without --dry-run to apply changes.")
    else:
        print(f"✨ Version update complete! All components updated to: {new_version}")


if __name__ == '__main__':
    main()