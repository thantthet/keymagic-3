#include "test_utils.h"
#include <iostream>
#include <vector>
#include <algorithm>

namespace keymagic_test {

std::optional<std::filesystem::path> KeyboardFinder::findKeyboardsDirectory() {
    std::filesystem::path currentDir = std::filesystem::current_path();
    
    // Search upwards through parent directories
    for (int depth = 0; depth < MAX_SEARCH_DEPTH; ++depth) {
        if (hasKeyboardsDirectory(currentDir)) {
            return currentDir / "keyboards" / "bundled";
        }
        
        // Move to parent directory
        std::filesystem::path parentDir = currentDir.parent_path();
        if (parentDir == currentDir) {
            // Reached filesystem root
            break;
        }
        currentDir = parentDir;
    }
    
    return std::nullopt;
}

std::optional<std::filesystem::path> KeyboardFinder::findKeyboardFile(const std::string& keyboardName) {
    auto keyboardsDir = findKeyboardsDirectory();
    if (!keyboardsDir) {
        return std::nullopt;
    }
    
    std::filesystem::path keyboardPath = *keyboardsDir / keyboardName;
    
    if (std::filesystem::exists(keyboardPath) && std::filesystem::is_regular_file(keyboardPath)) {
        return keyboardPath;
    }
    
    return std::nullopt;
}

std::vector<std::filesystem::path> KeyboardFinder::getAllKeyboardFiles() {
    std::vector<std::filesystem::path> keyboards;
    
    auto keyboardsDir = findKeyboardsDirectory();
    if (!keyboardsDir) {
        return keyboards;
    }
    
    try {
        for (const auto& entry : std::filesystem::directory_iterator(*keyboardsDir)) {
            if (entry.is_regular_file() && entry.path().extension() == ".km2") {
                keyboards.push_back(entry.path());
            }
        }
    } catch (const std::filesystem::filesystem_error& e) {
        std::cerr << "Error reading keyboards directory: " << e.what() << std::endl;
    }
    
    // Sort for consistent ordering
    std::sort(keyboards.begin(), keyboards.end());
    
    return keyboards;
}

bool KeyboardFinder::hasKeyboardsDirectory(const std::filesystem::path& dir) {
    std::filesystem::path keyboardsPath = dir / "keyboards" / "bundled";
    
    return std::filesystem::exists(keyboardsPath) && 
           std::filesystem::is_directory(keyboardsPath);
}

std::string getKeyboardLoadingHelp() {
    std::string help = "Keyboard Loading Debugging Information:\n";
    help += "=====================================\n";
    
    // Show current working directory
    help += "Current working directory: " + std::filesystem::current_path().string() + "\n";
    
    // Try to find keyboards directory
    auto keyboardsDir = KeyboardFinder::findKeyboardsDirectory();
    if (keyboardsDir) {
        help += "Found keyboards directory: " + keyboardsDir->string() + "\n";
        
        // List available keyboards
        auto keyboards = KeyboardFinder::getAllKeyboardFiles();
        if (!keyboards.empty()) {
            help += "Available keyboard files:\n";
            for (const auto& keyboard : keyboards) {
                help += "  - " + keyboard.filename().string() + "\n";
            }
        } else {
            help += "No .km2 files found in keyboards directory\n";
        }
    } else {
        help += "Could not find keyboards/bundled directory\n";
        help += "Searched upwards from current directory through " + 
               std::to_string(10) + " parent levels\n";
        
        // Show what we're looking for
        help += "\nLooking for directory structure:\n";
        help += "  some_parent_dir/\n";
        help += "    keyboards/\n";
        help += "      bundled/\n";
        help += "        *.km2 files\n";
    }
    
    return help;
}

} // namespace keymagic_test