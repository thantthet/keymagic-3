@echo off
REM Build script for KeyMagic Windows TSF DLL

echo Building KeyMagic Windows TSF DLL...
echo.

REM Check if cargo is available
where cargo >nul 2>nul
if %errorlevel% neq 0 (
    echo ERROR: Cargo not found. Please install Rust from https://rustup.rs/
    exit /b 1
)

REM Check if we're in the right directory
if not exist "Cargo.toml" (
    echo ERROR: Please run this script from the keymagic-windows directory
    exit /b 1
)

REM Clean previous builds
echo Cleaning previous builds...
cargo clean

REM Build release version
echo Building release version...
cargo build --release

if %errorlevel% neq 0 (
    echo.
    echo ERROR: Build failed!
    echo Please check the error messages above.
    exit /b 1
)

echo.
echo Build successful!
echo.
echo The DLL is located at: target\release\keymagic_windows.dll
echo.
echo To install:
echo 1. Run Command Prompt as Administrator
echo 2. Navigate to this directory
echo 3. Run: install.bat
echo.

pause