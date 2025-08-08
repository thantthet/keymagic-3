@echo off
setlocal enabledelayedexpansion

echo ===============================================
echo KeyMagic Unified Installer Build Script
echo Building for x64 and ARM64 architectures
echo ===============================================
echo.

REM Navigate to keymagic-windows directory
cd /d "%~dp0\.."

REM Build x64 components
echo [1/6] Building x64 components...
echo.

echo Building x64 TSF DLL and GUI...
call make.bat build x64 Release
if %errorlevel% neq 0 (
    echo [ERROR] Failed to build x64 components
    exit /b 1
)

echo Building x64 Tray Manager...
cd tray-manager
call make.bat build x64 Release
if %errorlevel% neq 0 (
    echo [ERROR] Failed to build x64 Tray Manager
    exit /b 1
)
cd ..

REM Build ARM64 components
echo.
echo [2/6] Building ARM64 components...
echo.

echo Building ARM64 TSF DLL and GUI...
call make.bat build arm64 Release
if %errorlevel% neq 0 (
    echo [ERROR] Failed to build ARM64 components
    exit /b 1
)

echo Building ARM64 Tray Manager...
cd tray-manager
call make.bat build arm64 Release
if %errorlevel% neq 0 (
    echo [ERROR] Failed to build ARM64 Tray Manager
    exit /b 1
)
cd ..

REM Build ARM64X native DLL
echo.
echo [3/6] Building ARM64X native TSF DLL...
call make-arm64x.bat Release
if %errorlevel% neq 0 (
    echo [ERROR] Failed to build ARM64X native TSF DLL
    exit /b 1
)

REM Verify all build artifacts
echo.
echo [4/6] Verifying build artifacts...
echo.

REM Check x64 components
echo Checking x64 components...
if not exist "tsf\build-x64\Release\KeyMagicTSF_x64.dll" (
    echo [ERROR] x64 TSF DLL not found
    exit /b 1
)
if not exist "..\target\x86_64-pc-windows-msvc\release\keymagic-gui.exe" (
    echo [ERROR] x64 GUI executable not found
    exit /b 1
)
if not exist "tray-manager\build-x64\bin\Release\keymagic-tray.exe" (
    echo [ERROR] x64 Tray Manager executable not found
    exit /b 1
)
echo [OK] All x64 components found

REM Check ARM64 components
echo Checking ARM64 components...
if not exist "tsf\build-arm64\Release\KeyMagicTSF_arm64.dll" (
    echo [ERROR] ARM64 TSF DLL not found
    exit /b 1
)
if not exist "..\target\aarch64-pc-windows-msvc\release\keymagic-gui.exe" (
    echo [ERROR] ARM64 GUI executable not found
    exit /b 1
)
if not exist "tray-manager\build-arm64\bin\Release\keymagic-tray.exe" (
    echo [ERROR] ARM64 Tray Manager executable not found
    exit /b 1
)
echo [OK] All ARM64 components found

REM Check ARM64X native DLL (now in target directory)
echo Checking ARM64X native DLL...
if not exist "target\release\KeyMagicTSF.dll" (
    echo [ERROR] ARM64X native DLL not found in target\release\
    echo Please run: make-arm64x.bat build Release
    exit /b 1
)
echo [OK] ARM64X native DLL found

REM Navigate to installer directory
echo.
echo [5/6] Preparing installer build...
cd installer

REM Create output directory
if not exist "output" mkdir "output"

REM Check if Inno Setup is installed
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

REM Build the unified installer
echo [6/6] Building unified installer...
"%INNO_PATH%" /Q "setup.iss"

if %errorlevel% neq 0 (
    echo.
    echo [ERROR] Failed to build unified installer
    exit /b 1
)

echo.
echo ===============================================
echo [SUCCESS] Unified installer built successfully!
echo ===============================================
echo.
echo Output location: output\
echo.

REM List output files
echo Installer files:
dir /b "output\KeyMagic3-Setup-*.exe"
echo.

endlocal