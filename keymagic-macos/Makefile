# Makefile for KeyMagic macOS
# This builds the Rust core library, Swift IMK server app, and Tauri GUI app

.PHONY: all clean install uninstall build-core build-swift build-gui install-imk install-gui uninstall-imk uninstall-gui dev help

# Configuration
IMK_APP_NAME = KeyMagic3-Server.app
GUI_APP_NAME = KeyMagic3.app
IMK_BUNDLE_ID = org.keymagic.inputmethod.KeyMagic3
GUI_BUNDLE_ID = net.keymagic.app
BUILD_DIR = build
CORE_LIB = libkeymagic_core.a
INSTALL_DIR = $(HOME)/Library/Input Methods

# Build configuration (default to release)
BUILD_CONFIG ?= release
ifeq ($(BUILD_CONFIG),debug)
    RUST_TARGET_DIR = ../target/debug
    GUI_BUILD_DIR = ../target/debug/bundle/macos
    SWIFT_FLAGS = -Onone -DDEBUG
    CARGO_FLAGS = 
    TAURI_FLAGS = --debug
else
    RUST_TARGET_DIR = ../target/release
    GUI_BUILD_DIR = ../target/release/bundle/macos
    SWIFT_FLAGS = -O
    CARGO_FLAGS = --release
    TAURI_FLAGS = 
endif

# Default target
all: build-core build-swift build-gui

# Build core Rust library
build-core:
	@echo "Building keymagic-core library ($(BUILD_CONFIG))..."
	cd .. && cargo build -p keymagic-core $(CARGO_FLAGS)

# Build Swift app bundle with C bridge
build-swift: build-core
	@echo "Building Swift app bundle ($(BUILD_CONFIG))..."
	@mkdir -p $(BUILD_DIR)
	
	# Compile Swift sources directly with keymagic-core (force static linking)
	swiftc \
		$(SWIFT_FLAGS) \
		-o $(BUILD_DIR)/KeyMagic3 \
		-module-name KeyMagic3 \
		-import-objc-header src/swift/KeyMagic-Bridging-Header.h \
		$(RUST_TARGET_DIR)/$(CORE_LIB) \
		-lc++ \
		-framework InputMethodKit \
		-framework Carbon \
		-framework Foundation \
		-framework AppKit \
		src/swift/main.swift \
		src/swift/KMInputController.swift \
		src/swift/KeycodeMapping.swift \
		src/swift/KMConfiguration.swift
	
	# Create app bundle structure
	@mkdir -p "$(BUILD_DIR)/$(IMK_APP_NAME)/Contents/MacOS"
	@mkdir -p "$(BUILD_DIR)/$(IMK_APP_NAME)/Contents/Resources"
	
	# Copy executable
	@cp $(BUILD_DIR)/KeyMagic3 "$(BUILD_DIR)/$(IMK_APP_NAME)/Contents/MacOS/"
	
	# Copy Info.plist
	@cp Info.plist "$(BUILD_DIR)/$(IMK_APP_NAME)/Contents/"
	
	# Copy app icon
	@cp ../resources/icons/KeyMagic.icns "$(BUILD_DIR)/$(IMK_APP_NAME)/Contents/Resources/KeyMagic3.icns"
	
	# Set executable permissions
	@chmod +x "$(BUILD_DIR)/$(IMK_APP_NAME)/Contents/MacOS/KeyMagic3"
	
	@echo "Build complete: $(BUILD_DIR)/$(IMK_APP_NAME)"

