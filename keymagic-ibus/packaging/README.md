# KeyMagic Linux Packaging Files

This directory contains packaging templates and scripts for creating Linux distribution packages.

## Structure

```
packaging/
├── debian/                  # Debian/Ubuntu packaging files
│   ├── control.in          # Control file template
│   ├── postinst            # Post-installation script
│   ├── postrm              # Post-removal script
│   ├── prerm               # Pre-removal script
│   └── keymagic3-ibus-refresh  # Helper script for IBus refresh
└── keymagic3.spec.in       # RPM spec file template
```

## Template Variables

The `.in` template files use the following variables that are replaced during build:

- `@VERSION@` - Package version (e.g., "0.0.1")
- `@ARCH@` - Architecture (e.g., "amd64", "x86_64")
- `@DATE@` - Build date for changelog entries
- `@PROJECT_ROOT@` - Absolute path to project root directory

## Usage

These templates are processed by the build scripts in `../build-scripts/`:

1. The build script reads the `.in` template files
2. Substitutes the variables with actual values
3. Generates the final packaging files

## Debian Package Scripts

- **postinst**: Runs after package installation
  - Installs the IBus refresh helper
  - Updates icon cache and desktop database
  - Provides user instructions

- **prerm**: Runs before package removal
  - Gracefully shuts down any running instances

- **postrm**: Runs after package removal
  - Cleans up configuration files
  - Removes the IBus refresh helper

## RPM Spec File

The `keymagic3.spec.in` template defines:
- Package metadata
- File installation locations
- Post-installation and post-uninstallation scripts
- Dependencies and requirements

## Adding New Package Formats

To add support for a new package format:

1. Create a new subdirectory (e.g., `packaging/arch/`)
2. Add necessary template files
3. Update the build script to process the new templates
4. Document the new format in this README