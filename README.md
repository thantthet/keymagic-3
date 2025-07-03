# KeyMagic Rust Rewrite

A modern rewrite of the KeyMagic input method editor in Rust, focusing on performance, memory safety, and cross-platform compatibility.

## Project Structure

This is a monorepo organized as a Cargo workspace:

```
keymagic-v3/
├── Cargo.toml                 # Workspace configuration
├── keymagic-core/            # Core engine library
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       └── types/            # KM2 format types and definitions
├── kms2km2/                  # KMS to KM2 converter
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── lexer/           # KMS lexical analyzer
│   │   ├── parser/          # KMS parser
│   │   ├── binary/          # KM2 compilation and writing
│   │   └── bin/
│   │       ├── kms2km2.rs   # CLI converter
│   │       └── km2_dump.rs  # KM2 file dumper
│   └── tests/
├── keymagic-ibus/           # Linux IBus integration (placeholder)
├── keymagic-macos/          # macOS IMK integration (placeholder)
└── keymagic-windows/        # Windows TSF integration (placeholder)
```

## Architecture

Following the Software Design Document (SDD.md), the project is structured as:

1. **keymagic-core**: Platform-agnostic core engine
   - KM2 format definitions
   - Layout parsing and rule matching
   - State management
   - FFI API for platform integrations

2. **kms2km2**: Converter utility (Phase 1 complete)
   - Lexer for KMS script format
   - Parser generating AST
   - Compiler to KM2 binary format
   - Binary writer with proper endianness

3. **Platform Integrations** (To be implemented):
   - keymagic-ibus: Linux desktop support via IBus
   - keymagic-macos: macOS support via Input Method Kit
   - keymagic-windows: Windows support via Text Services Framework

## Building

```bash
# Build all crates
cargo build --workspace

# Build specific crate
cargo build -p kms2km2

# Run tests
cargo test --workspace
```

## Usage

### Convert KMS to KM2

```bash
cargo run -p kms2km2 -- input.kms output.km2
```

### Dump KM2 file contents

```bash
cargo run -p kms2km2 --bin km2_dump -- file.km2
```

## Development Status

- ✅ Phase 1: KMS to KM2 Converter (Complete)
- ✅ Phase 2: Core Engine Development (Complete)
- ⏳ Phase 3: Linux Integration (Planned)
- ⏳ Phase 4: macOS Integration (Planned)
- 🚧 Phase 5: Windows Integration (In Progress)
- ⏳ Phase 6: Advanced Features & Optimization (Planned)

## Features

- Full KMS parsing support including:
  - Variable declarations
  - Unicode literals (U1000, u1000)
  - String literals
  - Virtual key combinations
  - Pattern matching with wildcards ([*], [^])
  - Back-references ($1, $2, etc.)
  - State management
  - Include directives
- Binary KM2 format generation (version 1.5)
- Command-line interface
- Cross-platform support

## License

GPL-2.0