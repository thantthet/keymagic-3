@echo off
:: make-arm64x.bat - Build native ARM64X DLL for KeyMagic TSF
:: This script builds both ARM64EC and ARM64 versions, then links them into a single ARM64X DLL

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
echo Building Native ARM64X DLL for KeyMagic
echo Configuration: %CONFIG%
echo ========================================
echo.

:: Step 1: Build Rust libraries for both architectures
echo [1/5] Building Rust libraries...
echo.

:: Build for ARM64EC target (Rust supports this as tier 2 target)
:: Note: Due to linking issues with cdylib on ARM64EC, we build as staticlib
echo Building Rust for ARM64EC (as static library)...
if /i "%CONFIG%"=="Release" (
    cargo rustc -p keymagic-core --release --target arm64ec-pc-windows-msvc --crate-type staticlib || (
        echo.
        echo [INFO] ARM64EC target may not be installed. Trying to add it...
        rustup target add arm64ec-pc-windows-msvc
        cargo rustc -p keymagic-core --release --target arm64ec-pc-windows-msvc --crate-type staticlib || exit /b 1
    )
) else (
    cargo rustc -p keymagic-core --target arm64ec-pc-windows-msvc --crate-type staticlib || (
        echo.
        echo [INFO] ARM64EC target may not be installed. Trying to add it...
        rustup target add arm64ec-pc-windows-msvc
        cargo rustc -p keymagic-core --target arm64ec-pc-windows-msvc --crate-type staticlib || exit /b 1
    )
)

echo Building Rust for ARM64 (as static library)...
if /i "%CONFIG%"=="Release" (
    cargo rustc -p keymagic-core --release --target aarch64-pc-windows-msvc --crate-type staticlib || exit /b 1
) else (
    cargo rustc -p keymagic-core --target aarch64-pc-windows-msvc --crate-type staticlib || exit /b 1
)

:: Step 2: Set up Visual Studio environment
echo.
echo [2/5] Setting up Visual Studio environment...
cd tsf

:: Determine VS path and architecture-specific script
if /i "%PROCESSOR_ARCHITECTURE%"=="ARM64" (
    echo Detected ARM64 system, using native ARM64 compiler...
    if exist "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvarsarm64.bat" (
        call "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvarsarm64.bat"
    ) else if exist "C:\Program Files\Microsoft Visual Studio\2022\Enterprise\VC\Auxiliary\Build\vcvarsarm64.bat" (
        call "C:\Program Files\Microsoft Visual Studio\2022\Enterprise\VC\Auxiliary\Build\vcvarsarm64.bat"
    ) else (
        echo [ERROR] Could not find Visual Studio 2022 ARM64 environment script
        exit /b 1
    )
) else if /i "%PROCESSOR_ARCHITECTURE%"=="AMD64" (
    echo Detected x64 system, using x64 to ARM64 cross compiler...
    if exist "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvarsamd64_arm64.bat" (
        call "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvarsamd64_arm64.bat"
    ) else if exist "C:\Program Files\Microsoft Visual Studio\2022\Enterprise\VC\Auxiliary\Build\vcvarsamd64_arm64.bat" (
        call "C:\Program Files\Microsoft Visual Studio\2022\Enterprise\VC\Auxiliary\Build\vcvarsamd64_arm64.bat"
    ) else (
        echo [ERROR] Could not find Visual Studio 2022 x64 to ARM64 cross-compiler environment script
        exit /b 1
    )
) else (
    echo [ERROR] Unsupported processor architecture: %PROCESSOR_ARCHITECTURE%
    exit /b 1
)

:: Verify correct environment is set
where cl.exe >nul 2>&1
if %errorlevel% neq 0 (
    echo [ERROR] Failed to set up Visual Studio environment
    exit /b 1
)

:: Create build directory for ARM64X
if not exist "build-arm64x" mkdir "build-arm64x"
if not exist "build-arm64x\arm64" mkdir "build-arm64x\arm64"
if not exist "build-arm64x\arm64ec" mkdir "build-arm64x\arm64ec"

:: Step 3: Compile all source files for ARM64
echo.
echo [3/5] Building ARM64 native objects...

:: Set compiler flags for ARM64
set "ARM64_FLAGS=/c /MT /O2 /DUNICODE /D_UNICODE /std:c++17"
if /i "%CONFIG%"=="Debug" set "ARM64_FLAGS=/c /MTd /Od /Zi /DUNICODE /D_UNICODE /std:c++17"

:: Include paths
set "INCLUDES=/I include /I ..\shared\include"

:: Compile each source file for ARM64
for %%f in (
    DllMain.cpp
    ClassFactory.cpp
    KeyMagicTextService.cpp
    DirectEditSession.cpp
    CompositionEditSession.cpp
    Composition.cpp
    DisplayAttribute.cpp
    Globals.cpp
    Registry.cpp
    ProcessDetector.cpp
    KeyProcessingUtils.cpp
    LanguageUtils.cpp
    HUD.cpp
    TrayClient.cpp
) do (
    echo Compiling %%f for ARM64...
    cl %ARM64_FLAGS% %INCLUDES% /Fobuild-arm64x\arm64\%%~nf.obj src\%%f || exit /b 1
)

