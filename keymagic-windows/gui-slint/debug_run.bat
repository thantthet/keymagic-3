@echo off
echo Starting KeyMagic GUI in debug mode...
echo.

REM Set environment variables for debugging
set RUST_BACKTRACE=full
set RUST_LOG=debug
set SLINT_DEBUG_PERFORMANCE=1

REM Force software rendering if having GPU issues
REM set SLINT_BACKEND=software

REM Create log directory
if not exist "%LOCALAPPDATA%\KeyMagic\logs" mkdir "%LOCALAPPDATA%\KeyMagic\logs"

echo Environment variables set:
echo   RUST_BACKTRACE=full
echo   RUST_LOG=debug
echo   SLINT_DEBUG_PERFORMANCE=1
echo.
echo Logs will be saved to: %LOCALAPPDATA%\KeyMagic\logs
echo Crash reports will be saved to: %LOCALAPPDATA%\KeyMagic\crashes
echo.

REM Run the application
..\target\release\keymagic-config-slint.exe

REM Check exit code
if %ERRORLEVEL% NEQ 0 (
    echo.
    echo Application exited with error code: %ERRORLEVEL%
    echo Check the logs in %LOCALAPPDATA%\KeyMagic\logs
    echo.
    pause
)