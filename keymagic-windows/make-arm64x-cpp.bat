@echo off
:: make-arm64x-cpp.bat - Build native ARM64X DLL using CMake with C++ core
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
echo Building Native ARM64X DLL with C++ Core
echo Configuration: %CONFIG%
echo ========================================
echo.
echo [INFO] This build uses CMake with C++ keymagic-core
echo.

:: Step 1: Configure and build ARM64 using CMake
echo [1/3] Configuring and building ARM64 components...
cd tsf

:: Configure using preset
echo Configuring ARM64 build with CMake preset...
cmake --preset arm64-%CONFIG_LOWER%-x || exit /b 1

:: Build
echo Building ARM64 components (includes C++ core)...
cmake --build build-arm64-%CONFIG_LOWER%-x --config %CONFIG% || exit /b 1

:: Step 2: Configure and build ARM64EC using CMake
echo.
echo [2/3] Configuring and building ARM64EC components...

:: Configure using preset
echo Configuring ARM64EC build with CMake preset...
cmake --preset arm64ec-%CONFIG_LOWER%-x || exit /b 1

:: Build (this will combine ARM64 and ARM64EC into ARM64X)
echo Building ARM64EC and linking ARM64X binary...
cmake --build build-arm64ec-%CONFIG_LOWER%-x --config %CONFIG% || exit /b 1

:: Step 3: Verify the output
echo.
echo [3/3] Verifying ARM64X DLL...
cd ..

:: Determine the output path based on configuration
if /i "%CONFIG%"=="Debug" (
    set "DLL_PATH=tsf\build-arm64ec-debug-x\Debug\KeyMagicTSF.dll"
) else (
    set "DLL_PATH=tsf\build-arm64ec-release-x\Release\KeyMagicTSF.dll"
)

if exist "%DLL_PATH%" (
    echo.
    echo [SUCCESS] ARM64X DLL built successfully:
    echo %DLL_PATH%
    
    :: Check if it's actually ARM64X using dumpbin
    echo.
    echo Verifying ARM64X format...
    dumpbin /headers "%DLL_PATH%" 2>nul | findstr /i "arm64x arm64 arm64ec" >nul
    if !errorlevel! equ 0 (
        echo [SUCCESS] DLL is confirmed to be ARM64X format
        
        :: Show machine types
        echo.
        echo Machine types in DLL:
        dumpbin /headers "%DLL_PATH%" 2>nul | findstr /i "machine"
    ) else (
        echo [WARNING] Could not verify ARM64X format. The DLL may still be valid.
    )
) else (
    echo [ERROR] ARM64X DLL not found at expected location:
    echo %DLL_PATH%
    exit /b 1
)

goto :end

:clean
echo ========================================
echo Cleaning ARM64X Build Artifacts
echo Configuration: %CONFIG%
echo ========================================
echo.

:: Clean CMake build directories
cd tsf
echo Cleaning ARM64 build directory...
if exist "build-arm64-%CONFIG_LOWER%-x" (
    rmdir /s /q "build-arm64-%CONFIG_LOWER%-x"
    echo Removed build-arm64-%CONFIG_LOWER%-x
)

echo Cleaning ARM64EC build directory...
if exist "build-arm64ec-%CONFIG_LOWER%-x" (
    rmdir /s /q "build-arm64ec-%CONFIG_LOWER%-x"
    echo Removed build-arm64ec-%CONFIG_LOWER%-x
)

:: Clean response files
echo Cleaning response files...
if exist "repros-%CONFIG_LOWER%" (
    rmdir /s /q "repros-%CONFIG_LOWER%"
    echo Removed repros-%CONFIG_LOWER%
)

cd ..

echo.
echo [SUCCESS] Clean complete for %CONFIG% configuration

goto :end

:help
echo ========================================
echo Native ARM64X Build Script (C++ Core)
echo ========================================
echo.
echo Usage: make-arm64x-cpp.bat [command] [configuration]
echo.
echo Commands:
echo   build   - Build the native ARM64X DLL
echo   clean   - Clean build artifacts
echo   help    - Show this help message
echo.
echo Configuration:
echo   Debug   - Debug build with symbols
echo   Release - Optimized release build (default)
echo.
echo Examples:
echo   make-arm64x-cpp.bat build           - Build Release ARM64X DLL
echo   make-arm64x-cpp.bat build Debug     - Build Debug ARM64X DLL  
echo   make-arm64x-cpp.bat clean           - Clean Release build
echo   make-arm64x-cpp.bat clean Debug     - Clean Debug build
echo.
echo Notes:
echo   - Requires Visual Studio 2022 17.11 or later with ARM64 tools
echo   - Uses CMake presets for configuration
echo   - Creates incremental builds (faster rebuilds)
echo   - C++ keymagic-core is built as part of the TSF build
echo.

:end
endlocal