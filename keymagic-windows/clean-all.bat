@echo off
:: clean-all.bat - Clean all KeyMagic build artifacts
:: This script cleans all possible build configurations

echo ========================================
echo KeyMagic Clean All Build Artifacts
echo ========================================
echo.

:: Counter for cleaning operations
set /a cleaned=0

:: Clean all combinations of architecture and configuration
for %%A in (x64 arm64) do (
    for %%C in (Debug Release) do (
        echo Cleaning %%A %%C...
        call "%~dp0make.bat" clean %%A %%C
        if !errorlevel! equ 0 (
            set /a cleaned+=1
        )
        echo.
    )
)

echo ========================================
echo Clean operations completed: %cleaned%/4
echo ========================================

exit /b 0