#include <keymagic/keymagic.h>
#include <keymagic/engine.h>
#include "../utils/utf8.h"
#include "../km2/loader.h"
#include <cstring>
#include <mutex>
#include <unordered_map>
#include <sstream>
#include <algorithm>
#include <vector>

namespace {
std::mutex g_handleMutex;
std::unordered_map<EngineHandle*, std::unique_ptr<keymagic::Engine>> g_engines;
std::unordered_map<Km2FileHandle*, std::unique_ptr<keymagic::KM2File>> g_km2Files;

// Helper to convert Windows VK codes to internal VirtualKey enum
keymagic::VirtualKey windowsVkToInternal(int vkCode) {
    switch (vkCode) {
        case 0x08: return keymagic::VirtualKey::Back;
        case 0x09: return keymagic::VirtualKey::Tab;
        case 0x0D: return keymagic::VirtualKey::Return;
        case 0x1B: return keymagic::VirtualKey::Escape;
        case 0x20: return keymagic::VirtualKey::Space;
        case 0x21: return keymagic::VirtualKey::Prior;
        case 0x22: return keymagic::VirtualKey::Next;
        case 0x23: return keymagic::VirtualKey::End;
        case 0x24: return keymagic::VirtualKey::Home;
        case 0x25: return keymagic::VirtualKey::Left;
        case 0x26: return keymagic::VirtualKey::Up;
        case 0x27: return keymagic::VirtualKey::Right;
        case 0x28: return keymagic::VirtualKey::Down;
        case 0x2D: return keymagic::VirtualKey::Insert;
        case 0x2E: return keymagic::VirtualKey::Delete;
        
        // Numbers
        case 0x30: return keymagic::VirtualKey::Key0;
        case 0x31: return keymagic::VirtualKey::Key1;
        case 0x32: return keymagic::VirtualKey::Key2;
        case 0x33: return keymagic::VirtualKey::Key3;
        case 0x34: return keymagic::VirtualKey::Key4;
        case 0x35: return keymagic::VirtualKey::Key5;
        case 0x36: return keymagic::VirtualKey::Key6;
        case 0x37: return keymagic::VirtualKey::Key7;
        case 0x38: return keymagic::VirtualKey::Key8;
        case 0x39: return keymagic::VirtualKey::Key9;
        
        // Letters
        case 0x41: return keymagic::VirtualKey::KeyA;
        case 0x42: return keymagic::VirtualKey::KeyB;
        case 0x43: return keymagic::VirtualKey::KeyC;
        case 0x44: return keymagic::VirtualKey::KeyD;
        case 0x45: return keymagic::VirtualKey::KeyE;
        case 0x46: return keymagic::VirtualKey::KeyF;
        case 0x47: return keymagic::VirtualKey::KeyG;
        case 0x48: return keymagic::VirtualKey::KeyH;
        case 0x49: return keymagic::VirtualKey::KeyI;
        case 0x4A: return keymagic::VirtualKey::KeyJ;
        case 0x4B: return keymagic::VirtualKey::KeyK;
        case 0x4C: return keymagic::VirtualKey::KeyL;
        case 0x4D: return keymagic::VirtualKey::KeyM;
        case 0x4E: return keymagic::VirtualKey::KeyN;
        case 0x4F: return keymagic::VirtualKey::KeyO;
        case 0x50: return keymagic::VirtualKey::KeyP;
        case 0x51: return keymagic::VirtualKey::KeyQ;
        case 0x52: return keymagic::VirtualKey::KeyR;
        case 0x53: return keymagic::VirtualKey::KeyS;
        case 0x54: return keymagic::VirtualKey::KeyT;
        case 0x55: return keymagic::VirtualKey::KeyU;
        case 0x56: return keymagic::VirtualKey::KeyV;
        case 0x57: return keymagic::VirtualKey::KeyW;
        case 0x58: return keymagic::VirtualKey::KeyX;
        case 0x59: return keymagic::VirtualKey::KeyY;
        case 0x5A: return keymagic::VirtualKey::KeyZ;
        
        // Numpad
        case 0x60: return keymagic::VirtualKey::Numpad0;
        case 0x61: return keymagic::VirtualKey::Numpad1;
        case 0x62: return keymagic::VirtualKey::Numpad2;
        case 0x63: return keymagic::VirtualKey::Numpad3;
        case 0x64: return keymagic::VirtualKey::Numpad4;
        case 0x65: return keymagic::VirtualKey::Numpad5;
        case 0x66: return keymagic::VirtualKey::Numpad6;
        case 0x67: return keymagic::VirtualKey::Numpad7;
        case 0x68: return keymagic::VirtualKey::Numpad8;
        case 0x69: return keymagic::VirtualKey::Numpad9;
        
        // Function keys
        case 0x70: return keymagic::VirtualKey::F1;
        case 0x71: return keymagic::VirtualKey::F2;
        case 0x72: return keymagic::VirtualKey::F3;
        case 0x73: return keymagic::VirtualKey::F4;
        case 0x74: return keymagic::VirtualKey::F5;
        case 0x75: return keymagic::VirtualKey::F6;
        case 0x76: return keymagic::VirtualKey::F7;
        case 0x77: return keymagic::VirtualKey::F8;
        case 0x78: return keymagic::VirtualKey::F9;
        case 0x79: return keymagic::VirtualKey::F10;
        case 0x7A: return keymagic::VirtualKey::F11;
        case 0x7B: return keymagic::VirtualKey::F12;
        
        // OEM keys
        case 0xBA: return keymagic::VirtualKey::Oem1;
        case 0xBB: return keymagic::VirtualKey::OemPlus;
        case 0xBC: return keymagic::VirtualKey::OemComma;
        case 0xBD: return keymagic::VirtualKey::OemMinus;
        case 0xBE: return keymagic::VirtualKey::OemPeriod;
        case 0xBF: return keymagic::VirtualKey::Oem2;
        case 0xC0: return keymagic::VirtualKey::Oem3;
        case 0xDB: return keymagic::VirtualKey::Oem4;
        case 0xDC: return keymagic::VirtualKey::Oem5;
        case 0xDD: return keymagic::VirtualKey::Oem6;
        case 0xDE: return keymagic::VirtualKey::Oem7;
        case 0xDF: return keymagic::VirtualKey::Oem8;
        
        // Special
        case 0x14: return keymagic::VirtualKey::Capital;
        case 0x10: return keymagic::VirtualKey::Shift;
        case 0x11: return keymagic::VirtualKey::Control;
        case 0x12: return keymagic::VirtualKey::Menu;
        case 0xA5: return keymagic::VirtualKey::Menu;  // Right Alt is same as Menu
        
        default: return keymagic::VirtualKey::Null;  // Use Null for unknown keys
    }
}

char* allocateAndCopyString(const std::string& str) {
    if (str.empty()) {
        return nullptr;
    }
    char* result = new char[str.length() + 1];
    std::strcpy(result, str.c_str());
    return result;
}

void fillProcessKeyOutput(const keymagic::Output& output, ProcessKeyOutput* cOutput) {
    if (!cOutput) return;
    
    // Map action type
    switch (output.action) {
        case keymagic::ActionType::None:
            cOutput->action_type = KeyMagicAction_None;
            break;
        case keymagic::ActionType::Insert:
            cOutput->action_type = KeyMagicAction_Insert;
            break;
        case keymagic::ActionType::BackspaceDelete:
            cOutput->action_type = KeyMagicAction_BackspaceDelete;
            break;
        case keymagic::ActionType::BackspaceDeleteAndInsert:
            cOutput->action_type = KeyMagicAction_BackspaceDeleteAndInsert;
            break;
    }
    
    cOutput->text = allocateAndCopyString(output.text);
    cOutput->delete_count = output.deleteCount;
    cOutput->composing_text = allocateAndCopyString(output.composingText);
    cOutput->is_processed = output.isProcessed ? 1 : 0;
}

} // anonymous namespace