# Build GUI application bundle
build-gui: build-core
	@echo "Building KeyMagic GUI application ($(BUILD_CONFIG))..."
	cd ../keymagic-shared/gui && cargo tauri build $(TAURI_FLAGS) --bundles app
	
	# Copy the built app to our build directory
	@mkdir -p $(BUILD_DIR)
	@rm -rf "$(BUILD_DIR)/$(GUI_APP_NAME)"
	@if [ -d "$(GUI_BUILD_DIR)/KeyMagic.app" ]; then \
		cp -R "$(GUI_BUILD_DIR)/KeyMagic.app" "$(BUILD_DIR)/$(GUI_APP_NAME)"; \
	else \
		echo "Error: GUI app bundle not found at $(GUI_BUILD_DIR)/KeyMagic.app"; \
		echo "Available files:"; \
		ls -la "$(GUI_BUILD_DIR)" 2>/dev/null || echo "Build directory not found"; \
		exit 1; \
	fi
	
	@echo "GUI build complete: $(BUILD_DIR)/$(GUI_APP_NAME)"

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	@rm -rf $(BUILD_DIR)
	@echo "Cleaning Tauri build artifacts..."
	cd ../keymagic-shared/gui && cargo clean

# Install IMK server to user's Input Methods
install-imk: build-core build-swift
	@echo "Installing KeyMagic IMK server..."
	@mkdir -p "$(INSTALL_DIR)"
	@rm -rf "$(INSTALL_DIR)/$(IMK_APP_NAME)"
	@cp -R "$(BUILD_DIR)/$(IMK_APP_NAME)" "$(INSTALL_DIR)/"
	
	@echo "IMK server installation complete."
	@echo "Please log out and log back in for the input method to be registered."
	@echo "Then go to System Settings > Keyboard > Input Sources to add KeyMagic."

# Install GUI application to Applications folder
install-gui: build-gui
	@echo "Installing KeyMagic GUI application..."
	@rm -rf "/Applications/$(GUI_APP_NAME)"
	@cp -R "$(BUILD_DIR)/$(GUI_APP_NAME)" "/Applications/"
	
	@echo "GUI application installed to /Applications/$(GUI_APP_NAME)"

# Install both IMK server and GUI
install: install-imk install-gui
	@echo "Full KeyMagic installation complete."

# Uninstall both IMK server and GUI
uninstall: uninstall-imk uninstall-gui
	@echo "Full KeyMagic uninstallation complete."

# Uninstall IMK server
uninstall-imk:
	@echo "Uninstalling KeyMagic IMK server..."
	@rm -rf "$(INSTALL_DIR)/$(IMK_APP_NAME)"
	@echo "IMK server uninstalled. You may need to log out and log back in."

# Uninstall GUI application
uninstall-gui:
	@echo "Uninstalling KeyMagic GUI application..."
	@rm -rf "/Applications/$(GUI_APP_NAME)"
	@echo "GUI application uninstalled."

# Development: build and install
dev: all install

# Show help
help:
	@echo "KeyMagic macOS Build System"
	@echo ""
	@echo "Build Configuration:"
	@echo "  BUILD_CONFIG=debug make <target>    - Build with debug symbols and logging"
	@echo "  BUILD_CONFIG=release make <target>  - Build optimized with redacted logging (default)"
	@echo ""
	@echo "Available targets:"
	@echo "  make all           - Build everything (core, IMK server, GUI)"
	@echo "  make build-core    - Build only the Rust core library"
	@echo "  make build-swift   - Build only the IMK server app bundle"
	@echo "  make build-gui     - Build only the GUI application"
	@echo "  make clean         - Clean all build artifacts"
	@echo ""
	@echo "Installation targets:"
	@echo "  make install       - Install both IMK server and GUI"
	@echo "  make install-imk   - Install only IMK server to ~/Library/Input Methods"
	@echo "  make install-gui   - Install only GUI to /Applications"
	@echo ""
	@echo "Uninstallation targets:"
	@echo "  make uninstall     - Uninstall both IMK server and GUI"
	@echo "  make uninstall-imk - Uninstall only IMK server"
	@echo "  make uninstall-gui - Uninstall only GUI"
	@echo ""
	@echo "Other:"
	@echo "  make dev           - Build and install everything (for development)"
	@echo "  make help          - Show this help message"
	@echo ""
	@echo "Examples:"
	@echo "  make BUILD_CONFIG=debug all         - Build everything with debug logging"
	@echo "  make BUILD_CONFIG=debug install-imk - Install IMK with debug logging"
	@echo "  make                                - Build release version (default)"