#ifndef KEYMAGIC_H
#define KEYMAGIC_H

// Include the unified KeyMagic Core header for C API definitions
// All C API functions and types are now defined in this header
#include "../../../keymagic-core/include/keymagic_core.h"

// Include C++ specific headers
#include "types.h"
#include "virtual_keys.h"
#include "km2_format.h"
#include <memory>
#include <string>

// ============================================================================
// C++ API - Modern C++ interface (when included from C++)
// ============================================================================

#ifdef __cplusplus

namespace keymagic {

// Forward declarations
class Engine;
class KeyboardLayout;

// Main engine class
class KEYMAGIC_API KeyMagicEngine {
public:
    KeyMagicEngine();
    ~KeyMagicEngine();
    
    // Disable copy, enable move
    KeyMagicEngine(const KeyMagicEngine&) = delete;
    KeyMagicEngine& operator=(const KeyMagicEngine&) = delete;
    KeyMagicEngine(KeyMagicEngine&&) noexcept;
    KeyMagicEngine& operator=(KeyMagicEngine&&) noexcept;
    
    // Keyboard loading
    Result loadKeyboard(const std::string& km2Path);
    Result loadKeyboardFromMemory(const uint8_t* data, size_t dataLen);
    
    // Key processing
    Output processKey(const Input& input);
    Output processKey(int keyCode, char32_t character, const Modifiers& modifiers);
    
    // Windows-specific processing
    Output processWindowsKey(int vkCode, char character, const Modifiers& modifiers);
    Output testProcessWindowsKey(int vkCode, char character, const Modifiers& modifiers);
    
    // Engine control
    void reset();
    std::string getComposition() const;
    void setComposition(const std::string& text);
    
    // Keyboard info
    bool hasKeyboard() const;
    std::string getKeyboardName() const;
    std::string getKeyboardDescription() const;
    
    // Version
    static std::string getVersion();
    
private:
    std::unique_ptr<Engine> engine_;
};

// KM2 file loader class
class KEYMAGIC_API KM2Loader {
public:
    // Load from file
    static std::unique_ptr<KM2File> loadFromFile(const std::string& path);
    
    // Load from memory
    static std::unique_ptr<KM2File> loadFromMemory(const uint8_t* data, size_t dataLen);
    
    // Validate file without fully loading
    static bool validateFile(const std::string& path);
    static bool validateMemory(const uint8_t* data, size_t dataLen);
};

// Hotkey parser
class KEYMAGIC_API HotkeyParser {
public:
    // Parse hotkey string (e.g., "CTRL+SHIFT+A")
    static bool parse(const std::string& hotkeyStr, HotkeyInfo& info);
    
    // Convert HotkeyInfo to string
    static std::string toString(const HotkeyInfo& info);
};

} // namespace keymagic

#endif // __cplusplus

#endif // KEYMAGIC_H