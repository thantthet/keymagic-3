# KeyMagic Core

The core engine for the KeyMagic input method editor, written in Rust. This crate provides the fundamental keyboard layout parsing, rule matching, and text processing capabilities that power KeyMagic across all platforms.

## Overview

`keymagic-core` is a platform-agnostic library that:
- Parses and loads KeyMagic keyboard layout files (`.km2` format)
- Processes keyboard input according to defined rules
- Manages composing text and state transitions
- Provides a C-compatible FFI for cross-language integration

## Architecture

The crate is organized into several key modules:

### Core Modules

- **`engine`** - The main processing engine that orchestrates keyboard input handling
  - `engine.rs` - Central `KeyMagicEngine` implementation
  - `input.rs` - Input event representation
  - `output.rs` - Processing results and actions
  - `state/` - State management (composing buffer, active states)
  - `matching/` - Rule matching logic
  - `processing/` - Action generation and recursive rule processing

- **`km2`** - Binary keyboard layout file handling
  - `loader.rs` - Parses `.km2` files into internal structures
  - `error.rs` - KM2-specific error types

- **`types`** - Common type definitions
  - `km2.rs` - KM2 file format structures
  - `opcodes.rs` - Binary opcode definitions
  - `virtual_keys.rs` - Virtual key code mappings
  - `errors.rs` - Error types

- **`ffi`** - Foreign Function Interface for C/C++ integration

## Usage

### As a Rust Library

```rust
use keymagic_core::{KeyMagicEngine, KeyInput, VirtualKey};

// Create a new engine
let mut engine = KeyMagicEngine::new();

// Load a keyboard layout
engine.load_keyboard("path/to/keyboard.km2")?;

// Process a key press
let input = KeyInput {
    key_code: VirtualKey::KeyA as u32,
    character: Some('a'),
    shift: false,
    ctrl: false,
    alt: false,
    caps_lock: false,
};

let output = engine.process_key(input);
```

### Via FFI (C/C++)

```c
#include "keymagic_ffi.h"

// Create engine
EngineHandle* engine = keymagic_engine_new();

// Load keyboard
keymagic_engine_load_keyboard(engine, "keyboard.km2");

// Process key
ProcessKeyOutput output;
keymagic_engine_process_key(
    engine, 
    VK_A,     // key code
    'a',      // character
    0, 0, 0, 0,  // modifiers
    &output
);

// Clean up
keymagic_free_string(output.text);
keymagic_free_string(output.composing_text);
keymagic_engine_free(engine);
```

## Key Features

### Persistent Composing Buffer
The engine maintains a composing text buffer that accumulates input across key events. This buffer is only cleared when explicitly reset or when a rule produces empty output.

### Rule-Based Processing
- Supports complex pattern matching with wildcards, back-references, and state conditions
- Rules are prioritized by: state-specific → virtual key → longer patterns → first match
- Recursive rule matching allows chained transformations

### State Management
- Transient states that activate for the next key press
- States must be explicitly maintained in rule outputs to persist

### Cross-Platform FFI
- Thread-safe C API for integration with platform-specific IME frameworks
- Used by Windows TSF, macOS IMK, and Linux IBus implementations

## Building

### Rust Library
```bash
cargo build --release
```

### Static Library for FFI
```bash
cargo build --release --lib
```

This produces:
- Dynamic library: `target/release/libkeymagic_core.so` (Linux), `.dylib` (macOS), `.dll` (Windows)
- Static library: `target/release/libkeymagic_core.a` (Unix), `.lib` (Windows)

## Testing

Run the test suite:
```bash
cargo test
```

Run FFI tests (requires Python):
```bash
./tests/run_ffi_tests.sh  # Unix
tests\run_ffi_tests.bat   # Windows
```

## Examples

See the `examples/` directory:
- `read_km2_info.rs` - Demonstrates loading and inspecting keyboard layouts

## Documentation

For detailed engine logic and behavior, see the [Engine Logic Documentation](ENGINE_LOGIC.md).

## License

Licensed under the same terms as the KeyMagic project.