@echo off
:: make-arm64x.bat - Build native ARM64X DLL using CMake
:: This script uses CMake presets to build both ARM64 and ARM64EC versions,
:: then links them into a single native ARM64X DLL with incremental build support

setlocal enabledelayedexpansion

:: Navigate to script directory
cd /d "%~dp0"

:: Parse command line arguments
set "COMMAND=%~1"
if "%COMMAND%"=="" set "COMMAND=help"

set "CONFIG=%~2"
if "%CONFIG%"=="" set "CONFIG=Release"

:: Validate configuration first (needed for all commands except help)
if /i not "%COMMAND%"=="help" (
    if /i not "%CONFIG%"=="Debug" if /i not "%CONFIG%"=="Release" (
        echo [ERROR] Invalid configuration: %CONFIG%
        echo Must be Debug or Release
        exit /b 1
    )
)

:: Use lowercase config for preset names
set "CONFIG_LOWER=%CONFIG%"
if /i "%CONFIG%"=="Debug" set "CONFIG_LOWER=debug"
if /i "%CONFIG%"=="Release" set "CONFIG_LOWER=release"

:: Route to command
if /i "%COMMAND%"=="build" goto :build
if /i "%COMMAND%"=="clean" goto :clean
if /i "%COMMAND%"=="help" goto :help

echo [ERROR] Unknown command: %COMMAND%
goto :help

:build

echo ========================================
echo Building Native ARM64X DLL with CMake
echo Configuration: %CONFIG%
echo ========================================
echo.
echo [INFO] This build uses CMake for incremental builds and caching
echo.

:: Step 1: Build Rust libraries for both architectures as static libs
echo [1/5] Building Rust static libraries...
echo.

:: Build for ARM64EC target (as static library for ARM64X)
echo Building Rust for ARM64EC (static library)...
if /i "%CONFIG%"=="Release" (
    cargo rustc -p keymagic-core --release --target arm64ec-pc-windows-msvc --crate-type staticlib || (
        echo.
        echo [INFO] ARM64EC target may not be installed. Trying to add it...
        rustup target add arm64ec-pc-windows-msvc
        cargo rustc -p keymagic-core --release --target arm64ec-pc-windows-msvc --crate-type staticlib || exit /b 1
    )
) else (
    cargo rustc -p keymagic-core --target arm64ec-pc-windows-msvc --crate-type staticlib || (
        echo.
        echo [INFO] ARM64EC target may not be installed. Trying to add it...
        rustup target add arm64ec-pc-windows-msvc
        cargo rustc -p keymagic-core --target arm64ec-pc-windows-msvc --crate-type staticlib || exit /b 1
    )
)

:: Build for ARM64 target (as static library for ARM64X)
echo Building Rust for ARM64 (static library)...
if /i "%CONFIG%"=="Release" (
    cargo rustc -p keymagic-core --release --target aarch64-pc-windows-msvc --crate-type staticlib || exit /b 1
) else (
    cargo rustc -p keymagic-core --target aarch64-pc-windows-msvc --crate-type staticlib || exit /b 1
)

:: Step 2: Configure and build ARM64 using CMake
echo.
echo [2/5] Configuring and building ARM64 components...
cd tsf

:: Configure using preset
echo Configuring ARM64 build with CMake preset...
cmake --preset arm64-%CONFIG_LOWER%-x || exit /b 1

:: Build
echo Building ARM64 components...
cmake --build build-arm64-%CONFIG_LOWER%-x --config %CONFIG% || exit /b 1

:: Step 3: Configure and build ARM64EC using CMake
echo.
echo [3/5] Configuring and building ARM64EC components...

:: Configure using preset
echo Configuring ARM64EC build with CMake preset...
cmake --preset arm64ec-%CONFIG_LOWER%-x || exit /b 1

:: Build (this will also link the ARM64X DLL)
echo Building ARM64EC components and linking ARM64X DLL...
cmake --build build-arm64ec-%CONFIG_LOWER%-x --config %CONFIG% || exit /b 1

:: Step 4: Verify the ARM64X DLL was created
echo.
echo [4/5] Verifying ARM64X DLL...

set "OUTPUT_DLL=build-arm64ec-%CONFIG_LOWER%-x\%CONFIG%\KeyMagicTSF.dll"
if not exist "%OUTPUT_DLL%" (
    echo [ERROR] ARM64X DLL not found at: %OUTPUT_DLL%
    cd ..
    exit /b 1
)

:: Get file size for display
for %%F in ("%OUTPUT_DLL%") do set "DLL_SIZE=%%~zF"
echo ARM64X DLL created: %OUTPUT_DLL% (%DLL_SIZE% bytes)

:: Step 5: Copy to target directory
echo.
echo [5/5] Copying to target directory...

:: Determine target directory
if /i "%CONFIG%"=="Release" (
    set "TARGET_DIR=..\target\release"
) else (
    set "TARGET_DIR=..\target\debug"
)

