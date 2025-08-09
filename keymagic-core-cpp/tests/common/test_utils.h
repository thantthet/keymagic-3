#ifndef KEYMAGIC_TEST_UTILS_H
#define KEYMAGIC_TEST_UTILS_H

#include <string>
#include <filesystem>
#include <optional>
#include <memory>
#include <vector>
#include <keymagic/km2_format.h>

namespace keymagic_test {

/**
 * Cross-platform utility to find the keyboards/bundled directory
 * starting from the current working directory and searching upwards
 */
class KeyboardFinder {
public:
    /**
     * Find the keyboards/bundled directory by searching upwards from current directory
     * @return Path to keyboards/bundled directory if found, empty optional if not found
     */
    static std::optional<std::filesystem::path> findKeyboardsDirectory();
    
    /**
     * Find a specific keyboard file in the keyboards/bundled directory
     * @param keyboardName Name of the keyboard file (e.g., "Pyidaungsu MM.km2")
     * @return Full path to the keyboard file if found, empty optional if not found
     */
    static std::optional<std::filesystem::path> findKeyboardFile(const std::string& keyboardName);
    
    /**
     * Get all available keyboard files in the keyboards/bundled directory
     * @return Vector of paths to all .km2 files found
     */
    static std::vector<std::filesystem::path> getAllKeyboardFiles();

private:
    /**
     * Check if a directory contains the keyboards/bundled subdirectory structure
     * @param dir Directory to check
     * @return true if keyboards/bundled exists under this directory
     */
    static bool hasKeyboardsDirectory(const std::filesystem::path& dir);
    
    /**
     * Maximum number of parent directories to search upwards
     */
    static constexpr int MAX_SEARCH_DEPTH = 10;
};

/**
 * Helper function to get a readable error message for keyboard loading failures
 */
std::string getKeyboardLoadingHelp();

/**
 * Creates a basic KM2 file with specified layout options for testing
 * @param autoBksp Enable smart backspace behavior
 * @param eat Consume keys that don't match any rule  
 * @param trackCaps Track CAPSLOCK key state
 * @return KM2File object with the specified options
 */
std::unique_ptr<keymagic::KM2File> createBasicKM2WithOptions(bool autoBksp = false, bool eat = false, bool trackCaps = true);

/**
 * Creates a KM2 file with a single rule and specified layout options
 * @param lhsPattern Left-hand side pattern (e.g., "ka")
 * @param rhsOutput Right-hand side output (e.g., "က")
 * @param autoBksp Enable smart backspace behavior
 * @param eat Consume keys that don't match any rule
 * @param trackCaps Track CAPSLOCK key state
 * @return KM2File object with the rule and options
 */
std::unique_ptr<keymagic::KM2File> createKM2WithRule(const std::string& lhsPattern, 
                                                     const std::string& rhsOutput,
                                                     bool autoBksp = false, 
                                                     bool eat = false, 
                                                     bool trackCaps = true);

} // namespace keymagic_test

#endif // KEYMAGIC_TEST_UTILS_H