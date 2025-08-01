#!/bin/bash
# Package KeyMagic for macOS as a DMG with embedded IMK bundle

set -e

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
BUILD_TYPE="${BUILD_TYPE:-release}"
BUILD_DIR="$PROJECT_ROOT/keymagic-macos/build"
DMG_DIR="$PROJECT_ROOT/target/dmg"
APP_NAME="KeyMagic3.app"
DMG_NAME="KeyMagic3-$1.dmg"

# Function to print colored output
print_status() {
    echo -e "${GREEN}[*]${NC} $1"
}

print_error() {
    echo -e "${RED}[!]${NC} $1" >&2
}

print_warning() {
    echo -e "${YELLOW}[!]${NC} $1"
}

# Check if version is provided
if [ -z "$1" ]; then
    print_error "Usage: $0 <version>"
    print_error "Example: $0 0.0.6"
    exit 1
fi

VERSION="$1"

# Check if app bundle exists (GUI with embedded IMK)
if [ ! -d "$BUILD_DIR/$APP_NAME" ]; then
    print_error "App bundle not found at: $BUILD_DIR/$APP_NAME"
    print_error "Please run 'make all' in keymagic-macos directory first"
    exit 1
fi

# Create DMG directory
print_status "Creating DMG staging directory..."
rm -rf "$DMG_DIR"
mkdir -p "$DMG_DIR/dmg"

# Copy app bundle (already has embedded IMK)
print_status "Copying app bundle..."
cp -R "$BUILD_DIR/$APP_NAME" "$DMG_DIR/dmg/"

# Copy bundled keyboards to app resources
print_status "Copying bundled keyboards..."
KEYBOARDS_DIR="$DMG_DIR/dmg/$APP_NAME/Contents/Resources/keyboards"
mkdir -p "$KEYBOARDS_DIR"
if [ -d "$PROJECT_ROOT/keyboards/bundled" ]; then
    cp "$PROJECT_ROOT/keyboards/bundled"/*.km2 "$KEYBOARDS_DIR/" 2>/dev/null || true
    KEYBOARD_COUNT=$(find "$KEYBOARDS_DIR" -name "*.km2" | wc -l)
    print_status "Copied $KEYBOARD_COUNT keyboard files"
else
    print_warning "No bundled keyboards directory found"
fi

# Update app bundle version to match
print_status "Updating app bundle version..."
/usr/libexec/PlistBuddy -c "Set :CFBundleShortVersionString $VERSION" \
    "$DMG_DIR/dmg/$APP_NAME/Contents/Info.plist"
/usr/libexec/PlistBuddy -c "Set :CFBundleVersion $VERSION" \
    "$DMG_DIR/dmg/$APP_NAME/Contents/Info.plist"

# Also update the embedded IMK server version
IMK_PLIST="$DMG_DIR/dmg/$APP_NAME/Contents/Resources/KeyMagic3-Server.app/Contents/Info.plist"
if [ -f "$IMK_PLIST" ]; then
    print_status "Updating embedded IMK server version..."
    /usr/libexec/PlistBuddy -c "Set :CFBundleShortVersionString $VERSION" "$IMK_PLIST"
    /usr/libexec/PlistBuddy -c "Set :CFBundleVersion $VERSION" "$IMK_PLIST"
fi

# Applications symlink will be created by create-dmg

# Sign the app bundle if certificates are available
if [ -n "$DEVELOPER_ID_APP" ]; then
    print_status "Signing app bundle..."
    "$PROJECT_ROOT/keymagic-macos/scripts/sign-bundle.sh" "$DMG_DIR/dmg/$APP_NAME"
else
    print_warning "DEVELOPER_ID_APP not set, skipping code signing"
fi

# Create DMG
print_status "Creating DMG..."
DMG_PATH="$PROJECT_ROOT/target/$DMG_NAME"

# Remove old DMG if exists
rm -f "$DMG_PATH"

# Check if custom background exists
BACKGROUND_IMAGE="$PROJECT_ROOT/keymagic-macos/assets/dmg-background.png"
if [ ! -f "$BACKGROUND_IMAGE" ]; then
    print_warning "DMG background image not found at: $BACKGROUND_IMAGE"
    print_warning "Using default DMG layout"
    
    # Create DMG using hdiutil (fallback)
    hdiutil create -volname "KeyMagic $VERSION" \
        -srcfolder "$DMG_DIR/dmg" \
        -ov -format UDZO \
        "$DMG_PATH"
else
    # Create DMG using create-dmg with custom background
    print_status "Creating DMG with custom background..."
    
    create-dmg \
        --volname "KeyMagic $VERSION" \
        --volicon "$PROJECT_ROOT/resources/icons/KeyMagic.icns" \
        --background "$BACKGROUND_IMAGE" \
        --window-pos 200 120 \
        --window-size 600 400 \
        --icon-size 100 \
        --icon "KeyMagic3.app" 150 200 \
        --hide-extension "KeyMagic3.app" \
        --app-drop-link 450 200 \
        --no-internet-enable \
        "$DMG_PATH" \
        "$DMG_DIR/dmg"
fi

# Sign DMG if certificates are available
if [ -n "$DEVELOPER_ID_APP" ]; then
    print_status "Signing DMG..."
    codesign --force --sign "$DEVELOPER_ID_APP" "$DMG_PATH"
    
    # Notarize if requested
    if [ "$2" == "--notarize" ]; then
        print_status "Notarizing DMG..."
        xcrun notarytool submit "$DMG_PATH" \
            --keychain-profile "${KEYCHAIN_PROFILE:-keymagic-notarize}" \
            --wait
        
        print_status "Stapling notarization ticket..."
        xcrun stapler staple "$DMG_PATH"
    fi
fi

# Clean up
print_status "Cleaning up..."
rm -rf "$DMG_DIR"

# Final output
print_status "DMG created successfully: $DMG_PATH"
print_status "Size: $(du -h "$DMG_PATH" | cut -f1)"

# Verify DMG
print_status "Verifying DMG..."
hdiutil verify "$DMG_PATH"

if [ -n "$DEVELOPER_ID_APP" ]; then
    print_status "Verifying signature..."
    spctl -a -t open --context context:primary-signature -v "$DMG_PATH"
fi

print_status "Done! The DMG is ready for distribution."

# Instructions
echo
echo "Installation instructions for users:"
echo "1. Open $DMG_NAME"
echo "2. Drag KeyMagic to Applications folder"
echo "3. Launch KeyMagic from Applications"
echo "4. KeyMagic will automatically install the input method on first launch"
echo "5. Add KeyMagic in System Preferences > Keyboard > Input Sources"