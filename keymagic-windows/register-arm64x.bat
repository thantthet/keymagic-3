@echo off
:: register-arm64x.bat - Register/unregister native ARM64X DLL
:: The ARM64X DLL works on both x64 and ARM64 Windows systems:
::   - On ARM64 Windows: Uses native ARM64 code
::   - On x64 Windows: Uses ARM64EC code (emulated)
:: For architecture-specific builds, use make.bat instead

setlocal

:: Check admin rights
net session >nul 2>&1
if %errorlevel% neq 0 (
    echo [ERROR] Administrator privileges required!
    echo Please run as administrator.
    exit /b 1
)

:: Parse command and configuration
set "COMMAND=%~1"
if "%COMMAND%"=="" set "COMMAND=register"

set "CONFIG=%~2"
if "%CONFIG%"=="" set "CONFIG=Release"

:: Validate configuration
if /i not "%CONFIG%"=="Debug" if /i not "%CONFIG%"=="Release" (
    echo [ERROR] Invalid configuration: %CONFIG%
    echo Must be Debug or Release
    exit /b 1
)

:: Navigate to script directory
cd /d "%~dp0"

if /i "%COMMAND%"=="register" goto :register
if /i "%COMMAND%"=="unregister" goto :unregister
if /i "%COMMAND%"=="status" goto :status

echo [ERROR] Unknown command: %COMMAND%
echo.
echo Usage: register-arm64x.bat [register^|unregister^|status] [Debug^|Release]
echo.
echo Examples:
echo   register-arm64x.bat register           # Register Release build
echo   register-arm64x.bat register Debug     # Register Debug build
echo   register-arm64x.bat register Release   # Register Release build
echo   register-arm64x.bat unregister         # Unregister all
echo   register-arm64x.bat status              # Check status
exit /b 1

:register
echo ========================================
echo Registering Native ARM64X DLL (%CONFIG%)
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

:x64_system
echo ========================================
echo [INFO] x64 Windows Detected
echo ========================================
echo.
echo This script is for registering native ARM64X builds on ARM64 Windows.
echo.
echo For x64 Windows, please use:
echo   make.bat register x64 %CONFIG%
echo.
echo The ARM64X DLL can technically run on x64 Windows using ARM64EC emulation,
echo but native x64 builds will provide better performance.
echo.
exit /b 0

:arm64_system
echo System: ARM64 Windows
set "IS_ARM64=1"
goto :check_dll

:check_dll
:: Check if the ARM64X DLL exists in target directory based on CONFIG
if /i "%CONFIG%"=="Release" (
    if exist "target\release\KeyMagicTSF.dll" (
        set "ARM64X_DLL=target\release\KeyMagicTSF.dll"
    ) else (
        echo [ERROR] KeyMagicTSF.dll not found in target\release\!
        echo Run make-arm64x.bat Release first to build the native ARM64X DLL.
        exit /b 1
    )
) else (
    if exist "target\debug\KeyMagicTSF.dll" (
        set "ARM64X_DLL=target\debug\KeyMagicTSF.dll"
    ) else (
        echo [ERROR] KeyMagicTSF.dll not found in target\debug\!
        echo Run make-arm64x.bat Debug first to build the native ARM64X DLL.
        exit /b 1
    )
)

:register_arm64x
echo Registering native ARM64X DLL...
echo Running on ARM64 Windows - will use native ARM64 code path
echo.
echo This DLL contains both:
echo   - ARM64 code for native ARM64 processes
echo   - ARM64EC code for x64 processes (when ARM64 apps host x64 components)
echo.

:: Create temp directory with unique name
set "RANDOM_NAME=KeyMagicTSF_ARM64X_%RANDOM%_%RANDOM%"
set "TEMP_DIR=%TEMP%\%RANDOM_NAME%"
mkdir "%TEMP_DIR%" 2>nul

:: Copy the ARM64X DLL to temp location
echo Copying DLL to temporary location...
copy /Y "%ARM64X_DLL%" "%TEMP_DIR%\" >nul
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

:: Also clean up any legacy x64 temp directories (for backwards compatibility)
for /d %%D in ("%TEMP%\KeyMagicTSF_x64_*") do (
    if exist "%%D\KeyMagicTSF.dll" (
        echo Unregistering legacy x64 from temp: %%D
        regsvr32 /s /u "%%D\KeyMagicTSF.dll" 2>nul
        if %errorlevel% equ 0 set "UNREGISTERED=1"
    )
    rmdir /s /q "%%D" 2>nul
)

:: Clean up any other KeyMagicTSF temp directories
for /d %%D in ("%TEMP%\KeyMagicTSF_*") do (
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
echo KeyMagic TSF Registration Status (%CONFIG%)
echo ========================================
echo.

:: Check system architecture
echo System Information:
echo -------------------
if /i "%PROCESSOR_ARCHITECTURE%"=="ARM64" (
    echo Architecture: ARM64
    echo ARM64X Support: Native
) else if /i "%PROCESSOR_ARCHITECTURE%"=="AMD64" (
    echo Architecture: x64/AMD64
    echo.
    echo [INFO] This script is for ARM64 Windows systems.
    echo For x64 Windows status, use: make.bat status x64 %CONFIG%
    echo.
    exit /b 0
) else (
    echo Architecture: %PROCESSOR_ARCHITECTURE%
    echo [ERROR] Unsupported architecture
    exit /b 1
)
echo.

:: Check build artifacts
echo Build Status (%CONFIG%):
echo ------------------------
if /i "%CONFIG%"=="Release" (
    if exist "target\release\KeyMagicTSF.dll" (
        for %%F in ("target\release\KeyMagicTSF.dll") do (
            echo [OK] Native ARM64X DLL built - Release (%%~zF bytes^)
        )
    ) else (
        echo [--] Native ARM64X DLL not built (Release)
    )
) else (
    if exist "target\debug\KeyMagicTSF.dll" (
        for %%F in ("target\debug\KeyMagicTSF.dll") do (
            echo [OK] Native ARM64X DLL built - Debug (%%~zF bytes^)
        )
    ) else (
        echo [--] Native ARM64X DLL not built (Debug)
    )
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