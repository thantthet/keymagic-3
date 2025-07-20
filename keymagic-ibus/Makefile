# KeyMagic IBus Engine Makefile
#
# This Makefile builds the KeyMagic IBus engine and prepares it for installation.
# It requires IBus development headers and links against the keymagic-core library.

# Project information
PROJECT_NAME = ibus-engine-keymagic
VERSION = 1.0.0

# Directories
SRCDIR = src
DATADIR = data
BUILDDIR = build
DESTDIR ?= 
PREFIX ?= /usr

# Installation paths
ENGINEDIR = $(DESTDIR)$(PREFIX)/lib/ibus-keymagic
COMPONENTDIR = $(DESTDIR)$(PREFIX)/share/ibus/component
ICONDIR = $(DESTDIR)$(PREFIX)/share/pixmaps

# Source files
SOURCES = $(SRCDIR)/main.c \
          $(SRCDIR)/engine.c \
          $(SRCDIR)/config.c \
          $(SRCDIR)/ffi_bridge.c \
          $(SRCDIR)/toml.c \
          $(SRCDIR)/keycode_map.c

HEADERS = $(SRCDIR)/engine.h \
          $(SRCDIR)/config.h \
          $(SRCDIR)/ffi_bridge.h \
          $(SRCDIR)/toml.h \
          $(SRCDIR)/keycode_map.h

OBJECTS = $(SOURCES:$(SRCDIR)/%.c=$(BUILDDIR)/%.o)

# Compiler and flags
CC ?= gcc
CFLAGS = -std=c99 -Wall -Wextra -O2 -fPIC
CPPFLAGS = -DVERSION=\"$(VERSION)\"

# Package config for dependencies
PKG_CONFIG ?= pkg-config
IBUS_CFLAGS = $(shell $(PKG_CONFIG) --cflags ibus-1.0)
IBUS_LIBS = $(shell $(PKG_CONFIG) --libs ibus-1.0)
GLIB_CFLAGS = $(shell $(PKG_CONFIG) --cflags glib-2.0 gio-2.0)
GLIB_LIBS = $(shell $(PKG_CONFIG) --libs glib-2.0 gio-2.0)

# TOML parsing library (we have our own parser, no external lib needed)
TOML_CFLAGS = 
TOML_LIBS = 

# KeyMagic core library (Rust FFI)
KEYMAGIC_CFLAGS = -I../keymagic-core/include
KEYMAGIC_LIBS = ../target/release/libkeymagic_core.a

# Combined flags
ALL_CFLAGS = $(CFLAGS) $(CPPFLAGS) $(IBUS_CFLAGS) $(GLIB_CFLAGS) $(TOML_CFLAGS) $(KEYMAGIC_CFLAGS)
ALL_LIBS = $(IBUS_LIBS) $(GLIB_LIBS) $(TOML_LIBS) $(KEYMAGIC_LIBS)

# Default target
all: $(PROJECT_NAME)

# Create build directory
$(BUILDDIR):
	mkdir -p $(BUILDDIR)

# Compile object files
$(BUILDDIR)/%.o: $(SRCDIR)/%.c $(HEADERS) | $(BUILDDIR)
	$(CC) $(ALL_CFLAGS) -c $< -o $@

# Link executable
$(PROJECT_NAME): $(OBJECTS)
	$(CC) $(OBJECTS) $(ALL_LIBS) -o $@

# Clean build artifacts
clean:
	rm -rf $(BUILDDIR)
	rm -f $(PROJECT_NAME)

# Install to system
install: $(PROJECT_NAME)
	# Create installation directories
	install -d $(ENGINEDIR)
	install -d $(COMPONENTDIR)
	install -d $(ICONDIR)
	
	# Install engine executable
	install -m 755 $(PROJECT_NAME) $(ENGINEDIR)/$(PROJECT_NAME)
	
	# Install IBus component definition
	install -m 644 $(DATADIR)/keymagic.xml $(COMPONENTDIR)/keymagic.xml
	
	# Install icon (if available)
	# install -m 644 $(DATADIR)/keymagic.png $(ICONDIR)/keymagic.png

# Uninstall from system
uninstall:
	rm -f $(ENGINEDIR)/$(PROJECT_NAME)
	rm -f $(COMPONENTDIR)/keymagic.xml
	rm -f $(ICONDIR)/keymagic.png
	rmdir --ignore-fail-on-non-empty $(ENGINEDIR)

# Development shortcuts
dev-install: install
	# Register component with IBus (for development)
	ibus restart || true
	sleep 2
	ibus register-component $(COMPONENTDIR)/keymagic.xml || true

dev-uninstall: uninstall
	# Restart IBus to remove component
	ibus restart || true

# Check dependencies
check-deps:
	@echo "Checking dependencies..."
	@$(PKG_CONFIG) --exists ibus-1.0 || (echo "ERROR: IBus development headers not found" && exit 1)
	@$(PKG_CONFIG) --exists glib-2.0 || (echo "ERROR: GLib development headers not found" && exit 1)
	@$(PKG_CONFIG) --exists gio-2.0 || (echo "ERROR: GIO development headers not found" && exit 1)
	@echo "Dependencies OK"

# Show build information
info:
	@echo "KeyMagic IBus Engine Build Information"
	@echo "======================================"
	@echo "Project: $(PROJECT_NAME)"
	@echo "Version: $(VERSION)"
	@echo "CC: $(CC)"
	@echo "CFLAGS: $(ALL_CFLAGS)"
	@echo "LIBS: $(ALL_LIBS)"
	@echo "Install prefix: $(PREFIX)"

# Debug build
debug: CFLAGS += -g -DDEBUG -O0
debug: $(PROJECT_NAME)

# Help target
help:
	@echo "KeyMagic IBus Engine Makefile"
	@echo ""
	@echo "Targets:"
	@echo "  all          - Build the engine (default)"
	@echo "  clean        - Remove build artifacts"
	@echo "  install      - Install to system"
	@echo "  uninstall    - Remove from system"
	@echo "  dev-install  - Install and register with IBus"
	@echo "  dev-uninstall- Uninstall and restart IBus"
	@echo "  check-deps   - Check build dependencies"
	@echo "  debug        - Build with debug symbols"
	@echo "  info         - Show build information"
	@echo "  help         - Show this help"
	@echo ""
	@echo "Variables:"
	@echo "  PREFIX       - Installation prefix (default: /usr)"
	@echo "  DESTDIR      - Destination directory for packaging"
	@echo "  CC           - C compiler (default: gcc)"

.PHONY: all clean install uninstall dev-install dev-uninstall check-deps info debug help