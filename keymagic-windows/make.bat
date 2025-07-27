@echo off
:: make.bat - KeyMagic build system
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
if /i "%COMMAND%"=="register" goto :register
if /i "%COMMAND%"=="unregister" goto :unregister
if /i "%COMMAND%"=="status" goto :status
if /i "%COMMAND%"=="help" goto :help

echo [ERROR] Unknown command: %COMMAND%
goto :help

:: ========== BUILD ==========
:build
echo Building KeyMagic (%ARCH% %CONFIG%)...
echo.

:: Build Rust
echo Building Rust libraries...
if /i "%CONFIG%"=="Release" (
    cargo build -p keymagic-core --release --target %RUST_TARGET% || exit /b 1
    cargo build -p keymagic-gui --release --target %RUST_TARGET% || exit /b 1
) else (
    cargo build -p keymagic-core --target %RUST_TARGET% || exit /b 1
    cargo build -p keymagic-gui --target %RUST_TARGET% || exit /b 1
)

:: Build TSF
echo.
echo Building TSF...
cd tsf
if not exist "build-%ARCH%" mkdir "build-%ARCH%"
cd "build-%ARCH%"
cmake -G "Visual Studio 17 2022" -A %CMAKE_ARCH% .. || exit /b 1
cmake --build . --config %CONFIG% || exit /b 1
cd ..\..

echo.
echo [SUCCESS] Build complete!
exit /b 0

:: ========== CLEAN ==========
:clean
echo Cleaning build artifacts (%ARCH% %CONFIG%)...

:: Clean Rust
cargo clean

:: Clean TSF
if exist "tsf\build-%ARCH%" (
    rmdir /s /q "tsf\build-%ARCH%"
)

echo [SUCCESS] Clean complete!
exit /b 0

:: ========== REGISTER ==========
:register
:: Check admin rights
net session >nul 2>&1
if %errorlevel% neq 0 (
    echo [ERROR] Administrator privileges required!
    echo Please run as administrator.
    exit /b 1
)

:: Set DLL path
set "DLL_PATH=tsf\build-%ARCH%\%CONFIG%\KeyMagicTSF_%ARCH%.dll"

if not exist "%DLL_PATH%" (
    echo [ERROR] TSF DLL not found at: %DLL_PATH%
    echo Run: make.bat build %ARCH% %CONFIG%
    exit /b 1
)

:: Generate random temp directory name
set "RANDOM_NAME=KeyMagicTSF_%RANDOM%_%RANDOM%"
set "TEMP_DIR=%TEMP%\%RANDOM_NAME%"
set "TEMP_DLL=%TEMP_DIR%\KeyMagicTSF.dll"

:: Create temp directory
mkdir "%TEMP_DIR%" 2>nul
if %errorlevel% neq 0 (
    echo [ERROR] Failed to create temp directory: %TEMP_DIR%
    exit /b 1
)

:: Copy DLL to temp location
echo Copying TSF to temporary location...
copy /Y "%DLL_PATH%" "%TEMP_DLL%" >nul 2>&1
if %errorlevel% neq 0 (
    echo [ERROR] Failed to copy DLL to temp location
    rmdir /s /q "%TEMP_DIR%" 2>nul
    exit /b 1
)

echo Registering TSF (%ARCH% %CONFIG%) from temp location...
regsvr32 /s "%TEMP_DLL%"
if %errorlevel% equ 0 (
    echo [SUCCESS] TSF registered!
    echo Temp location: %TEMP_DLL%
) else (
    echo [ERROR] Registration failed!
    :: Clean up temp directory on failure
    rmdir /s /q "%TEMP_DIR%" 2>nul
    exit /b 1
)
exit /b 0

:: ========== UNREGISTER ==========
:unregister
:: Check admin rights
net session >nul 2>&1
if %errorlevel% neq 0 (
    echo [ERROR] Administrator privileges required!
    echo Please run as administrator.
    exit /b 1
)

echo Unregistering TSF (%ARCH% %CONFIG%)...

:: Try unregistering from build directory (old method)
regsvr32 /s /u "tsf\build-%ARCH%\%CONFIG%\KeyMagicTSF.dll" 2>nul

:: Unregister any TSF DLLs in temp directories
for /d %%D in ("%TEMP%\KeyMagicTSF_*") do (
    if exist "%%D\KeyMagicTSF.dll" (
        echo Unregistering from: %%D
        regsvr32 /s /u "%%D\KeyMagicTSF.dll" 2>nul
    )
)

:: Clean up temp directories after unregistering
echo Cleaning up temporary TSF directories...
for /d %%D in ("%TEMP%\KeyMagicTSF_*") do (
    echo Removing: %%D
    rmdir /s /q "%%D" 2>nul
)

echo [SUCCESS] TSF unregistered!
exit /b 0

:: ========== STATUS ==========
:status
echo KeyMagic Status (%ARCH% %CONFIG%)
echo ================================
echo.

:: Check TSF DLL
set "TSF_DLL=tsf\build-%ARCH%\%CONFIG%\KeyMagicTSF.dll"
if exist "%TSF_DLL%" (
    echo [OK] TSF DLL found
) else (
    echo [--] TSF DLL not found
)

:: Check GUI EXE
if /i "%CONFIG%"=="Release" (
    set "GUI_EXE=target\%RUST_TARGET%\release\keymagic-config.exe"
) else (
    set "GUI_EXE=target\%RUST_TARGET%\debug\keymagic-config.exe"
)

if exist "%GUI_EXE%" (
    echo [OK] GUI executable found
) else (
    echo [--] GUI executable not found
)

:: Check if TSF is registered
reg query "HKEY_CLASSES_ROOT\CLSID\{094A562B-D08B-4CAF-8E95-8F8031CFD24C}" >nul 2>&1
if %errorlevel% equ 0 (
    echo [OK] TSF registered
) else (
    echo [--] TSF not registered
)

echo.
exit /b 0

:: ========== HELP ==========
:help
echo KeyMagic Build System
echo ============================
echo.
echo Usage: make.bat [command] [arch] [config]
echo.
echo Commands:
echo   build       Build all components
echo   clean       Clean build artifacts
echo   register    Register TSF (requires admin)
echo   unregister  Unregister TSF (requires admin)
echo   status      Check build status
echo   help        Show this help
echo.
echo Arguments:
echo   arch      x64 or arm64 (default: arm64)
echo   config    Debug or Release (default: Release)
echo.
echo Examples:
echo   make.bat build
echo   make.bat build x64
echo   make.bat build x64 Debug
echo   make.bat register x64
echo   make.bat unregister x64
echo   make.bat status arm64 Debug
echo.
exit /b 0

endlocal