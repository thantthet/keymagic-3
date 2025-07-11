@echo off
:: build-installer.bat - Build KeyMagic installer
:: This script builds all components and creates the installer

setlocal enabledelayedexpansion

echo ===============================================
echo KeyMagic Installer Build Script
echo ===============================================
echo.

:: Navigate to keymagic-windows directory
cd /d "%~dp0\.."

:: Check if running as admin (needed for TSF registration tests)
net session >nul 2>&1
if %errorlevel% neq 0 (
    echo [WARNING] Not running as administrator.
    echo Some post-build tests may be skipped.
    echo.
)

:: Build all components
echo [1/5] Building x64 TSF DLL...
call make.bat build x64 Release
if %errorlevel% neq 0 (
    echo [ERROR] Failed to build x64 TSF DLL
    exit /b 1
)

echo.
echo [2/5] Building ARM64 TSF DLL...
call make.bat build arm64 Release
if %errorlevel% neq 0 (
    echo [ERROR] Failed to build ARM64 TSF DLL
    exit /b 1
)

echo.
echo [3/5] Building GUI (x64 only)...
pushd gui-tauri
call build.bat
if %errorlevel% neq 0 (
    echo [ERROR] Failed to build GUI
    popd
    exit /b 1
)
popd

echo.
echo [4/5] Verifying build artifacts...

:: Check x64 TSF
if not exist "tsf\build-x64\Release\KeyMagicTSF.dll" (
    echo [ERROR] x64 TSF DLL not found
    exit /b 1
)
echo [OK] x64 TSF DLL found

:: Check ARM64 TSF
if not exist "tsf\build-ARM64\Release\KeyMagicTSF.dll" (
    echo [ERROR] ARM64 TSF DLL not found
    exit /b 1
)
echo [OK] ARM64 TSF DLL found

:: Check GUI
if not exist "target\x86_64-pc-windows-msvc\release\gui-tauri.exe" (
    echo [ERROR] GUI executable not found
    exit /b 1
)
echo [OK] GUI executable found

:: Check resources
if not exist "resources\icons\keymagic.ico" (
    echo [WARNING] Icon file not found, installer will use default icon
)

echo.
echo [5/5] Building installer...

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
"%INNO_PATH%" /Q "setup.iss"
if %errorlevel% neq 0 (
    echo [ERROR] Failed to build installer
    exit /b 1
)

echo.
echo ===============================================
echo [SUCCESS] Installer built successfully!
echo ===============================================
echo.
echo Output location: output\
echo.

:: List output files
echo Installer files:
dir /b "output\*.exe"
echo.

endlocal