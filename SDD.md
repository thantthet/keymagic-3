# Software Design Document: KeyMagic Rust Rewrite (Desktop Focus)

## 1. Introduction
*   **1.1 Purpose:** This document outlines the design for rewriting the KeyMagic input method editor (IME) using the Rust programming language. The primary goal is to modernize the codebase, improve performance, enhance memory safety, and ensure cross-platform compatibility for desktop operating systems (Linux, macOS, and Windows). This rewrite aims to provide a robust, efficient, and maintainable foundation for future development.
*   **1.2 Target Audience:** This document is intended for developers, architects, and stakeholders involved in the KeyMagic project. It provides a high-level overview and detailed technical specifications for the new system.
*   **1.3 References:**
    *   Existing KeyMagic documentation and codebase.
    *   Rust Programming Language Documentation.
    *   Relevant platform-specific IME documentation (IBus, IMK, TSF).

## 2. System Overview
### 2.1 High-Level Architecture
The new KeyMagic system will follow a layered architecture, separating the core logic from platform-specific integrations.

```
+---------------------+
|     User Interface  | (Platform-specific IME UI)
+----------+----------+
           |
+----------v----------+
| Platform Integration| (FFI Bridge, OS-specific IME APIs)
| (Linux, macOS, Win) |
+----------+----------+
           |
+----------v----------+
|    KeyMagic Core    | (Rust Crate: keymagic-core)
| (Layout Parsing,    |
|  Rule Matching,     |
|  State Management)  |
+---------------------+
```

### 2.2 Key Components
*   **KeyMagic Core Engine (`keymagic-core`):** A Rust library responsible for all core IME functionalities, independent of any specific operating system.
*   **Desktop Platform Integrations:** Separate modules or crates that provide the necessary Foreign Function Interface (FFI) bindings and interact with the native IME frameworks of Linux (IBus), macOS (IMK), and Windows (TSF).
*   **Build System:** Leveraging Cargo for Rust projects, with configurations for cross-compilation and platform-specific builds.

### 2.3 Technology Stack
*   **Core Logic:** Rust
*   **Platform Integration (FFI):** Rust's FFI capabilities interacting with C/C++ APIs.
*   **Linux IME:** IBus framework (C/C++).
*   **macOS IME:** Input Method Kit (IMK) framework (Objective-C/Swift).
*   **Windows IME:** Text Services Framework (TSF) (C++).

## 3. Architectural Design
### 3.1 Core Engine (`keymagic-core` crate)
*   **3.1.1 Responsibilities:**
    *   Parsing and loading keyboard layout definition files (`.km2`).
    *   Maintaining the current IME state (e.g., active layout, composing buffer, cursor position).
    *   Processing raw key events (keydown, keyup).
    *   Applying layout rules to transform input sequences into output characters.
    *   Managing rule matching and context-sensitive transformations.
    *   Providing an API for platform-specific layers to interact with the core logic.
*   **3.1.2 Data Structures:**
    *   `KeyboardLayout`: Represents a loaded `.km2` file, including rules, states, and key mappings.
    *   `Rule`: Defines a single transformation rule (e.g., input pattern, output, next state).
    *   `EngineState`: Encapsulates the current state of the IME, including the composing string, active rule set, and any internal flags.
    *   `KeyInput`: A struct representing a raw key event (keycode, modifiers).
*   **3.1.3 Key Modules/Traits:**
    *   `layout_parser`: Module responsible for parsing `.km2` files into `KeyboardLayout` structures.
    *   `engine`: Main module containing the `KeyMagicEngine` struct, which orchestrates input processing and state management.
    *   `rule_processor`: Module handling the logic for matching input sequences against rules and generating output.
    *   `ffi_api`: Module defining the public functions exposed via FFI.
*   **3.1.4 Error Handling Strategy:**
    *   Utilize Rust's `Result<T, E>` enum for fallible operations.
    *   Define custom error types for parsing, engine state, and FFI interactions.
    *   Propagate errors up to the FFI boundary, where they can be translated into platform-specific error codes or logging.
*   **3.1.5 Performance Considerations:**
    *   Minimize memory allocations during critical input processing paths.
    *   Optimize rule matching algorithms (e.g., using Aho-Corasick or similar for pattern matching).
    *   Leverage Rust's zero-cost abstractions and efficient data structures.

### 3.2 Desktop Platform Integration Layers
Each platform will have a thin wrapper around the `keymagic-core` library, handling the communication between the native IME framework and the Rust core.
*   **3.2.1 Foreign Function Interface (FFI) Design:**
    *   **Exposure:** Rust functions will be exposed as C-compatible functions using `#[no_mangle]` and `extern "C"`.
    *   **Data Marshalling:** Primitive types will pass directly. Strings will be passed as C-style null-terminated strings (`*const c_char`, `*mut c_char`) and converted to/from Rust `String` or `&str`. Complex data structures will be passed as opaque pointers or serialized/deserialized.
    *   **Error Propagation:** FFI functions will return integer error codes or use out-parameters for error details.
