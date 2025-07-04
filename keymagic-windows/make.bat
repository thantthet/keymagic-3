@echo off
setlocal enabledelayedexpansion

:: KeyMagic Windows Build System
:: Usage: make.bat [command] [options]

cd "%~dp0"

:: Load configuration
call scripts\config.bat
call scripts\functions.bat

:: Default build configuration
set "BUILD_CONFIG=Release"

:: Parse command and arguments
set "COMMAND=%~1"
if "%COMMAND%"=="" set "COMMAND=help"

:: Initialize arguments
set "ARG1="
set "ARG2="

:: Parse remaining arguments
set "SHIFT_COUNT=0"
:parse_args_loop
shift
set /a SHIFT_COUNT+=1
if "%~1"=="" goto :done_parsing

:: Check for build config flags
if /i "%~1"=="--debug" (
    set "BUILD_CONFIG=Debug"
    goto :parse_args_loop
)
if /i "%~1"=="--release" (
    set "BUILD_CONFIG=Release"
    goto :parse_args_loop
)

:: Store non-flag arguments
if not defined ARG1 (
    set "ARG1=%~1"
) else if not defined ARG2 (
    set "ARG2=%~1"
)
goto :parse_args_loop

:done_parsing

:: Update paths based on build configuration
call :update_paths

:: Route to appropriate action
if /i "%COMMAND%"=="build" goto :build
if /i "%COMMAND%"=="clean" goto :clean
if /i "%COMMAND%"=="register" goto :register
if /i "%COMMAND%"=="unregister" goto :unregister
if /i "%COMMAND%"=="test" goto :test
if /i "%COMMAND%"=="setup" goto :setup
if /i "%COMMAND%"=="status" goto :status
if /i "%COMMAND%"=="help" goto :help
goto :invalid_command

:build
:: Build components
set "TARGET=%ARG1%"
if "%TARGET%"=="" set "TARGET=all"

echo Building KeyMagic (%TARGET% - %BUILD_CONFIG%)...
echo.

if /i "%TARGET%"=="all" (
    call :build_tsf
    if !errorlevel! neq 0 exit /b !errorlevel!
    call :build_gui
    if !errorlevel! neq 0 exit /b !errorlevel!
    echo [SUCCESS] All components built
) else if /i "%TARGET%"=="tsf" (
    call :build_tsf
) else if /i "%TARGET%"=="gui" (
    call :build_gui
) else (
    echo [ERROR] Unknown build target: %TARGET%
    echo Valid targets: all, tsf, gui
    exit /b 1
)
goto :eof

:clean
:: Clean build artifacts
set "TARGET=%ARG1%"
if "%TARGET%"=="" set "TARGET=all"

echo Cleaning KeyMagic (%TARGET% - %BUILD_CONFIG%)...
echo.

if /i "%TARGET%"=="all" (
    call :clean_tsf
    call :clean_gui
    echo [SUCCESS] All components cleaned
) else if /i "%TARGET%"=="tsf" (
    call :clean_tsf
) else if /i "%TARGET%"=="gui" (
    call :clean_gui
) else (
    echo [ERROR] Unknown clean target: %TARGET%
    exit /b 1
)
goto :eof

:register
:: Register TSF (requires admin)
call :check_admin || exit /b 1

echo Registering KeyMagic TSF...
if not exist "%TSF_OUTPUT_PATH%" (
    echo [ERROR] TSF DLL not found. Run 'make.bat build tsf' first.
    exit /b 1
)

:: Generate unique temp filename
set "TEMP_NAME=KeyMagicTSF_%RANDOM%_%BUILD_CONFIG%.dll"
set "TEMP_DLL_PATH=%TEMP%\%TEMP_NAME%"

:: Unregister any previously registered temp DLLs
call :unregister_temp_dlls

:: Copy DLL to temp location
echo Copying DLL to temp location: %TEMP_NAME%
copy /Y "%TSF_OUTPUT_PATH%" "%TEMP_DLL_PATH%" >nul
if %errorlevel% neq 0 (
    echo [ERROR] Failed to copy DLL to temp location
    exit /b 1
)

:: Register from temp location
regsvr32 /s "%TEMP_DLL_PATH%"
if %errorlevel% equ 0 (
    echo [SUCCESS] TSF registered successfully from temp location
    :: Store temp DLL path in registry for later cleanup
    reg add "%KEYMAGIC_REG_SETTINGS%" /v "TempDllPath_%BUILD_CONFIG%" /t REG_SZ /d "%TEMP_DLL_PATH%" /f >nul
) else (
    echo [ERROR] Failed to register TSF
    del "%TEMP_DLL_PATH%" 2>nul
    exit /b 1
)
goto :eof

:unregister
:: Unregister TSF (requires admin)
call :check_admin || exit /b 1

echo Unregistering KeyMagic TSF...

:: Unregister temp DLLs
call :unregister_temp_dlls

:: Also try to unregister from original location (legacy)
regsvr32 /s /u "%TSF_OUTPUT_PATH%" 2>nul

