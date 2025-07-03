@echo off
REM Build script for KeyMagic Manager GUI

echo Building KeyMagic Manager...
echo.

REM Check for cl.exe
where cl >nul 2>nul
if %errorlevel% neq 0 (
    echo ERROR: Visual Studio compiler not found.
    echo Please run this from Visual Studio Developer Command Prompt.
    pause
    exit /b 1
)

REM Compile
cl /W3 /EHsc /Fe:KeyMagicManager.exe KeyMagicManager.cpp user32.lib gdi32.lib comctl32.lib shlwapi.lib shell32.lib advapi32.lib

if %errorlevel% neq 0 (
    echo.
    echo ERROR: Build failed!
    pause
    exit /b 1
)

echo.
echo Build successful!
echo Run KeyMagicManager.exe to manage keyboard layouts.
echo.

pause