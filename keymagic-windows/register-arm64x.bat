@echo off
:: register-arm64x.bat - Register/unregister native ARM64X DLL
:: Works on both x64 and ARM64 Windows systems

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
echo ========================================
echo Registering Native ARM64X DLL
echo ========================================
echo.

:: Detect system architecture
if /i "%PROCESSOR_ARCHITECTURE%"=="ARM64" goto :arm64_system
if /i "%PROCESSOR_ARCHITECTURE%"=="AMD64" goto :x64_system
if /i "%PROCESSOR_ARCHITECTURE%"=="x86" goto :x86_check
echo [ERROR] Unknown architecture: %PROCESSOR_ARCHITECTURE%
exit /b 1

:x86_check
if defined PROCESSOR_ARCHITEW6432 goto :x64_system
echo [ERROR] x86 systems are not supported
exit /b 1

:arm64_system
echo System: ARM64 Windows
set "IS_ARM64=1"
goto :check_dll

:x64_system
echo System: x64 Windows
set "IS_ARM64=0"
goto :check_dll

:check_dll
:: Check if the ARM64X DLL exists
if not exist "tsf\build-arm64x\KeyMagicTSF.dll" (
    echo [ERROR] KeyMagicTSF.dll not found!
    echo Run make-arm64x.bat first to build the native ARM64X DLL.
    exit /b 1
)

:: For ARM64 systems, use the native ARM64X DLL directly
if "%IS_ARM64%"=="1" goto :register_arm64x
goto :register_x64

:register_arm64x
echo Registering native ARM64X DLL...
echo This DLL contains both:
echo   - ARM64 code for native ARM64 processes
echo   - ARM64EC code for x64 processes
echo.

:: Create temp directory with unique name
set "RANDOM_NAME=KeyMagicTSF_ARM64X_%RANDOM%_%RANDOM%"
set "TEMP_DIR=%TEMP%\%RANDOM_NAME%"
mkdir "%TEMP_DIR%" 2>nul

:: Copy the ARM64X DLL to temp location
echo Copying DLL to temporary location...
copy /Y "tsf\build-arm64x\KeyMagicTSF.dll" "%TEMP_DIR%\" >nul
if %errorlevel% neq 0 (
    echo [ERROR] Failed to copy DLL to temporary directory
    rmdir /s /q "%TEMP_DIR%" 2>nul
    exit /b 1
)

:: Set permissions for broad access (including sandboxed apps)
echo Setting permissions for broad access...

:: Grant access to EVERYONE (includes all users and processes)
icacls "%TEMP_DIR%" /grant "Everyone:(OI)(CI)(RX)" >nul 2>&1
icacls "%TEMP_DIR%\KeyMagicTSF.dll" /grant "Everyone:(RX)" >nul 2>&1

:: Grant access to ALL APPLICATION PACKAGES for UWP/Store apps
icacls "%TEMP_DIR%" /grant "ALL APPLICATION PACKAGES:(OI)(CI)(RX)" >nul 2>&1
icacls "%TEMP_DIR%\KeyMagicTSF.dll" /grant "ALL APPLICATION PACKAGES:(RX)" >nul 2>&1

:: Explicitly grant access to Low Integrity Level (for protected mode browsers)
icacls "%TEMP_DIR%" /grant "*S-1-16-4096:(OI)(CI)(RX)" >nul 2>&1
icacls "%TEMP_DIR%\KeyMagicTSF.dll" /grant "*S-1-16-4096:(RX)" >nul 2>&1

:: Register the DLL from temp location
echo Registering from: %TEMP_DIR%
regsvr32 /s "%TEMP_DIR%\KeyMagicTSF.dll"
if %errorlevel% equ 0 (
    echo [SUCCESS] Native ARM64X DLL registered!
    echo Location: %TEMP_DIR%\KeyMagicTSF.dll
    echo.
    echo The DLL will automatically use:
    echo   - ARM64 code in ARM64 processes
    echo   - ARM64EC code in x64/AMD64 processes
) else (
    echo [ERROR] Registration failed!
    rmdir /s /q "%TEMP_DIR%" 2>nul
    exit /b 1
)
exit /b 0

