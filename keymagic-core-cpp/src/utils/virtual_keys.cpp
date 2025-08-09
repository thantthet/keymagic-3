#include <keymagic/virtual_keys.h>
#include <algorithm>
#include <cctype>

namespace keymagic {

// Static member definitions
std::unordered_map<std::string, VirtualKey> VirtualKeyHelper::aliasMap;
std::unordered_map<int, VirtualKey> VirtualKeyHelper::windowsVKMap;
std::unordered_map<VirtualKey, int> VirtualKeyHelper::toWindowsVKMap;
bool VirtualKeyHelper::mapsInitialized = false;

const char* VirtualKeyHelper::toDisplayString(VirtualKey key) {
    switch (key) {
        // Control keys
        case VirtualKey::Back: return "Backspace";
        case VirtualKey::Tab: return "Tab";
        case VirtualKey::Return: return "Enter";
        case VirtualKey::Shift: return "Shift";
        case VirtualKey::Control: return "Ctrl";
        case VirtualKey::Menu: return "Alt";
        case VirtualKey::Pause: return "Pause";
        case VirtualKey::Capital: return "CapsLock";
        case VirtualKey::Kanji: return "Kanji";
        case VirtualKey::Escape: return "Esc";
        case VirtualKey::Space: return "Space";
        case VirtualKey::Prior: return "PageUp";
        case VirtualKey::Next: return "PageDown";
        case VirtualKey::Delete: return "Delete";
        
        // Number keys
        case VirtualKey::Key0: return "0";
        case VirtualKey::Key1: return "1";
        case VirtualKey::Key2: return "2";
        case VirtualKey::Key3: return "3";
        case VirtualKey::Key4: return "4";
        case VirtualKey::Key5: return "5";
        case VirtualKey::Key6: return "6";
        case VirtualKey::Key7: return "7";
        case VirtualKey::Key8: return "8";
        case VirtualKey::Key9: return "9";
        
        // Letter keys
        case VirtualKey::KeyA: return "A";
        case VirtualKey::KeyB: return "B";
        case VirtualKey::KeyC: return "C";
        case VirtualKey::KeyD: return "D";
        case VirtualKey::KeyE: return "E";
        case VirtualKey::KeyF: return "F";
        case VirtualKey::KeyG: return "G";
        case VirtualKey::KeyH: return "H";
        case VirtualKey::KeyI: return "I";
        case VirtualKey::KeyJ: return "J";
        case VirtualKey::KeyK: return "K";
        case VirtualKey::KeyL: return "L";
        case VirtualKey::KeyM: return "M";
        case VirtualKey::KeyN: return "N";
        case VirtualKey::KeyO: return "O";
        case VirtualKey::KeyP: return "P";
        case VirtualKey::KeyQ: return "Q";
        case VirtualKey::KeyR: return "R";
        case VirtualKey::KeyS: return "S";
        case VirtualKey::KeyT: return "T";
        case VirtualKey::KeyU: return "U";
        case VirtualKey::KeyV: return "V";
        case VirtualKey::KeyW: return "W";
        case VirtualKey::KeyX: return "X";
        case VirtualKey::KeyY: return "Y";
        case VirtualKey::KeyZ: return "Z";
        
        // Function keys
        case VirtualKey::F1: return "F1";
        case VirtualKey::F2: return "F2";
        case VirtualKey::F3: return "F3";
        case VirtualKey::F4: return "F4";
        case VirtualKey::F5: return "F5";
        case VirtualKey::F6: return "F6";
        case VirtualKey::F7: return "F7";
        case VirtualKey::F8: return "F8";
        case VirtualKey::F9: return "F9";
        case VirtualKey::F10: return "F10";
        case VirtualKey::F11: return "F11";
        case VirtualKey::F12: return "F12";
        
        // Numpad
        case VirtualKey::Numpad0: return "Num0";
        case VirtualKey::Numpad1: return "Num1";
        case VirtualKey::Numpad2: return "Num2";
        case VirtualKey::Numpad3: return "Num3";
        case VirtualKey::Numpad4: return "Num4";
        case VirtualKey::Numpad5: return "Num5";
        case VirtualKey::Numpad6: return "Num6";
        case VirtualKey::Numpad7: return "Num7";
        case VirtualKey::Numpad8: return "Num8";
        case VirtualKey::Numpad9: return "Num9";
        
        // OEM keys
        case VirtualKey::Oem1: return ";";
        case VirtualKey::OemPlus: return "+";
        case VirtualKey::OemComma: return ",";
        case VirtualKey::OemMinus: return "-";
        case VirtualKey::OemPeriod: return ".";
        case VirtualKey::Oem2: return "/";
        case VirtualKey::Oem3: return "`";
        case VirtualKey::Oem4: return "[";
        case VirtualKey::Oem5: return "\\";
        case VirtualKey::Oem6: return "]";
        case VirtualKey::Oem7: return "'";
        
        default: return "Unknown";
    }
}

std::string VirtualKeyHelper::toString(VirtualKey key) {
    switch (key) {
        case VirtualKey::Back: return "BACK";
        case VirtualKey::Tab: return "TAB";
        case VirtualKey::Return: return "RETURN";
        case VirtualKey::Shift: return "SHIFT";
        case VirtualKey::Control: return "CONTROL";
        case VirtualKey::Menu: return "MENU";
        case VirtualKey::Escape: return "ESCAPE";
        case VirtualKey::Space: return "SPACE";
        case VirtualKey::Key0: return "KEY_0";
        case VirtualKey::Key1: return "KEY_1";
        case VirtualKey::Key2: return "KEY_2";
        case VirtualKey::Key3: return "KEY_3";
        case VirtualKey::Key4: return "KEY_4";
        case VirtualKey::Key5: return "KEY_5";
        case VirtualKey::Key6: return "KEY_6";
        case VirtualKey::Key7: return "KEY_7";
        case VirtualKey::Key8: return "KEY_8";
        case VirtualKey::Key9: return "KEY_9";
        case VirtualKey::KeyA: return "KEY_A";
        case VirtualKey::KeyB: return "KEY_B";
        case VirtualKey::KeyC: return "KEY_C";
        case VirtualKey::KeyD: return "KEY_D";
        case VirtualKey::KeyE: return "KEY_E";
        case VirtualKey::KeyF: return "KEY_F";
        case VirtualKey::KeyG: return "KEY_G";
        case VirtualKey::KeyH: return "KEY_H";
        case VirtualKey::KeyI: return "KEY_I";
        case VirtualKey::KeyJ: return "KEY_J";
        case VirtualKey::KeyK: return "KEY_K";
        case VirtualKey::KeyL: return "KEY_L";
        case VirtualKey::KeyM: return "KEY_M";
        case VirtualKey::KeyN: return "KEY_N";
        case VirtualKey::KeyO: return "KEY_O";
        case VirtualKey::KeyP: return "KEY_P";
        case VirtualKey::KeyQ: return "KEY_Q";
        case VirtualKey::KeyR: return "KEY_R";
        case VirtualKey::KeyS: return "KEY_S";
        case VirtualKey::KeyT: return "KEY_T";
        case VirtualKey::KeyU: return "KEY_U";
        case VirtualKey::KeyV: return "KEY_V";
        case VirtualKey::KeyW: return "KEY_W";
        case VirtualKey::KeyX: return "KEY_X";
        case VirtualKey::KeyY: return "KEY_Y";
        case VirtualKey::KeyZ: return "KEY_Z";
        default: return "UNKNOWN";
    }
}

VirtualKey VirtualKeyHelper::fromString(const std::string& str) {
    if (!mapsInitialized) {
        initializeAliasMap();
    }
    
    // Convert to uppercase for case-insensitive matching
    std::string upper = str;
    std::transform(upper.begin(), upper.end(), upper.begin(), ::toupper);
    
    // Remove VK_ prefix if present
    if (upper.substr(0, 3) == "VK_") {
        upper = upper.substr(3);
    }
    
    auto it = aliasMap.find(upper);
    if (it != aliasMap.end()) {
        return it->second;
    }
    
    return VirtualKey::Null;
}

VirtualKey VirtualKeyHelper::fromWindowsVK(int vkCode) {
    if (!mapsInitialized) {
        initializeAliasMap();
    }
    
    auto it = windowsVKMap.find(vkCode);
    if (it != windowsVKMap.end()) {
        return it->second;
    }
    
    return VirtualKey::Null;
}

int VirtualKeyHelper::toWindowsVK(VirtualKey key) {
    if (!mapsInitialized) {
        initializeAliasMap();
    }
    
    auto it = toWindowsVKMap.find(key);
    if (it != toWindowsVKMap.end()) {
        return it->second;
    }
    
    return 0;
}

void VirtualKeyHelper::initializeAliasMap() {
    if (mapsInitialized) return;
    
    // String to VirtualKey aliases
    aliasMap["BACK"] = VirtualKey::Back;
    aliasMap["BACKSPACE"] = VirtualKey::Back;
    aliasMap["TAB"] = VirtualKey::Tab;
    aliasMap["RETURN"] = VirtualKey::Return;
    aliasMap["ENTER"] = VirtualKey::Return;
    aliasMap["SHIFT"] = VirtualKey::Shift;
    aliasMap["CONTROL"] = VirtualKey::Control;
    aliasMap["CTRL"] = VirtualKey::Control;
    aliasMap["MENU"] = VirtualKey::Menu;
    aliasMap["ALT"] = VirtualKey::Menu;
    aliasMap["ESCAPE"] = VirtualKey::Escape;
    aliasMap["ESC"] = VirtualKey::Escape;
    aliasMap["SPACE"] = VirtualKey::Space;
    aliasMap["CAPITAL"] = VirtualKey::Capital;
    aliasMap["CAPSLOCK"] = VirtualKey::Capital;
    aliasMap["DELETE"] = VirtualKey::Delete;
    
    // Number keys
    for (int i = 0; i <= 9; ++i) {
        std::string keyName = "KEY_" + std::to_string(i);
        aliasMap[keyName] = static_cast<VirtualKey>(static_cast<int>(VirtualKey::Key0) + i);
        aliasMap[std::to_string(i)] = static_cast<VirtualKey>(static_cast<int>(VirtualKey::Key0) + i);
    }
    
    // Letter keys
    for (char c = 'A'; c <= 'Z'; ++c) {
        std::string keyName = "KEY_";
        keyName += c;
        aliasMap[keyName] = static_cast<VirtualKey>(static_cast<int>(VirtualKey::KeyA) + (c - 'A'));
        aliasMap[std::string(1, c)] = static_cast<VirtualKey>(static_cast<int>(VirtualKey::KeyA) + (c - 'A'));
    }
    
    // Windows VK code mappings
    windowsVKMap[0x08] = VirtualKey::Back;      // VK_BACK
    windowsVKMap[0x09] = VirtualKey::Tab;       // VK_TAB
    windowsVKMap[0x0D] = VirtualKey::Return;    // VK_RETURN
    windowsVKMap[0x10] = VirtualKey::Shift;     // VK_SHIFT
    windowsVKMap[0x11] = VirtualKey::Control;   // VK_CONTROL
    windowsVKMap[0x12] = VirtualKey::Menu;      // VK_MENU
    windowsVKMap[0x13] = VirtualKey::Pause;     // VK_PAUSE
    windowsVKMap[0x14] = VirtualKey::Capital;   // VK_CAPITAL
    windowsVKMap[0x1B] = VirtualKey::Escape;    // VK_ESCAPE
    windowsVKMap[0x20] = VirtualKey::Space;     // VK_SPACE
    windowsVKMap[0x21] = VirtualKey::Prior;     // VK_PRIOR
    windowsVKMap[0x22] = VirtualKey::Next;      // VK_NEXT
    windowsVKMap[0x2E] = VirtualKey::Delete;    // VK_DELETE
    
    // Number keys
    for (int i = 0; i <= 9; ++i) {
        windowsVKMap[0x30 + i] = static_cast<VirtualKey>(static_cast<int>(VirtualKey::Key0) + i);
    }
    
    // Letter keys
    for (int i = 0; i < 26; ++i) {
        windowsVKMap[0x41 + i] = static_cast<VirtualKey>(static_cast<int>(VirtualKey::KeyA) + i);
    }
    
    // Numpad keys
    for (int i = 0; i <= 9; ++i) {
        windowsVKMap[0x60 + i] = static_cast<VirtualKey>(static_cast<int>(VirtualKey::Numpad0) + i);
    }
    
    // Function keys
    for (int i = 0; i < 12; ++i) {
        windowsVKMap[0x70 + i] = static_cast<VirtualKey>(static_cast<int>(VirtualKey::F1) + i);
    }
    
    // OEM keys
    windowsVKMap[0xBA] = VirtualKey::Oem1;      // VK_OEM_1
    windowsVKMap[0xBB] = VirtualKey::OemPlus;   // VK_OEM_PLUS
    windowsVKMap[0xBC] = VirtualKey::OemComma;  // VK_OEM_COMMA
    windowsVKMap[0xBD] = VirtualKey::OemMinus;  // VK_OEM_MINUS
    windowsVKMap[0xBE] = VirtualKey::OemPeriod; // VK_OEM_PERIOD
    windowsVKMap[0xBF] = VirtualKey::Oem2;      // VK_OEM_2
    windowsVKMap[0xC0] = VirtualKey::Oem3;      // VK_OEM_3
    windowsVKMap[0xDB] = VirtualKey::Oem4;      // VK_OEM_4
    windowsVKMap[0xDC] = VirtualKey::Oem5;      // VK_OEM_5
    windowsVKMap[0xDD] = VirtualKey::Oem6;      // VK_OEM_6
    windowsVKMap[0xDE] = VirtualKey::Oem7;      // VK_OEM_7
    
    // Build reverse map
    for (const auto& pair : windowsVKMap) {
        toWindowsVKMap[pair.second] = pair.first;
    }
    
    mapsInitialized = true;
}

const std::unordered_map<std::string, VirtualKey>& VirtualKeyHelper::getAliasMap() {
    if (!mapsInitialized) {
        initializeAliasMap();
    }
    return aliasMap;
}

} // namespace keymagic