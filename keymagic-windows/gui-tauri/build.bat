@echo off
echo Building KeyMagic Tauri GUI...
echo.

cd src-tauri

echo Building release version...
cargo build --release

if %ERRORLEVEL% NEQ 0 (
    echo Build failed!
    exit /b 1
)

echo.
echo Build successful!
echo Executable location: ..\..\target\release\gui-tauri.exe
echo.
echo To run the app: cargo run --release
echo.