@echo off
:: KeyMagic Windows Build Configuration
:: ====================================

:: Project paths
set "PROJECT_ROOT=%~dp0.."
set "TSF_DIR=%PROJECT_ROOT%\tsf"
set "TSF_BUILD_DIR=%TSF_DIR%\build"

:: Build configuration dependent paths (will be set after BUILD_CONFIG is determined)
:: These are updated by make.bat based on BUILD_CONFIG
set "TSF_OUTPUT_PATH=%TSF_BUILD_DIR%\Release\KeyMagicTSF.dll"
set "GUI_OUTPUT_PATH=%PROJECT_ROOT%\target\release\keymagic-config.exe"

:: Resource paths
set "SCRIPTS_DIR=%PROJECT_ROOT%\scripts"
set "ICONS_DIR=%PROJECT_ROOT%\resources\icons"
set "KEYBOARDS_DIR=%PROJECT_ROOT%\keyboards"

:: Test configuration
set "TEST_KEYBOARD_SOURCE=%PROJECT_ROOT%\..\ZawCode.km2"
set "TEST_KEYBOARD_PATH=%KEYBOARDS_DIR%\ZawCode.km2"

:: Build configuration
set "CMAKE_GENERATOR=Visual Studio 17 2022"

:: Registry keys
set "TSF_REG_KEY=HKEY_CLASSES_ROOT\CLSID\{12345678-1234-1234-1234-123456789ABC}"
set "KEYMAGIC_REG_ROOT=HKEY_CURRENT_USER\Software\KeyMagic"
set "KEYMAGIC_REG_SETTINGS=%KEYMAGIC_REG_ROOT%\Settings"
set "KEYMAGIC_REG_KEYBOARDS=%KEYMAGIC_REG_ROOT%\Keyboards"

:: Return to original directory
cd "%~dp0.."