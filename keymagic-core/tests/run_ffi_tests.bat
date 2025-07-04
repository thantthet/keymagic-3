@echo off
REM Script to build the library and run FFI tests on Windows

echo === Building KeyMagic Core library ===
cd /d "%~dp0\..\.."

REM Detect Python architecture and build accordingly
for /f "tokens=*" %%i in ('python -c "import sys; print(\"x64\" if \"AMD64\" in sys.version else \"arm64\")"') do set PYARCH=%%i

if "%PYARCH%"=="x64" (
    echo Building for x64 architecture...
    cargo build --release -p keymagic-core --target x86_64-pc-windows-msvc
) else (
    echo Building for ARM64 architecture...
    cargo build --release -p keymagic-core --target aarch64-pc-windows-msvc
)

if %errorlevel% neq 0 (
    echo Build failed!
    exit /b %errorlevel%
)

echo.
echo === Running Python FFI tests ===
python keymagic-core\tests\test_ffi.py
if %errorlevel% neq 0 (
    echo Some tests failed!
    exit /b %errorlevel%
)

echo.
echo === All tests passed! ===
pause