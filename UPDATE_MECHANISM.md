# KeyMagic Update Mechanism

## Overview

KeyMagic uses a JSON-based update system hosted on GitHub Pages to manage version information across multiple platforms. This approach provides better control over multi-platform releases compared to directly using GitHub Releases API.

## Update JSON Structure

The `updates.json` file should be hosted at: `https://thantthet.github.io/keymagic-3/updates.json`

### JSON Schema

```json
{
  "name": "KeyMagic",
  "platforms": {
    "<os>": {
      "<architecture>": {
        "version": "semantic version",
        "releaseDate": "ISO 8601 date",
        "minimumSystemVersion": "minimum OS version",
        "url": "download URL",
        "signature": "optional signature",
        "size": file size in bytes,
        "sha256": "optional SHA256 hash"
      }
    }
  },
  "releaseNotes": {
    "<version>": {
      "<language>": "Release notes in markdown"
    }
  }
}
```

### Supported Platforms

- **OS**: `windows`, `macos`, `linux`
- **Architecture**: `x86_64`, `aarch64`, `x86`, `armv7`

## How It Works

1. **Version Check**: The updater fetches `updates.json` from GitHub Pages
2. **Platform Detection**: Automatically detects the current OS and architecture
3. **Version Comparison**: Compares current version with the latest available for the platform
4. **Update Info**: Returns download URL and release notes if update is available

## Setting Up GitHub Pages

1. Create a `gh-pages` branch in your repository
2. Place `updates.json` at the root
3. Enable GitHub Pages for the `gh-pages` branch
4. The file will be accessible at `https://thantthet.github.io/keymagic-3/updates.json`

## Updating Versions

When releasing a new version:

1. Build installers for all platforms
2. Upload installers to GitHub Releases
3. Update `updates.json` with new version info:
   - Version numbers
   - Download URLs (pointing to GitHub Release assets)
   - Release notes
   - File sizes and hashes (optional but recommended)
4. Commit and push to `gh-pages` branch

## Example Workflow

```bash
# After creating a new release
git checkout gh-pages
# Update updates.json with new version info
git add updates.json
git commit -m "Update to version 0.3.1"
git push origin gh-pages
```

## Sample File

A sample `updates.sample.json` file is provided in the project root directory as a template. Copy it and rename it to `updates.json` when setting up GitHub Pages.

## Benefits

- **Platform-specific versions**: Different platforms can have different latest versions
- **Gradual rollout**: Update the JSON file separately from releases
- **Multiple architectures**: Easy support for x64, ARM64, etc.
- **Localized release notes**: Support for multiple languages
- **Better control**: Can temporarily disable updates by not updating the JSON

## Security Considerations

- Always use HTTPS URLs for downloads
- Consider implementing signature verification
- Add SHA256 hashes for integrity checking
- Keep the updates.json file minimal to prevent information leakage

## Testing

To test the update mechanism locally:

1. Host `updates.json` on a local server
2. Temporarily change `UPDATE_JSON_URL` in `updater.rs`
3. Test with different version numbers and platforms