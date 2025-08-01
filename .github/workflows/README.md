# GitHub Actions Workflows

This directory contains GitHub Actions workflows for building, testing, and releasing KeyMagic across multiple platforms.

## Workflows

### CI (`ci.yml`)
- **Trigger**: On every push and pull request to main branches
- **Purpose**: Continuous integration testing
- **Jobs**:
  - Run tests on all platforms
  - Check code formatting
  - Run clippy lints
  - Build verification
  - Security audit

### Release (`release.yml`)
- **Trigger**: On version tags (`v*`) or manual workflow dispatch
- **Purpose**: Create official releases with all platform builds
- **Jobs**:
  - Build Windows installer (x64, ARM64, ARM64X)
  - Build Linux packages (Debian/RPM for amd64/arm64)
  - Build macOS DMG (Universal binary)
  - Create GitHub release with all artifacts
  - Update `updates.json` for auto-update mechanism

### Platform-Specific Build Workflows

#### Windows Build (`build-windows.yml`)
- **Purpose**: Reusable workflow for Windows builds
- **Features**: Cross-compilation for both x64 and ARM64 on a single runner
- **Outputs**:
  - TSF DLLs for x64 and ARM64
  - GUI executables for both architectures
  - Tray manager executables for both architectures
  - ARM64X forwarder DLL
  - Unified installer supporting both architectures

#### Linux Build (`build-linux.yml`)
- **Purpose**: Reusable workflow for Linux/IBus builds
- **Outputs**:
  - Debian packages (amd64, arm64)
  - RPM packages (x86_64, aarch64)

#### macOS Build (`build-macos.yml`)
- **Purpose**: Reusable workflow for macOS builds
- **Outputs**:
  - Universal app bundle
  - DMG installer
  - Code signing and notarization support

## Required Secrets

For full functionality, configure these secrets in your repository:

### Windows
- None required for basic builds

### Linux
- None required for basic builds

### macOS
- `CERTIFICATES_P12`: Base64-encoded P12 certificate for code signing
- `CERTIFICATES_P12_PASSWORD`: Password for the P12 certificate
- `KEYCHAIN_PASSWORD`: Temporary keychain password
- `DEVELOPER_ID_APP`: Developer ID Application certificate name
- `DEVELOPER_ID_INSTALLER`: Developer ID Installer certificate name
- `KEYCHAIN_PROFILE`: Notarization keychain profile name
- `APPLE_ID`: Apple ID for notarization
- `APPLE_TEAM_ID`: Apple Team ID

## Usage

### Manual Release
1. Go to Actions tab
2. Select "Release" workflow
3. Click "Run workflow"
4. Enter version number (e.g., "0.0.6")
5. Optionally check "draft" for draft release

### Automatic Release
1. Create and push a version tag:
   ```bash
   git tag v0.0.6
   git push origin v0.0.6
   ```

### Testing Changes
All pull requests automatically trigger:
- Unit tests on all platforms
- Build verification
- Code quality checks

## Architecture Support

- **Windows**: x64, ARM64 (with ARM64X forwarder)
- **Linux**: amd64, arm64
- **macOS**: Universal binary (Intel + Apple Silicon)

## Build Times

Approximate build times:
- Windows: 15-20 minutes
- Linux: 10-15 minutes
- macOS: 20-25 minutes (with notarization)
- Full release: 30-40 minutes (parallel builds)

## Troubleshooting

### Windows Builds
- Ensure Visual Studio components are correctly installed
- Check CMake compatibility with MSVC version

### Linux Builds
- Verify all development dependencies are listed
- Check cross-compilation setup for ARM64

### macOS Builds
- Ensure certificates are valid and not expired
- Check notarization credentials
- Verify Xcode command line tools version

## Future Improvements

- [ ] Add caching for build artifacts
- [ ] Implement incremental builds
- [ ] Add automated testing for installers
- [ ] Support for more Linux distributions
- [ ] Add performance benchmarking