:register_x64
:: For x64 systems, check if we have a regular x64 build
if exist "tsf\build-x64\Release\KeyMagicTSF_x64.dll" (
    echo x64 system detected. Using x64-specific DLL...
    set "DLL_PATH=tsf\build-x64\Release\KeyMagicTSF_x64.dll"
) else if exist "tsf\build-x64\Debug\KeyMagicTSF_x64.dll" (
    echo x64 system detected. Using x64-specific DLL (Debug)...
    set "DLL_PATH=tsf\build-x64\Debug\KeyMagicTSF_x64.dll"
) else if exist "target\release\KeyMagicTSF.dll" (
    echo x64 system detected. Using standard DLL from target\release...
    set "DLL_PATH=target\release\KeyMagicTSF.dll"
) else if exist "target\debug\KeyMagicTSF.dll" (
    echo x64 system detected. Using standard DLL from target\debug...
    set "DLL_PATH=target\debug\KeyMagicTSF.dll"
) else (
    echo [ERROR] No compatible DLL found for x64 system!
    echo Please build the x64 version first.
    exit /b 1
)

:: Create temp directory with unique name
set "RANDOM_NAME=KeyMagicTSF_x64_%RANDOM%_%RANDOM%"
set "TEMP_DIR=%TEMP%\%RANDOM_NAME%"
mkdir "%TEMP_DIR%" 2>nul

:: Copy and register from temp location
echo Copying DLL to temporary location...
copy /Y "%DLL_PATH%" "%TEMP_DIR%\KeyMagicTSF.dll" >nul
if %errorlevel% neq 0 (
    echo [ERROR] Failed to copy DLL to temporary directory
    rmdir /s /q "%TEMP_DIR%" 2>nul
    exit /b 1
)

:: Set permissions for broad access (including sandboxed apps)
echo Setting permissions for broad access...

:: Grant access to EVERYONE (includes all users and processes)
icacls "%TEMP_DIR%" /grant "Everyone:(OI)(CI)(RX)" >nul 2>&1
icacls "%TEMP_DIR%\KeyMagicTSF.dll" /grant "Everyone:(RX)" >nul 2>&1

:: Grant access to ALL APPLICATION PACKAGES for UWP/Store apps
icacls "%TEMP_DIR%" /grant "ALL APPLICATION PACKAGES:(OI)(CI)(RX)" >nul 2>&1
icacls "%TEMP_DIR%\KeyMagicTSF.dll" /grant "ALL APPLICATION PACKAGES:(RX)" >nul 2>&1

:: Explicitly grant access to Low Integrity Level (for protected mode browsers)
icacls "%TEMP_DIR%" /grant "*S-1-16-4096:(OI)(CI)(RX)" >nul 2>&1
icacls "%TEMP_DIR%\KeyMagicTSF.dll" /grant "*S-1-16-4096:(RX)" >nul 2>&1

echo Registering from: %TEMP_DIR%
regsvr32 /s "%TEMP_DIR%\KeyMagicTSF.dll"
if %errorlevel% equ 0 (
    echo [SUCCESS] x64 DLL registered!
    echo Location: %TEMP_DIR%\KeyMagicTSF.dll
) else (
    echo [ERROR] Registration failed!
    rmdir /s /q "%TEMP_DIR%" 2>nul
    exit /b 1
)
exit /b 0

:unregister
echo ========================================
echo Unregistering KeyMagic TSF
echo ========================================
echo.

:: Try to unregister from various possible locations
set "UNREGISTERED=0"

:: Clean up ARM64X temp directories
for /d %%D in ("%TEMP%\KeyMagicTSF_ARM64X_*") do (
    if exist "%%D\KeyMagicTSF.dll" (
        echo Unregistering from temp: %%D
        regsvr32 /s /u "%%D\KeyMagicTSF.dll" 2>nul
        if %errorlevel% equ 0 set "UNREGISTERED=1"
    )
    rmdir /s /q "%%D" 2>nul
)

