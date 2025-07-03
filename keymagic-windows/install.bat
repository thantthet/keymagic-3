@echo off
REM Installation script for KeyMagic Windows TSF DLL
REM This script must be run as Administrator

echo KeyMagic Windows TSF DLL Installer
echo ==================================
echo.

REM Check for admin rights
net session >nul 2>&1
if %errorlevel% neq 0 (
    echo ERROR: This script must be run as Administrator!
    echo Please right-click and select "Run as administrator"
    pause
    exit /b 1
)

REM Check if DLL exists
if not exist "target\release\keymagic_windows.dll" (
    echo ERROR: keymagic_windows.dll not found!
    echo Please run build_windows.bat first.
    pause
    exit /b 1
)

REM Create installation directory
set INSTALL_DIR=C:\Program Files\KeyMagic
echo Creating installation directory...
if not exist "%INSTALL_DIR%" mkdir "%INSTALL_DIR%"

REM Copy DLL
echo Copying DLL to installation directory...
copy /y "target\release\keymagic_windows.dll" "%INSTALL_DIR%\" >nul
if %errorlevel% neq 0 (
    echo ERROR: Failed to copy DLL!
    pause
    exit /b 1
)

REM Unregister old version if exists
echo Unregistering previous version (if any)...
regsvr32 /s /u "%INSTALL_DIR%\keymagic_windows.dll" >nul 2>&1

REM Register new DLL
echo Registering KeyMagic TSF DLL...
regsvr32 /s "%INSTALL_DIR%\keymagic_windows.dll"
if %errorlevel% neq 0 (
    echo ERROR: Failed to register DLL!
    echo Make sure you have administrator rights.
    pause
    exit /b 1
)

echo.
echo Installation successful!
echo.
echo Next steps:
echo 1. Go to Windows Settings - Time and Language - Language
echo 2. Click on your language and select Options
echo 3. Add KeyMagic as a keyboard
echo 4. Use Win+Space to switch to KeyMagic
echo.

pause