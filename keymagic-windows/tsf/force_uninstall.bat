@echo off
REM Force uninstall script for KeyMagic Windows TSF DLL
REM This script must be run as Administrator

echo KeyMagic Windows TSF Force Uninstaller
echo ======================================
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

echo Step 1: Stopping Text Services Framework...
net stop "Touch Keyboard and Handwriting Panel Service" >nul 2>&1
taskkill /F /IM ctfmon.exe >nul 2>&1
timeout /t 2 >nul

echo Step 2: Unregistering DLL...
regsvr32 /s /u "%INSTALL_DIR%\keymagic_windows.dll" >nul 2>&1

echo Step 3: Removing registry entries...
reg delete "HKCR\CLSID\{12345678-1234-1234-1234-123456789ABC}" /f >nul 2>&1
reg delete "HKLM\SOFTWARE\Classes\CLSID\{12345678-1234-1234-1234-123456789ABC}" /f >nul 2>&1

echo Step 4: Killing any processes using the DLL...
REM Kill any process that might be using our DLL
powershell -Command "Get-Process | Where-Object {$_.Modules.FileName -like '*keymagic_windows.dll'} | Stop-Process -Force" >nul 2>&1

echo Step 5: Attempting to delete DLL...
timeout /t 2 >nul
del /f /q "%INSTALL_DIR%\keymagic_windows.dll" >nul 2>&1

if exist "%INSTALL_DIR%\keymagic_windows.dll" (
    echo.
    echo WARNING: Could not delete DLL immediately.
    echo Scheduling deletion on next reboot...
    
    REM Schedule file deletion on reboot
    reg add "HKLM\SYSTEM\CurrentControlSet\Control\Session Manager" /v PendingFileRenameOperations /t REG_MULTI_SZ /d "\??\%INSTALL_DIR%\keymagic_windows.dll\0\0" /f >nul 2>&1
    
    echo.
    echo The DLL will be deleted after restarting Windows.
    echo.
    echo Additional steps:
    echo 1. Restart Windows
    echo 2. The DLL will be automatically deleted during startup
) else (
    echo.
    echo SUCCESS: DLL has been deleted!
)

echo.
echo Step 6: Restarting Text Services...
start ctfmon.exe >nul 2>&1

echo.
echo Uninstallation complete!
echo.
echo If you still see KeyMagic in language settings:
echo 1. Go to Settings - Time and Language - Language
echo 2. Click on your language and select Options
echo 3. Remove KeyMagic keyboard if it's still listed
echo.

pause