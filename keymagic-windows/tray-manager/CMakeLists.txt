cmake_minimum_required(VERSION 3.16)
project(KeyMagicTray VERSION 1.0.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Set output directories
set(CMAKE_RUNTIME_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/bin)
set(CMAKE_LIBRARY_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/lib)
set(CMAKE_ARCHIVE_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/lib)

# Define sources
set(SOURCES
    src/main.cpp
    src/TrayManager.cpp
    src/NamedPipeServer.cpp
    src/RegistryMonitor.cpp
    src/IconCacheManager.cpp
    src/ProfileMonitor.cpp
    src/TrayIcon.cpp
    src/SecurityUtils.cpp
)

set(HEADERS
    include/TrayManager.h
    include/NamedPipeServer.h
    include/RegistryMonitor.h
    include/IconCacheManager.h
    include/ProfileMonitor.h
    include/TrayIcon.h
    include/TrayProtocol.h
    include/SecurityUtils.h
    include/Common.h
)

# Create executable
add_executable(keymagic-tray WIN32 ${SOURCES} ${HEADERS})

# Include directories
target_include_directories(keymagic-tray PRIVATE 
    ${CMAKE_CURRENT_SOURCE_DIR}/include
    ${CMAKE_CURRENT_SOURCE_DIR}/../tsf/include
)

# Windows specific settings
target_compile_definitions(keymagic-tray PRIVATE
    UNICODE
    _UNICODE
    WIN32_LEAN_AND_MEAN
    NOMINMAX
)

# Link libraries
target_link_libraries(keymagic-tray PRIVATE
    kernel32
    user32
    shell32
    advapi32
    ole32
    oleaut32
    uuid
    comdlg32
    gdi32
    shlwapi
    gdiplus
    psapi
    ws2_32
    userenv
    ntdll
    comctl32
)

# Determine Rust target based on architecture
if(CMAKE_GENERATOR_PLATFORM STREQUAL "ARM64")
    set(RUST_TARGET "aarch64-pc-windows-msvc")
elseif(CMAKE_GENERATOR_PLATFORM STREQUAL "x64")
    set(RUST_TARGET "x86_64-pc-windows-msvc")
else()
    message(FATAL_ERROR "Unsupported architecture: ${CMAKE_GENERATOR_PLATFORM}")
endif()

# Link keymagic-core based on configuration
if(CMAKE_BUILD_TYPE STREQUAL "Debug")
    target_link_libraries(keymagic-tray PRIVATE
        ${CMAKE_CURRENT_SOURCE_DIR}/../../target/${RUST_TARGET}/debug/keymagic_core.lib
    )
else()
    target_link_libraries(keymagic-tray PRIVATE
        ${CMAKE_CURRENT_SOURCE_DIR}/../../target/${RUST_TARGET}/release/keymagic_core.lib
    )
endif()

# Set static runtime
set_property(TARGET keymagic-tray PROPERTY
    MSVC_RUNTIME_LIBRARY "MultiThreaded$<$<CONFIG:Debug>:Debug>")

# Add manifest for high DPI support
if(MSVC)
    target_sources(keymagic-tray PRIVATE manifest.xml)
endif()

# Installation
install(TARGETS keymagic-tray
    RUNTIME DESTINATION bin
)