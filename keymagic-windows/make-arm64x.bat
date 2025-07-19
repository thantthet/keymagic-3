@echo off
:: make-arm64x.bat - Build ARM64X forwarder DLL for KeyMagic TSF
:: This script builds both x64 and ARM64 versions, then creates an ARM64X forwarder DLL

setlocal enabledelayedexpansion

:: Navigate to script directory
cd /d "%~dp0"

:: Parse configuration (default: Release)
set "CONFIG=%~1"
if "%CONFIG%"=="" set "CONFIG=Release"

if /i not "%CONFIG%"=="Debug" if /i not "%CONFIG%"=="Release" (
    echo [ERROR] Invalid config: %CONFIG%
    echo Must be Debug or Release
    exit /b 1
)

echo ========================================
echo Building ARM64X Forwarder for KeyMagic
echo Configuration: %CONFIG%
echo ========================================
echo.

:: Step 1: Build Rust libraries for both architectures
echo [1/6] Building Rust libraries...
echo.

echo Building Rust for x64...
if /i "%CONFIG%"=="Release" (
    cargo build -p keymagic-core --release --target x86_64-pc-windows-msvc || exit /b 1
) else (
    cargo build -p keymagic-core --target x86_64-pc-windows-msvc || exit /b 1
)

echo Building Rust for ARM64...
if /i "%CONFIG%"=="Release" (
    cargo build -p keymagic-core --release --target aarch64-pc-windows-msvc || exit /b 1
) else (
    cargo build -p keymagic-core --target aarch64-pc-windows-msvc || exit /b 1
)

:: Step 2: Build TSF DLL for x64
echo.
echo [2/6] Building TSF for x64...
cd tsf
if not exist "build-x64" mkdir "build-x64"
cd "build-x64"
cmake -G "Visual Studio 17 2022" -A x64 .. || exit /b 1
cmake --build . --config %CONFIG% || exit /b 1
cd ..\..

:: Step 3: Build TSF DLL for ARM64
echo.
echo [3/6] Building TSF for ARM64...
cd tsf
if not exist "build-arm64" mkdir "build-arm64"
cd "build-arm64"
cmake -G "Visual Studio 17 2022" -A ARM64 .. || exit /b 1
cmake --build . --config %CONFIG% || exit /b 1
cd ..\..

:: Step 4: Create empty object files for forwarder
echo.
echo [4/6] Creating empty object files...
cd tsf

:: Set up Visual Studio environment if not already set
where cl.exe >nul 2>&1
if %errorlevel% neq 0 (
    echo Setting up Visual Studio environment...
    
    :: Determine VS path and architecture-specific script
    if /i "%PROCESSOR_ARCHITECTURE%"=="ARM64" (
        echo Detected ARM64 system, using native ARM64 compiler...
        call "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvarsarm64.bat"
    ) else if /i "%PROCESSOR_ARCHITECTURE%"=="AMD64" (
        echo Detected x64 system, using x64 to ARM64 cross compiler...
        call "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvarsamd64_arm64.bat"
    ) else (
        echo [ERROR] Unsupported processor architecture: %PROCESSOR_ARCHITECTURE%
        exit /b 1
    )
)

:: Create build directory for ARM64X
if not exist "build-arm64x" mkdir "build-arm64x"

:: Compile empty objects
cl /c /Fobuild-arm64x\empty_arm64.obj src\empty.cpp || exit /b 1
cl /c /arm64EC /Fobuild-arm64x\empty_x64.obj src\empty.cpp || exit /b 1

:: Step 5: Create import libraries from DEF files
echo.
echo [5/6] Creating import libraries...

link /lib /machine:arm64ec /def:src\KeyMagicTSF_x64.def /out:build-arm64x\KeyMagicTSF_x64.lib || exit /b 1
link /lib /machine:arm64 /def:src\KeyMagicTSF_arm64.def /out:build-arm64x\KeyMagicTSF_arm64.lib || exit /b 1

:: Step 6: Link the ARM64X forwarder DLL
echo.
echo [6/6] Creating ARM64X forwarder DLL...

link /dll /noentry /machine:arm64x ^
    /defArm64Native:src\KeyMagicTSF_arm64.def /def:src\KeyMagicTSF_x64.def ^
    build-arm64x\empty_arm64.obj build-arm64x\empty_x64.obj ^
    /out:build-arm64x\KeyMagicTSF.dll ^
    build-arm64x\KeyMagicTSF_arm64.lib build-arm64x\KeyMagicTSF_x64.lib || exit /b 1

:: Copy the architecture-specific DLLs to the ARM64X directory
echo.
echo Copying architecture-specific DLLs...
copy /Y "build-x64\%CONFIG%\KeyMagicTSF_x64.dll" "build-arm64x\" >nul
copy /Y "build-arm64\%CONFIG%\KeyMagicTSF_arm64.dll" "build-arm64x\" >nul

:: Copy to target directory
echo.
echo Copying to target directory...
if /i "%CONFIG%"=="Release" (
    set "TARGET_DIR=..\target\release"
) else (
    set "TARGET_DIR=..\target\debug"
)

if not exist "%TARGET_DIR%" mkdir "%TARGET_DIR%"
copy /Y "build-arm64x\KeyMagicTSF.dll" "%TARGET_DIR%\" >nul
copy /Y "build-arm64x\KeyMagicTSF_x64.dll" "%TARGET_DIR%\" >nul
copy /Y "build-arm64x\KeyMagicTSF_arm64.dll" "%TARGET_DIR%\" >nul

cd ..

echo.
echo ========================================
echo [SUCCESS] ARM64X build complete!
echo ========================================
echo.
echo Output files in tsf\build-arm64x:
echo   - KeyMagicTSF.dll (ARM64X forwarder)
echo   - KeyMagicTSF_x64.dll (x64 implementation)
echo   - KeyMagicTSF_arm64.dll (ARM64 implementation)
echo.
echo Files also copied to: %TARGET_DIR%
echo.
echo To register the TSF:
echo   1. Run as Administrator
echo   2. regsvr32 tsf\build-arm64x\KeyMagicTSF.dll
echo.

exit /b 0