@echo off
:: register-arm64x.bat - Register/unregister ARM64X forwarder DLL

setlocal

:: Check admin rights
net session >nul 2>&1
if %errorlevel% neq 0 (
    echo [ERROR] Administrator privileges required!
    echo Please run as administrator.
    exit /b 1
)

:: Parse command
set "COMMAND=%~1"
if "%COMMAND%"=="" set "COMMAND=register"

:: Navigate to script directory
cd /d "%~dp0"

if /i "%COMMAND%"=="register" goto :register
if /i "%COMMAND%"=="unregister" goto :unregister
if /i "%COMMAND%"=="status" goto :status

echo [ERROR] Unknown command: %COMMAND%
echo.
echo Usage: register-arm64x.bat [register^|unregister^|status]
exit /b 1

:register
echo Registering ARM64X forwarder DLL...

:: Check if files exist
if not exist "tsf\build-arm64x\KeyMagicTSF.dll" (
    echo [ERROR] KeyMagicTSF.dll not found!
    echo Run make-arm64x.bat first to build the forwarder.
    exit /b 1
)

if not exist "tsf\build-arm64x\KeyMagicTSF_x64.dll" (
    echo [ERROR] KeyMagicTSF_x64.dll not found!
    exit /b 1
)

if not exist "tsf\build-arm64x\KeyMagicTSF_arm64.dll" (
    echo [ERROR] KeyMagicTSF_arm64.dll not found!
    exit /b 1
)

:: Create temp directory with all DLLs
set "RANDOM_NAME=KeyMagicTSF_ARM64X_%RANDOM%_%RANDOM%"
set "TEMP_DIR=%TEMP%\%RANDOM_NAME%"
mkdir "%TEMP_DIR%" 2>nul

:: Copy all DLLs to temp location
echo Copying DLLs to temporary location...
copy /Y "tsf\build-arm64x\KeyMagicTSF.dll" "%TEMP_DIR%\" >nul || exit /b 1
copy /Y "tsf\build-arm64x\KeyMagicTSF_x64.dll" "%TEMP_DIR%\" >nul || exit /b 1
copy /Y "tsf\build-arm64x\KeyMagicTSF_arm64.dll" "%TEMP_DIR%\" >nul || exit /b 1

:: Register the forwarder
echo Registering from: %TEMP_DIR%
regsvr32 /s "%TEMP_DIR%\KeyMagicTSF.dll"
if %errorlevel% equ 0 (
    echo [SUCCESS] ARM64X forwarder registered!
    echo Location: %TEMP_DIR%
    echo.
    echo The forwarder will automatically load:
    echo   - KeyMagicTSF_x64.dll for x64/ARM64EC processes
    echo   - KeyMagicTSF_arm64.dll for ARM64 processes
) else (
    echo [ERROR] Registration failed!
    rmdir /s /q "%TEMP_DIR%" 2>nul
    exit /b 1
)
exit /b 0

:unregister
echo Unregistering ARM64X forwarder...

:: Unregister from any temp directories
for /d %%D in ("%TEMP%\KeyMagicTSF_ARM64X_*") do (
    if exist "%%D\KeyMagicTSF.dll" (
        echo Unregistering from: %%D
        regsvr32 /s /u "%%D\KeyMagicTSF.dll" 2>nul
    )
)

:: Clean up temp directories
echo Cleaning up temporary directories...
for /d %%D in ("%TEMP%\KeyMagicTSF_ARM64X_*") do (
    echo Removing: %%D
    rmdir /s /q "%%D" 2>nul
)

echo [SUCCESS] ARM64X forwarder unregistered!
exit /b 0

:status
echo KeyMagic ARM64X Status
echo ======================
echo.

:: Check build artifacts
if exist "tsf\build-arm64x\KeyMagicTSF.dll" (
    echo [OK] ARM64X forwarder built
) else (
    echo [--] ARM64X forwarder not built
)

if exist "tsf\build-arm64x\KeyMagicTSF_x64.dll" (
    echo [OK] x64 DLL built
) else (
    echo [--] x64 DLL not built
)

if exist "tsf\build-arm64x\KeyMagicTSF_arm64.dll" (
    echo [OK] ARM64 DLL built
) else (
    echo [--] ARM64 DLL not built
)

:: Check registration
reg query "HKEY_CLASSES_ROOT\CLSID\{094A562B-D08B-4CAF-8E95-8F8031CFD24C}" >nul 2>&1
if %errorlevel% equ 0 (
    echo [OK] TSF registered
    
    :: Check which temp directory it's in
    for /d %%D in ("%TEMP%\KeyMagicTSF_ARM64X_*") do (
        if exist "%%D\KeyMagicTSF.dll" (
            echo      Location: %%D
        )
    )
) else (
    echo [--] TSF not registered
)

echo.
exit /b 0

endlocal