echo [SUCCESS] TSF unregistered
goto :eof

:test
:: Run tests
set "TARGET=%ARG1%"
if "%TARGET%"=="" (
    :: Interactive test
    call :test_interactive
) else if /i "%TARGET%"=="status" (
    call :check_status
) else (
    echo [ERROR] Unknown test target: %TARGET%
    exit /b 1
)
goto :eof

:setup
:: Setup development environment
set "TARGET=%ARG1%"
if "%TARGET%"=="" set "TARGET=dev"

if /i "%TARGET%"=="dev" (
    :: Development setup
    call :setup_dev
) else if /i "%TARGET%"=="icons" (
    :: Download icons only
    call :download_icons
) else (
    echo [ERROR] Unknown setup target: %TARGET%
    exit /b 1
)
goto :eof

:status
:: Check system status
call :check_status
goto :eof

:help
:: Display help
echo KeyMagic Windows Build System
echo =============================
echo.
echo Usage: make.bat [command] [options]
echo.
echo Commands:
echo   build [all^|tsf^|gui]     Build components (default: all)
echo   clean [all^|tsf^|gui]     Clean build artifacts
echo   register                 Register TSF with Windows (requires admin)
echo   unregister              Unregister TSF from Windows (requires admin)
echo   test [status]           Run interactive test or check status
echo   setup [dev^|icons]       Setup development environment
echo   status                  Check installation status
echo   help                    Show this help message
echo.
echo Global Options:
echo   --debug                 Build in Debug configuration
echo   --release               Build in Release configuration (default)
echo.
echo Examples:
echo   make.bat build          Build everything in Release mode
echo   make.bat build --debug  Build everything in Debug mode
echo   make.bat build tsf --debug      Build only TSF in Debug mode
echo   make.bat test           Launch test environment
echo   make.bat clean --debug  Clean Debug build artifacts
echo.
goto :eof

:invalid_command
echo [ERROR] Invalid command: %COMMAND%
echo Run 'make.bat help' for usage information
exit /b 1

:: ========== Build Functions ==========

:build_tsf
echo Building TSF (%BUILD_CONFIG%)...
if not exist "%TSF_BUILD_DIR%" mkdir "%TSF_BUILD_DIR%"
cd "%TSF_BUILD_DIR%"
cmake -G "%CMAKE_GENERATOR%" -A ARM64 .. || (cd "%PROJECT_ROOT%" & exit /b 1)
cmake --build . --config %BUILD_CONFIG% || (cd "%PROJECT_ROOT%" & exit /b 1)
cd "%PROJECT_ROOT%"
echo [OK] TSF built successfully (%BUILD_CONFIG%)
exit /b 0

:build_gui
echo Building GUI (%BUILD_CONFIG%)...
if /i "%BUILD_CONFIG%"=="Debug" (
    cargo build -p keymagic-config
) else (
    cargo build --release -p keymagic-config
)
if %errorlevel% neq 0 (
    echo [ERROR] Failed to build GUI
    exit /b 1
)
echo [OK] GUI built successfully (%BUILD_CONFIG%)
exit /b 0

:: ========== Clean Functions ==========

:clean_tsf
echo Cleaning TSF...

:: Unregister temp DLLs first
call :unregister_temp_dlls

if exist "%TSF_BUILD_DIR%" (
    :: Try to unregister from original location too (legacy)
    regsvr32 /s /u "%TSF_OUTPUT_PATH%" 2>nul
    timeout /t 2 /nobreak >nul
    rmdir /s /q "%TSF_BUILD_DIR%"
)
echo [OK] TSF cleaned
exit /b 0

:clean_gui
echo Cleaning GUI...
cargo clean
echo [OK] GUI cleaned
exit /b 0

:: ========== Setup Functions ==========

:setup_dev
echo Setting up development environment...
echo.

:: Create directories
if not exist "%KEYBOARDS_DIR%" mkdir "%KEYBOARDS_DIR%"
if not exist "%ICONS_DIR%" mkdir "%ICONS_DIR%"

:: Download icons if not present
if not exist "%ICONS_DIR%\keymagic.ico" (
    call :download_icons
)

:: Copy test keyboard
if exist "%TEST_KEYBOARD_SOURCE%" (
    copy /Y "%TEST_KEYBOARD_SOURCE%" "%TEST_KEYBOARD_PATH%" >nul
    echo [OK] Test keyboard copied
)

:: Configure registry
call :configure_registry

echo [OK] Development environment ready
exit /b 0

:download_icons
echo Downloading icons...
powershell -ExecutionPolicy Bypass -File "%SCRIPTS_DIR%\tools\download-icons.ps1"
if %errorlevel% equ 0 (
    echo [OK] Icons downloaded
) else (
    echo [WARNING] Failed to download icons
)
exit /b 0

:: ========== Test Functions ==========

:test_interactive
echo Launching test environment...
echo.

:: Check prerequisites
if not exist "%TSF_OUTPUT_PATH%" (
    echo [ERROR] TSF not built. Run 'make.bat build tsf' first.
    exit /b 1
)

