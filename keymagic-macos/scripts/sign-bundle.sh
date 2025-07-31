#!/bin/bash
# macOS code signing script for KeyMagic 3

set -e

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration - can be overridden by environment variables
DEVELOPER_ID_APP="${DEVELOPER_ID_APP:-}"
KEYCHAIN_PROFILE="${KEYCHAIN_PROFILE:-keymagic-notarize}"
ENTITLEMENTS="${ENTITLEMENTS:-keymagic-macos/entitlements.plist}"

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

# Check prerequisites
check_prerequisites() {
    if [ -z "$DEVELOPER_ID_APP" ]; then
        print_error "DEVELOPER_ID_APP environment variable not set"
        print_error "Export it with: export DEVELOPER_ID_APP=\"Developer ID Application: Your Name (TEAMID)\""
        exit 1
    fi
    
    if ! command -v codesign &> /dev/null; then
        print_error "codesign command not found. Please install Xcode command line tools."
        exit 1
    fi
    
    if [ ! -f "$ENTITLEMENTS" ]; then
        print_status "No entitlements file found - proceeding with standard hardened runtime"
        ENTITLEMENTS=""
    fi
}

# Sign the application
sign_app() {
    local app_path="$1"
    
    if [ ! -d "$app_path" ]; then
        print_error "Application not found: $app_path"
        exit 1
    fi
    
    print_status "Signing application: $app_path"
    
    # First, find and sign any nested .app bundles (working from deepest to outermost)
    find "$app_path" -name "*.app" -type d | sort -r | while read -r nested_app; do
        # Skip the main app itself
        if [ "$nested_app" != "$app_path" ]; then
            print_status "Signing nested app: ${nested_app#$app_path/}"
            
            # Sign nested app with hardened runtime and timestamp
            local nested_sign_cmd="codesign --force --sign \"$DEVELOPER_ID_APP\" --options runtime --timestamp"
            
            if [ -n "$ENTITLEMENTS" ]; then
                nested_sign_cmd="$nested_sign_cmd --entitlements \"$ENTITLEMENTS\""
            fi
            
            nested_sign_cmd="$nested_sign_cmd \"$nested_app\""
            eval $nested_sign_cmd
        fi
    done
    
    # Now sign the main application
    local sign_cmd="codesign --force --deep --sign \"$DEVELOPER_ID_APP\" --options runtime --timestamp"
    
    if [ -n "$ENTITLEMENTS" ]; then
        sign_cmd="$sign_cmd --entitlements \"$ENTITLEMENTS\""
    fi
    
    sign_cmd="$sign_cmd \"$app_path\""
    
    # Execute signing
    eval $sign_cmd
    
    print_status "Verifying signature..."
    codesign --verify --deep --strict --verbose=2 "$app_path"
    
    print_status "Application signed successfully!"
}


# Main function
main() {
    if [ $# -eq 0 ]; then
        echo "Usage: $0 <app_path>"
        echo ""
        echo "This script signs a macOS application bundle."
        echo ""
        echo "Environment variables:"
        echo "  DEVELOPER_ID_APP         Developer ID Application certificate"
        echo "  KEYCHAIN_PROFILE         Notarization keychain profile (default: keymagic-notarize)"
        echo "  ENTITLEMENTS            Path to entitlements file (default: keymagic-macos/entitlements.plist)"
        exit 1
    fi
    
    local app_path="$1"
    
    # Check prerequisites
    check_prerequisites
    
    # Sign the application
    sign_app "$app_path"
    
    print_status "All done!"
}

# Run main function
main "$@"