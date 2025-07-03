@echo off
REM Uninstallation script for KeyMagic Windows TSF DLL
REM This script must be run as Administrator

echo KeyMagic Windows TSF DLL Uninstaller
echo ====================================
echo.

REM Check for admin rights
net session >nul 2>&1
if %errorlevel% neq 0 (
    echo ERROR: This script must be run as Administrator!
    echo Please right-click and select "Run as administrator"
    pause
    exit /b 1
)

set INSTALL_DIR=C:\Program Files\KeyMagic

REM Check if installed
if not exist "%INSTALL_DIR%\keymagic_windows.dll" (
    echo KeyMagic TSF DLL is not installed.
    pause
    exit /b 0
)

echo This will uninstall KeyMagic TSF DLL.
echo Make sure to remove KeyMagic from your Windows language settings first!
echo.
set /p CONFIRM=Continue with uninstallation? (Y/N): 
if /i "%CONFIRM%" neq "Y" (
    echo Uninstallation cancelled.
    pause
    exit /b 0
)

REM Unregister DLL
echo Unregistering KeyMagic TSF DLL...
regsvr32 /s /u "%INSTALL_DIR%\keymagic_windows.dll"
if %errorlevel% neq 0 (
    echo WARNING: Failed to unregister DLL. It may not be registered.
)

REM Delete files
echo Removing files...
del /f /q "%INSTALL_DIR%\keymagic_windows.dll" >nul 2>&1

REM Remove directory if empty
rmdir "%INSTALL_DIR%" >nul 2>&1

echo.
echo Uninstallation complete!
echo.
echo Note: If KeyMagic still appears in your language settings,
echo please remove it manually from Windows Settings.
echo.

pause