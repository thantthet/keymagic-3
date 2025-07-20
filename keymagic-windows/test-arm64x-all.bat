@echo off
:: test-arm64x-all.bat - Comprehensive test for ARM64X DLL loading
:: This script builds test apps for both architectures and runs tests

setlocal enabledelayedexpansion

:: Navigate to script directory
cd /d "%~dp0"

:: Parse configuration (default: Release)
set "CONFIG=%~1"
if "%CONFIG%"=="" set "CONFIG=Release"

if /i not "%CONFIG%"=="Debug" if /i not "%CONFIG%"=="Release" (
    echo [ERROR] Invalid config: %CONFIG%
    echo Must be Debug or Release
    exit /b 1
)

echo ================================================
echo ARM64X DLL Loading Comprehensive Test
echo Configuration: %CONFIG%
echo ================================================
echo.

:: Step 1: Ensure ARM64X DLLs are built
echo [1/4] Checking for ARM64X DLLs...
if not exist "tsf\build-arm64x\KeyMagicTSF.dll" (
    echo ARM64X DLLs not found. Building them first...
    echo.
    call make-arm64x.bat %CONFIG%
    if %errorlevel% neq 0 (
        echo [ERROR] Failed to build ARM64X DLLs
        exit /b 1
    )
)

:: Step 2: Build test app for x64
echo.
echo [2/4] Building test app for x64...
call build-test-loader.bat x64 %CONFIG%
if %errorlevel% neq 0 (
    echo [ERROR] Failed to build x64 test app
    exit /b 1
)

:: Step 3: Build test app for ARM64
echo.
echo [3/4] Building test app for ARM64...
call build-test-loader.bat arm64 %CONFIG%
if %errorlevel% neq 0 (
    echo [ERROR] Failed to build ARM64 test app
    exit /b 1
)

:: Step 4: Run tests
echo.
echo [4/4] Running tests...
echo.
echo ================================================
echo TEST 1: x64 Process Loading DLLs
echo ================================================
"test-build\test-arm64x-loader-x64-%CONFIG%.exe" %CONFIG%

echo.
echo.
echo ================================================
echo TEST 2: ARM64 Process Loading DLLs
echo ================================================
"test-build\test-arm64x-loader-arm64-%CONFIG%.exe" %CONFIG%

echo.
echo ================================================
echo All tests complete!
echo ================================================
echo.
echo Summary:
echo - x64 test app should successfully load:
echo   * ARM64X forwarder (forwards to x64 DLL)
echo   * x64 implementation DLLs
echo.
echo - ARM64 test app should successfully load:
echo   * ARM64X forwarder (forwards to ARM64 DLL)
echo   * ARM64 implementation DLLs
echo.
echo - Cross-architecture loads should fail with
echo   "Bad Image" or similar errors
echo.

exit /b 0