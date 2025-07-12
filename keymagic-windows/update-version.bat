@echo off
REM Update version across all KeyMagic Windows components

setlocal enabledelayedexpansion

REM Get the directory where this script is located
set SCRIPT_DIR=%~dp0

REM Try different PowerShell locations
if exist "%SystemRoot%\System32\WindowsPowerShell\v1.0\powershell.exe" (
    set PWSH="%SystemRoot%\System32\WindowsPowerShell\v1.0\powershell.exe"
    goto :run_script
)

if exist "%SystemRoot%\SysWOW64\WindowsPowerShell\v1.0\powershell.exe" (
    set PWSH="%SystemRoot%\SysWOW64\WindowsPowerShell\v1.0\powershell.exe"
    goto :run_script
)

REM Try PowerShell Core
if exist "%ProgramFiles%\PowerShell\7\pwsh.exe" (
    set PWSH="%ProgramFiles%\PowerShell\7\pwsh.exe"
    goto :run_script
)

REM Last resort - try without full path
where powershell >nul 2>nul
if %errorlevel% equ 0 (
    set PWSH=powershell
    goto :run_script
)

echo ERROR: PowerShell not found in standard locations
echo Please ensure PowerShell is installed
exit /b 1

:run_script
REM Run the PowerShell script
if "%~1"=="" (
    echo Updating version from version.txt...
    %PWSH% -ExecutionPolicy Bypass -File "%SCRIPT_DIR%scripts\update-version.ps1"
) else (
    echo Updating version to: %1
    %PWSH% -ExecutionPolicy Bypass -File "%SCRIPT_DIR%scripts\update-version.ps1" -NewVersion %1
)

if %errorlevel% neq 0 (
    echo ERROR: Version update failed
    exit /b 1
)

exit /b 0