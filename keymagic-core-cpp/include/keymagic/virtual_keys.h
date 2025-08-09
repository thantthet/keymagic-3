#ifndef KEYMAGIC_VIRTUAL_KEYS_H
#define KEYMAGIC_VIRTUAL_KEYS_H

#include <cstdint>
#include <string>
#include <unordered_map>

namespace keymagic {

// Virtual key codes - internal representation (NOT Windows VK codes)
// These match the Rust VirtualKey enum values for compatibility
enum class VirtualKey : uint16_t {
    // Special value
    Null = 1,              // NULL output (delete)
    
    // Control keys
    Back = 2,              // Backspace
    Tab = 3,               // Tab
    Return = 4,            // Enter
    Shift = 5,             // Shift
    Control = 6,           // Ctrl
    Menu = 7,              // Alt
    Pause = 8,             // Pause
    Capital = 9,           // Caps Lock
    Kanji = 10,            // Kanji
    Escape = 11,           // Escape
    Space = 12,            // Space
    Prior = 13,            // Page Up
    Next = 14,             // Page Down
    Delete = 15,           // Delete
    
    // Number keys (0-9)
    Key0 = 16,
    Key1 = 17,
    Key2 = 18,
    Key3 = 19,
    Key4 = 20,
    Key5 = 21,
    Key6 = 22,
    Key7 = 23,
    Key8 = 24,
    Key9 = 25,
    
    // Letter keys (A-Z)
    KeyA = 26,
    KeyB = 27,
    KeyC = 28,
    KeyD = 29,
    KeyE = 30,
    KeyF = 31,
    KeyG = 32,
    KeyH = 33,
    KeyI = 34,
    KeyJ = 35,
    KeyK = 36,
    KeyL = 37,
    KeyM = 38,
    KeyN = 39,
    KeyO = 40,
    KeyP = 41,
    KeyQ = 42,
    KeyR = 43,
    KeyS = 44,
    KeyT = 45,
    KeyU = 46,
    KeyV = 47,
    KeyW = 48,
    KeyX = 49,
    KeyY = 50,
    KeyZ = 51,
    
    // Numpad keys
    Numpad0 = 52,
    Numpad1 = 53,
    Numpad2 = 54,
    Numpad3 = 55,
    Numpad4 = 56,
    Numpad5 = 57,
    Numpad6 = 58,
    Numpad7 = 59,
    Numpad8 = 60,
    Numpad9 = 61,
    
    // Numpad operators
    Multiply = 62,
    Add = 63,
    Separator = 64,
    Subtract = 65,
    Decimal = 66,
    Divide = 67,
    
    // Function keys
    F1 = 68,
    F2 = 69,
    F3 = 70,
    F4 = 71,
    F5 = 72,
    F6 = 73,
    F7 = 74,
    F8 = 75,
    F9 = 76,
    F10 = 77,
    F11 = 78,
    F12 = 79,
    
    // Modifier keys (left/right variants)
    LShift = 80,
    RShift = 81,
    LControl = 82,
    RControl = 83,
    LMenu = 84,            // Left Alt
    RMenu = 85,            // Right Alt/AltGr
    
    // OEM keys (keyboard-specific)
    Oem1 = 86,             // ;: for US
    OemPlus = 87,          // + key
    OemComma = 88,         // , key
    OemMinus = 89,         // - key
    OemPeriod = 90,        // . key
    Oem2 = 91,             // /? for US
    Oem3 = 92,             // `~ for US
    Oem4 = 93,             // [{ for US
    Oem5 = 94,             // \| for US
    Oem6 = 95,             // ]} for US
    Oem7 = 96,             // '" for US
    Oem8 = 97,
    OemAx = 98,
    Oem102 = 99,           // <> or \| on 102-key keyboard
    IcoHelp = 100,
    Ico00 = 101,
    
    // Navigation keys
    End = 102,
    Home = 103,
    Left = 104,
    Up = 105,
    Right = 106,
    Down = 107,
    Insert = 108,
    
    // Additional OEM keys
    CapsLock = 109,        // Alias for Capital
    Cflex = 110,           // Circumflex/caret key ^
    Colon = 111,           // Alias for Oem1
    Quote = 112,           // Alias for Oem7
    BackSlash = 113,       // Alias for Oem5
    OpenSquareBracket = 114,  // Alias for Oem4
    CloseSquareBracket = 115, // Alias for Oem6
    BackQuote = 116,       // Alias for Oem3
    ForwardSlash = 117,    // Alias for Oem2
    
    // Special aliases
    Enter = 118,           // Alias for Return
    Ctrl = 119,            // Alias for Control
    Alt = 120,             // Alias for Menu
    Esc = 121,             // Alias for Escape
    AltGr = 122,           // Alias for RMenu
    
    // Max value for validation
    MaxValue = 122
};

// Helper class for VirtualKey operations
class VirtualKeyHelper {
public:
    // Convert VirtualKey to display string
    static const char* toDisplayString(VirtualKey key);
    
    // Convert VirtualKey to string representation (for debugging)
    static std::string toString(VirtualKey key);
    
    // Parse VirtualKey from string (e.g., "VK_KEY_A" -> VirtualKey::KeyA)
    static VirtualKey fromString(const std::string& str);
    
    // Convert Windows VK code to internal VirtualKey
    static VirtualKey fromWindowsVK(int vkCode);
    
    // Convert internal VirtualKey to Windows VK code
    static int toWindowsVK(VirtualKey key);
    
    // Check if a value is a valid VirtualKey
    static bool isValid(uint16_t value) {
        return value >= static_cast<uint16_t>(VirtualKey::Null) && 
               value <= static_cast<uint16_t>(VirtualKey::MaxValue);
    }
    
    // Get alias mappings
    static const std::unordered_map<std::string, VirtualKey>& getAliasMap();
    
private:
    static void initializeAliasMap();
    static std::unordered_map<std::string, VirtualKey> aliasMap;
    static std::unordered_map<int, VirtualKey> windowsVKMap;
    static std::unordered_map<VirtualKey, int> toWindowsVKMap;
    static bool mapsInitialized;
};

// Inline implementations for frequently used functions
inline bool isModifierKey(VirtualKey key) {
    switch (key) {
        case VirtualKey::Shift:
        case VirtualKey::Control:
        case VirtualKey::Menu:
        case VirtualKey::LShift:
        case VirtualKey::RShift:
        case VirtualKey::LControl:
        case VirtualKey::RControl:
        case VirtualKey::LMenu:
        case VirtualKey::RMenu:
        case VirtualKey::Ctrl:
        case VirtualKey::Alt:
        case VirtualKey::AltGr:
            return true;
        default:
            return false;
    }
}

inline bool isLetterKey(VirtualKey key) {
    return key >= VirtualKey::KeyA && key <= VirtualKey::KeyZ;
}

inline bool isNumberKey(VirtualKey key) {
    return key >= VirtualKey::Key0 && key <= VirtualKey::Key9;
}

inline bool isNumpadKey(VirtualKey key) {
    return key >= VirtualKey::Numpad0 && key <= VirtualKey::Divide;
}

inline bool isFunctionKey(VirtualKey key) {
    return key >= VirtualKey::F1 && key <= VirtualKey::F12;
}

inline bool isOemKey(VirtualKey key) {
    return (key >= VirtualKey::Oem1 && key <= VirtualKey::Ico00) ||
           (key >= VirtualKey::Colon && key <= VirtualKey::ForwardSlash);
}

} // namespace keymagic

#endif // KEYMAGIC_VIRTUAL_KEYS_H