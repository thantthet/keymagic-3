@echo off
setlocal enabledelayedexpansion

echo KeyMagic Cross-Compilation Build Script
echo ======================================
echo.

REM Check if target architecture is specified
if "%1"=="" (
    echo Usage: build-cross.bat [x64^|arm64] [debug^|release]
    echo Example: build-cross.bat x64 release
    exit /b 1
)

set TARGET_ARCH=%1
set BUILD_TYPE=%2
if "%BUILD_TYPE%"=="" set BUILD_TYPE=release

REM Set Rust target based on architecture
if /I "%TARGET_ARCH%"=="x64" (
    set RUST_TARGET=x86_64-pc-windows-msvc
    set CMAKE_ARCH=x64
    set CMAKE_GENERATOR_PLATFORM=x64
) else if /I "%TARGET_ARCH%"=="arm64" (
    set RUST_TARGET=aarch64-pc-windows-msvc
    set CMAKE_ARCH=ARM64
    set CMAKE_GENERATOR_PLATFORM=ARM64
) else (
    echo Invalid architecture: %TARGET_ARCH%
    echo Must be either x64 or arm64
    exit /b 1
)

echo Building for: %TARGET_ARCH% (%BUILD_TYPE%)
echo Rust target: %RUST_TARGET%
echo.

REM Navigate to project root
cd /d %~dp0\..

REM Build Rust libraries for target architecture
echo Building Rust libraries for %RUST_TARGET%...
if /I "%BUILD_TYPE%"=="debug" (
    cargo build --target %RUST_TARGET%
) else (
    cargo build --release --target %RUST_TARGET%
)

if !errorlevel! neq 0 (
    echo Rust build failed!
    exit /b !errorlevel!
)

REM Build TSF DLL with CMake
echo.
echo Building TSF DLL for %TARGET_ARCH%...
cd keymagic-windows\tsf

REM Create build directory for target architecture
set BUILD_DIR=build-%TARGET_ARCH%
if not exist %BUILD_DIR% mkdir %BUILD_DIR%
cd %BUILD_DIR%

REM Configure CMake with target architecture
cmake -G "Visual Studio 17 2022" -A %CMAKE_GENERATOR_PLATFORM% ^
    -DCMAKE_BUILD_TYPE=%BUILD_TYPE% ^
    -DTARGET_ARCH=%TARGET_ARCH% ^
    -DRUST_TARGET=%RUST_TARGET% ^
    ..

if !errorlevel! neq 0 (
    echo CMake configuration failed!
    exit /b !errorlevel!
)

REM Build the project
if /I "%BUILD_TYPE%"=="debug" (
    cmake --build . --config Debug
) else (
    cmake --build . --config Release
)

if !errorlevel! neq 0 (
    echo CMake build failed!
    exit /b !errorlevel!
)

echo.
echo Build completed successfully!
echo.
echo Output files:
echo - Rust library: ..\..\target\%RUST_TARGET%\%BUILD_TYPE%\keymagic_core.lib
if /I "%BUILD_TYPE%"=="debug" (
    echo - TSF DLL: %BUILD_DIR%\Debug\KeyMagicTSF.dll
) else (
    echo - TSF DLL: %BUILD_DIR%\Release\KeyMagicTSF.dll
)

endlocal