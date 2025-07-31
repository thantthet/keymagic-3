#!/bin/bash
# Linux package signing script for KeyMagic 3

set -e

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
GPG_KEY="${GPG_KEY:-}"
SIGN_PASSPHRASE="${SIGN_PASSPHRASE:-}"

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

# Check if GPG key is available
check_gpg_key() {
    if [ -z "$GPG_KEY" ]; then
        # Try to get from ~/.rpmmacros or ~/.devscripts
        if [ -f ~/.rpmmacros ]; then
            GPG_KEY=$(grep "^%_gpg_name" ~/.rpmmacros | cut -d' ' -f2-)
        elif [ -f ~/.devscripts ]; then
            GPG_KEY=$(grep "^DEBSIGN_KEYID=" ~/.devscripts | cut -d'=' -f2 | tr -d '"')
        fi
    fi
    
    if [ -z "$GPG_KEY" ]; then
        print_error "GPG_KEY not set. Please export GPG_KEY=your@email.com"
        exit 1
    fi
    
    # Check if key exists
    if ! gpg --list-secret-keys "$GPG_KEY" &> /dev/null; then
        print_error "GPG key not found: $GPG_KEY"
        print_error "Available keys:"
        gpg --list-secret-keys
        exit 1
    fi
    
    print_status "Using GPG key: $GPG_KEY"
}

# Sign RPM packages
sign_rpm() {
    local packages=("$@")
    
    if [ ${#packages[@]} -eq 0 ]; then
        print_error "No RPM packages specified"
        return 1
    fi
    
    # Check if rpm is available
    if ! command -v rpm &> /dev/null; then
        print_error "rpm command not found. Please install rpm-build package."
        return 1
    fi
    
    # Configure rpm if needed
    if [ ! -f ~/.rpmmacros ] || ! grep -q "%_signature gpg" ~/.rpmmacros; then
        print_status "Configuring RPM macros..."
        echo "%_signature gpg" >> ~/.rpmmacros
        echo "%_gpg_name $GPG_KEY" >> ~/.rpmmacros
    fi
    
    # Sign each package
    for rpm in "${packages[@]}"; do
        if [ ! -f "$rpm" ]; then
            print_error "RPM package not found: $rpm"
            continue
        fi
        
        print_status "Signing RPM: $rpm"
        
        if [ -n "$SIGN_PASSPHRASE" ]; then
            # Use expect or similar for automated signing
            echo "$SIGN_PASSPHRASE" | rpm --addsign "$rpm" 2>/dev/null || {
                # Fallback to manual signing
                rpm --addsign "$rpm"
            }
        else
            rpm --addsign "$rpm"
        fi
        
        # Verify signature
        print_status "Verifying signature..."
        if rpm --checksig "$rpm" | grep -q "OK"; then
            print_status "Successfully signed: $rpm"
        else
            print_error "Failed to sign: $rpm"
        fi
    done
}

# Sign DEB packages
sign_deb() {
    local packages=("$@")
    
    if [ ${#packages[@]} -eq 0 ]; then
        print_error "No DEB packages specified"
        return 1
    fi
    
    # Check if dpkg-sig is available
    if ! command -v dpkg-sig &> /dev/null; then
        print_error "dpkg-sig not found. Please install it with: sudo apt-get install dpkg-sig"
        return 1
    fi
    
    # Sign each package
    for deb in "${packages[@]}"; do
        if [ ! -f "$deb" ]; then
            print_error "DEB package not found: $deb"
            continue
        fi
        
        print_status "Signing DEB: $deb"
        
        if [ -n "$SIGN_PASSPHRASE" ]; then
            # Use GPG agent for automated signing
            echo "$SIGN_PASSPHRASE" | gpg --batch --yes --passphrase-fd 0 \
                --pinentry-mode loopback --detach-sign --armor "$deb.sig" < "$deb"
            dpkg-sig --import-sig "$deb.sig" "$deb"
            rm -f "$deb.sig"
        else
            dpkg-sig --sign builder -k "$GPG_KEY" "$deb"
        fi
        
        # Verify signature
        print_status "Verifying signature..."
        if dpkg-sig --verify "$deb" 2>&1 | grep -q "GOODSIG"; then
            print_status "Successfully signed: $deb"
        else
            print_error "Failed to sign: $deb"
        fi
    done
}

# Export public key
export_public_key() {
    local output_file="${1:-RPM-GPG-KEY-keymagic}"
    
    print_status "Exporting public key to: $output_file"
    gpg --armor --export "$GPG_KEY" > "$output_file"
    
    print_status "Public key exported successfully!"
    print_status "Users can import it with:"
    echo "  RPM: sudo rpm --import $output_file"
    echo "  DEB: sudo apt-key add $output_file"
}

# Sign repository metadata
sign_repository() {
    local repo_type="$1"
    local repo_path="${2:-.}"
    
    case "$repo_type" in
        apt|deb)
            print_status "Signing APT repository..."
            cd "$repo_path"
            
            # Generate Release file
            apt-ftparchive release . > Release
            
            # Sign Release file
            gpg --clearsign --digest-algo SHA512 --local-user "$GPG_KEY" -o InRelease Release
            gpg -abs --digest-algo SHA512 --local-user "$GPG_KEY" -o Release.gpg Release
            
            print_status "APT repository signed successfully!"
            ;;
            
        yum|rpm|dnf)
            print_status "Signing YUM/DNF repository..."
            cd "$repo_path"
            
            # Create repository if needed
            if [ ! -d "repodata" ]; then
                createrepo_c .
            fi
            
            # Sign repository metadata
            gpg --detach-sign --armor --local-user "$GPG_KEY" repodata/repomd.xml
            
            print_status "YUM/DNF repository signed successfully!"
            ;;
            
        *)
            print_error "Unknown repository type: $repo_type"
            print_error "Supported types: apt, deb, yum, rpm, dnf"
            return 1
            ;;
    esac
}

# Main function
main() {
    if [ $# -eq 0 ]; then
        echo "Usage: $0 <command> [options]"
        echo ""
        echo "Commands:"
        echo "  rpm <package...>         Sign RPM packages"
        echo "  deb <package...>         Sign DEB packages"
        echo "  export-key [filename]    Export public GPG key"
        echo "  sign-repo <type> [path]  Sign repository metadata"
        echo ""
        echo "Environment variables:"
        echo "  GPG_KEY          GPG key ID or email"
        echo "  SIGN_PASSPHRASE  GPG passphrase (for automation)"
        echo ""
        echo "Examples:"
        echo "  $0 rpm dist/*.rpm"
        echo "  $0 deb dist/*.deb"
        echo "  $0 export-key"
        echo "  $0 sign-repo apt dist/repo"
        exit 1
    fi
    
    local command="$1"
    shift
    
    # Check GPG key for all operations
    check_gpg_key
    
    case "$command" in
        rpm)
            sign_rpm "$@"
            ;;
        deb)
            sign_deb "$@"
            ;;
        export-key)
            export_public_key "$@"
            ;;
        sign-repo)
            sign_repository "$@"
            ;;
        *)
            print_error "Unknown command: $command"
            exit 1
            ;;
    esac
}

# Run main function
main "$@"