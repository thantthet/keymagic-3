@echo off
:: build-test-loader.bat - Build the ARM64X DLL loader test application
:: Usage: build-test-loader.bat [arch] [config]
:: Example: build-test-loader.bat x64 Debug

setlocal enabledelayedexpansion

:: Navigate to script directory
cd /d "%~dp0"

:: Parse arguments with defaults
set "ARCH=%~1"
if "%ARCH%"=="" set "ARCH=arm64"

set "CONFIG=%~2"
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

echo ========================================
echo Building ARM64X Loader Test
echo Architecture: %ARCH%
echo Configuration: %CONFIG%
echo ========================================
echo.

:: Set up Visual Studio environment if not already set
where cl.exe >nul 2>&1
if %errorlevel% neq 0 (
    echo Setting up Visual Studio environment for %ARCH%...
    
    if /i "%ARCH%"=="x64" (
        if /i "%PROCESSOR_ARCHITECTURE%"=="ARM64" (
            :: On ARM64, use ARM64 to x64 cross compiler
            echo Using ARM64 to x64 cross compiler...
            call "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvarsarm64_amd64.bat"
        ) else (
            :: On x64, use native compiler
            echo Using native x64 compiler...
            call "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat"
        )
    ) else (
        :: ARM64 target
        if /i "%PROCESSOR_ARCHITECTURE%"=="ARM64" (
            :: On ARM64, use native compiler
            echo Using native ARM64 compiler...
            call "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvarsarm64.bat"
        ) else (
            :: On x64, use x64 to ARM64 cross compiler
            echo Using x64 to ARM64 cross compiler...
            call "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvarsamd64_arm64.bat"
        )
    )
)

:: Create output directory
if not exist "test-build" mkdir "test-build"

:: Set compiler flags
set "CFLAGS=/nologo /W3 /EHsc /std:c++17"
if /i "%CONFIG%"=="Debug" (
    set "CFLAGS=%CFLAGS% /Od /MDd /Zi /D_DEBUG"
) else (
    set "CFLAGS=%CFLAGS% /O2 /MD /DNDEBUG"
)

:: Set output file name
set "OUTPUT=test-build\test-arm64x-loader-%ARCH%-%CONFIG%.exe"

:: Compile
echo Compiling test-arm64x-loader.cpp for %ARCH%...
cl %CFLAGS% test-arm64x-loader.cpp /Fe:"%OUTPUT%" /Fd:"test-build\\" /link /SUBSYSTEM:CONSOLE || exit /b 1

echo.
echo ========================================
echo [SUCCESS] Build complete!
echo ========================================
echo.
echo Output: %OUTPUT%
echo.
echo To run the test:
echo   %OUTPUT% [Debug^|Release]
echo.
echo Example:
echo   %OUTPUT% %CONFIG%
echo.

exit /b 0