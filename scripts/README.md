# KeyMagic Scripts

This directory contains utility scripts for the KeyMagic project.

## Update Management Scripts

### generate-updates-json.py / generate-updates-json.ps1

Automatically generates `updates.json` from installer files in the output directory.

**Features:**
- Reads installer files from `keymagic-windows/installer/output/`
- Extracts version and architecture information from filenames
- Calculates file sizes and SHA256 hashes
- Generates a complete `updates.json` file

**Usage:**

```bash
# On macOS/Linux:
./scripts/generate-updates-json.py

# On Windows:
powershell -ExecutionPolicy Bypass -File scripts\generate-updates-json.ps1
```

**What it does:**
1. Scans for `KeyMagic3-Setup-*.exe` files
2. Extracts version and architecture from filenames
3. Calculates SHA256 hash for each file
4. Creates `updates.json` with proper GitHub release URLs
5. Adds placeholder entries for other platforms (macOS, Linux)

### deploy-updates.sh

Deploys `updates.json` to GitHub Pages for the update mechanism.

**Usage:**

```bash
./scripts/deploy-updates.sh
```

**Requirements:**
- `updates.json` must exist in the project root
- Git must be configured with push access
- GitHub Pages should be enabled on the `gh-pages` branch

## Release Workflow

1. **Build installers** using the appropriate build scripts
2. **Generate updates.json**: `./scripts/generate-updates-json.py`
3. **Edit updates.json**: Add release notes and verify information
4. **Deploy to GitHub Pages**: `./scripts/deploy-updates.sh`
5. **Create GitHub Release**: Upload the installer files

## Notes

- The scripts expect installer files to follow the naming pattern: `KeyMagic3-Setup-{version}-{arch}.exe`
- SHA256 hashes are automatically calculated for security
- The generated `updates.json` includes placeholder entries for non-Windows platforms