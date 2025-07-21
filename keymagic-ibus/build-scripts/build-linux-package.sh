#!/bin/bash
# Build script for KeyMagic Linux packages
# This script builds both the GUI and IBus engine, then creates packages

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script directory
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
# Get project root (two levels up from build-scripts)
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
cd "$PROJECT_ROOT"

echo -e "${BLUE}KeyMagic 3 Linux Package Builder${NC}"
echo "==================================="
echo "Version: 0.0.1"
echo "Contact: contact@keymagic.net"
echo ""

# Parse command line arguments
BUILD_TYPE="release"
PACKAGE_FORMAT="all"

while [[ $# -gt 0 ]]; do
    case $1 in
        --debug)
            BUILD_TYPE="debug"
            shift
            ;;
        --all)
            PACKAGE_FORMAT="all"
            shift
            ;;
        --deb)
            PACKAGE_FORMAT="deb"
            shift
            ;;
        --rpm)
            PACKAGE_FORMAT="rpm"
            shift
            ;;
        --appimage)
            PACKAGE_FORMAT="appimage"
            shift
            ;;
        --help)
            echo "Usage: $0 [options]"
            echo "Options:"
            echo "  --debug      Build debug version"
            echo "  --all        Build all package formats (default)"
            echo "  --deb        Build only Debian package"
            echo "  --rpm        Build only RPM package"
            echo "  --appimage   Build only AppImage"
            echo "  --help       Show this help"
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            exit 1
            ;;
    esac
done

# Check dependencies
echo -e "${YELLOW}Checking build dependencies...${NC}"
command -v rustc >/dev/null 2>&1 || { echo -e "${RED}Rust is required but not installed.${NC}"; exit 1; }
command -v cargo >/dev/null 2>&1 || { echo -e "${RED}Cargo is required but not installed.${NC}"; exit 1; }
command -v gcc >/dev/null 2>&1 || { echo -e "${RED}GCC is required but not installed.${NC}"; exit 1; }
command -v pkg-config >/dev/null 2>&1 || { echo -e "${RED}pkg-config is required but not installed.${NC}"; exit 1; }

# Check IBus development headers
pkg-config --exists ibus-1.0 || { echo -e "${RED}IBus development headers not found.${NC}"; exit 1; }
pkg-config --exists glib-2.0 || { echo -e "${RED}GLib development headers not found.${NC}"; exit 1; }

echo -e "${GREEN}Dependencies OK${NC}"
echo ""

# Step 1: Build keymagic-core
echo -e "${BLUE}Step 1: Building keymagic-core...${NC}"
cd "$PROJECT_ROOT/keymagic-core"
if [ "$BUILD_TYPE" = "release" ]; then
    cargo build --release
else
    cargo build
fi
echo -e "${GREEN}✓ keymagic-core built${NC}"
echo ""

# Step 2: Build IBus engine
echo -e "${BLUE}Step 2: Building IBus engine...${NC}"
cd "$PROJECT_ROOT/keymagic-ibus"
make clean
if [ "$BUILD_TYPE" = "release" ]; then
    make release
else
    make debug
fi
echo -e "${GREEN}✓ IBus engine built${NC}"
echo ""

# Step 3: Build GUI
echo -e "${BLUE}Step 3: Building GUI application...${NC}"
cd "$PROJECT_ROOT/keymagic-shared/gui/src-tauri"
if [ "$BUILD_TYPE" = "release" ]; then
    cargo build --release
else
    cargo build
fi
echo -e "${GREEN}✓ GUI application built${NC}"
echo ""

# Step 4: Create packages
cd "$PROJECT_ROOT/keymagic-ibus"
mkdir -p dist