extern "C" {

// Engine management
KEYMAGIC_API EngineHandle* keymagic_engine_new(void) {
    std::lock_guard<std::mutex> lock(g_handleMutex);
    
    auto engine = std::make_unique<keymagic::Engine>();
    auto handle = reinterpret_cast<EngineHandle*>(engine.get());
    g_engines[handle] = std::move(engine);
    
    return handle;
}

KEYMAGIC_API void keymagic_engine_free(EngineHandle* handle) {
    if (!handle) return;
    
    std::lock_guard<std::mutex> lock(g_handleMutex);
    g_engines.erase(handle);
}

// Keyboard loading
KEYMAGIC_API KeyMagicResult keymagic_engine_load_keyboard(EngineHandle* handle, const char* km2_path) {
    if (!handle || !km2_path) {
        return KeyMagicResult_ErrorInvalidParameter;
    }
    
    std::lock_guard<std::mutex> lock(g_handleMutex);
    auto it = g_engines.find(handle);
    if (it == g_engines.end()) {
        return KeyMagicResult_ErrorInvalidHandle;
    }
    
    auto result = it->second->loadKeyboardFromPath(km2_path);
    
    switch (result) {
        case keymagic::Result::Success:
            return KeyMagicResult_Success;
        case keymagic::Result::ErrorFileNotFound:
        case keymagic::Result::ErrorInvalidFormat:
            return KeyMagicResult_ErrorNoKeyboard;
        default:
            return KeyMagicResult_ErrorEngineFailure;
    }
}

KEYMAGIC_API KeyMagicResult keymagic_engine_load_keyboard_from_memory(
    EngineHandle* handle, 
    const uint8_t* km2_data, 
    size_t data_len
) {
    if (!handle || !km2_data || data_len == 0) {
        return KeyMagicResult_ErrorInvalidParameter;
    }
    
    std::lock_guard<std::mutex> lock(g_handleMutex);
    auto it = g_engines.find(handle);
    if (it == g_engines.end()) {
        return KeyMagicResult_ErrorInvalidHandle;
    }
    
    auto km2 = keymagic::KM2Loader::loadFromMemory(km2_data, data_len);
    if (!km2) {
        return KeyMagicResult_ErrorNoKeyboard;
    }
    
    auto result = it->second->loadKeyboard(std::move(km2));
    
    switch (result) {
        case keymagic::Result::Success:
            return KeyMagicResult_Success;
        default:
            return KeyMagicResult_ErrorEngineFailure;
    }
}

// Key processing
KEYMAGIC_API KeyMagicResult keymagic_engine_process_key(
    EngineHandle* handle,
    KeyMagicVirtualKey key_code,
    char character,
    int shift,
    int ctrl,
    int alt,
    int caps_lock,
    ProcessKeyOutput* output
) {
    if (!handle || !output) {
        return KeyMagicResult_ErrorInvalidParameter;
    }
    
    std::lock_guard<std::mutex> lock(g_handleMutex);
    auto it = g_engines.find(handle);
    if (it == g_engines.end()) {
        return KeyMagicResult_ErrorInvalidHandle;
    }
    
    keymagic::Input input;
    input.keyCode = static_cast<keymagic::VirtualKey>(key_code);
    if (character != 0) {
        input.character = static_cast<char32_t>(static_cast<unsigned char>(character));
    }
    input.modifiers.shift = shift != 0;
    input.modifiers.ctrl = ctrl != 0;
    input.modifiers.alt = alt != 0;
    input.modifiers.capsLock = caps_lock != 0;
    
    auto result = it->second->processKey(input);
    fillProcessKeyOutput(result, output);
    
    return KeyMagicResult_Success;
}

// Windows-specific key processing with VK codes
KEYMAGIC_API KeyMagicResult keymagic_engine_process_key_win(
    EngineHandle* handle,
    int vk_code,
    char character,
    int shift,
    int ctrl,
    int alt,
    int caps_lock,
    ProcessKeyOutput* output
) {
    if (!handle || !output) {
        return KeyMagicResult_ErrorInvalidParameter;
    }
    
    std::lock_guard<std::mutex> lock(g_handleMutex);
    auto it = g_engines.find(handle);
    if (it == g_engines.end()) {
        return KeyMagicResult_ErrorInvalidHandle;
    }
    
    keymagic::Input input;
    input.keyCode = windowsVkToInternal(vk_code);
    if (character != 0) {
        input.character = static_cast<char32_t>(static_cast<unsigned char>(character));
    }
    input.modifiers.shift = shift != 0;
    input.modifiers.ctrl = ctrl != 0;
    input.modifiers.alt = alt != 0;
    input.modifiers.capsLock = caps_lock != 0;
    
    auto result = it->second->processKey(input);
    fillProcessKeyOutput(result, output);
    
    return KeyMagicResult_Success;
}

// Test mode - non-modifying key processing for preview
KEYMAGIC_API KeyMagicResult keymagic_engine_process_key_test_win(
    EngineHandle* handle,
    int vk_code,
    char character,
    int shift,
    int ctrl,
    int alt,
    int caps_lock,
    ProcessKeyOutput* output
) {
    if (!handle || !output) {
        return KeyMagicResult_ErrorInvalidParameter;
    }
    
    std::lock_guard<std::mutex> lock(g_handleMutex);
    auto it = g_engines.find(handle);
    if (it == g_engines.end()) {
        return KeyMagicResult_ErrorInvalidHandle;
    }
    
    // Save current state
    auto savedComposition = it->second->getComposingText();
    
    keymagic::Input input;
    input.keyCode = windowsVkToInternal(vk_code);
    if (character != 0) {
        input.character = static_cast<char32_t>(static_cast<unsigned char>(character));
    }
    input.modifiers.shift = shift != 0;
    input.modifiers.ctrl = ctrl != 0;
    input.modifiers.alt = alt != 0;
    input.modifiers.capsLock = caps_lock != 0;
    
    auto result = it->second->processKey(input);
    fillProcessKeyOutput(result, output);
    
    // Restore state
    it->second->setComposingText(savedComposition);
    
    return KeyMagicResult_Success;
}

// Memory management
KEYMAGIC_API void keymagic_free_string(char* s) {
    delete[] s;
}

// Engine control
KEYMAGIC_API KeyMagicResult keymagic_engine_reset(EngineHandle* handle) {
    if (!handle) {
        return KeyMagicResult_ErrorInvalidParameter;
    }
    
    std::lock_guard<std::mutex> lock(g_handleMutex);
    auto it = g_engines.find(handle);
    if (it == g_engines.end()) {
        return KeyMagicResult_ErrorInvalidHandle;
    }
    
    it->second->reset();
    return KeyMagicResult_Success;
}

KEYMAGIC_API char* keymagic_engine_get_composition(EngineHandle* handle) {
    if (!handle) {
        return nullptr;
    }
    
    std::lock_guard<std::mutex> lock(g_handleMutex);
    auto it = g_engines.find(handle);
    if (it == g_engines.end()) {
        return nullptr;
    }
    
    std::u16string composing = it->second->getComposingText();
    return allocateAndCopyString(keymagic::utils::utf16ToUtf8(composing));
}

KEYMAGIC_API KeyMagicResult keymagic_engine_set_composition(EngineHandle* handle, const char* text) {
    if (!handle) {
        return KeyMagicResult_ErrorInvalidParameter;
    }
    
    std::lock_guard<std::mutex> lock(g_handleMutex);
    auto it = g_engines.find(handle);
    if (it == g_engines.end()) {
        return KeyMagicResult_ErrorInvalidHandle;
    }
    
    it->second->setComposingText(text ? keymagic::utils::utf8ToUtf16(text) : std::u16string());
    return KeyMagicResult_Success;
}

// Version info
KEYMAGIC_API const char* keymagic_get_version(void) {
    return "1.0.0";
}

// Hotkey parsing
KEYMAGIC_API int keymagic_parse_hotkey(const char* hotkey_str, KeyMagicHotkeyInfo* info) {
    if (!hotkey_str || !info) {
        return 0;
    }
    
    // Initialize info
    info->key_code = KeyMagic_VK_Null;
    info->ctrl = 0;
    info->alt = 0;
    info->shift = 0;
    info->meta = 0;
    
    std::string str(hotkey_str);
    
    // Trim whitespace
    str.erase(0, str.find_first_not_of(" \t\r\n"));
    str.erase(str.find_last_not_of(" \t\r\n") + 1);
    
    if (str.empty()) {
        return 0;
    }
    
    // Convert to uppercase for case-insensitive parsing
    std::transform(str.begin(), str.end(), str.begin(), ::toupper);
    
    // Split by + or space
    std::vector<std::string> parts;
    std::string current;
    for (char c : str) {
        if (c == '+' || c == ' ') {
            if (!current.empty()) {
                parts.push_back(current);
                current.clear();
            }
        } else {
            current += c;
        }
    }
    if (!current.empty()) {
        parts.push_back(current);
    }
    
    if (parts.empty()) {
        return 0;
    }
    
    // Parse each part
    int key_count = 0;
    for (const auto& part : parts) {
        // Check for modifiers
        if (part == "CTRL" || part == "CONTROL") {
            info->ctrl = 1;
        } else if (part == "ALT" || part == "OPTION") {
            info->alt = 1;
        } else if (part == "SHIFT") {
            info->shift = 1;
        } else if (part == "META" || part == "CMD" || part == "COMMAND" || 
                   part == "WIN" || part == "SUPER") {
            info->meta = 1;
        } else {
            // Parse as key
            if (key_count > 0) {
                // Multiple keys specified - error
                return 0;
            }
            
            KeyMagicVirtualKey vk_code = KeyMagic_VK_Null;  // Will store VirtualKey enum value
            
            // Single character keys
            if (part.length() == 1) {
                char ch = part[0];
                if (ch >= 'A' && ch <= 'Z') {
                    vk_code = static_cast<KeyMagicVirtualKey>(static_cast<int>(keymagic::VirtualKey::KeyA) + (ch - 'A'));
                } else if (ch >= '0' && ch <= '9') {
                    vk_code = static_cast<KeyMagicVirtualKey>(static_cast<int>(keymagic::VirtualKey::Key0) + (ch - '0'));
                } else {
                    // Special single characters (matching Rust OEM key mappings)
                    switch (ch) {
                        case '=': vk_code = static_cast<KeyMagicVirtualKey>(keymagic::VirtualKey::OemPlus); break;
                        case '-': vk_code = static_cast<KeyMagicVirtualKey>(keymagic::VirtualKey::OemMinus); break;
                        case ',': vk_code = static_cast<KeyMagicVirtualKey>(keymagic::VirtualKey::OemComma); break;
                        case '.': vk_code = static_cast<KeyMagicVirtualKey>(keymagic::VirtualKey::OemPeriod); break;
                        case ';': vk_code = static_cast<KeyMagicVirtualKey>(keymagic::VirtualKey::Oem1); break;
                        case '/': vk_code = static_cast<KeyMagicVirtualKey>(keymagic::VirtualKey::Oem2); break;
                        case '`': vk_code = static_cast<KeyMagicVirtualKey>(keymagic::VirtualKey::Oem3); break;
                        case '[': vk_code = static_cast<KeyMagicVirtualKey>(keymagic::VirtualKey::Oem4); break;
                        case '\\': vk_code = static_cast<KeyMagicVirtualKey>(keymagic::VirtualKey::Oem5); break;
                        case ']': vk_code = static_cast<KeyMagicVirtualKey>(keymagic::VirtualKey::Oem6); break;
                        case '\'': vk_code = static_cast<KeyMagicVirtualKey>(keymagic::VirtualKey::Oem7); break;
                    }
                }
            } else {
                // Multi-character key names
                if (part == "SPACE") {
                    vk_code = static_cast<KeyMagicVirtualKey>(keymagic::VirtualKey::Space);
                } else if (part == "ENTER" || part == "RETURN") {
                    vk_code = static_cast<KeyMagicVirtualKey>(keymagic::VirtualKey::Return);
                } else if (part == "TAB") {
                    vk_code = static_cast<KeyMagicVirtualKey>(keymagic::VirtualKey::Tab);
                } else if (part == "BACKSPACE" || part == "BACK") {
                    // Note: "DELETE" is not mapped to Back in Rust
                    vk_code = static_cast<KeyMagicVirtualKey>(keymagic::VirtualKey::Back);
                } else if (part == "DELETE") {
                    // This should be VK_DELETE, not BACK (matching Rust behavior)
                    vk_code = static_cast<KeyMagicVirtualKey>(keymagic::VirtualKey::Delete);
                } else if (part == "ESCAPE" || part == "ESC") {
                    vk_code = static_cast<KeyMagicVirtualKey>(keymagic::VirtualKey::Escape);
                } else if (part == "CAPSLOCK" || part == "CAPS" || part == "CAPITAL") {
                    vk_code = static_cast<KeyMagicVirtualKey>(keymagic::VirtualKey::Capital);
                } else if (part == "INSERT" || part == "INS") {
                    vk_code = static_cast<KeyMagicVirtualKey>(keymagic::VirtualKey::Insert);
                } else if (part == "DEL") {
                    vk_code = static_cast<KeyMagicVirtualKey>(keymagic::VirtualKey::Delete);
                } else if (part == "HOME") {
                    vk_code = static_cast<KeyMagicVirtualKey>(keymagic::VirtualKey::Home);
                } else if (part == "END") {
                    vk_code = static_cast<KeyMagicVirtualKey>(keymagic::VirtualKey::End);
                } else if (part == "PAGEUP" || part == "PGUP" || part == "PRIOR") {
                    vk_code = static_cast<KeyMagicVirtualKey>(keymagic::VirtualKey::Prior);
                } else if (part == "PAGEDOWN" || part == "PGDN" || part == "NEXT") {
                    vk_code = static_cast<KeyMagicVirtualKey>(keymagic::VirtualKey::Next);
                } else if (part == "LEFT") {
                    vk_code = static_cast<KeyMagicVirtualKey>(keymagic::VirtualKey::Left);
                } else if (part == "UP") {
                    vk_code = static_cast<KeyMagicVirtualKey>(keymagic::VirtualKey::Up);
                } else if (part == "RIGHT") {
                    vk_code = static_cast<KeyMagicVirtualKey>(keymagic::VirtualKey::Right);
                } else if (part == "DOWN") {
                    vk_code = static_cast<KeyMagicVirtualKey>(keymagic::VirtualKey::Down);
                } else if (part == "PLUS") {
                    vk_code = static_cast<KeyMagicVirtualKey>(keymagic::VirtualKey::OemPlus);
                } else if (part == "MINUS") {
                    vk_code = static_cast<KeyMagicVirtualKey>(keymagic::VirtualKey::OemMinus);
                } else if (part == "COMMA") {
                    vk_code = static_cast<KeyMagicVirtualKey>(keymagic::VirtualKey::OemComma);
                } else if (part == "PERIOD") {
                    vk_code = static_cast<KeyMagicVirtualKey>(keymagic::VirtualKey::OemPeriod);
                } else if (part == "SEMICOLON") {
                    vk_code = static_cast<KeyMagicVirtualKey>(keymagic::VirtualKey::Oem1);
                } else if (part == "SLASH") {
                    vk_code = static_cast<KeyMagicVirtualKey>(keymagic::VirtualKey::Oem2);
                } else if (part == "GRAVE") {
                    vk_code = static_cast<KeyMagicVirtualKey>(keymagic::VirtualKey::Oem3);
                } else if (part == "LEFTBRACKET" || part == "LBRACKET") {
                    vk_code = static_cast<KeyMagicVirtualKey>(keymagic::VirtualKey::Oem4);
                } else if (part == "BACKSLASH") {
                    vk_code = static_cast<KeyMagicVirtualKey>(keymagic::VirtualKey::Oem5);
                } else if (part == "RIGHTBRACKET" || part == "RBRACKET") {
                    vk_code = static_cast<KeyMagicVirtualKey>(keymagic::VirtualKey::Oem6);
                } else if (part == "QUOTE" || part == "APOSTROPHE") {
                    vk_code = static_cast<KeyMagicVirtualKey>(keymagic::VirtualKey::Oem7);
                } else if (part.substr(0, 1) == "F" && part.length() <= 3) {
                    // Function keys F1-F12 (matching Rust - only supports F1-F12)
                    std::string fnum = part.substr(1);
                    try {
                        int num = std::stoi(fnum);
                        if (num >= 1 && num <= 12) {
                            vk_code = static_cast<KeyMagicVirtualKey>(static_cast<int>(keymagic::VirtualKey::F1) + (num - 1));
                        }
                    } catch (...) {
                        // Not a valid function key
                    }
                } else if (part.substr(0, 6) == "NUMPAD" && part.length() == 7) {
                    // Numpad keys
                    char numpad_ch = part[6];
                    if (numpad_ch >= '0' && numpad_ch <= '9') {
                        vk_code = static_cast<KeyMagicVirtualKey>(static_cast<int>(keymagic::VirtualKey::Numpad0) + (numpad_ch - '0'));
                    }
                }
            }
            
            if (vk_code == KeyMagic_VK_Null) {
                // Unknown key
                return 0;
            }
            
            info->key_code = vk_code;
            key_count++;
        }
    }
    
    // Must have exactly one key
    return (info->key_code != KeyMagic_VK_Null) ? 1 : 0;
}

// KM2 file loading and metadata access
KEYMAGIC_API Km2FileHandle* keymagic_km2_load(const char* path) {
    if (!path) {
        return nullptr;
    }
    
    auto km2 = keymagic::KM2Loader::loadFromFile(path);
    if (!km2) {
        return nullptr;
    }
    
    auto handle = reinterpret_cast<Km2FileHandle*>(km2.get());
    
    std::lock_guard<std::mutex> lock(g_handleMutex);
    g_km2Files[handle] = std::move(km2);
    
    return handle;
}

KEYMAGIC_API void keymagic_km2_free(Km2FileHandle* handle) {
    if (!handle) return;
    
    std::lock_guard<std::mutex> lock(g_handleMutex);
    g_km2Files.erase(handle);
}

KEYMAGIC_API char* keymagic_km2_get_name(Km2FileHandle* handle) {
    if (!handle) return nullptr;
    
    std::lock_guard<std::mutex> lock(g_handleMutex);
    auto it = g_km2Files.find(handle);
    if (it == g_km2Files.end()) {
        return nullptr;
    }
    
    return allocateAndCopyString(it->second->metadata.getName());
}

KEYMAGIC_API char* keymagic_km2_get_description(Km2FileHandle* handle) {
    if (!handle) return nullptr;
    
    std::lock_guard<std::mutex> lock(g_handleMutex);
    auto it = g_km2Files.find(handle);
    if (it == g_km2Files.end()) {
        return nullptr;
    }
    
    return allocateAndCopyString(it->second->metadata.getDescription());
}

KEYMAGIC_API char* keymagic_km2_get_hotkey(Km2FileHandle* handle) {
    if (!handle) return nullptr;
    
    std::lock_guard<std::mutex> lock(g_handleMutex);
    auto it = g_km2Files.find(handle);
    if (it == g_km2Files.end()) {
        return nullptr;
    }
    
    return allocateAndCopyString(it->second->metadata.getHotkey());
}

KEYMAGIC_API size_t keymagic_km2_get_icon_data(Km2FileHandle* handle, uint8_t* buffer, size_t buffer_size) {
    if (!handle) return 0;
    
    std::lock_guard<std::mutex> lock(g_handleMutex);
    auto it = g_km2Files.find(handle);
    if (it == g_km2Files.end()) {
        return 0;
    }
    
    const auto* iconData = it->second->metadata.getIcon();
    if (!iconData || iconData->empty()) {
        return 0;
    }
    
    if (!buffer) {
        // Return required size
        return iconData->size();
    }
    
    size_t copySize = std::min(iconData->size(), buffer_size);
    std::memcpy(buffer, iconData->data(), copySize);
    return copySize;
}

// Virtual key utilities
KEYMAGIC_API char* keymagic_virtual_key_to_string(KeyMagicVirtualKey key_code) {
    std::string result;
    
    switch (key_code) {
        // Control keys
        case KeyMagic_VK_Back: result = "BACK"; break;
        case KeyMagic_VK_Tab: result = "TAB"; break;
        case KeyMagic_VK_Return: result = "RETURN"; break;
        case KeyMagic_VK_Shift: result = "SHIFT"; break;
        case KeyMagic_VK_Control: result = "CONTROL"; break;
        case KeyMagic_VK_Menu: result = "MENU"; break;  // Alt key
        case KeyMagic_VK_Pause: result = "PAUSE"; break;
        case KeyMagic_VK_Capital: result = "CAPITAL"; break;  // Caps Lock
        case KeyMagic_VK_Escape: result = "ESCAPE"; break;
        case KeyMagic_VK_Space: result = "SPACE"; break;
        
        // Navigation keys
        case KeyMagic_VK_Prior: result = "PRIOR"; break;  // Page Up
        case KeyMagic_VK_Next: result = "NEXT"; break;   // Page Down
        case KeyMagic_VK_End: result = "END"; break;
        case KeyMagic_VK_Home: result = "HOME"; break;
        case KeyMagic_VK_Left: result = "LEFT"; break;
        case KeyMagic_VK_Up: result = "UP"; break;
        case KeyMagic_VK_Right: result = "RIGHT"; break;
        case KeyMagic_VK_Down: result = "DOWN"; break;
        case KeyMagic_VK_Insert: result = "INSERT"; break;
        case KeyMagic_VK_Delete: result = "DELETE"; break;
        
        // Number keys
        case KeyMagic_VK_Key0: result = "0"; break;
        case KeyMagic_VK_Key1: result = "1"; break;
        case KeyMagic_VK_Key2: result = "2"; break;
        case KeyMagic_VK_Key3: result = "3"; break;
        case KeyMagic_VK_Key4: result = "4"; break;
        case KeyMagic_VK_Key5: result = "5"; break;
        case KeyMagic_VK_Key6: result = "6"; break;
        case KeyMagic_VK_Key7: result = "7"; break;
        case KeyMagic_VK_Key8: result = "8"; break;
        case KeyMagic_VK_Key9: result = "9"; break;
        
        // Letter keys
        case KeyMagic_VK_KeyA: result = "A"; break;
        case KeyMagic_VK_KeyB: result = "B"; break;
        case KeyMagic_VK_KeyC: result = "C"; break;
        case KeyMagic_VK_KeyD: result = "D"; break;
        case KeyMagic_VK_KeyE: result = "E"; break;
        case KeyMagic_VK_KeyF: result = "F"; break;
        case KeyMagic_VK_KeyG: result = "G"; break;
        case KeyMagic_VK_KeyH: result = "H"; break;
        case KeyMagic_VK_KeyI: result = "I"; break;
        case KeyMagic_VK_KeyJ: result = "J"; break;
        case KeyMagic_VK_KeyK: result = "K"; break;
        case KeyMagic_VK_KeyL: result = "L"; break;
        case KeyMagic_VK_KeyM: result = "M"; break;
        case KeyMagic_VK_KeyN: result = "N"; break;
        case KeyMagic_VK_KeyO: result = "O"; break;
        case KeyMagic_VK_KeyP: result = "P"; break;
        case KeyMagic_VK_KeyQ: result = "Q"; break;
        case KeyMagic_VK_KeyR: result = "R"; break;
        case KeyMagic_VK_KeyS: result = "S"; break;
        case KeyMagic_VK_KeyT: result = "T"; break;
        case KeyMagic_VK_KeyU: result = "U"; break;
        case KeyMagic_VK_KeyV: result = "V"; break;
        case KeyMagic_VK_KeyW: result = "W"; break;
        case KeyMagic_VK_KeyX: result = "X"; break;
        case KeyMagic_VK_KeyY: result = "Y"; break;
        case KeyMagic_VK_KeyZ: result = "Z"; break;
        
        // Numpad keys
        case KeyMagic_VK_Numpad0: result = "NUMPAD0"; break;
        case KeyMagic_VK_Numpad1: result = "NUMPAD1"; break;
        case KeyMagic_VK_Numpad2: result = "NUMPAD2"; break;
        case KeyMagic_VK_Numpad3: result = "NUMPAD3"; break;
        case KeyMagic_VK_Numpad4: result = "NUMPAD4"; break;
        case KeyMagic_VK_Numpad5: result = "NUMPAD5"; break;
        case KeyMagic_VK_Numpad6: result = "NUMPAD6"; break;
        case KeyMagic_VK_Numpad7: result = "NUMPAD7"; break;
        case KeyMagic_VK_Numpad8: result = "NUMPAD8"; break;
        case KeyMagic_VK_Numpad9: result = "NUMPAD9"; break;
        case KeyMagic_VK_Multiply: result = "MULTIPLY"; break;
        case KeyMagic_VK_Add: result = "ADD"; break;
        case KeyMagic_VK_Separator: result = "SEPARATOR"; break;
        case KeyMagic_VK_Subtract: result = "SUBTRACT"; break;
        case KeyMagic_VK_Decimal: result = "DECIMAL"; break;
        case KeyMagic_VK_Divide: result = "DIVIDE"; break;
        
        // Function keys
        case KeyMagic_VK_F1: result = "F1"; break;
        case KeyMagic_VK_F2: result = "F2"; break;
        case KeyMagic_VK_F3: result = "F3"; break;
        case KeyMagic_VK_F4: result = "F4"; break;
        case KeyMagic_VK_F5: result = "F5"; break;
        case KeyMagic_VK_F6: result = "F6"; break;
        case KeyMagic_VK_F7: result = "F7"; break;
        case KeyMagic_VK_F8: result = "F8"; break;
        case KeyMagic_VK_F9: result = "F9"; break;
        case KeyMagic_VK_F10: result = "F10"; break;
        case KeyMagic_VK_F11: result = "F11"; break;
        case KeyMagic_VK_F12: result = "F12"; break;
        
        // Modifier keys
        case KeyMagic_VK_LShift: result = "LSHIFT"; break;
        case KeyMagic_VK_RShift: result = "RSHIFT"; break;
        case KeyMagic_VK_LControl: result = "LCONTROL"; break;
        case KeyMagic_VK_RControl: result = "RCONTROL"; break;
        case KeyMagic_VK_LMenu: result = "LMENU"; break;  // Left Alt
        case KeyMagic_VK_RMenu: result = "RMENU"; break;  // Right Alt
        
        // OEM keys
        case KeyMagic_VK_Oem1: result = "OEM_1"; break;      // ; :
        case KeyMagic_VK_OemPlus: result = "OEM_PLUS"; break;   // = +
        case KeyMagic_VK_OemComma: result = "OEM_COMMA"; break;  // , <
        case KeyMagic_VK_OemMinus: result = "OEM_MINUS"; break;  // - _
        case KeyMagic_VK_OemPeriod: result = "OEM_PERIOD"; break; // . >
        case KeyMagic_VK_Oem2: result = "OEM_2"; break;      // / ?
        case KeyMagic_VK_Oem3: result = "OEM_3"; break;      // ` ~
        case KeyMagic_VK_Oem4: result = "OEM_4"; break;      // [ {
        case KeyMagic_VK_Oem5: result = "OEM_5"; break;      // \ |
        case KeyMagic_VK_Oem6: result = "OEM_6"; break;      // ] }
        case KeyMagic_VK_Oem7: result = "OEM_7"; break;      // ' "
        case KeyMagic_VK_Oem8: result = "OEM_8"; break;
        case KeyMagic_VK_Oem102: result = "OEM_102"; break;    // < > on UK/Germany keyboards
        
        default: 
            // For unknown keys, return the numeric value
            result = "VK_" + std::to_string(static_cast<int>(key_code));
            break;
    }
    
    return allocateAndCopyString(result);
}

} // extern "C"