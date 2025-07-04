@echo off
REM Script to build the library and run FFI tests on Windows

echo === Building KeyMagic Core library ===
cd /d "%~dp0\..\.."

REM Build in release mode
cargo build --release -p keymagic-core
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