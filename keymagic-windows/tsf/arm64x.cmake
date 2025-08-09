# arm64x.cmake - CMake support for building native ARM64X binaries
# This creates a true ARM64X DLL containing both ARM64 and ARM64EC code

# Directory where the link.rsp file generated during arm64 build will be stored
# Use separate directories for Debug and Release to avoid conflicts
# Check multiple ways to determine the configuration
if(CMAKE_PRESET_NAME MATCHES "debug" OR CMAKE_BUILD_TYPE STREQUAL "Debug" OR CMAKE_CURRENT_BINARY_DIR MATCHES "debug")
    set(arm64ReproDir "${CMAKE_CURRENT_SOURCE_DIR}/repros-debug")
    message(STATUS "Using repros-debug directory for ARM64X response files")
elseif(CMAKE_PRESET_NAME MATCHES "release" OR CMAKE_BUILD_TYPE STREQUAL "Release" OR CMAKE_CURRENT_BINARY_DIR MATCHES "release")
    set(arm64ReproDir "${CMAKE_CURRENT_SOURCE_DIR}/repros-release")
    message(STATUS "Using repros-release directory for ARM64X response files")
else()
    # Fallback to single directory if configuration can't be determined
    set(arm64ReproDir "${CMAKE_CURRENT_SOURCE_DIR}/repros")
    message(STATUS "Using default repros directory for ARM64X response files")
endif()

# This function reads in the content of the rsp file outputted from arm64 build for a target.
# Then passes the arm64 libs, objs and def file to the linker using /machine:arm64x to combine
# them with the arm64ec counterparts and create an arm64x binary.
function(set_arm64_dependencies n)
    set(REPRO_FILE "${arm64ReproDir}/${n}.rsp")
    if(NOT EXISTS "${REPRO_FILE}")
        message(WARNING "ARM64 response file not found: ${REPRO_FILE}")
        message(WARNING "Make sure to build ARM64 configuration first")
        return()
    endif()
    
    file(STRINGS "${REPRO_FILE}" ARM64_OBJS REGEX obj\"$)
    file(STRINGS "${REPRO_FILE}" ARM64_DEF REGEX def\"$)
    file(STRINGS "${REPRO_FILE}" ARM64_LIBS REGEX lib\"$)
    string(REPLACE "\"" ";" ARM64_OBJS "${ARM64_OBJS}")
    string(REPLACE "\"" ";" ARM64_LIBS "${ARM64_LIBS}")
    string(REPLACE "\"" ";" ARM64_DEF "${ARM64_DEF}")
    string(REPLACE "/def:" "/defArm64Native:" ARM64_DEF "${ARM64_DEF}")

    # Add ARM64 objects and libraries to the ARM64EC target
    target_sources(${n} PRIVATE ${ARM64_OBJS})
    target_link_options(${n} PRIVATE /machine:arm64x "${ARM64_DEF}" "${ARM64_LIBS}")
    
    message(STATUS "Configured ARM64X for target: ${n}")
endfunction()

# During the ARM64 build, create link.rsp files that contain the absolute path to the inputs
# passed to the linker (objs, def files, libs).
if("${BUILD_AS_ARM64X}" STREQUAL "ARM64")
    message(STATUS "Building ARM64 components for ARM64X...")
    
    # Create the repros directory if it doesn't exist
    add_custom_target(mkdirs ALL COMMAND ${CMAKE_COMMAND} -E make_directory "${arm64ReproDir}")
    
    foreach (n ${ARM64X_TARGETS})
        add_dependencies(${n} mkdirs)
        # Tell the linker to produce this special rsp file that has absolute paths to its inputs
        # This requires Visual Studio 17.11 or later
        # Force full link to ensure complete response file generation
        target_link_options(${n} PRIVATE 
            "/LINKREPROFULLPATHRSP:${arm64ReproDir}/${n}.rsp"
            "/INCREMENTAL:NO"  # Disable incremental linking to ensure full response file
        )
        message(STATUS "ARM64 target ${n} configured to generate response file")
    endforeach()

# During the ARM64EC build, modify the link step appropriately to produce an arm64x binary
elseif("${BUILD_AS_ARM64X}" STREQUAL "ARM64EC")
    message(STATUS "Building ARM64EC components and linking ARM64X binary...")
    
    foreach (n ${ARM64X_TARGETS})
        set_arm64_dependencies(${n})
    endforeach()
endif()

# Function to handle C++ core library for ARM64X builds
function(configure_cpp_core_for_arm64x target)
    if("${BUILD_AS_ARM64X}" STREQUAL "ARM64" OR "${BUILD_AS_ARM64X}" STREQUAL "ARM64EC")
        # Build the C++ core library as a subdirectory
        # Use a unique build directory for each configuration
        set(CPP_BUILD_DIR "keymagic-core-cpp-${BUILD_AS_ARM64X}-${CMAKE_BUILD_TYPE}")
        add_subdirectory(${CMAKE_CURRENT_SOURCE_DIR}/../../keymagic-core-cpp ${CPP_BUILD_DIR})
        
        # Link to the C++ core library
        target_link_libraries(${target} PRIVATE keymagic_core)
        
        # Include C++ core headers
        target_include_directories(${target} PRIVATE
            ${CMAKE_CURRENT_SOURCE_DIR}/../../keymagic-core-cpp/include
        )
        
        message(STATUS "Configured C++ core lib for ${target} (${BUILD_AS_ARM64X})")
    endif()
endfunction()