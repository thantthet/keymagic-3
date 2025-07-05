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
    src/Composition.cpp
    src/Globals.cpp
    src/Registry.cpp
    src/KeyMagicTSF.rc
)

# Create the DLL
add_library(KeyMagicTSF SHARED ${SOURCES})

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
)

# Link to Rust keymagic-core static library based on configuration
# Using relative paths from the CMake source directory
target_link_libraries(KeyMagicTSF PRIVATE
    debug "${CMAKE_CURRENT_SOURCE_DIR}/../../target/debug/keymagic_core.lib"
    optimized "${CMAKE_CURRENT_SOURCE_DIR}/../../target/release/keymagic_core.lib"
)

# Module definition file
set_target_properties(KeyMagicTSF PROPERTIES
    LINK_FLAGS "/DEF:${CMAKE_CURRENT_SOURCE_DIR}/src/KeyMagicTSF.def"
)

# Copy DLL to output directory based on configuration
# Using generator expressions for multi-config generators
add_custom_command(TARGET KeyMagicTSF POST_BUILD
    COMMAND ${CMAKE_COMMAND} -E make_directory 
        "$<IF:$<CONFIG:Debug>,${CMAKE_CURRENT_BINARY_DIR}/../../target/debug/,${CMAKE_CURRENT_BINARY_DIR}/../../target/release/>"
    COMMAND ${CMAKE_COMMAND} -E copy_if_different
        $<TARGET_FILE:KeyMagicTSF>
        "$<IF:$<CONFIG:Debug>,${CMAKE_CURRENT_BINARY_DIR}/../../target/debug/,${CMAKE_CURRENT_BINARY_DIR}/../../target/release/>"
)