@echo off
:: build-installer-arm64.bat - Build KeyMagic ARM64 installer with ARM64X forwarder
:: This script builds ARM64X forwarder components and creates the ARM64 installer

setlocal enabledelayedexpansion

echo ===============================================
echo KeyMagic ARM64 Installer Build Script
echo ===============================================
echo.

:: Navigate to keymagic-windows directory
cd /d "%~dp0\.."

:: Build dll and gui
echo [1/3] Building ARM64 components...

:: Build GUI and keymagic-core
echo Building GUI and libraries...
call make.bat build arm64 Release
if %errorlevel% neq 0 (
    echo [ERROR] Failed to build ARM64 components
    exit /b 1
)

:: Build ARM64X forwarder TSF DLL
echo Building ARM64X forwarder TSF DLL...
call make-arm64x.bat Release
if %errorlevel% neq 0 (
    echo [ERROR] Failed to build ARM64X forwarder TSF DLL
    exit /b 1
)

echo.
echo [2/3] Verifying build artifacts...

:: Check ARM64X forwarder and its dependencies
if not exist "tsf\build-arm64x\KeyMagicTSF.dll" (
    echo [ERROR] ARM64X forwarder DLL not found
    exit /b 1
)
echo [OK] ARM64X forwarder DLL found

if not exist "tsf\build-arm64x\KeyMagicTSF_arm64.dll" (
    echo [ERROR] ARM64 implementation DLL not found
    exit /b 1
)
echo [OK] ARM64 implementation DLL found

if not exist "tsf\build-arm64x\KeyMagicTSF_x64.dll" (
    echo [ERROR] x64 implementation DLL not found
    exit /b 1
)
echo [OK] x64 implementation DLL found

:: Check GUI
if not exist "..\target\aarch64-pc-windows-msvc\release\keymagic-gui.exe" (
    echo [ERROR] GUI executable not found
    exit /b 1
)
echo [OK] GUI executable found

:: Check resources
if not exist "resources\icons\keymagic.ico" (
    echo [WARNING] Icon file not found, installer will use default icon
)

echo.
echo [3/3] Building ARM64 installer...

cd installer

:: Create output directory
if not exist "output" mkdir "output"

:: Check if Inno Setup is installed
set "INNO_PATH="
if exist "%ProgramFiles(x86)%\Inno Setup 6\ISCC.exe" (
    set "INNO_PATH=%ProgramFiles(x86)%\Inno Setup 6\ISCC.exe"
) else if exist "%ProgramFiles%\Inno Setup 6\ISCC.exe" (
    set "INNO_PATH=%ProgramFiles%\Inno Setup 6\ISCC.exe"
) else (
    echo [ERROR] Inno Setup 6 not found!
    echo Please install Inno Setup 6 from: https://jrsoftware.org/isdl.php
    exit /b 1
)

echo Using Inno Setup: %INNO_PATH%
echo.

:: Build installer
"%INNO_PATH%" /Q "setup-arm64.iss"
if %errorlevel% neq 0 (
    echo [ERROR] Failed to build installer
    exit /b 1
)

echo.
echo ===============================================
echo [SUCCESS] ARM64 Installer built successfully!
echo ===============================================
echo.
echo Output location: output\
echo.

:: List output files
echo Installer files:
dir /b "output\*arm64*.exe"
echo.

endlocal