:: Clean up x64 temp directories
for /d %%D in ("%TEMP%\KeyMagicTSF_x64_*") do (
    if exist "%%D\KeyMagicTSF.dll" (
        echo Unregistering from temp: %%D
        regsvr32 /s /u "%%D\KeyMagicTSF.dll" 2>nul
        if %errorlevel% equ 0 set "UNREGISTERED=1"
    )
    rmdir /s /q "%%D" 2>nul
)

if "%UNREGISTERED%"=="1" (
    echo [SUCCESS] KeyMagic TSF unregistered!
) else (
    echo [INFO] No registered KeyMagic TSF found.
)
exit /b 0

:status
echo ========================================
echo KeyMagic TSF Registration Status
echo ========================================
echo.

:: Check system architecture
echo System Information:
echo -------------------
if /i "%PROCESSOR_ARCHITECTURE%"=="ARM64" (
    echo Architecture: ARM64
    echo ARM64X Support: Yes
) else if /i "%PROCESSOR_ARCHITECTURE%"=="AMD64" (
    echo Architecture: x64/AMD64
    echo ARM64X Support: No (x64 system)
) else (
    echo Architecture: %PROCESSOR_ARCHITECTURE%
)
echo.

:: Check build artifacts
echo Build Status:
echo -------------
if exist "tsf\build-arm64x\KeyMagicTSF.dll" (
    for %%F in ("tsf\build-arm64x\KeyMagicTSF.dll") do (
        echo [OK] Native ARM64X DLL built (%%~zF bytes^)
    )
) else (
    echo [--] Native ARM64X DLL not built
)

if exist "tsf\build-x64\Release\KeyMagicTSF_x64.dll" (
    echo [OK] x64 DLL built (Release)
) else if exist "tsf\build-x64\Debug\KeyMagicTSF_x64.dll" (
    echo [OK] x64 DLL built (Debug)
) else (
    echo [--] x64 DLL not built
)

if exist "tsf\build-arm64\Release\KeyMagicTSF_arm64.dll" (
    echo [OK] ARM64 DLL built (Release)
) else if exist "tsf\build-arm64\Debug\KeyMagicTSF_arm64.dll" (
    echo [OK] ARM64 DLL built (Debug)
) else (
    echo [--] ARM64 DLL not built
)
echo.

:: Check registration
echo Registration Status:
echo --------------------
reg query "HKEY_CLASSES_ROOT\CLSID\{094A562B-D08B-4CAF-8E95-8F8031CFD24C}" >nul 2>&1
if %errorlevel% equ 0 (
    echo [OK] TSF registered in registry
    
    :: Check temp directory installations
    set "INSTALL_FOUND=0"
    
    :: Check ARM64X temp directories
    for /d %%D in ("%TEMP%\KeyMagicTSF_ARM64X_*") do (
        if exist "%%D\KeyMagicTSF.dll" (
            if "%INSTALL_FOUND%"=="0" (
                echo      Type: Native ARM64X
                set "INSTALL_FOUND=1"
            )
            echo      Location: %%D
            
            :: Verify it's actually ARM64X using dumpbin if available
            where dumpbin >nul 2>&1
            if %errorlevel% equ 0 (
                dumpbin /headers "%%D\KeyMagicTSF.dll" 2>nul | findstr /i "arm64x" >nul
                if %errorlevel% equ 0 (
                    echo      Verified: ARM64X binary confirmed
                )
            )
        )
    )
    
    :: Check x64 temp directories
    for /d %%D in ("%TEMP%\KeyMagicTSF_x64_*") do (
        if exist "%%D\KeyMagicTSF.dll" (
            if "%INSTALL_FOUND%"=="0" (
                echo      Type: x64
                set "INSTALL_FOUND=1"
            )
            echo      Location: %%D
        )
    )
) else (
    echo [--] TSF not registered
)

echo.
exit /b 0

endlocal