# Helper function to create package structure
create_package_structure() {
    local pkg_dir="$1"
    
    # Create directory structure
    mkdir -p "$pkg_dir/usr/bin"
    mkdir -p "$pkg_dir/usr/lib/ibus-keymagic3"
    mkdir -p "$pkg_dir/usr/share/ibus/component"
    mkdir -p "$pkg_dir/usr/share/applications"
    mkdir -p "$pkg_dir/usr/share/icons/hicolor/256x256/apps"
    mkdir -p "$pkg_dir/usr/share/doc/keymagic3"
    mkdir -p "$pkg_dir/usr/share/keymagic3"
    mkdir -p "$pkg_dir/usr/share/keymagic3/keyboards"
    
    # Copy binaries
    if [ "$BUILD_TYPE" = "release" ]; then
        cp "$PROJECT_ROOT/target/release/keymagic-gui" "$pkg_dir/usr/bin/keymagic3-gui"
    else
        cp "$PROJECT_ROOT/target/debug/keymagic-gui" "$pkg_dir/usr/bin/keymagic3-gui"
    fi
    
    cp "$PROJECT_ROOT/keymagic-ibus/ibus-engine-keymagic3" "$pkg_dir/usr/lib/ibus-keymagic3/ibus-engine-keymagic3"
    
    # Copy data files
    cp "$PROJECT_ROOT/keymagic-ibus/data/keymagic3.xml" "$pkg_dir/usr/share/ibus/component/"
    cp "$PROJECT_ROOT/keymagic-ibus/data/keymagic3.desktop" "$pkg_dir/usr/share/applications/" 2>/dev/null || true
    
    # Copy icon if exists
    if [ -f "$PROJECT_ROOT/resources/icons/keymagic.png" ]; then
        cp "$PROJECT_ROOT/resources/icons/keymagic.png" "$pkg_dir/usr/share/icons/hicolor/256x256/apps/keymagic3.png"
    fi
    
    # Copy documentation
    cp "$PROJECT_ROOT/README.md" "$pkg_dir/usr/share/doc/keymagic3/" 2>/dev/null || true
    echo "KeyMagic 3 version 0.0.1" > "$pkg_dir/usr/share/doc/keymagic3/VERSION"
    
    # Copy helper scripts
    if [ -f "$PROJECT_ROOT/keymagic-ibus/packaging/debian/keymagic3-ibus-refresh" ]; then
        cp "$PROJECT_ROOT/keymagic-ibus/packaging/debian/keymagic3-ibus-refresh" "$pkg_dir/usr/share/keymagic3/"
    fi
    
    # Copy bundled keyboard files
    if [ -d "$PROJECT_ROOT/keymagic-ibus/data/keyboards" ]; then
        echo "Copying bundled keyboard files..."
        for km2_file in "$PROJECT_ROOT/keymagic-ibus/data/keyboards"/*.km2; do
            if [ -f "$km2_file" ]; then
                cp "$km2_file" "$pkg_dir/usr/share/keymagic3/keyboards/"
                echo "  - $(basename "$km2_file")"
            fi
        done
    fi
}

# Build Debian package
if [ "$PACKAGE_FORMAT" = "all" ] || [ "$PACKAGE_FORMAT" = "deb" ]; then
    echo -e "${BLUE}Creating Debian package...${NC}"
    
    PKG_DIR="dist/keymagic3_0.0.1_$(dpkg --print-architecture)"
    rm -rf "$PKG_DIR"
    create_package_structure "$PKG_DIR"
    
    # Create DEBIAN directory
    mkdir -p "$PKG_DIR/DEBIAN"
    
    # Generate control file from template
    if [ -f "$PROJECT_ROOT/keymagic-ibus/packaging/debian/control.in" ]; then
        sed -e "s/@VERSION@/0.0.1/g" \
            -e "s/@ARCH@/$(dpkg --print-architecture)/g" \
            "$PROJECT_ROOT/keymagic-ibus/packaging/debian/control.in" > "$PKG_DIR/DEBIAN/control"
    else
        echo -e "${RED}Debian control template not found${NC}"
        exit 1
    fi

    # Copy maintainer scripts
    if [ -d "$PROJECT_ROOT/keymagic-ibus/packaging/debian" ]; then
        cp "$PROJECT_ROOT/keymagic-ibus/packaging/debian/postinst" "$PKG_DIR/DEBIAN/" 2>/dev/null || true
        cp "$PROJECT_ROOT/keymagic-ibus/packaging/debian/prerm" "$PKG_DIR/DEBIAN/" 2>/dev/null || true
        cp "$PROJECT_ROOT/keymagic-ibus/packaging/debian/postrm" "$PKG_DIR/DEBIAN/" 2>/dev/null || true
        chmod 755 "$PKG_DIR/DEBIAN/"* 2>/dev/null || true
    fi
    
    # Build package
    dpkg-deb --build "$PKG_DIR"
    echo -e "${GREEN}✓ Debian package created: ${PKG_DIR}.deb${NC}"
fi

# Build RPM package
if [ "$PACKAGE_FORMAT" = "all" ] || [ "$PACKAGE_FORMAT" = "rpm" ]; then
    if command -v rpmbuild >/dev/null 2>&1; then
        echo -e "${BLUE}Creating RPM package...${NC}"
        
        # Create RPM build structure
        RPMBUILD_DIR="$HOME/rpmbuild"
        mkdir -p "$RPMBUILD_DIR"/{BUILD,RPMS,SOURCES,SPECS,SRPMS}
        
        # Generate spec file from template
        if [ -f "$PROJECT_ROOT/keymagic-ibus/packaging/keymagic3.spec.in" ]; then
            sed -e "s/@VERSION@/0.0.1/g" \
                -e "s/@ARCH@/$(uname -m)/g" \
                -e "s/@DATE@/$(date +"%a %b %d %Y")/g" \
                -e "s|@PROJECT_ROOT@|$PROJECT_ROOT|g" \
                "$PROJECT_ROOT/keymagic-ibus/packaging/keymagic3.spec.in" > "$RPMBUILD_DIR/SPECS/keymagic3.spec"
        else
            echo -e "${RED}RPM spec template not found${NC}"
            exit 1
        fi

        # Build RPM
        cd "$RPMBUILD_DIR/SPECS"
        rpmbuild -bb keymagic3.spec
        
        # Copy to dist directory
        cp "$RPMBUILD_DIR/RPMS/$(uname -m)/keymagic3-0.0.1-1."*".rpm" "$PROJECT_ROOT/keymagic-ibus/dist/"
        echo -e "${GREEN}✓ RPM package created${NC}"
    else
        echo -e "${YELLOW}rpmbuild not found, skipping RPM creation${NC}"
    fi
fi

# Build AppImage
if [ "$PACKAGE_FORMAT" = "all" ] || [ "$PACKAGE_FORMAT" = "appimage" ]; then
    if command -v appimagetool >/dev/null 2>&1; then
        echo -e "${BLUE}Creating AppImage...${NC}"
        # AppImage building would go here
        echo -e "${YELLOW}AppImage building not yet implemented${NC}"
    fi
fi

echo ""
echo -e "${GREEN}Build complete!${NC}"
echo "Packages are available in the keymagic-ibus/dist/ directory"
echo ""
echo "To install the Debian package:"
echo "  sudo dpkg -i keymagic-ibus/dist/keymagic3_0.0.1_*.deb"
echo "  sudo apt-get install -f  # Install any missing dependencies"
echo ""
echo "To test IBus engine directly:"
echo "  cd $PROJECT_ROOT/keymagic-ibus && ./test-debug.sh"