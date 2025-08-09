cmake_minimum_required(VERSION 3.20)

# Test executable
add_executable(keymagic_core_tests
    unit/test_main.cpp
    unit/test_utf8.cpp
    unit/test_types.cpp
    unit/test_virtual_keys.cpp
    unit/test_km2_format.cpp
)

# Link with Google Test and KeyMagic Core
target_link_libraries(keymagic_core_tests
    PRIVATE
        keymagic_core
        GTest::gtest
        GTest::gtest_main
        GTest::gmock
)

# Include directories
target_include_directories(keymagic_core_tests
    PRIVATE
        ${CMAKE_SOURCE_DIR}/include
        ${CMAKE_SOURCE_DIR}/src
)

# Add tests to CTest
include(GoogleTest)
gtest_discover_tests(keymagic_core_tests)

# Copy test data files
file(GLOB TEST_KM2_FILES "${CMAKE_SOURCE_DIR}/tests/data/*.km2")
foreach(TEST_FILE ${TEST_KM2_FILES})
    configure_file(${TEST_FILE} ${CMAKE_CURRENT_BINARY_DIR} COPYONLY)
endforeach()

# Integration tests
add_executable(keymagic_integration_tests
    integration/test_main.cpp
    integration/test_keyboard_discovery.cpp
    integration/test_pyidaungsu_keystrokes.cpp
    integration/test_backspace_history.cpp
    common/test_utils.cpp
)

target_link_libraries(keymagic_integration_tests
    PRIVATE
        keymagic_core
        GTest::gtest
        GTest::gtest_main
)

target_include_directories(keymagic_integration_tests
    PRIVATE
        ${CMAKE_SOURCE_DIR}/include
        ${CMAKE_SOURCE_DIR}/src
)

gtest_discover_tests(keymagic_integration_tests)

# Memory leak detection on Linux
if(UNIX AND NOT APPLE)
    add_custom_target(test_valgrind
        COMMAND valgrind --leak-check=full --error-exitcode=1 ./keymagic_core_tests
        DEPENDS keymagic_core_tests
        WORKING_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR}
        COMMENT "Running tests with Valgrind memory leak detection"
    )
endif()

# Coverage report (if enabled)
if(CMAKE_BUILD_TYPE STREQUAL "Debug" AND CMAKE_CXX_COMPILER_ID MATCHES "GNU|Clang")
    find_program(GCOV gcov)
    find_program(LCOV lcov)
    find_program(GENHTML genhtml)
    
    if(GCOV AND LCOV AND GENHTML)
        add_custom_target(coverage
            COMMAND ${LCOV} --directory . --zerocounters
            COMMAND ${CMAKE_CTEST_COMMAND}
            COMMAND ${LCOV} --directory . --capture --output-file coverage.info
            COMMAND ${LCOV} --remove coverage.info '/usr/*' '*/tests/*' '*/googletest/*' --output-file coverage.filtered.info
            COMMAND ${GENHTML} coverage.filtered.info --output-directory coverage_report
            WORKING_DIRECTORY ${CMAKE_BINARY_DIR}
            COMMENT "Generating code coverage report"
        )
    endif()
endif()