*   **3.2.2 Linux Integration (IBus):**
    *   A C/C++ wrapper will implement the IBus engine interface.
    *   This wrapper will call into the Rust `keymagic-core` library via FFI for input processing.
    *   Output from the Rust core will be sent back to IBus for display.
*   **3.2.3 macOS Integration (IMK):**
    *   An Objective-C/Swift wrapper will implement the `IMKInputController` and `IMKServer` protocols.
    *   Key events received by the IMK controller will be passed to the Rust core via FFI.
    *   Composing text and committed text from the Rust core will be sent back to the IMK framework.
*   **3.2.4 Windows Integration:**
    *   A C++ wrapper will interact with the Text Services Framework (TSF) or potentially a simpler keyboard hook mechanism.
    *   Input events from TSF will be forwarded to the Rust core via FFI.
    *   Output and composition string updates from the Rust core will be used to update the TSF context.

### 3.3 Build System
*   **3.3.1 Cargo Workspaces:** The project will be structured as a Cargo workspace, with `keymagic-core` as a library crate and separate binary/library crates for each platform integration (e.g., `keymagic-ibus`, `keymagic-macos`, `keymagic-windows`).
*   **3.3.2 Cross-compilation:** Cargo's built-in cross-compilation features will be utilized. Platform-specific build scripts (e.g., `build.rs`) will handle linking with native libraries.
*   **3.3.3 CI/CD Considerations:** Automated builds and tests will be set up for each target platform to ensure continuous integration and delivery.

## 4. Data Design
*   **4.1 Keyboard Layout Format:**
    *   The existing `.km2` file format will be supported initially.
    *   The `layout_parser` module in `keymagic-core` will be responsible for deserializing `.km2` content into Rust data structures (`KeyboardLayout`).
    *   Consideration for a more modern, Rust-native layout definition format (e.g., TOML, YAML) for future enhancements, while maintaining backward compatibility.
*   **4.2 Internal State Management:**
    *   The `EngineState` struct will hold all mutable state required for the IME's operation.
    *   This includes the current composing string, the active rule set, the current state in a state machine (if applicable for complex rules), and any temporary buffers.
    *   State updates will be managed immutably where possible, or through clear mutable references within the `KeyMagicEngine`'s methods.

## 5. Non-Functional Requirements
*   **5.1 Performance:**
    *   **Latency:** Input processing latency must be minimal (target < 10ms) to ensure a smooth typing experience.
    *   **Throughput:** Capable of handling rapid key presses without dropping events.
*   **5.2 Security:**
    *   **Input Handling:** Securely handle raw input events, preventing injection or manipulation.
    *   **FFI Safety:** Minimize `unsafe` Rust code, especially at FFI boundaries, and rigorously audit any `unsafe` blocks.
    *   **Memory Safety:** Leverage Rust's ownership system to prevent memory-related vulnerabilities.
*   **5.3 Maintainability:**
    *   **Code Quality:** Adhere to Rust best practices, use `rustfmt` and `clippy`.
    *   **Modularity:** Clear separation of concerns between core logic and platform integrations.
    *   **Documentation:** Comprehensive inline code documentation and external design documents.
*   **5.4 Portability:**
    *   The `keymagic-core` should be entirely platform-agnostic.
    *   New desktop platforms should be supportable by implementing a new, thin integration layer.
*   **5.5 Testability:**
    *   **Unit Tests:** Extensive unit tests for `keymagic-core` modules (parsing, rule processing).
    *   **Integration Tests:** Tests for the FFI boundaries and interactions between the core and platform layers.
    *   **End-to-End Tests:** Automated tests simulating user input and verifying IME output on each platform.

## 6. Development Plan (High-Level)
*   **6.1 Phases/Milestones:**
    *   **Phase 1: KMS to KM2 Converter:** Develop a utility to convert existing KMS (KeyMagic Script) files to the KM2 format.
    *   **Phase 2: Core Engine Development:** Implement `keymagic-core` with `.km2` parsing, basic rule matching, and state management.
    *   **Phase 3: Linux Integration:** Develop the IBus integration layer and demonstrate basic functionality.
    *   **Phase 4: macOS Integration:** Develop the IMK integration layer and demonstrate basic functionality.
    *   **Phase 5: Windows Integration:** Develop the TSF integration layer and demonstrate basic functionality.
    *   **Phase 6: Advanced Features & Optimization:** Implement more complex rule types, performance optimizations, and comprehensive error handling.
*   **6.2 Key Tasks:**
    *   Define `.km2` parsing logic and data structures in Rust.
    *   Implement the core rule matching engine.
    *   Set up Cargo workspace and build configurations for all target platforms.
    *   Develop FFI bindings for each platform.
    *   Implement platform-specific IME interfaces.
    *   Write comprehensive test suites.
    *   Integrate CI/CD pipelines.
