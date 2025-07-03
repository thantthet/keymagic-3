@echo off
REM Developer script for quick rebuild and reinstall
REM Run as Administrator for the registration to work

echo KeyMagic Developer Rebuild Script
echo =================================
echo.

REM Check for admin rights
net session >nul 2>&1
if %errorlevel% neq 0 (
    echo WARNING: Not running as administrator.
    echo The DLL will be built but not registered.
    echo.
)

REM Build
echo Building DLL...
cargo build --release
if %errorlevel% neq 0 (
    echo Build failed!
    pause
    exit /b 1
)

REM Only proceed with installation if admin
net session >nul 2>&1
if %errorlevel% eq 0 (
    set INSTALL_DIR=C:\Program Files\KeyMagic
    
    echo Unregistering old DLL...
    regsvr32 /s /u "%INSTALL_DIR%\keymagic_windows.dll" >nul 2>&1
    
    echo Copying new DLL...
    if not exist "%INSTALL_DIR%" mkdir "%INSTALL_DIR%"
    copy /y "target\release\keymagic_windows.dll" "%INSTALL_DIR%\" >nul
    
    echo Registering new DLL...
    regsvr32 /s "%INSTALL_DIR%\keymagic_windows.dll"
    
    if %errorlevel% eq 0 (
        echo.
        echo Rebuild and reinstall successful!
        echo You may need to restart applications to use the new version.
    ) else (
        echo.
        echo Registration failed!
    )
) else (
    echo.
    echo Build complete. Run as administrator to install.
)

echo.
pause