if not exist "%GUI_OUTPUT_PATH%" (
    echo [ERROR] GUI not built. Run 'make.bat build gui' first.
    exit /b 1
)

:: Check registration
call :check_tsf_registered
if %errorlevel% neq 0 (
    echo [WARNING] TSF not registered. Run 'make.bat register' as admin.
    echo.
)

:: Launch GUI
echo Starting KeyMagic GUI...
start "" "%GUI_OUTPUT_PATH%"

echo.
echo Test Instructions:
echo -----------------
echo 1. Check system tray for KeyMagic icon
echo 2. Right-click tray icon for menu
echo 3. Open Windows Settings - Time ^& Language
echo 4. Add "KeyMagic Text Service" as input method
echo 5. Test typing in Notepad
echo.
exit /b 0

:: ========== Utility Functions ==========

:check_admin
net session >nul 2>&1
if %errorlevel% neq 0 (
    echo [ERROR] Administrator privileges required!
    echo Please run as administrator.
    exit /b 1
)
exit /b 0

:check_tsf_registered
reg query "%TSF_REG_KEY%" >nul 2>&1
exit /b %errorlevel%

:configure_registry
:: Set up default keyboard configuration
reg add "%KEYMAGIC_REG_SETTINGS%" /v "DefaultKeyboard" /t REG_SZ /d "ZawCode" /f >nul
reg add "%KEYMAGIC_REG_KEYBOARDS%\ZawCode" /v "Path" /t REG_SZ /d "%TEST_KEYBOARD_PATH%" /f >nul
reg add "%KEYMAGIC_REG_KEYBOARDS%\ZawCode" /v "Name" /t REG_SZ /d "ZawCode" /f >nul
reg add "%KEYMAGIC_REG_KEYBOARDS%\ZawCode" /v "Enabled" /t REG_DWORD /d 1 /f >nul
echo [OK] Registry configured
exit /b 0

:check_status
echo KeyMagic Status Check (%BUILD_CONFIG%)
echo ===================================
echo.

:: Check files
echo Files:
if exist "%TSF_OUTPUT_PATH%" (
    echo   [OK] TSF DLL found
) else (
    echo   [--] TSF DLL not found
)

if exist "%GUI_OUTPUT_PATH%" (
    echo   [OK] GUI executable found
) else (
    echo   [--] GUI executable not found
)

echo.
echo Registration:
call :check_tsf_registered
if %errorlevel% equ 0 (
    echo   [OK] TSF registered
) else (
    echo   [--] TSF not registered
)

:: Check processes
echo.
echo Processes:
tasklist | findstr /i ctfmon.exe >nul
if %errorlevel% equ 0 (
    echo   [OK] Text Services Framework running
) else (
    echo   [--] ctfmon.exe not running
)

tasklist | findstr /i keymagic-config.exe >nul
if %errorlevel% equ 0 (
    echo   [OK] KeyMagic GUI running
) else (
    echo   [--] KeyMagic GUI not running
)

echo.
exit /b 0

:: ========== Path Update Function ==========

:update_paths
:: Update paths based on BUILD_CONFIG
if /i "%BUILD_CONFIG%"=="Debug" (
    set "TSF_OUTPUT_PATH=%TSF_BUILD_DIR%\Debug\KeyMagicTSF.dll"
    set "GUI_OUTPUT_PATH=%PROJECT_ROOT%\target\debug\keymagic-config.exe"
) else (
    set "TSF_OUTPUT_PATH=%TSF_BUILD_DIR%\Release\KeyMagicTSF.dll"
    set "GUI_OUTPUT_PATH=%PROJECT_ROOT%\target\release\keymagic-config.exe"
)
exit /b 0

:: ========== Temp DLL Management ==========

:unregister_temp_dlls
:: Unregister any previously registered temp DLLs
for %%C in (Debug Release) do (
    :: Read temp DLL path from registry
    for /f "tokens=2*" %%a in ('reg query "%KEYMAGIC_REG_SETTINGS%" /v "TempDllPath_%%C" 2^>nul ^| findstr "TempDllPath_%%C"') do (
        set "OLD_TEMP_DLL=%%b"
        if exist "!OLD_TEMP_DLL!" (
            echo Unregistering old temp DLL: !OLD_TEMP_DLL!
            regsvr32 /s /u "!OLD_TEMP_DLL!" 2>nul
            timeout /t 1 /nobreak >nul
            del "!OLD_TEMP_DLL!" 2>nul
        )
        :: Remove registry entry
        reg delete "%KEYMAGIC_REG_SETTINGS%" /v "TempDllPath_%%C" /f >nul 2>&1
    )
)

:: Also clean up any orphaned KeyMagicTSF_*.dll files in temp
echo Cleaning up orphaned temp DLLs...
for %%f in ("%TEMP%\KeyMagicTSF_*.dll") do (
    regsvr32 /s /u "%%f" 2>nul
    timeout /t 1 /nobreak >nul
    del "%%f" 2>nul
)
exit /b 0

endlocal