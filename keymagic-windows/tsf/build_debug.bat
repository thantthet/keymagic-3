@echo off
REM Build script for KeyMagic Windows TSF DLL (Debug version)

echo Building KeyMagic Windows TSF DLL (Debug)...
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

REM Build debug version
echo Building debug version...
set RUSTFLAGS=-C target-feature=+crt-static
cargo build --features debug

if %errorlevel% neq 0 (
    echo.
    echo ERROR: Build failed!
    echo Please check the error messages above.
    exit /b 1
)

echo.
echo Build successful!
echo.
echo The DLL is located at: target\debug\keymagic_windows.dll
echo.
echo Debug output can be viewed with:
echo 1. DebugView from Windows Sysinternals
echo 2. Check log files in %%TEMP%%\KeyMagicTSF_*.log
echo.

pause