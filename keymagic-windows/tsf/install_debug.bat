@echo off
REM Installation script for KeyMagic Windows TSF DLL (Debug Version)
REM This script must be run as Administrator

echo KeyMagic Windows TSF DLL Installer (DEBUG)
echo ==========================================
echo.

REM Get the directory where this script is located
set SCRIPT_DIR=%~dp0
echo Script directory: %SCRIPT_DIR%

REM Check for admin rights
net session >nul 2>&1
if %errorlevel% neq 0 (
    echo ERROR: This script must be run as Administrator!
    echo Please right-click and select "Run as administrator"
    pause
    exit /b 1
)

REM Determine the DLL path relative to script location
set DLL_PATH=%SCRIPT_DIR%..\target\debug\keymagic_windows.dll

REM Check if DLL exists
if not exist "%DLL_PATH%" (
    echo ERROR: keymagic_windows.dll not found!
    echo Looking for: %DLL_PATH%
    echo Please run build_debug.bat first.
    pause
    exit /b 1
)

REM Create installation directory
set INSTALL_DIR=C:\Program Files\KeyMagic
echo Creating installation directory...
if not exist "%INSTALL_DIR%" mkdir "%INSTALL_DIR%"

REM Copy DLL
echo Copying DEBUG DLL to installation directory...
copy /y "%DLL_PATH%" "%INSTALL_DIR%\" >nul
if %errorlevel% neq 0 (
    echo ERROR: Failed to copy DLL!
    pause
    exit /b 1
)

REM Unregister old version if exists
echo Unregistering previous version (if any)...
regsvr32 /s /u "%INSTALL_DIR%\keymagic_windows.dll" >nul 2>&1

REM Register new DLL
echo Registering KeyMagic TSF DLL (DEBUG)...
regsvr32 /s "%INSTALL_DIR%\keymagic_windows.dll"
if %errorlevel% neq 0 (
    echo ERROR: Failed to register DLL!
    echo Make sure you have administrator rights.
    pause
    exit /b 1
)

echo.
echo DEBUG Installation successful!
echo.
echo Debug output locations:
echo 1. Use DebugView.exe to see real-time debug output
echo 2. Check log files in: %%TEMP%%\KeyMagicTSF_*.log
echo.
echo Next steps:
echo 1. Go to Windows Settings - Time and Language - Language
echo 2. Click on your language and select Options
echo 3. Add KeyMagic as a keyboard
echo 4. Use Win+Space to switch to KeyMagic
echo 5. Run DebugView.exe to see debug messages
echo.

pause