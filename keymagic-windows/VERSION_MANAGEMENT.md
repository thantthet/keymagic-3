# KeyMagic Windows Version Management

This document describes the centralized version management system for KeyMagic Windows components.

## Overview

The version management system ensures that all KeyMagic Windows components maintain consistent version numbers across different files and formats.

## Version File

The master version is stored in `version.txt` at the root of the keymagic-windows directory. This file contains a single line with the version number in semantic versioning format (e.g., `3.0.0`).

## Components That Use Version

The following components require version updates:

1. **GUI Application (Tauri)**
   - `gui-tauri/src-tauri/Cargo.toml` - Rust package version
   - `gui-tauri/src-tauri/tauri.conf.json` - Tauri application version

2. **TSF DLL**
   - `tsf/src/KeyMagicTSF.rc` - Windows resource file with FILEVERSION and PRODUCTVERSION

3. **Installers**
   - `installer/setup-x64.iss` - Inno Setup script for x64
   - `installer/setup-arm64.iss` - Inno Setup script for ARM64

## Usage

### Checking Current Versions

To check the current version status across all components:

```batch
check-versions.bat
```

This will display:
- The master version from `version.txt`
- The version in each component file
- Whether versions are synchronized (green) or mismatched (red)

### Updating Versions

To update all versions to match the version in `version.txt`:

```batch
update-version.bat
```

To update all versions to a specific new version:

```batch
update-version.bat 3.1.0
```

The update script will:
1. Validate the version format
2. Update all component files
3. Update `version.txt` with the new version
4. Display which files were updated

### Version Formats

Different components use different version formats:

- **Semantic Version** (e.g., `3.0.0`): Used in Cargo.toml, tauri.conf.json, and Inno Setup scripts
- **File Version** (e.g., `3,0,0,0`): Used in Windows resource files (FILEVERSION/PRODUCTVERSION)
- **Dotted File Version** (e.g., `3.0.0.0`): Used in Windows resource file strings

The update script automatically converts between these formats as needed.

## Best Practices

1. **Always use the scripts** to update versions rather than editing files manually
2. **Check versions** before building releases to ensure consistency
3. **Update version** as part of the release process
4. **Commit version changes** as a separate commit with message like "Bump version to X.Y.Z"

## Troubleshooting

### PowerShell Execution Policy

If you encounter execution policy errors, the batch files automatically use `-ExecutionPolicy Bypass`. Alternatively, you can run:

```powershell
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
```

### File Not Found

If the scripts report files as not found, ensure you're running them from the `keymagic-windows` directory.

### Version Format Errors

The version must follow semantic versioning: `MAJOR.MINOR.PATCH` or `MAJOR.MINOR.PATCH-SUFFIX`

Examples of valid versions:
- `3.0.0`
- `3.1.0`
- `3.0.0-beta.1`
- `3.0.0-rc.1`