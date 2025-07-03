# KeyMagic Windows Project Structure

## Current Directory Organization

```
keymagic-windows/
├── Cargo.toml              # Rust project configuration
├── build.rs                # Rust build script
├── src/                    # TSF source files (Rust + C++)
│   ├── ffi.rs             # Rust FFI implementation
│   ├── lib.rs             # Rust library entry point
│   ├── KeyMagicTextService.cpp/h  # TSF implementation
│   ├── DllMain.cpp        # DLL entry point
│   ├── Globals.cpp        # Global variables
│   ├── Debug.cpp/h        # Debug utilities
│   ├── KeyMagicGuids.h    # COM GUIDs
│   └── keymagic.def       # DLL exports
├── include/                # Public headers
│   └── keymagic_ffi.h     # FFI interface
├── manager/                # GUI Manager application
│   ├── KeyMagicManager.cpp
│   ├── KeyMagicManager.exe
│   └── build scripts
├── examples/               # Example code
├── tests/                  # Unit tests
├── build_windows.bat       # TSF build script
├── install.bat            # TSF install script
└── Documentation files

## Proposed Reorganization

```
keymagic-windows/
├── README.md               # Main project overview
├── tsf/                    # Text Services Framework component
│   ├── Cargo.toml
│   ├── build.rs
│   ├── src/               # TSF source files
│   │   ├── rust/          # Rust code
│   │   │   ├── ffi.rs
│   │   │   └── lib.rs
│   │   ├── cpp/           # C++ code
│   │   │   ├── KeyMagicTextService.cpp/h
│   │   │   ├── DllMain.cpp
│   │   │   ├── Globals.cpp
│   │   │   ├── Debug.cpp/h
│   │   │   └── KeyMagicGuids.h
│   │   └── keymagic.def
│   ├── include/
│   │   └── keymagic_ffi.h
│   ├── build.bat
│   ├── install.bat
│   └── README.md
├── manager/                # GUI Manager application
│   ├── src/
│   │   ├── KeyMagicManager.cpp
│   │   └── resource.h
│   ├── build.bat
│   ├── KeyMagicManager.exe
│   └── README.md
├── docs/                   # Documentation
│   ├── BUILD_INSTALL.md
│   ├── TSF_DEBUGGING.md
│   ├── UNINSTALL_HELP.md
│   └── USAGE_GUIDE.md
├── examples/
└── tests/
```

## Benefits of Reorganization

1. **Clear separation** between TSF and Manager components
2. **Better organization** of source files by language
3. **Centralized documentation** in docs folder
4. **Each component** has its own README and build scripts
5. **Easier to navigate** and understand project structure