#include "test_utils.h"
#include <iostream>
#include <vector>
#include <algorithm>
#include <keymagic/km2_format.h>
#include <memory>

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

std::unique_ptr<keymagic::KM2File> createBasicKM2WithOptions(bool autoBksp, bool eat, bool trackCaps) {
    auto km2 = std::make_unique<keymagic::KM2File>();
    
    // Set header with version info
    km2->header.magicCode[0] = 'K';
    km2->header.magicCode[1] = 'M';
    km2->header.magicCode[2] = 'K';
    km2->header.magicCode[3] = 'L';
    km2->header.majorVersion = 1;
    km2->header.minorVersion = 5;
    
    // Set layout options
    km2->header.layoutOptions.trackCaps = trackCaps ? 1 : 0;
    km2->header.layoutOptions.autoBksp = autoBksp ? 1 : 0;
    km2->header.layoutOptions.eat = eat ? 1 : 0;
    km2->header.layoutOptions.posBased = 0;
    km2->header.layoutOptions.rightAlt = 1;
    
    // Initialize counts
    km2->header.stringCount = 0;
    km2->header.infoCount = 0; 
    km2->header.ruleCount = 0;
    
    return km2;
}

std::unique_ptr<keymagic::KM2File> createKM2WithRule(const std::string& lhsPattern, 
                                                     const std::string& rhsOutput,
                                                     bool autoBksp, 
                                                     bool eat, 
                                                     bool trackCaps) {
    auto km2 = createBasicKM2WithOptions(autoBksp, eat, trackCaps);
    
    // Add strings for the rule - need to convert std::string to std::u16string
    keymagic::StringEntry lhsString;
    // Simple ASCII to UTF-16 conversion
    for (char ch : lhsPattern) {
        lhsString.value.push_back(static_cast<char16_t>(ch));
    }
    km2->strings.push_back(lhsString);
    
    keymagic::StringEntry rhsString;
    // Proper UTF-8 to UTF-16 conversion for rhsOutput
    // This handles Myanmar and other Unicode characters correctly
    const char* ptr = rhsOutput.c_str();
    const char* end = ptr + rhsOutput.size();
    while (ptr < end) {
        uint32_t codepoint = 0;
        unsigned char ch = *ptr;
        
        if (ch < 0x80) {
            // ASCII
            codepoint = ch;
            ptr++;
        } else if ((ch & 0xE0) == 0xC0) {
            // 2-byte UTF-8
            codepoint = ((ch & 0x1F) << 6) | (ptr[1] & 0x3F);
            ptr += 2;
        } else if ((ch & 0xF0) == 0xE0) {
            // 3-byte UTF-8
            codepoint = ((ch & 0x0F) << 12) | ((ptr[1] & 0x3F) << 6) | (ptr[2] & 0x3F);
            ptr += 3;
        } else if ((ch & 0xF8) == 0xF0) {
            // 4-byte UTF-8
            codepoint = ((ch & 0x07) << 18) | ((ptr[1] & 0x3F) << 12) | 
                       ((ptr[2] & 0x3F) << 6) | (ptr[3] & 0x3F);
            ptr += 4;
        } else {
            // Invalid UTF-8, skip
            ptr++;
            continue;
        }
        
        // Convert to UTF-16
        if (codepoint < 0x10000) {
            rhsString.value.push_back(static_cast<char16_t>(codepoint));
        } else {
            // Surrogate pair
            codepoint -= 0x10000;
            rhsString.value.push_back(static_cast<char16_t>(0xD800 + (codepoint >> 10)));
            rhsString.value.push_back(static_cast<char16_t>(0xDC00 + (codepoint & 0x3FF)));
        }
    }
    km2->strings.push_back(rhsString);
    
    km2->header.stringCount = 2;
    
    // Create a simple rule: lhsPattern => rhsOutput
    keymagic::BinaryRule rule;
    
    // LHS: STRING(lhsPattern)
    rule.lhs.push_back(keymagic::OP_STRING);
    rule.lhs.push_back(static_cast<uint16_t>(lhsPattern.size()));
    for (char ch : lhsPattern) {
        rule.lhs.push_back(static_cast<uint16_t>(ch));
    }
    
    // RHS: STRING(rhsOutput) - use the UTF-16 value we already converted
    rule.rhs.push_back(keymagic::OP_STRING);
    rule.rhs.push_back(static_cast<uint16_t>(rhsString.value.size()));
    for (char16_t ch : rhsString.value) {
        rule.rhs.push_back(static_cast<uint16_t>(ch));
    }
    
    km2->rules.push_back(rule);
    km2->header.ruleCount = 1;
    
    return km2;
}

} // namespace keymagic_test