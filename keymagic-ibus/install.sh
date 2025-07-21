#!/bin/bash
# Installation/Uninstallation script for KeyMagic IBus engine

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default action is install
ACTION="install"

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --uninstall|-u)
            ACTION="uninstall"
            shift
            ;;
        --help|-h)
            echo "Usage: $0 [options]"
            echo "Options:"
            echo "  --uninstall, -u    Uninstall KeyMagic IBus engine"
            echo "  --help, -h         Show this help message"
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done
# Check if running as root for system install
if [ "$EUID" -eq 0 ]; then 
   PREFIX="/usr"
   INSTALL_TYPE="system-wide"
else
   PREFIX="$HOME/.local"
   INSTALL_TYPE="user"
fi

# Uninstall function
uninstall_keymagic() {
    echo -e "${BLUE}KeyMagic 3 IBus Engine Uninstallation${NC}"
    echo "========================================"
    echo "Uninstalling from $INSTALL_TYPE installation..."
    
    # Remove engine executable
    if [ -f "$PREFIX/lib/ibus-keymagic3/ibus-engine-keymagic3" ]; then
        echo "Removing engine executable..."
        rm -f "$PREFIX/lib/ibus-keymagic3/ibus-engine-keymagic3"
        # Remove directory if empty
        rmdir "$PREFIX/lib/ibus-keymagic3" 2>/dev/null || true
    fi
    
    # Remove component XML
    if [ -f "$PREFIX/share/ibus/component/keymagic3.xml" ]; then
        echo "Removing IBus component definition..."
        rm -f "$PREFIX/share/ibus/component/keymagic3.xml"
    fi
    
    # Restart IBus to update component list
    echo "Restarting IBus..."
    ibus restart || true
    
    echo ""
    echo -e "${GREEN}Uninstallation complete!${NC}"
    echo ""
    echo "Note: User configuration and keyboard files in ~/.config/keymagic3"
    echo "and ~/.local/share/keymagic3 have been preserved."
}

# Install function
install_keymagic() {
    echo -e "${GREEN}KeyMagic 3 IBus Engine Installation${NC}"
    echo "======================================"
    echo "Installing $INSTALL_TYPE..."
    
    # Create directories
    echo "Creating directories..."
    mkdir -p "$PREFIX/lib/ibus-keymagic3"
    mkdir -p "$PREFIX/share/ibus/component"
    
    # Build if not already built
    if [ ! -f ibus-engine-keymagic3 ]; then
        echo "Building engine..."
        make clean
        make
    fi
    
    # Install engine
    echo "Installing engine executable..."
    install -m 755 ibus-engine-keymagic3 "$PREFIX/lib/ibus-keymagic3/ibus-engine-keymagic3"
    
    # Update component XML with correct path
    echo "Installing IBus component definition..."
    sed "s|/usr/lib/ibus-keymagic3|$PREFIX/lib/ibus-keymagic3|g" data/keymagic3.xml > /tmp/keymagic3.xml
    install -m 644 /tmp/keymagic3.xml "$PREFIX/share/ibus/component/"
    rm /tmp/keymagic3.xml
    
    # Restart IBus to pick up the new component
    echo "Restarting IBus..."
    ibus restart || true
    
    echo ""
    echo -e "${GREEN}Installation complete!${NC}"
    echo ""
    echo "To use KeyMagic 3:"
    echo "1. Run 'ibus-setup' and add KeyMagic 3 to your input methods"
    echo "2. Switch to KeyMagic 3 using your input method switcher"
    echo ""
    echo "To test immediately:"
    echo "  ibus engine keymagic3"
    echo ""
    echo "For debugging, run:"
    echo "  ./test-debug.sh"
    echo ""
    echo "To uninstall, run:"
    echo "  $0 --uninstall"
}

# Execute the appropriate action
case $ACTION in
    install)
        install_keymagic
        ;;
    uninstall)
        uninstall_keymagic
        ;;
    *)
        echo -e "${RED}Unknown action: $ACTION${NC}"
        exit 1
        ;;
esac