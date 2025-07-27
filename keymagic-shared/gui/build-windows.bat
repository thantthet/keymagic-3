@echo off
echo Building KeyMagic Shared GUI for Windows...
echo.

cd src-tauri

if "%1"=="" (
    set TARGET=aarch64-pc-windows-msvc
) else (
    set TARGET=%1
)

echo Building release version for %TARGET%...
cargo build --release --target %TARGET%

if %ERRORLEVEL% NEQ 0 (
    echo Build failed!
    exit /b 1
)

echo.
echo Build successful!
echo Executable location: ..\..\target\%TARGET%\release\keymagic-gui.exe
echo.