if not exist "%TARGET_DIR%" mkdir "%TARGET_DIR%"
copy /Y "%OUTPUT_DLL%" "%TARGET_DIR%\KeyMagicTSF.dll" >nul

cd ..

echo.
echo ========================================
echo [SUCCESS] Native ARM64X build complete!
echo ========================================
echo.
echo Output file: tsf\build-arm64ec-%CONFIG_LOWER%-x\%CONFIG%\KeyMagicTSF.dll
echo.
echo This is a native ARM64X DLL that contains both:
echo   - ARM64 native code (for ARM64 processes)
echo   - ARM64EC code (for x64/AMD64 processes on ARM64)
echo.
echo File also copied to: %TARGET_DIR%\KeyMagicTSF.dll
echo.
echo Benefits of CMake approach:
echo   - Incremental builds (only rebuilds changed files)
echo   - Build caching for faster rebuilds
echo   - Better dependency tracking
echo   - Integration with Visual Studio and other IDEs
echo.
echo To register the TSF:
echo   1. Run as Administrator
echo   2. regsvr32 %TARGET_DIR%\KeyMagicTSF.dll
echo.
echo To clean build artifacts:
echo   make-arm64x.bat clean %CONFIG%
echo.

exit /b 0

:clean
echo ========================================
echo Cleaning ARM64X Build Artifacts
echo Configuration: %CONFIG%
echo ========================================
echo.

:: Clean CMake build directories
echo Cleaning CMake build directories...
if exist "tsf\build-arm64-%CONFIG_LOWER%-x" (
    echo   Removing tsf\build-arm64-%CONFIG_LOWER%-x...
    rmdir /s /q "tsf\build-arm64-%CONFIG_LOWER%-x"
)

if exist "tsf\build-arm64ec-%CONFIG_LOWER%-x" (
    echo   Removing tsf\build-arm64ec-%CONFIG_LOWER%-x...
    rmdir /s /q "tsf\build-arm64ec-%CONFIG_LOWER%-x"
)

:: Clean the appropriate repros directory based on configuration
if /i "%CONFIG%"=="Debug" (
    if exist "tsf\repros-debug" (
        echo   Removing tsf\repros-debug...
        rmdir /s /q "tsf\repros-debug"
    )
) else (
    if exist "tsf\repros-release" (
        echo   Removing tsf\repros-release...
        rmdir /s /q "tsf\repros-release"
    )
)

:: Also clean old unified repros directory if it exists (backward compatibility)
if exist "tsf\repros" (
    echo   Removing old tsf\repros directory...
    rmdir /s /q "tsf\repros"
)

:: Clean Rust build artifacts for ARM64X targets
echo.
echo Cleaning Rust static libraries...
if /i "%CONFIG%"=="Release" (
    if exist "target\aarch64-pc-windows-msvc\release\keymagic_core.lib" (
        echo   Removing ARM64 release static lib...
        del /q "target\aarch64-pc-windows-msvc\release\keymagic_core.lib"
    )
    if exist "target\arm64ec-pc-windows-msvc\release\keymagic_core.lib" (
        echo   Removing ARM64EC release static lib...
        del /q "target\arm64ec-pc-windows-msvc\release\keymagic_core.lib"
    )
) else (
    if exist "target\aarch64-pc-windows-msvc\debug\keymagic_core.lib" (
        echo   Removing ARM64 debug static lib...
        del /q "target\aarch64-pc-windows-msvc\debug\keymagic_core.lib"
    )
    if exist "target\arm64ec-pc-windows-msvc\debug\keymagic_core.lib" (
        echo   Removing ARM64EC debug static lib...
        del /q "target\arm64ec-pc-windows-msvc\debug\keymagic_core.lib"
    )
)

echo.
echo [SUCCESS] Clean complete!
echo.
echo To rebuild:
echo   make-arm64x.bat build %CONFIG%
echo.

exit /b 0

:help
echo ARM64X CMake Build System
echo =========================
echo.
echo Usage: make-arm64x.bat [command] [config]
echo.
echo Commands:
echo   build    Build native ARM64X DLL (default)
echo   clean    Clean build artifacts
echo   help     Show this help
echo.
echo Arguments:
echo   config   Debug or Release (default: Release)
echo.
echo Examples:
echo   make-arm64x.bat                   (builds Release)
echo   make-arm64x.bat build             (builds Release)
echo   make-arm64x.bat build Debug       (builds Debug)
echo   make-arm64x.bat clean             (cleans Release)
echo   make-arm64x.bat clean Debug       (cleans Debug)
echo.
echo What this does:
echo   - Builds Rust libraries as static libs for ARM64 and ARM64EC
echo   - Uses CMake to compile C++ code for both architectures
echo   - Links everything into a single native ARM64X DLL
echo   - Provides incremental builds with CMake caching
echo.
echo Output:
echo   Release: target\release\KeyMagicTSF.dll
echo   Debug:   target\debug\KeyMagicTSF.dll
echo.
exit /b 0