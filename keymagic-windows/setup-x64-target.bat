@echo off
echo Installing x64 target for Rust...
echo.

rustup target add x86_64-pc-windows-msvc

if %errorlevel% equ 0 (
    echo.
    echo [SUCCESS] x64 target installed successfully!
    echo.
    echo You can now build for x64 using:
    echo   make.bat build --x64
    echo.
    echo Or use the cross-compilation script:
    echo   build-cross.bat x64 release
) else (
    echo.
    echo [ERROR] Failed to install x64 target
    echo Make sure Rust is installed and rustup is in PATH
)

pause