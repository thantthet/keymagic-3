#!/bin/bash
# Installation script for KeyMagic IBus engine

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}KeyMagic IBus Engine Installation${NC}"
echo "=================================="

# Check if running as root for system install
if [ "$EUID" -eq 0 ]; then 
   echo "Installing system-wide..."
   PREFIX="/usr"
else
   echo -e "${YELLOW}Not running as root. Installing to user directory...${NC}"
   PREFIX="$HOME/.local"
fi

# Create directories
echo "Creating directories..."
mkdir -p "$PREFIX/lib/ibus-keymagic"
mkdir -p "$PREFIX/share/ibus/component"

# Build if not already built
if [ ! -f ibus-engine-keymagic ]; then
    echo "Building engine..."
    make clean
    make
fi

# Install engine
echo "Installing engine executable..."
install -m 755 ibus-engine-keymagic "$PREFIX/lib/ibus-keymagic/"

# Update component XML with correct path
echo "Installing IBus component definition..."
sed "s|/usr/lib/ibus-keymagic|$PREFIX/lib/ibus-keymagic|g" data/keymagic.xml > /tmp/keymagic.xml
install -m 644 /tmp/keymagic.xml "$PREFIX/share/ibus/component/"
rm /tmp/keymagic.xml

# Restart IBus
echo "Restarting IBus..."
ibus restart || true
sleep 2

# Register component
echo "Registering component..."
ibus register-component "$PREFIX/share/ibus/component/keymagic.xml" || true

echo ""
echo -e "${GREEN}Installation complete!${NC}"
echo ""
echo "To use KeyMagic:"
echo "1. Run 'ibus-setup' and add KeyMagic to your input methods"
echo "2. Switch to KeyMagic using your input method switcher"
echo ""
echo "To test immediately:"
echo "  ibus engine keymagic"
echo ""
echo "For debugging, run:"
echo "  ./test-debug.sh"