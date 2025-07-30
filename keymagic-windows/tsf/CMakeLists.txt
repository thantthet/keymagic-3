cmake_minimum_required(VERSION 3.20)
project(KeyMagicTSF)

# Set C++ standard
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Define Unicode
add_definitions(-DUNICODE -D_UNICODE)

# TSF DLL sources
set(SOURCES
    src/DllMain.cpp
    src/ClassFactory.cpp
    src/KeyMagicTextService.cpp
    src/DirectEditSession.cpp
    src/CompositionEditSession.cpp
    src/Composition.cpp
    src/DisplayAttribute.cpp
    src/Globals.cpp
    src/Registry.cpp
    src/ProcessDetector.cpp
    src/KeyProcessingUtils.cpp
    src/LanguageUtils.cpp
    src/HUD.cpp
    src/TrayClient.cpp
    src/KeyMagicTSF.rc
)

# Create the DLL
add_library(KeyMagicTSF SHARED ${SOURCES})

# Set output name based on architecture for ARM64X support
if(CMAKE_GENERATOR_PLATFORM STREQUAL "x64" OR CMAKE_GENERATOR_PLATFORM STREQUAL "X64")
    set_target_properties(KeyMagicTSF PROPERTIES OUTPUT_NAME "KeyMagicTSF_x64")
elseif(CMAKE_GENERATOR_PLATFORM STREQUAL "ARM64")
    set_target_properties(KeyMagicTSF PROPERTIES OUTPUT_NAME "KeyMagicTSF_arm64")
endif()

# Include directories
target_include_directories(KeyMagicTSF PRIVATE 
    ${CMAKE_CURRENT_SOURCE_DIR}/include
)

# Link libraries
target_link_libraries(KeyMagicTSF PRIVATE
    uuid
    ole32
    oleaut32
    advapi32
    user32
    kernel32
    ws2_32      # Windows Sockets
    userenv     # User environment
    ntdll       # NT functions
    gdi32       # Graphics Device Interface for HUD
    shlwapi     # Path functions
    psapi       # Process information
)

# Determine Rust target directory based on architecture
if(DEFINED RUST_TARGET)
    set(RUST_TARGET_DIR "${RUST_TARGET}")
elseif(CMAKE_GENERATOR_PLATFORM STREQUAL "x64" OR CMAKE_GENERATOR_PLATFORM STREQUAL "X64")
    set(RUST_TARGET_DIR "x86_64-pc-windows-msvc")
elseif(CMAKE_GENERATOR_PLATFORM STREQUAL "ARM64")
    set(RUST_TARGET_DIR "aarch64-pc-windows-msvc")
else()
    # Default to native compilation
    set(RUST_TARGET_DIR "")
endif()

# Link to Rust keymagic-core static library based on configuration
# Using relative paths from the CMake source directory
if(RUST_TARGET_DIR)
    # Cross-compilation - use target-specific directory
    target_link_libraries(KeyMagicTSF PRIVATE
        debug "${CMAKE_CURRENT_SOURCE_DIR}/../../target/${RUST_TARGET_DIR}/debug/keymagic_core.lib"
        optimized "${CMAKE_CURRENT_SOURCE_DIR}/../../target/${RUST_TARGET_DIR}/release/keymagic_core.lib"
    )
else()
    # Native compilation - use default directory
    target_link_libraries(KeyMagicTSF PRIVATE
        debug "${CMAKE_CURRENT_SOURCE_DIR}/../../target/debug/keymagic_core.lib"
        optimized "${CMAKE_CURRENT_SOURCE_DIR}/../../target/release/keymagic_core.lib"
    )
endif()

# Set static runtime
set_property(TARGET KeyMagicTSF PROPERTY
    MSVC_RUNTIME_LIBRARY "MultiThreaded$<$<CONFIG:Debug>:Debug>")

# Module definition file
set_target_properties(KeyMagicTSF PROPERTIES
    LINK_FLAGS "/DEF:${CMAKE_CURRENT_SOURCE_DIR}/src/KeyMagicTSF.def"
)

# Copy DLL to output directory based on configuration
# Using generator expressions for multi-config generators
if(RUST_TARGET_DIR)
    # Cross-compilation - copy to target-specific directory
    add_custom_command(TARGET KeyMagicTSF POST_BUILD
        COMMAND ${CMAKE_COMMAND} -E make_directory 
            "$<IF:$<CONFIG:Debug>,${CMAKE_CURRENT_BINARY_DIR}/../../target/${RUST_TARGET_DIR}/debug/,${CMAKE_CURRENT_BINARY_DIR}/../../target/${RUST_TARGET_DIR}/release/>"
        COMMAND ${CMAKE_COMMAND} -E copy_if_different
            $<TARGET_FILE:KeyMagicTSF>
            "$<IF:$<CONFIG:Debug>,${CMAKE_CURRENT_BINARY_DIR}/../../target/${RUST_TARGET_DIR}/debug/,${CMAKE_CURRENT_BINARY_DIR}/../../target/${RUST_TARGET_DIR}/release/>"
    )
else()
    # Native compilation - copy to default directory
    add_custom_command(TARGET KeyMagicTSF POST_BUILD
        COMMAND ${CMAKE_COMMAND} -E make_directory 
            "$<IF:$<CONFIG:Debug>,${CMAKE_CURRENT_BINARY_DIR}/../../target/debug/,${CMAKE_CURRENT_BINARY_DIR}/../../target/release/>"
        COMMAND ${CMAKE_COMMAND} -E copy_if_different
            $<TARGET_FILE:KeyMagicTSF>
            "$<IF:$<CONFIG:Debug>,${CMAKE_CURRENT_BINARY_DIR}/../../target/debug/,${CMAKE_CURRENT_BINARY_DIR}/../../target/release/>"
    )
endif()