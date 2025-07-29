@echo off
:: make.bat - KeyMagic Tray Manager build system
:: Usage: make.bat [command] [arch] [config]
:: Example: make.bat build x64 Debug

setlocal enabledelayedexpansion

:: Navigate to script directory
cd /d "%~dp0"

:: Parse arguments with defaults
set "COMMAND=%~1"
if "%COMMAND%"=="" set "COMMAND=help"

set "ARCH=%~2"
if "%ARCH%"=="" set "ARCH=arm64"

set "CONFIG=%~3"
if "%CONFIG%"=="" set "CONFIG=Release"

:: Validate arguments
if /i not "%ARCH%"=="x64" if /i not "%ARCH%"=="arm64" (
    echo [ERROR] Invalid architecture: %ARCH%
    echo Must be x64 or arm64
    exit /b 1
)

if /i not "%CONFIG%"=="Debug" if /i not "%CONFIG%"=="Release" (
    echo [ERROR] Invalid config: %CONFIG%
    echo Must be Debug or Release
    exit /b 1
)

:: Set common variables
if /i "%ARCH%"=="x64" (
    set "RUST_TARGET=x86_64-pc-windows-msvc"
    set "CMAKE_ARCH=x64"
) else (
    set "RUST_TARGET=aarch64-pc-windows-msvc"
    set "CMAKE_ARCH=ARM64"
)

:: Route to command
if /i "%COMMAND%"=="build" goto :build
if /i "%COMMAND%"=="clean" goto :clean
if /i "%COMMAND%"=="status" goto :status
if /i "%COMMAND%"=="help" goto :help

echo [ERROR] Unknown command: %COMMAND%
goto :help

:: ========== BUILD ==========
:build
echo Building KeyMagic Tray Manager (%ARCH% %CONFIG%)...
echo.

:: Check if we need to build keymagic-core first
if /i "%CONFIG%"=="Release" (
    set "CORE_LIB=..\..\target\%RUST_TARGET%\release\keymagic_core.lib"
) else (
    set "CORE_LIB=..\..\target\%RUST_TARGET%\debug\keymagic_core.lib"
)

if not exist "%CORE_LIB%" (
    echo Building keymagic-core library...
    pushd ..\..
    if /i "%CONFIG%"=="Release" (
        cargo build -p keymagic-core --release --target %RUST_TARGET% || exit /b 1
    ) else (
        cargo build -p keymagic-core --target %RUST_TARGET% || exit /b 1
    )
    popd
)

:: Create build directory
if not exist "build-%ARCH%" mkdir "build-%ARCH%"
cd "build-%ARCH%"

:: Configure with CMake
echo Configuring with CMake...
cmake -G "Visual Studio 17 2022" -A %CMAKE_ARCH% .. || exit /b 1

:: Build
echo Building...
cmake --build . --config %CONFIG% || exit /b 1

cd ..
echo.
echo [SUCCESS] Build complete!
echo Output: build-%ARCH%\bin\%CONFIG%\keymagic-tray.exe
exit /b 0

:: ========== CLEAN ==========
:clean
echo Cleaning build artifacts (%ARCH% %CONFIG%)...

:: Clean build directory
if exist "build-%ARCH%" (
    rmdir /s /q "build-%ARCH%"
)

echo [SUCCESS] Clean complete!
exit /b 0

:: ========== STATUS ==========
:status
echo KeyMagic Tray Manager Status (%ARCH% %CONFIG%)
echo ========================================
echo.

:: Check Tray executable
set "TRAY_EXE=build-%ARCH%\bin\%CONFIG%\keymagic-tray.exe"
if exist "%TRAY_EXE%" (
    echo [OK] Tray Manager executable found
) else (
    echo [--] Tray Manager executable not found
)

:: Check keymagic-core library
if /i "%CONFIG%"=="Release" (
    set "CORE_LIB=..\..\target\%RUST_TARGET%\release\keymagic_core.lib"
) else (
    set "CORE_LIB=..\..\target\%RUST_TARGET%\debug\keymagic_core.lib"
)

if exist "%CORE_LIB%" (
    echo [OK] keymagic-core library found
) else (
    echo [--] keymagic-core library not found
)

:: Check if tray manager is running
tasklist /FI "IMAGENAME eq keymagic-tray.exe" 2>NUL | find /I /N "keymagic-tray.exe">NUL
if %errorlevel% equ 0 (
    echo [OK] Tray Manager is running
) else (
    echo [--] Tray Manager is not running
)

echo.
exit /b 0

:: ========== HELP ==========
:help
echo KeyMagic Tray Manager Build System
echo ==================================
echo.
echo Usage: make.bat [command] [arch] [config]
echo.
echo Commands:
echo   build     Build tray manager
echo   clean     Clean build artifacts
echo   status    Check build status
echo   help      Show this help
echo.
echo Arguments:
echo   arch      x64 or arm64 (default: arm64)
echo   config    Debug or Release (default: Release)
echo.
echo Examples:
echo   make.bat build
echo   make.bat build x64
echo   make.bat build x64 Debug
echo   make.bat clean x64
echo   make.bat status arm64 Debug
echo.
exit /b 0

endlocal