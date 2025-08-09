@echo off
:: build-windows.bat - Build KeyMagic Core C++ for Windows
:: Supports x64, ARM64, and ARM64EC targets

setlocal enabledelayedexpansion

:: Parse arguments
set "ARCH=%~1"
set "CONFIG=%~2"

if "%ARCH%"=="" set "ARCH=x64"
if "%CONFIG%"=="" set "CONFIG=Release"

:: Validate architecture
if /i not "%ARCH%"=="x64" if /i not "%ARCH%"=="arm64" if /i not "%ARCH%"=="arm64ec" (
    echo [ERROR] Invalid architecture: %ARCH%
    echo Must be x64, arm64, or arm64ec
    goto :usage
)

:: Validate configuration
if /i not "%CONFIG%"=="Debug" if /i not "%CONFIG%"=="Release" (
    echo [ERROR] Invalid configuration: %CONFIG%
    echo Must be Debug or Release
    goto :usage
)

echo ========================================
echo Building KeyMagic Core C++
echo Architecture: %ARCH%
echo Configuration: %CONFIG%
echo ========================================
echo.

:: Set generator platform based on architecture
if /i "%ARCH%"=="x64" set "PLATFORM=x64"
if /i "%ARCH%"=="arm64" set "PLATFORM=ARM64"
if /i "%ARCH%"=="arm64ec" set "PLATFORM=ARM64EC"

:: Create build directory
set "BUILD_DIR=build-%ARCH%-%CONFIG%"

if not exist "%BUILD_DIR%" mkdir "%BUILD_DIR%"
cd "%BUILD_DIR%"

:: Configure with CMake
echo Configuring with CMake...
cmake .. -G "Visual Studio 17 2022" -A %PLATFORM% -DCMAKE_BUILD_TYPE=%CONFIG% -DBUILD_TESTING=ON || exit /b 1

:: Build
echo.
echo Building...
cmake --build . --config %CONFIG% || exit /b 1

:: Run tests (only for x64)
if /i "%ARCH%"=="x64" (
    echo.
    echo Running tests...
    ctest -C %CONFIG% --output-on-failure
    if !errorlevel! neq 0 (
        echo [WARNING] Some tests failed
    ) else (
        echo [SUCCESS] All tests passed
    )
)

cd ..

echo.
echo ========================================
echo Build Complete
echo Output: %BUILD_DIR%\%CONFIG%\
echo ========================================

goto :end

:usage
echo.
echo Usage: build-windows.bat [architecture] [configuration]
echo.
echo Architecture:
echo   x64      - 64-bit x86 (default)
echo   arm64    - ARM64 native
echo   arm64ec  - ARM64EC (x64-compatible ARM64)
echo.
echo Configuration:
echo   Debug    - Debug build with symbols
echo   Release  - Optimized release build (default)
echo.
echo Examples:
echo   build-windows.bat                  - Build x64 Release
echo   build-windows.bat x64 Debug        - Build x64 Debug
echo   build-windows.bat arm64 Release    - Build ARM64 Release
echo   build-windows.bat arm64ec Debug    - Build ARM64EC Debug

:end
endlocal