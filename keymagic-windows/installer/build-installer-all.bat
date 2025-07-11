@echo off
:: build-installer-all.bat - Build all KeyMagic installers
:: This script builds both x64 and ARM64 installers

setlocal enabledelayedexpansion

echo ===============================================
echo KeyMagic Build All Installers Script
echo ===============================================
echo.

:: Get script directory
set "SCRIPT_DIR=%~dp0"

echo [1/2] Building x64 installer...
echo.
call "%SCRIPT_DIR%build-installer-x64.bat"
if %errorlevel% neq 0 (
    echo [ERROR] Failed to build x64 installer
    exit /b 1
)

echo.
echo ===============================================
echo.
echo [2/2] Building ARM64 installer...
echo.
call "%SCRIPT_DIR%build-installer-arm64.bat"
if %errorlevel% neq 0 (
    echo [ERROR] Failed to build ARM64 installer
    exit /b 1
)

echo.
echo ===============================================
echo [SUCCESS] All installers built successfully!
echo ===============================================
echo.
echo Output files:
cd "%SCRIPT_DIR%"
dir /b "output\*.exe"
echo.

endlocal