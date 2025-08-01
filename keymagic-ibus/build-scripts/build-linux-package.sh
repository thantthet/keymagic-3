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

# Read version from version.txt file
if [ -f "$PROJECT_ROOT/version.txt" ]; then
    VERSION=$(cat "$PROJECT_ROOT/version.txt" | tr -d '\n\r')
else
    echo -e "${RED}Error: version.txt not found in project root${NC}"
    exit 1
fi

echo -e "${BLUE}KeyMagic 3 Linux Package Builder${NC}"
echo "==================================="
echo "Version: $VERSION"
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

# Add target flag if cross-compiling
CARGO_FLAGS=""
if [ -n "$CARGO_BUILD_TARGET" ]; then
    CARGO_FLAGS="--target $CARGO_BUILD_TARGET"
fi

if [ "$BUILD_TYPE" = "release" ]; then
    cargo build --release $CARGO_FLAGS
else
    cargo build $CARGO_FLAGS
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
    cargo build --release $CARGO_FLAGS
else
    cargo build $CARGO_FLAGS
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
    mkdir -p "$pkg_dir/usr/share/icons/hicolor/32x32/apps"
    mkdir -p "$pkg_dir/usr/share/icons/hicolor/128x128/apps"
    mkdir -p "$pkg_dir/usr/share/icons/hicolor/256x256/apps"
    mkdir -p "$pkg_dir/usr/share/doc/keymagic3"
    mkdir -p "$pkg_dir/usr/share/keymagic3"
    mkdir -p "$pkg_dir/usr/share/keymagic3/keyboards"
    
    # Copy binaries
    # Determine target directory based on cross-compilation
    if [ -n "$CARGO_BUILD_TARGET" ]; then
        TARGET_DIR="$PROJECT_ROOT/target/$CARGO_BUILD_TARGET"
    else
        TARGET_DIR="$PROJECT_ROOT/target"
    fi
    
    if [ "$BUILD_TYPE" = "release" ]; then
        cp "$TARGET_DIR/release/keymagic-gui" "$pkg_dir/usr/bin/keymagic3-gui"
    else
        cp "$TARGET_DIR/debug/keymagic-gui" "$pkg_dir/usr/bin/keymagic3-gui"
    fi
    
    cp "$PROJECT_ROOT/keymagic-ibus/ibus-engine-keymagic3" "$pkg_dir/usr/lib/ibus-keymagic3/ibus-engine-keymagic3"
    
    # Copy data files
    cp "$PROJECT_ROOT/keymagic-ibus/data/keymagic3.xml" "$pkg_dir/usr/share/ibus/component/"
    cp "$PROJECT_ROOT/keymagic-ibus/data/keymagic3.desktop" "$pkg_dir/usr/share/applications/" 2>/dev/null || true
    
    # Copy icons
    cp "$PROJECT_ROOT/keymagic-ibus/data/icon-32.png" "$pkg_dir/usr/share/icons/hicolor/32x32/apps/keymagic3.png"
    cp "$PROJECT_ROOT/keymagic-ibus/data/icon-128.png" "$pkg_dir/usr/share/icons/hicolor/128x128/apps/keymagic3.png"
    cp "$PROJECT_ROOT/keymagic-ibus/data/icon-256.png" "$pkg_dir/usr/share/icons/hicolor/256x256/apps/keymagic3.png"
    
    # Copy documentation
    cp "$PROJECT_ROOT/README.md" "$pkg_dir/usr/share/doc/keymagic3/" 2>/dev/null || true
    echo "KeyMagic 3 version $VERSION" > "$pkg_dir/usr/share/doc/keymagic3/VERSION"
    
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
    
    # Determine architecture (support cross-compilation)
    DEB_ARCH="${DEB_TARGET_ARCH:-$(dpkg --print-architecture)}"
    
    PKG_DIR="dist/keymagic3_${VERSION}_${DEB_ARCH}"
    rm -rf "$PKG_DIR"
    create_package_structure "$PKG_DIR"
    
    # Create DEBIAN directory
    mkdir -p "$PKG_DIR/DEBIAN"
    
    # Generate control file from template
    if [ -f "$PROJECT_ROOT/keymagic-ibus/packaging/debian/control.in" ]; then
        sed -e "s/@VERSION@/$VERSION/g" \
            -e "s/@ARCH@/$DEB_ARCH/g" \
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
        
        # Parse version for RPM (handle pre-release versions)
        # RPM doesn't allow hyphens in version, so split version and release
        if [[ "$VERSION" =~ ^([0-9.]+)(-(.+))?$ ]]; then
            RPM_VERSION="${BASH_REMATCH[1]}"
            RPM_RELEASE="${BASH_REMATCH[3]:-1}"
        else
            RPM_VERSION="$VERSION"
            RPM_RELEASE="1"
        fi
        
        # Determine architecture (support cross-compilation)
        RPM_ARCH="${RPM_TARGET_ARCH:-$(uname -m)}"
        
        # Generate spec file from template
        if [ -f "$PROJECT_ROOT/keymagic-ibus/packaging/keymagic3.spec.in" ]; then
            sed -e "s/@VERSION@/$RPM_VERSION/g" \
                -e "s/@RELEASE@/$RPM_RELEASE/g" \
                -e "s/@ARCH@/$RPM_ARCH/g" \
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
        cp "$RPMBUILD_DIR/RPMS/$RPM_ARCH/keymagic3-$RPM_VERSION-$RPM_RELEASE."*".rpm" "$PROJECT_ROOT/keymagic-ibus/dist/"
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
echo "  sudo dpkg -i keymagic-ibus/dist/keymagic3_${VERSION}_*.deb"
echo "  sudo apt-get install -f  # Install any missing dependencies"
echo ""
echo "To test IBus engine directly:"
echo "  cd $PROJECT_ROOT/keymagic-ibus && ./test-debug.sh"