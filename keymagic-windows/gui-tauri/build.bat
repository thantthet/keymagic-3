@echo off
echo Building KeyMagic Tauri GUI...
echo.

cd src-tauri

echo Building release version for x64...
cargo build -p gui-tauri --release --target x86_64-pc-windows-msvc

if %ERRORLEVEL% NEQ 0 (
    echo Build failed!
    exit /b 1
)

echo.
echo Build successful!
echo Executable location: ..\..\target\x86_64-pc-windows-msvc\release\gui-tauri.exe
echo.
echo To run the app: cargo run --release --target x86_64-pc-windows-msvc
echo.