:: Compile resource file for ARM64
echo Compiling resources for ARM64...
rc /fo build-arm64x\arm64\KeyMagicTSF.res src\KeyMagicTSF.rc || exit /b 1

:: Step 4: Compile all source files for ARM64EC
echo.
echo [4/5] Building ARM64EC objects...

:: Set compiler flags for ARM64EC
set "ARM64EC_FLAGS=/c /arm64EC /MT /O2 /DUNICODE /D_UNICODE /std:c++17"
if /i "%CONFIG%"=="Debug" set "ARM64EC_FLAGS=/c /arm64EC /MTd /Od /Zi /DUNICODE /D_UNICODE /std:c++17"

:: Compile each source file for ARM64EC
for %%f in (
    DllMain.cpp
    ClassFactory.cpp
    KeyMagicTextService.cpp
    DirectEditSession.cpp
    CompositionEditSession.cpp
    Composition.cpp
    DisplayAttribute.cpp
    Globals.cpp
    Registry.cpp
    ProcessDetector.cpp
    KeyProcessingUtils.cpp
    LanguageUtils.cpp
    HUD.cpp
    TrayClient.cpp
) do (
    echo Compiling %%f for ARM64EC...
    cl %ARM64EC_FLAGS% %INCLUDES% /Fobuild-arm64x\arm64ec\%%~nf.obj src\%%f || exit /b 1
)

:: Compile resource file for ARM64EC
echo Compiling resources for ARM64EC...
rc /fo build-arm64x\arm64ec\KeyMagicTSF.res src\KeyMagicTSF.rc || exit /b 1

:: Step 5: Link the native ARM64X DLL
echo.
echo [5/5] Linking native ARM64X DLL...

:: Set Rust library paths (static libraries on Windows use .lib)
if /i "%CONFIG%"=="Release" (
    set "RUST_ARM64_LIB=..\..\target\aarch64-pc-windows-msvc\release\keymagic_core.lib"
    set "RUST_ARM64EC_LIB=..\..\target\arm64ec-pc-windows-msvc\release\keymagic_core.lib"
) else (
    set "RUST_ARM64_LIB=..\..\target\aarch64-pc-windows-msvc\debug\keymagic_core.lib"
    set "RUST_ARM64EC_LIB=..\..\target\arm64ec-pc-windows-msvc\debug\keymagic_core.lib"
)

:: System libraries
set "SYS_LIBS=uuid.lib ole32.lib oleaut32.lib advapi32.lib user32.lib kernel32.lib ws2_32.lib userenv.lib ntdll.lib gdi32.lib shlwapi.lib psapi.lib shell32.lib"

:: Link ARM64 objects first
echo Linking ARM64 native code...
link /dll /machine:arm64 /def:src\KeyMagicTSF.def ^
    /out:build-arm64x\KeyMagicTSF_arm64.dll ^
    build-arm64x\arm64\*.obj ^
    build-arm64x\arm64\KeyMagicTSF.res ^
    %RUST_ARM64_LIB% ^
    %SYS_LIBS% || exit /b 1

:: Link ARM64EC objects
echo Linking ARM64EC code...
link /dll /machine:arm64ec /def:src\KeyMagicTSF.def ^
    /out:build-arm64x\KeyMagicTSF_arm64ec.dll ^
    build-arm64x\arm64ec\*.obj ^
    build-arm64x\arm64ec\KeyMagicTSF.res ^
    %RUST_ARM64EC_LIB% ^
    %SYS_LIBS% || exit /b 1

:: Create the final ARM64X DLL by merging both architectures
echo Creating native ARM64X DLL...
link /dll /machine:arm64x ^
    /defArm64Native:src\KeyMagicTSF.def ^
    /def:src\KeyMagicTSF.def ^
    build-arm64x\arm64\*.obj ^
    build-arm64x\arm64ec\*.obj ^
    build-arm64x\arm64\KeyMagicTSF.res ^
    /out:build-arm64x\KeyMagicTSF.dll ^
    %RUST_ARM64_LIB% ^
    %RUST_ARM64EC_LIB% ^
    %SYS_LIBS% || exit /b 1

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

cd ..

echo.
echo ========================================
echo [SUCCESS] Native ARM64X build complete!
echo ========================================
echo.
echo Output file: tsf\build-arm64x\KeyMagicTSF.dll
echo.
echo This is a native ARM64X DLL that contains both:
echo   - ARM64 native code (for ARM64 processes)
echo   - ARM64EC code (for x64/AMD64 processes on ARM64)
echo.
echo File also copied to: %TARGET_DIR%
echo.
echo To register the TSF:
echo   1. Run as Administrator
echo   2. regsvr32 %TARGET_DIR%\KeyMagicTSF.dll
echo.

exit /b 0