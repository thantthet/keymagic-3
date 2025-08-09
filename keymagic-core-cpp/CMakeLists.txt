cmake_minimum_required(VERSION 3.20)

# Project configuration
project(KeyMagicCore 
    VERSION 1.0.0
    DESCRIPTION "KeyMagic Core Engine C++ Implementation"
    LANGUAGES CXX
)

# C++ Standard
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

# Build options
option(BUILD_SHARED_LIBS "Build shared libraries" OFF)
option(BUILD_TESTING "Build test suite" ON)
option(ENABLE_ASAN "Enable Address Sanitizer" OFF)
option(ENABLE_UBSAN "Enable Undefined Behavior Sanitizer" OFF)

# Platform detection
if(WIN32)
    add_compile_definitions(KEYMAGIC_WINDOWS)
    
    # Windows-specific settings
    add_compile_options(/utf-8)  # UTF-8 source files
    add_compile_options(/W4)     # Warning level 4
    
    # Disable specific warnings
    add_compile_options(/wd4100) # unreferenced formal parameter
    add_compile_options(/wd4996) # deprecated functions
    
    # ARM64EC support
    if(CMAKE_GENERATOR_PLATFORM STREQUAL "ARM64EC")
        add_compile_definitions(KEYMAGIC_ARM64EC)
        message(STATUS "Building for ARM64EC")
    elseif(CMAKE_GENERATOR_PLATFORM STREQUAL "ARM64")
        add_compile_definitions(KEYMAGIC_ARM64)
        message(STATUS "Building for ARM64")
    endif()
    
    # Multi-processor compilation
    add_compile_options(/MP)
    
    # Runtime library
    set(CMAKE_MSVC_RUNTIME_LIBRARY "MultiThreaded$<$<CONFIG:Debug>:Debug>")
    
elseif(UNIX)
    add_compile_definitions(KEYMAGIC_UNIX)
    
    # Unix-specific settings
    add_compile_options(-Wall -Wextra -Wpedantic)
    add_compile_options(-Wno-unused-parameter)
    
    # Position-independent code for shared libraries
    set(CMAKE_POSITION_INDEPENDENT_CODE ON)
endif()

# Sanitizers
if(ENABLE_ASAN)
    add_compile_options(-fsanitize=address)
    add_link_options(-fsanitize=address)
endif()

if(ENABLE_UBSAN)
    add_compile_options(-fsanitize=undefined)
    add_link_options(-fsanitize=undefined)
endif()

# Debug/Release configurations
if(CMAKE_BUILD_TYPE STREQUAL "Debug")
    add_compile_definitions(KEYMAGIC_DEBUG)
    if(MSVC)
        add_compile_options(/Od /RTC1)  # No optimization, runtime checks
    else()
        add_compile_options(-O0 -g)     # No optimization, debug symbols
    endif()
else()
    if(MSVC)
        add_compile_options(/O2 /GL)    # Full optimization, whole program optimization
        add_link_options(/LTCG)          # Link-time code generation
    else()
        add_compile_options(-O3)        # Maximum optimization
    endif()
endif()

# Source files
set(KEYMAGIC_CORE_SOURCES
    src/engine/engine.cpp
    src/engine/state.cpp
    src/engine/buffer.cpp
    
    src/matching/matcher.cpp
    src/matching/pattern.cpp
    src/matching/context.cpp
    src/matching/capture.cpp
    
    src/processing/rule_application.cpp
    src/processing/recursive.cpp
    src/processing/action_generator.cpp
    
    src/km2/loader.cpp
    src/km2/parser.cpp
    src/km2/validator.cpp
    
    src/api/c_api.cpp
    src/api/memory.cpp
    
    src/utils/utf8.cpp
    src/utils/debug.cpp
    src/utils/virtual_keys.cpp
)

# Header files
set(KEYMAGIC_CORE_HEADERS
    include/keymagic/keymagic.h
    include/keymagic/engine.h
    include/keymagic/types.h
    include/keymagic/km2_format.h
    include/keymagic/virtual_keys.h
)

# Create library
add_library(keymagic_core ${KEYMAGIC_CORE_SOURCES})

# Include directories
target_include_directories(keymagic_core
    PUBLIC
        $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/include>
        $<INSTALL_INTERFACE:include>
    PRIVATE
        ${CMAKE_CURRENT_SOURCE_DIR}/src
)

# Export symbols for shared library
if(BUILD_SHARED_LIBS)
    target_compile_definitions(keymagic_core
        PRIVATE KEYMAGIC_CORE_EXPORTS
        INTERFACE KEYMAGIC_CORE_IMPORTS
    )
endif()

# Debug matching disabled in production build

# Installation
install(TARGETS keymagic_core
    EXPORT KeyMagicCoreTargets
    LIBRARY DESTINATION lib
    ARCHIVE DESTINATION lib
    RUNTIME DESTINATION bin
    INCLUDES DESTINATION include
)

install(FILES ${KEYMAGIC_CORE_HEADERS}
    DESTINATION include/keymagic
)


# Testing
if(BUILD_TESTING)
    enable_testing()
    
    # Download and configure Google Test
    include(FetchContent)
    FetchContent_Declare(
        googletest
        GIT_REPOSITORY https://github.com/google/googletest.git
        GIT_TAG v1.14.0
    )
    
    # For Windows: Prevent overriding the parent project's compiler/linker settings
    set(gtest_force_shared_crt ON CACHE BOOL "" FORCE)
    
    FetchContent_MakeAvailable(googletest)
    
    # Add test subdirectory
    add_subdirectory(tests)
endif()

# Package configuration
include(CMakePackageConfigHelpers)

write_basic_package_version_file(
    "${CMAKE_CURRENT_BINARY_DIR}/KeyMagicCoreConfigVersion.cmake"
    VERSION ${PROJECT_VERSION}
    COMPATIBILITY AnyNewerVersion
)

install(EXPORT KeyMagicCoreTargets
    FILE KeyMagicCoreTargets.cmake
    NAMESPACE KeyMagic::
    DESTINATION lib/cmake/KeyMagicCore
)

configure_file(cmake/KeyMagicCoreConfig.cmake.in
    "${CMAKE_CURRENT_BINARY_DIR}/KeyMagicCoreConfig.cmake"
    @ONLY
)

install(FILES
    "${CMAKE_CURRENT_BINARY_DIR}/KeyMagicCoreConfig.cmake"
    "${CMAKE_CURRENT_BINARY_DIR}/KeyMagicCoreConfigVersion.cmake"
    DESTINATION lib/cmake/KeyMagicCore
)

# Status messages
message(STATUS "KeyMagic Core C++ Configuration:")
message(STATUS "  Version: ${PROJECT_VERSION}")
message(STATUS "  Build type: ${CMAKE_BUILD_TYPE}")
message(STATUS "  C++ Standard: ${CMAKE_CXX_STANDARD}")
message(STATUS "  Shared libraries: ${BUILD_SHARED_LIBS}")
message(STATUS "  Testing: ${BUILD_TESTING}")
message(STATUS "  Platform: ${CMAKE_SYSTEM_NAME}")
message(STATUS "  Architecture: ${CMAKE_SYSTEM_PROCESSOR}")
if(WIN32)
    message(STATUS "  Generator Platform: ${CMAKE_GENERATOR_PLATFORM}")
endif()