# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

KeyMagic 3 is a Rust rewrite of the KeyMagic input method editor (IME), targeting Windows, macOS, and Linux. The core engine is platform-agnostic Rust, with platform-specific integrations via FFI.

## Documentation

- KeyMagic Script format: @KMS_FORMAT.md
- KeyMagic Binary format: @KM2_FORMAT.md
- Engine processing logic: @keymagic-core/ENGINE_LOGIC.md

## Build & Test Commands

```bash
# Build all Rust workspace crates
cargo build --workspace

# Run all tests
cargo test --workspace

# Run tests for a specific crate
cargo test -p keymagic-core
cargo test -p kms2km2

# Run a single test by name
cargo test -p keymagic-core -- test_name

# Lint and format
cargo fmt --all -- --check
cargo clippy --all -- -D warnings

# KMS compiler CLI
cargo run -p kms2km2 -- input.kms output.km2

# KM2 file inspector
cargo run -p kms2km2 --bin km2_dump -- file.km2
```

### Platform-specific builds

```bash
# macOS (universal binary)
cd keymagic-macos && make BUILD_CONFIG=release all

# Linux (IBus engine)
cd keymagic-ibus && make BUILD_TYPE=release

# Windows (from Windows VM)
cd keymagic-windows && make.bat build arm64 Release
cd keymagic-windows && make.bat build x64 Release
cd keymagic-windows && make.bat register    # Register TSF (requires admin)
cd keymagic-windows && make.bat unregister  # Unregister TSF (requires admin)
cd keymagic-windows && make.bat status      # Check build status
cd keymagic-windows && make.bat clean       # Clean build artifacts

# Tauri GUI (cross-platform)
cargo tauri build
```

## Architecture

### Cargo workspace members

| Crate | Path | Purpose |
|-------|------|---------|
| `keymagic-core` | `keymagic-core/` | Core engine (cdylib + staticlib + rlib) |
| `kms2km2` | `kms2km2/` | KMS script compiler (two binaries) |
| `keymagic-gui` | `keymagic-shared/gui/src-tauri/` | Tauri 2.x cross-platform GUI |

### keymagic-core

The platform-agnostic engine. Exposes a C FFI (`ffi.rs`) consumed by all platform integrations.

- `engine/` — Key processing orchestration, recursive rule matching, action generation
  - `engine.rs` — `KeyMagicEngine` main struct
  - `matching/` — Rule matcher, pattern evaluation, capture groups
  - `processing/` — Recursive rule application, action generation
  - `state/` — Composing text buffer, active state tracking
  - `input.rs` / `output.rs` — Input/output types (`ActionType`: Insert, BackspaceDelete, BackspaceDeleteAndInsert)
- `km2/loader.rs` — Binary KM2 file parser
- `types/` — Data structures: `Km2File`, `Rule`, `VirtualKey`, opcodes
- `ffi.rs` — C-compatible FFI API for all platforms
- `hotkey.rs` — Hotkey parsing

### kms2km2

KMS script to KM2 binary compiler:

- `lexer/` — Tokenization via `logos` crate
- `parser/` — AST generation
- `binary/` — KM2 compilation and writing
- `bin/kms2km2.rs` — CLI converter; `bin/km2_dump.rs` — KM2 inspector

### Platform integrations

| Platform | Path | Language | Integration |
|----------|------|----------|-------------|
| Windows | `keymagic-windows/tsf/` | C++ | Text Services Framework (TSF) DLL |
| Windows | `keymagic-windows/tray-manager/` | C++ | System tray app, named pipe IPC |
| macOS | `keymagic-macos/` | Swift | Input Method Kit (IMK) |
| Linux | `keymagic-ibus/` | C | IBus engine |
| GUI | `keymagic-shared/gui/` | Rust + HTML/JS/CSS | Tauri 2.x app (all platforms) |

All platform integrations link against `keymagic-core` via its C FFI. The GUI is a Tauri app with vanilla HTML/JS/CSS frontend (no framework).

### Key data flow

1. Platform IME receives key event → converts to `KeyInput` via FFI
2. `KeyMagicEngine::process_key()` matches rules against composing buffer
3. Engine returns `EngineOutput` with action (insert/delete/backspace+insert)
4. Platform IME applies action to the text editor

### KM2 format conventions

- All multi-byte values are little-endian
- Strings are UTF-16LE
- Variable indices are **1-based** (index 1 = first string in table)
- State names from KMS are compiled to integer indices
- opPREDEFINED values are internal enum values, NOT platform VK codes

## Testing

Integration tests live in `keymagic-core/tests/` and `kms2km2/tests/`. Test fixtures (`.kms` and `.km2` files) are in `tests/fixtures/` within each crate. Common test helpers are in `keymagic-core/tests/common/`. Bundled keyboard layouts (`.km2`) are in `keyboards/bundled/`.

There is also a Python FFI test suite (`keymagic-core/tests/test_ffi.py`) runnable via `run_ffi_tests.sh`.

## CI

GitHub Actions workflows in `.github/workflows/`:
- `ci.yml` — Tests, formatting, clippy, audit (all 3 OS)
- `release.yml` — Orchestrates platform builds on tag push `v*`
- `build-windows.yml`, `build-macos.yml`, `build-linux.yml` — Reusable platform build workflows

## Versioning

Current version is in `version.txt`. Use `update-version.sh` (or `update-version.ps1`) to bump version across all Cargo.toml files, Inno Setup scripts, and other locations.
