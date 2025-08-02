#!/bin/bash

# KeyMagic Linux Installer Script
# Usage: curl -fsSL https://thantthet.github.io/keymagic-3/install.sh | bash

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Repository base URL
REPO_BASE_URL="https://thantthet.github.io/keymagic-3"
GPG_KEY_URL="${REPO_BASE_URL}/keymagic.gpg"

# Print colored output
print_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

# Check if running as root or with sudo
check_sudo() {
    if [[ $EUID -eq 0 ]]; then
        SUDO=""
    elif command -v sudo &> /dev/null; then
        SUDO="sudo"
        # Test sudo access
        if ! sudo -n true 2>/dev/null; then
            print_error "This script requires sudo privileges. Please run with sudo or as root."
            exit 1
        fi
    else
        print_error "This script requires root privileges, but sudo is not available."
        exit 1
    fi
}

# Detect Linux distribution
detect_distro() {
    if [ -f /etc/os-release ]; then
        . /etc/os-release
        OS=$ID
        VER=$VERSION_ID
        PRETTY_NAME=$PRETTY_NAME
    elif [ -f /etc/debian_version ]; then
        OS="debian"
        VER=$(cat /etc/debian_version)
    elif [ -f /etc/redhat-release ]; then
        OS="rhel"
        VER=$(rpm -q --queryformat '%{VERSION}' centos-release || rpm -q --queryformat '%{VERSION}' redhat-release)
    else
        print_error "Cannot detect Linux distribution"
        exit 1
    fi
    
    # Normalize OS names
    case "$OS" in
        ubuntu|debian|linuxmint|pop|elementary|zorin)
            PKG_MANAGER="apt"
            ;;
        fedora|rhel|centos|rocky|almalinux|ol)
            PKG_MANAGER="dnf"
            # Check if dnf is available, fallback to yum
            if ! command -v dnf &> /dev/null; then
                PKG_MANAGER="yum"
            fi
            ;;
        *)
            print_error "Unsupported distribution: $OS"
            print_info "This installer supports: Ubuntu, Debian, Fedora, RHEL, CentOS, Rocky Linux, AlmaLinux"
            exit 1
            ;;
    esac
}

# Install APT repository
install_apt_repo() {
    print_info "Setting up APT repository for $PRETTY_NAME..."
    
    # Update package list
    print_info "Updating package list..."
    $SUDO apt-get update -qq
    
    # Install required packages
    print_info "Installing required packages..."
    $SUDO apt-get install -y -qq curl gnupg
    
    # Create keyrings directory if it doesn't exist
    $SUDO mkdir -p /usr/share/keyrings
    
    # Download and add GPG key
    print_info "Adding GPG key..."
    curl -fsSL "$GPG_KEY_URL" | $SUDO gpg --dearmor -o /usr/share/keyrings/keymagic.gpg
    
    # Add repository
    print_info "Adding repository..."
    echo "deb [signed-by=/usr/share/keyrings/keymagic.gpg] ${REPO_BASE_URL}/deb stable main" | \
        $SUDO tee /etc/apt/sources.list.d/keymagic.list > /dev/null
    
    # Update package list with new repository
    print_info "Updating package list with new repository..."
    $SUDO apt-get update -qq
    
    # Install KeyMagic
    print_info "Installing KeyMagic..."
    $SUDO apt-get install -y keymagic3
}

# Install YUM/DNF repository
install_dnf_repo() {
    print_info "Setting up $PKG_MANAGER repository for $PRETTY_NAME..."
    
    # Install required packages
    print_info "Installing required packages..."
    $SUDO $PKG_MANAGER install -y -q curl
    
    # Download and add repository config
    print_info "Adding repository configuration..."
    $SUDO curl -fsSL "${REPO_BASE_URL}/rpm/keymagic.repo" -o /etc/yum.repos.d/keymagic.repo
    
    # Import GPG key
    print_info "Importing GPG key..."
    $SUDO rpm --import "$GPG_KEY_URL"
    
    # Clean cache
    print_info "Cleaning package cache..."
    $SUDO $PKG_MANAGER clean all -q
    
    # Install KeyMagic
    print_info "Installing KeyMagic..."
    $SUDO $PKG_MANAGER install -y keymagic3
}

# Post-installation instructions
post_install() {
    print_success "KeyMagic has been successfully installed!"
    echo ""
    print_info "Post-installation steps:"
    echo "  1. Restart IBus: ibus restart"
    echo "  2. Or log out and log back in"
    echo "  3. Add KeyMagic in your system's input method settings"
    echo ""
    print_info "For more information, visit: https://github.com/thantthet/keymagic-3"
}

# Main installation flow
main() {
    echo "========================================="
    echo "       KeyMagic Linux Installer"
    echo "========================================="
    echo ""
    
    # Check sudo/root access
    check_sudo
    
    # Detect distribution
    detect_distro
    
    print_info "Detected: $PRETTY_NAME"
    print_info "Package manager: $PKG_MANAGER"
    echo ""
    
    # Confirm installation
    read -p "Do you want to install KeyMagic? [Y/n] " -n 1 -r
    echo ""
    if [[ ! $REPLY =~ ^[Yy]$ ]] && [[ ! -z $REPLY ]]; then
        print_info "Installation cancelled."
        exit 0
    fi
    
    # Install based on package manager
    case "$PKG_MANAGER" in
        apt)
            install_apt_repo
            ;;
        dnf|yum)
            install_dnf_repo
            ;;
        *)
            print_error "Unsupported package manager: $PKG_MANAGER"
            exit 1
            ;;
    esac
    
    # Show post-installation instructions
    post_install
}

# Handle errors
trap 'print_error "An error occurred during installation. Please check the output above for details."' ERR

# Run main function
main