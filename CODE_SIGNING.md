# Code Signing Guide for KeyMagic 3

This guide covers code signing setup for macOS and Linux packages to ensure users can install and run KeyMagic without security warnings.

## macOS Code Signing

### Prerequisites

1. **Apple Developer Account**: Required for distributing outside the App Store
   - Enroll at https://developer.apple.com ($99/year)
   - For open source projects, you might qualify for a fee waiver

2. **Developer ID Certificates**:
   - Developer ID Application certificate (for apps)
   - Developer ID Installer certificate (for .pkg installers)

### Setting Up Certificates

1. **Generate certificates in Xcode**:
   ```bash
   # Open Xcode preferences
   open -a Xcode
   # Go to Preferences > Accounts > Manage Certificates
   # Click + to create "Developer ID Application" and "Developer ID Installer"
   ```

2. **Or use command line**:
   ```bash
   # List available certificates
   security find-identity -v -p codesigning
   
   # You should see something like:
   # "Developer ID Application: Your Name (TEAMID)"
   # "Developer ID Installer: Your Name (TEAMID)"
   ```

### Code Signing Process

KeyMagic uses DMG distribution with embedded app bundles. The signing process is automated through the packaging script.

1. **Build your applications**:
   ```bash
   # Build Tauri app
   cargo tauri build
   
   # Build IMK bundle
   cd keymagic-macos
   make
   ```

2. **Package and sign everything**:
   ```bash
   # This script handles all signing internally
   ./keymagic-macos/scripts/package-macos-dmg.sh 0.0.6 --notarize
   ```

   The packaging script automatically:
   - Embeds the IMK bundle inside the Tauri app
   - Signs all nested app bundles (from innermost to outermost)
   - Creates and signs the DMG
   - Notarizes the DMG (with --notarize flag)
   - Staples the notarization ticket

3. **Manual signing (if needed)**:
   ```bash
   # Use the sign-bundle.sh script for individual app bundles
   ./keymagic-macos/scripts/sign-bundle.sh path/to/KeyMagic3.app
   ```

### Notarization (Required for macOS 10.15+)

1. **Create app-specific password**:
   - Go to https://appleid.apple.com
   - Generate app-specific password under Security

2. **Store credentials**:
   ```bash
   # Store credentials in keychain
   xcrun notarytool store-credentials "keymagic-notarize" \
     --apple-id "your@email.com" \
     --team-id "TEAMID" \
     --password "app-specific-password"
   ```

3. **Notarize through the packaging script**:
   ```bash
   # The packaging script handles notarization with --notarize flag
   ./keymagic-macos/scripts/package-macos-dmg.sh 0.0.6 --notarize
   ```

   Or manually notarize a DMG:
   ```bash
   # Submit for notarization
   xcrun notarytool submit KeyMagic3-0.0.6.dmg \
     --keychain-profile "keymagic-notarize" \
     --wait
   
   # Staple the ticket
   xcrun stapler staple KeyMagic3-0.0.6.dmg
   ```

### Automated Signing Scripts

#### Bundle Signing Script

The `keymagic-macos/scripts/sign-bundle.sh` script handles signing of app bundles:
- Signs nested app bundles from innermost to outermost
- Applies hardened runtime and timestamp (required for notarization)
- Uses entitlements if provided
- Verifies signatures after signing

#### DMG Packaging Script

The `keymagic-macos/scripts/package-macos-dmg.sh` script handles the complete distribution process:
```bash
# Usage
./keymagic-macos/scripts/package-macos-dmg.sh <version> [--notarize]

# Example
./keymagic-macos/scripts/package-macos-dmg.sh 0.0.6 --notarize
```

This script:
1. Builds the IMK bundle if needed
2. Copies the Tauri app bundle
3. Embeds the IMK bundle inside the Tauri app
4. Calls `sign-bundle.sh` to sign everything
5. Creates a DMG with custom background
6. Signs the DMG
7. Notarizes and staples (with --notarize flag)

### Required Entitlements

Create `keymagic-macos/entitlements.plist`:
```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <!-- No special entitlements needed for KeyMagic -->
    <!-- Works purely within IMK framework with statically linked Rust library -->
</dict>
</plist>
```

