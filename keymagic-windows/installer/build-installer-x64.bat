@echo off
:: build-installer-x64.bat - Build KeyMagic x64 installer
:: This script builds x64 components and creates the x64 installer

setlocal enabledelayedexpansion

echo ===============================================
echo KeyMagic x64 Installer Build Script
echo ===============================================
echo.

:: Navigate to keymagic-windows directory
cd /d "%~dp0\.."

:: Build dll and gui
echo [1/4] Building x64 TSF DLL...
call make.bat build x64 Release
if %errorlevel% neq 0 (
    echo [ERROR] Failed to build x64 TSF DLL
    exit /b 1
)

echo.
echo [2/4] Building x64 Tray Manager...
cd tray-manager
call make.bat build x64 Release
if %errorlevel% neq 0 (
    echo [ERROR] Failed to build x64 Tray Manager
    exit /b 1
)
cd ..

echo.
echo [3/4] Verifying build artifacts...

:: Check x64 TSF
if not exist "tsf\build-x64\Release\KeyMagicTSF_x64.dll" (
    echo [ERROR] x64 TSF DLL not found
    exit /b 1
)
echo [OK] x64 TSF DLL found

:: Check GUI
if not exist "..\target\x86_64-pc-windows-msvc\release\keymagic-gui.exe" (
    echo [ERROR] GUI executable not found
    exit /b 1
)
echo [OK] GUI executable found

:: Check Tray Manager
if not exist "tray-manager\build-x64\bin\Release\keymagic-tray.exe" (
    echo [ERROR] Tray Manager executable not found
    exit /b 1
)
echo [OK] Tray Manager executable found

:: Check resources
if not exist "resources\icons\keymagic.ico" (
    echo [WARNING] Icon file not found, installer will use default icon
)

echo.
echo [4/4] Building x64 installer...

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
"%INNO_PATH%" /Q "setup-x64.iss"
if %errorlevel% neq 0 (
    echo [ERROR] Failed to build installer
    exit /b 1
)

echo.
echo ===============================================
echo [SUCCESS] x64 Installer built successfully!
echo ===============================================
echo.
echo Output location: output\
echo.

:: List output files
echo Installer files:
dir /b "output\*x64*.exe"
echo.

endlocal