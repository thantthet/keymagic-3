#ifndef KEYMAGIC_H
#define KEYMAGIC_H

#include "types.h"
#include "virtual_keys.h"
#include "km2_format.h"
#include <memory>
#include <string>

// Export/import macros for shared library
#ifdef _WIN32
    #ifdef KEYMAGIC_CORE_EXPORTS
        #define KEYMAGIC_API __declspec(dllexport)
    #elif defined(KEYMAGIC_CORE_IMPORTS)
        #define KEYMAGIC_API __declspec(dllimport)
    #else
        #define KEYMAGIC_API
    #endif
#else
    #define KEYMAGIC_API __attribute__((visibility("default")))
#endif

// C API compatibility - wrap in extern "C" when included from C++
#ifdef __cplusplus
extern "C" {
#endif

// Opaque handle to the engine (for C API)
typedef struct EngineHandle EngineHandle;

// Result codes (C-compatible)
typedef enum {
    KeyMagicResult_Success = 0,
    KeyMagicResult_ErrorInvalidHandle = -1,
    KeyMagicResult_ErrorInvalidParameter = -2,
    KeyMagicResult_ErrorEngineFailure = -3,
    KeyMagicResult_ErrorUtf8Conversion = -4,
    KeyMagicResult_ErrorNoKeyboard = -5,
} KeyMagicResult;

// Output structure from key processing (C-compatible)
typedef struct {
    int action_type;      // 0=None, 1=Insert, 2=BackspaceDelete, 3=BackspaceDeleteAndInsert
    char* text;           // UTF-8 encoded, null-terminated (needs to be freed)
    int delete_count;     // Number of characters to delete
    char* composing_text; // UTF-8 encoded, null-terminated (needs to be freed)
    int is_processed;     // 0=false, 1=true
} ProcessKeyOutput;

// Hotkey info structure (C-compatible)
typedef struct {
    int key_code;       // VirtualKey as int
    int ctrl;           // 0 or 1
    int alt;            // 0 or 1
    int shift;          // 0 or 1
    int meta;           // 0 or 1
} HotkeyInfo;

// KM2 file handle (C-compatible)
typedef struct Km2FileHandle Km2FileHandle;

// ============================================================================
// C API Functions - These match the Rust FFI interface exactly
// ============================================================================

// Engine management
KEYMAGIC_API EngineHandle* keymagic_engine_new(void);
KEYMAGIC_API void keymagic_engine_free(EngineHandle* handle);

// Keyboard loading
KEYMAGIC_API KeyMagicResult keymagic_engine_load_keyboard(EngineHandle* handle, const char* km2_path);
KEYMAGIC_API KeyMagicResult keymagic_engine_load_keyboard_from_memory(
    EngineHandle* handle, 
    const uint8_t* km2_data, 
    size_t data_len
);

// Key processing
KEYMAGIC_API KeyMagicResult keymagic_engine_process_key(
    EngineHandle* handle,
    int key_code,
    char character,
    int shift,
    int ctrl,
    int alt,
    int caps_lock,
    ProcessKeyOutput* output
);

// Memory management
KEYMAGIC_API void keymagic_free_string(char* s);

// Engine control
KEYMAGIC_API KeyMagicResult keymagic_engine_reset(EngineHandle* handle);
KEYMAGIC_API char* keymagic_engine_get_composition(EngineHandle* handle);
KEYMAGIC_API KeyMagicResult keymagic_engine_set_composition(EngineHandle* handle, const char* text);

// Version info
KEYMAGIC_API const char* keymagic_get_version(void);

// Windows-specific key processing with VK codes
KEYMAGIC_API KeyMagicResult keymagic_engine_process_key_win(
    EngineHandle* handle,
    int vk_code,          // Windows VK code (e.g., 0x41 for VK_A)
    char character,
    int shift,
    int ctrl,
    int alt,
    int caps_lock,
    ProcessKeyOutput* output
);

// Test mode - non-modifying key processing for preview
KEYMAGIC_API KeyMagicResult keymagic_engine_process_key_test_win(
    EngineHandle* handle,
    int vk_code,          // Windows VK code (e.g., 0x41 for VK_A)
    char character,
    int shift,
    int ctrl,
    int alt,
    int caps_lock,
    ProcessKeyOutput* output
);

// Hotkey parsing
KEYMAGIC_API int keymagic_parse_hotkey(const char* hotkey_str, HotkeyInfo* info);

// KM2 file loading and metadata access
KEYMAGIC_API Km2FileHandle* keymagic_km2_load(const char* path);
KEYMAGIC_API void keymagic_km2_free(Km2FileHandle* handle);
KEYMAGIC_API char* keymagic_km2_get_name(Km2FileHandle* handle);
KEYMAGIC_API char* keymagic_km2_get_description(Km2FileHandle* handle);
KEYMAGIC_API char* keymagic_km2_get_hotkey(Km2FileHandle* handle);
KEYMAGIC_API size_t keymagic_km2_get_icon_data(Km2FileHandle* handle, uint8_t* buffer, size_t buffer_size);

// Virtual key utilities
KEYMAGIC_API char* keymagic_virtual_key_to_string(int key_code);

#ifdef __cplusplus
}
#endif

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