**Note**: KeyMagic requires NO special entitlements because it:
- Works purely within the Input Method Kit (IMK) framework
- Uses a statically linked Rust library (no dynamic loading)
- Doesn't intercept or send Apple Events
- Doesn't require JIT or unsigned memory access
- Runs entirely within the sandboxed IMK environment

## Linux Package Signing

### RPM Signing (Fedora/RHEL/openSUSE)

1. **Generate GPG key**:
   ```bash
   # Generate a new GPG key
   gpg --gen-key
   
   # List keys
   gpg --list-secret-keys --keyid-format LONG
   
   # Export public key
   gpg --armor --export your@email.com > RPM-GPG-KEY-keymagic
   ```

2. **Configure RPM macros**:
   ```bash
   # Add to ~/.rpmmacros
   echo "%_signature gpg" >> ~/.rpmmacros
   echo "%_gpg_name Your Name <your@email.com>" >> ~/.rpmmacros
   ```

3. **Sign RPM packages**:
   ```bash
   # Sign the package
   rpm --addsign keymagic3-*.rpm
   
   # Verify signature
   rpm --checksig keymagic3-*.rpm
   ```

### DEB Signing (Debian/Ubuntu)

1. **Set up GPG for debsign**:
   ```bash
   # Install required tools
   sudo apt-get install devscripts
   
   # Configure debsign
   echo 'DEBSIGN_KEYID="your-gpg-key-id"' >> ~/.devscripts
   ```

2. **Sign packages**:
   ```bash
   # Sign changes file (which references the .deb)
   debsign keymagic3_*.changes
   
   # Or sign .deb directly
   dpkg-sig --sign builder keymagic3_*.deb
   
   # Verify signature
   dpkg-sig --verify keymagic3_*.deb
   ```

### Repository Signing

For APT repositories:
```bash
# Create Release file
apt-ftparchive release . > Release

# Sign Release file
gpg --clearsign --digest-algo SHA512 -o InRelease Release
gpg -abs --digest-algo SHA512 -o Release.gpg Release
```

For YUM/DNF repositories:
```bash
# Sign repository metadata
createrepo_c .
gpg --detach-sign --armor repodata/repomd.xml
```

### Automated Linux Signing Script

The `keymagic-ibus/scripts/sign-package.sh` script provides comprehensive Linux package signing:
```bash
# Usage
./keymagic-ibus/scripts/sign-package.sh <command> [options]

# Sign RPM packages
./keymagic-ibus/scripts/sign-package.sh rpm dist/*.rpm

# Sign DEB packages  
./keymagic-ibus/scripts/sign-package.sh deb dist/*.deb

# Export GPG public key
./keymagic-ibus/scripts/sign-package.sh export-key

# Sign repository metadata
./keymagic-ibus/scripts/sign-package.sh sign-repo apt dist/repo
```

This script features:
- Automatic GPG key detection from system configuration
- Support for both RPM and DEB package signing
- Repository metadata signing for APT and YUM/DNF
- Public key export functionality
- Colored output and comprehensive error handling
- Signature verification after signing

## CI/CD Integration

### GitHub Actions for macOS

```yaml
# .github/workflows/sign-macos.yml
name: Sign macOS Build

on:
  workflow_dispatch:
  release:
    types: [created]

jobs:
  sign:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Import certificates
        env:
          CERTIFICATES_P12: ${{ secrets.CERTIFICATES_P12 }}
          CERTIFICATES_P12_PASSWORD: ${{ secrets.CERTIFICATES_P12_PASSWORD }}
        run: |
          # Create temporary keychain
          security create-keychain -p actions temp.keychain
          security set-keychain-settings -lut 21600 temp.keychain
          security unlock-keychain -p actions temp.keychain
          
          # Import certificates
          echo "$CERTIFICATES_P12" | base64 --decode > certificate.p12
          security import certificate.p12 -P "$CERTIFICATES_P12_PASSWORD" \
            -A -t cert -f pkcs12 -k temp.keychain
          security list-keychain -d user -s temp.keychain
      
      - name: Build applications
        run: |
          cargo tauri build
          cd keymagic-macos && make
      
      - name: Package and sign DMG
        run: |
          keymagic-macos/scripts/package-macos-dmg.sh ${{ github.event.release.tag_name }} --notarize
        env:
          DEVELOPER_ID_APP: ${{ secrets.DEVELOPER_ID_APP }}
          KEYCHAIN_PROFILE: keymagic-notarize
```

