@echo off
REM Build script for KeyMagic Manager GUI using VS Build Tools

echo Building KeyMagic Manager...
echo.

REM Try to find VS installation
set "VS_PATH="
if exist "C:\Program Files\Microsoft Visual Studio\2022\Community\Common7\Tools\VsDevCmd.bat" (
    set "VS_PATH=C:\Program Files\Microsoft Visual Studio\2022\Community\Common7\Tools\VsDevCmd.bat"
    goto found_vs
)
if exist "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\Common7\Tools\VsDevCmd.bat" (
    set "VS_PATH=C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\Common7\Tools\VsDevCmd.bat"
    goto found_vs
)

echo ERROR: Visual Studio not found!
echo Please install Visual Studio or Build Tools for Visual Studio.
pause
exit /b 1

:found_vs
REM Initialize VS environment
call "%VS_PATH%" -arch=arm64

REM Compile
cl /W3 /EHsc /D_UNICODE /DUNICODE /Fe:KeyMagicManager.exe KeyMagicManager.cpp ^
   user32.lib gdi32.lib comctl32.lib shlwapi.lib shell32.lib advapi32.lib comdlg32.lib

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