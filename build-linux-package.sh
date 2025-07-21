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
cd "$SCRIPT_DIR"

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
cd "$SCRIPT_DIR/keymagic-core"
if [ "$BUILD_TYPE" = "release" ]; then
    cargo build --release
else
    cargo build
fi
echo -e "${GREEN}✓ keymagic-core built${NC}"
echo ""

# Step 2: Build IBus engine
echo -e "${BLUE}Step 2: Building IBus engine...${NC}"
cd "$SCRIPT_DIR/keymagic-ibus"
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
cd "$SCRIPT_DIR/keymagic-shared/gui/src-tauri"
if [ "$BUILD_TYPE" = "release" ]; then
    cargo build --release
else
    cargo build
fi
echo -e "${GREEN}✓ GUI application built${NC}"
echo ""

# Step 4: Create packages
cd "$SCRIPT_DIR"
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
    
    # Copy binaries
    if [ "$BUILD_TYPE" = "release" ]; then
        cp "target/release/keymagic-gui" "$pkg_dir/usr/bin/keymagic3-gui"
    else
        cp "target/debug/keymagic-gui" "$pkg_dir/usr/bin/keymagic3-gui"
    fi
    
    cp "keymagic-ibus/ibus-engine-keymagic3" "$pkg_dir/usr/lib/ibus-keymagic3/ibus-engine-keymagic3"
    
    # Copy data files
    cp "keymagic-ibus/data/keymagic3.xml" "$pkg_dir/usr/share/ibus/component/"
    cp "keymagic-shared/gui/assets/keymagic3.desktop" "$pkg_dir/usr/share/applications/" 2>/dev/null || true
    
    # Copy icon if exists
    if [ -f "./resources/icons/keymagic.png" ]; then
        cp "./resources/icons/keymagic.png" "$pkg_dir/usr/share/icons/hicolor/256x256/apps/keymagic3.png"
    fi
    
    # Copy documentation
    cp "README.md" "$pkg_dir/usr/share/doc/keymagic3/" 2>/dev/null || true
    echo "KeyMagic 3 version 0.0.1" > "$pkg_dir/usr/share/doc/keymagic3/VERSION"
}

# Build Debian package
if [ "$PACKAGE_FORMAT" = "all" ] || [ "$PACKAGE_FORMAT" = "deb" ]; then
    echo -e "${BLUE}Creating Debian package...${NC}"
    
    PKG_DIR="dist/keymagic3_0.0.1_$(dpkg --print-architecture)"
    rm -rf "$PKG_DIR"
    create_package_structure "$PKG_DIR"
    
    # Create DEBIAN directory
    mkdir -p "$PKG_DIR/DEBIAN"
    
    # Create control file
    cat > "$PKG_DIR/DEBIAN/control" << EOF
Package: keymagic3
Version: 0.0.1
Section: utils
Priority: optional
Architecture: $(dpkg --print-architecture)
Maintainer: Thant Thet Khin Zaw <contact@keymagic.net>
Depends: libc6 (>= 2.31), libgtk-3-0 (>= 3.24), ibus (>= 1.5.0)
Recommends: fonts-myanmar
Homepage: https://github.com/thantthet/keymagic-3
Description: KeyMagic 3 - Smart keyboard input method
 KeyMagic 3 is a powerful and flexible input method that allows users to type
 in Myanmar and other complex scripts using standard keyboards.
EOF

    # Copy maintainer scripts
    if [ -d "keymagic-shared/gui/debian" ]; then
        cp keymagic-shared/gui/debian/postinst "$PKG_DIR/DEBIAN/" 2>/dev/null || true
        cp keymagic-shared/gui/debian/prerm "$PKG_DIR/DEBIAN/" 2>/dev/null || true
        cp keymagic-shared/gui/debian/postrm "$PKG_DIR/DEBIAN/" 2>/dev/null || true
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
        
        # Create spec file
        cat > "$RPMBUILD_DIR/SPECS/keymagic3.spec" << EOF
Name:           keymagic3
Version:        0.0.1
Release:        1%{?dist}
Summary:        KeyMagic 3 - Smart keyboard input method
License:        GPL-3.0
URL:            https://github.com/thantthet/keymagic-3
BuildArch:      $(uname -m)

%description
KeyMagic 3 is a powerful and flexible input method that allows users to type
in Myanmar and other complex scripts using standard keyboards.

%prep
# No prep needed as we're using pre-built binaries

%build
# No build needed as we're using pre-built binaries

%install
rm -rf %{buildroot}
mkdir -p %{buildroot}/usr/bin
mkdir -p %{buildroot}/usr/lib/ibus-keymagic3
mkdir -p %{buildroot}/usr/share/ibus/component
mkdir -p %{buildroot}/usr/share/applications
mkdir -p %{buildroot}/usr/share/icons/hicolor/256x256/apps
mkdir -p %{buildroot}/usr/share/doc/keymagic3

# Copy files from our build
cp $SCRIPT_DIR/target/release/keymagic-gui %{buildroot}/usr/bin/keymagic3-gui
cp $SCRIPT_DIR/keymagic-ibus/ibus-engine-keymagic3 %{buildroot}/usr/lib/ibus-keymagic3/
cp $SCRIPT_DIR/keymagic-ibus/data/keymagic3.xml %{buildroot}/usr/share/ibus/component/
cp $SCRIPT_DIR/keymagic-shared/gui/assets/keymagic3.desktop %{buildroot}/usr/share/applications/ 2>/dev/null || true
cp $SCRIPT_DIR/README.md %{buildroot}/usr/share/doc/keymagic3/ 2>/dev/null || true

%files
%defattr(-,root,root,-)
/usr/bin/keymagic3-gui
/usr/lib/ibus-keymagic3/ibus-engine-keymagic3
/usr/share/ibus/component/keymagic3.xml
/usr/share/applications/keymagic3.desktop
%doc /usr/share/doc/keymagic3/

%post
# Register with IBus
if command -v ibus write-cache >/dev/null 2>&1; then
    ibus write-cache
fi

%postun
# Cleanup IBus cache
if [ "$1" = "0" ] && command -v ibus write-cache >/dev/null 2>&1; then
    ibus write-cache
fi

%changelog
* $(date +"%a %b %d %Y") Thant Thet Khin Zaw <contact@keymagic.net> - 0.0.1-1
- Initial release of KeyMagic 3
EOF

        # Build RPM
        cd "$RPMBUILD_DIR/SPECS"
        SCRIPT_DIR="$SCRIPT_DIR" rpmbuild -bb keymagic3.spec
        
        # Copy to dist directory
        cp "$RPMBUILD_DIR/RPMS/$(uname -m)/keymagic3-0.0.1-1."*".rpm" "$SCRIPT_DIR/dist/"
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
echo "Packages are available in the dist/ directory"
echo ""
echo "To install the Debian package:"
echo "  sudo dpkg -i dist/keymagic3_0.0.1_*.deb"
echo "  sudo apt-get install -f  # Install any missing dependencies"
echo ""
echo "To test IBus engine directly:"
echo "  cd keymagic-ibus && ./test-debug.sh"