### GitHub Actions for Linux

```yaml
# .github/workflows/sign-linux.yml
name: Sign Linux Packages

on:
  workflow_dispatch:
  release:
    types: [created]

jobs:
  sign:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Import GPG key
        env:
          GPG_PRIVATE_KEY: ${{ secrets.GPG_PRIVATE_KEY }}
          GPG_PASSPHRASE: ${{ secrets.GPG_PASSPHRASE }}
        run: |
          echo "$GPG_PRIVATE_KEY" | gpg --import
          echo "$GPG_PASSPHRASE" | gpg --batch --yes --passphrase-fd 0 \
            --pinentry-mode loopback --change-passphrase $(gpg --list-secret-keys --keyid-format LONG | grep sec | cut -d' ' -f4 | cut -d'/' -f2)
      
      - name: Sign packages
        run: |
          # Sign RPMs
          keymagic-ibus/scripts/sign-package.sh rpm dist/*.rpm
          
          # Sign DEBs
          keymagic-ibus/scripts/sign-package.sh deb dist/*.deb
```

## Best Practices

1. **Security**:
   - Never commit signing certificates or private keys
   - Use environment variables or secure secret storage
   - Rotate keys periodically

2. **Automation**:
   - Integrate signing into CI/CD pipelines
   - Sign immediately after building
   - Verify signatures before distribution

3. **Distribution**:
   - Provide public keys for verification
   - Document verification steps for users
   - Consider using package managers' built-in signing

4. **Testing**:
   - Test signed packages on clean systems
   - Verify installation without warnings
   - Check signature verification works

## Troubleshooting

### macOS Issues

- **"unnotarized developer"**: Ensure notarization completed
- **"damaged application"**: Check code signature integrity
- **Gatekeeper blocks**: Verify Developer ID certificate is valid

### Linux Issues

- **"gpg: no default secret key"**: Set GPG_KEY or update ~/.rpmmacros
- **"dpkg-sig: error"**: Ensure GPG key is available and unlocked
- **Repository errors**: Check that Release files are properly signed

## macOS Distribution

### Creating a DMG Installer

KeyMagic for macOS is distributed as a DMG file containing both the Tauri GUI app and the embedded IMK bundle:

```bash
# Set up environment
export DEVELOPER_ID_APP="Developer ID Application: Your Name (TEAMID)"

# Build and package everything
cargo tauri build
cd keymagic-macos && make && cd ..
./keymagic-macos/scripts/package-macos-dmg.sh 0.0.6 --notarize
```

The packaging script produces a notarized DMG ready for distribution.

### How it Works

1. **DMG Contents**:
   - KeyMagic3.app (Tauri GUI app)
     - Contains embedded IMK bundle at: Contents/Resources/resources/KeyMagic3.app
   - Symbolic link to Applications folder
   - Custom background with instructions

2. **Installation Flow**:
   - User drags KeyMagic3.app to Applications
   - Launches the app from Applications
   - App automatically installs the embedded IMK bundle to /Library/Input Methods
   - IMK bundle self-registers as input method

3. **Architecture**:
   - Tauri app: User interface and keyboard management
   - IMK bundle: Actual input method implementation
   - Both are signed and notarized together in the DMG

## Additional Resources

- [Apple Developer - Notarizing macOS Software](https://developer.apple.com/documentation/security/notarizing_macos_software_before_distribution)
- [RPM Signing](https://rpm-software-management.github.io/rpm/manual/rpmsign.html)
- [Debian Package Signing](https://wiki.debian.org/SecureApt)
- [GPG Best Practices](https://riseup.net/en/security/message-security/openpgp/best-practices)