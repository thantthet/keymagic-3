/**
 * KeyMagic Core FFI Interface
 * 
 * Unified C header for KeyMagic Core library that can be used across all platforms
 * (Windows, macOS, Linux/IBus, etc.)
 * 
 * This header defines the standard FFI interface to the KeyMagic Core library,
 * providing keyboard layout loading, key processing, and metadata access.
 */

#ifndef KEYMAGIC_CORE_H
#define KEYMAGIC_CORE_H

#ifdef __cplusplus
extern "C" {
#endif

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

/* Platform-specific export/import macros */
#ifdef _WIN32
    #ifdef KEYMAGIC_CORE_EXPORTS
        #define KEYMAGIC_API __declspec(dllexport)
    #elif defined(KEYMAGIC_CORE_IMPORTS)
        #define KEYMAGIC_API __declspec(dllimport)
    #else
        #define KEYMAGIC_API
    #endif
#else
    #ifdef KEYMAGIC_CORE_EXPORTS
        #define KEYMAGIC_API __attribute__((visibility("default")))
    #else
        #define KEYMAGIC_API
    #endif
#endif

/* ============================================================================
 * Type Definitions
 * ============================================================================ */

/* Opaque handle to the KeyMagic engine */
typedef struct EngineHandle EngineHandle;

/* Opaque handle to a loaded KM2 file */
typedef struct Km2FileHandle Km2FileHandle;

/* Result codes for API functions */
typedef enum {
    KeyMagicResult_Success = 0,
    KeyMagicResult_ErrorInvalidHandle = -1,
    KeyMagicResult_ErrorInvalidParameter = -2,
    KeyMagicResult_ErrorEngineFailure = -3,
    KeyMagicResult_ErrorUtf8Conversion = -4,
    KeyMagicResult_ErrorNoKeyboard = -5,
    KeyMagicResult_ErrorFileNotFound = -6,
    KeyMagicResult_ErrorInvalidFormat = -7,
    KeyMagicResult_ErrorOutOfMemory = -8
} KeyMagicResult;

/* Action types for key processing output */
typedef enum {
    KeyMagicAction_None = 0,
    KeyMagicAction_Insert = 1,
    KeyMagicAction_BackspaceDelete = 2,
    KeyMagicAction_BackspaceDeleteAndInsert = 3
} KeyMagicActionType;

/* Output structure from key processing */
typedef struct {
    KeyMagicActionType action_type;  /* Action to perform */
    char* text;                      /* UTF-8 text to insert (may be NULL, caller must free) */
    int delete_count;                /* Number of characters to delete */
    char* composing_text;            /* Current composing text (may be NULL, caller must free) */
    int is_processed;                /* 0=false, 1=true - whether engine handled the key */
} ProcessKeyOutput;

/* Hotkey information structure */
typedef struct {
    KeyMagicVirtualKey key_code;  /* VirtualKey enum value (not platform VK code) */
    int ctrl;                     /* 0 or 1 */
    int alt;                      /* 0 or 1 */
    int shift;                    /* 0 or 1 */
    int meta;                     /* 0 or 1 (Windows/Command/Super key) */
} HotkeyInfo;

/* VirtualKey enum - Internal key codes (not Windows VK codes) */
typedef enum {
    KeyMagic_VK_Null = 1,
    KeyMagic_VK_Back = 2,
    KeyMagic_VK_Tab = 3,
    KeyMagic_VK_Return = 4,
    KeyMagic_VK_Shift = 5,
    KeyMagic_VK_Control = 6,
    KeyMagic_VK_Menu = 7,  /* Alt key */
    KeyMagic_VK_Pause = 8,
    KeyMagic_VK_Capital = 9,  /* Caps Lock */
    KeyMagic_VK_Kanji = 10,
    KeyMagic_VK_Escape = 11,
    KeyMagic_VK_Space = 12,
    KeyMagic_VK_Prior = 13,  /* Page Up */
    KeyMagic_VK_Next = 14,   /* Page Down */
    KeyMagic_VK_Delete = 15,
    
    /* Number keys */
    KeyMagic_VK_Key0 = 16,
    KeyMagic_VK_Key1 = 17,
    KeyMagic_VK_Key2 = 18,
    KeyMagic_VK_Key3 = 19,
    KeyMagic_VK_Key4 = 20,
    KeyMagic_VK_Key5 = 21,
    KeyMagic_VK_Key6 = 22,
    KeyMagic_VK_Key7 = 23,
    KeyMagic_VK_Key8 = 24,
    KeyMagic_VK_Key9 = 25,
    
    /* Letter keys */
    KeyMagic_VK_KeyA = 26,
    KeyMagic_VK_KeyB = 27,
    KeyMagic_VK_KeyC = 28,
    KeyMagic_VK_KeyD = 29,
    KeyMagic_VK_KeyE = 30,
    KeyMagic_VK_KeyF = 31,
    KeyMagic_VK_KeyG = 32,
    KeyMagic_VK_KeyH = 33,
    KeyMagic_VK_KeyI = 34,
    KeyMagic_VK_KeyJ = 35,
    KeyMagic_VK_KeyK = 36,
    KeyMagic_VK_KeyL = 37,
    KeyMagic_VK_KeyM = 38,
    KeyMagic_VK_KeyN = 39,
    KeyMagic_VK_KeyO = 40,
    KeyMagic_VK_KeyP = 41,
    KeyMagic_VK_KeyQ = 42,
    KeyMagic_VK_KeyR = 43,
    KeyMagic_VK_KeyS = 44,
    KeyMagic_VK_KeyT = 45,
    KeyMagic_VK_KeyU = 46,
    KeyMagic_VK_KeyV = 47,
    KeyMagic_VK_KeyW = 48,
    KeyMagic_VK_KeyX = 49,
    KeyMagic_VK_KeyY = 50,
    KeyMagic_VK_KeyZ = 51,
    
    /* Numpad */
    KeyMagic_VK_Numpad0 = 52,
    KeyMagic_VK_Numpad1 = 53,
    KeyMagic_VK_Numpad2 = 54,
    KeyMagic_VK_Numpad3 = 55,
    KeyMagic_VK_Numpad4 = 56,
    KeyMagic_VK_Numpad5 = 57,
    KeyMagic_VK_Numpad6 = 58,
    KeyMagic_VK_Numpad7 = 59,
    KeyMagic_VK_Numpad8 = 60,
    KeyMagic_VK_Numpad9 = 61,
    
    /* Numpad operators */
    KeyMagic_VK_Multiply = 62,
    KeyMagic_VK_Add = 63,
    KeyMagic_VK_Separator = 64,
    KeyMagic_VK_Subtract = 65,
    KeyMagic_VK_Decimal = 66,
    KeyMagic_VK_Divide = 67,
    
    /* Function keys */
    KeyMagic_VK_F1 = 68,
    KeyMagic_VK_F2 = 69,
    KeyMagic_VK_F3 = 70,
    KeyMagic_VK_F4 = 71,
    KeyMagic_VK_F5 = 72,
    KeyMagic_VK_F6 = 73,
    KeyMagic_VK_F7 = 74,
    KeyMagic_VK_F8 = 75,
    KeyMagic_VK_F9 = 76,
    KeyMagic_VK_F10 = 77,
    KeyMagic_VK_F11 = 78,
    KeyMagic_VK_F12 = 79,
    
    /* Modifier keys */
    KeyMagic_VK_LShift = 80,
    KeyMagic_VK_RShift = 81,
    KeyMagic_VK_LControl = 82,
    KeyMagic_VK_RControl = 83,
    KeyMagic_VK_LMenu = 84,    /* Left Alt */
    KeyMagic_VK_RMenu = 85,    /* Right Alt/AltGr */
    
    /* OEM keys */
    KeyMagic_VK_Oem1 = 86,     /* ;: for US */
    KeyMagic_VK_OemPlus = 87,  /* + key */
    KeyMagic_VK_OemComma = 88, /* , key */
    KeyMagic_VK_OemMinus = 89, /* - key */
    KeyMagic_VK_OemPeriod = 90, /* . key */
    KeyMagic_VK_Oem2 = 91,     /* /? for US */
    KeyMagic_VK_Oem3 = 92,     /* `~ for US */
    KeyMagic_VK_Oem4 = 93,     /* [{ for US */
    KeyMagic_VK_Oem5 = 94,     /* \| for US */
    KeyMagic_VK_Oem6 = 95,     /* ]} for US */
    KeyMagic_VK_Oem7 = 96,     /* '" for US */
    KeyMagic_VK_Oem8 = 97,
    KeyMagic_VK_OemAx = 98,
    KeyMagic_VK_Oem102 = 99,   /* <> or \| on 102-key keyboard */
    KeyMagic_VK_IcoHelp = 100,
    KeyMagic_VK_Ico00 = 101,
    
    /* Navigation keys */
    KeyMagic_VK_End = 102,
    KeyMagic_VK_Home = 103,
    KeyMagic_VK_Left = 104,
    KeyMagic_VK_Up = 105,
    KeyMagic_VK_Right = 106,
    KeyMagic_VK_Down = 107,
    KeyMagic_VK_Insert = 108
} KeyMagicVirtualKey;

/* ============================================================================
 * Engine Management Functions
 * ============================================================================ */

/**
 * Create a new KeyMagic engine instance
 * 
 * @return New engine handle or NULL on failure
 */
KEYMAGIC_API EngineHandle* keymagic_engine_new(void);

/**
 * Free/destroy an engine instance
 * 
 * @param handle Engine handle to destroy
 */
KEYMAGIC_API void keymagic_engine_free(EngineHandle* handle);

/* ============================================================================
 * Keyboard Loading Functions
 * ============================================================================ */

/**
 * Load a keyboard layout from a .km2 file
 * 
 * @param handle Engine handle
 * @param km2_path Path to .km2 keyboard file
 * @return Result code
 */
KEYMAGIC_API KeyMagicResult keymagic_engine_load_keyboard(
    EngineHandle* handle, 
    const char* km2_path
);

/**
 * Load a keyboard layout from memory buffer
 * 
 * @param handle Engine handle
 * @param km2_data Pointer to KM2 file data
 * @param data_len Size of KM2 data in bytes
 * @return Result code
 */
KEYMAGIC_API KeyMagicResult keymagic_engine_load_keyboard_from_memory(
    EngineHandle* handle, 
    const uint8_t* km2_data, 
    size_t data_len
);

/* ============================================================================
 * Key Processing Functions
 * ============================================================================ */

/**
 * Process a key event (platform-independent)
 * 
 * @param handle Engine handle
 * @param key_code Internal VirtualKey code (use conversion functions for platform codes)
 * @param character Unicode character (0 if none)
 * @param shift Shift key state (0 or 1)
 * @param ctrl Control key state (0 or 1)
 * @param alt Alt key state (0 or 1)
 * @param caps_lock Caps Lock state (0 or 1)
 * @param output Output structure (caller must free strings)
 * @return Result code
 */
KEYMAGIC_API KeyMagicResult keymagic_engine_process_key(
    EngineHandle* handle,
    KeyMagicVirtualKey key_code,
    char character,
    int shift,
    int ctrl,
    int alt,
    int caps_lock,
    ProcessKeyOutput* output
);

/* ============================================================================
 * Platform-Specific Functions
 * ============================================================================ */

#ifdef _WIN32
/**
 * Process a key event using Windows VK codes
 * 
 * @param handle Engine handle
 * @param vk_code Windows Virtual Key code (e.g., 0x41 for VK_A)
 * @param character Unicode character (0 if none)
 * @param shift Shift key state (0 or 1)
 * @param ctrl Control key state (0 or 1)
 * @param alt Alt key state (0 or 1)
 * @param caps_lock Caps Lock state (0 or 1)
 * @param output Output structure (caller must free strings)
 * @return Result code
 */
KEYMAGIC_API KeyMagicResult keymagic_engine_process_key_win(
    EngineHandle* handle,
    int vk_code,
    char character,
    int shift,
    int ctrl,
    int alt,
    int caps_lock,
    ProcessKeyOutput* output
);

/**
 * Test mode - process key without modifying engine state (Windows)
 * Useful for preview/testing without affecting the actual input
 * 
 * @param handle Engine handle
 * @param vk_code Windows Virtual Key code
 * @param character Unicode character (0 if none)
 * @param shift Shift key state (0 or 1)
 * @param ctrl Control key state (0 or 1)
 * @param alt Alt key state (0 or 1)
 * @param caps_lock Caps Lock state (0 or 1)
 * @param output Output structure (caller must free strings)
 * @return Result code
 */
KEYMAGIC_API KeyMagicResult keymagic_engine_process_key_test_win(
    EngineHandle* handle,
    int vk_code,
    char character,
    int shift,
    int ctrl,
    int alt,
    int caps_lock,
    ProcessKeyOutput* output
);
#endif /* _WIN32 */

/* ============================================================================
 * Engine State Management
 * ============================================================================ */

/**
 * Reset engine state (clear composing text and internal state)
 * 
 * @param handle Engine handle
 * @return Result code
 */
KEYMAGIC_API KeyMagicResult keymagic_engine_reset(EngineHandle* handle);

/**
 * Get current composing text from engine
 * 
 * @param handle Engine handle
 * @return Composing text (caller must free) or NULL
 */
KEYMAGIC_API char* keymagic_engine_get_composition(EngineHandle* handle);

/**
 * Set composing text in engine (for synchronization)
 * 
 * @param handle Engine handle
 * @param text Text to set as composing text (UTF-8)
 * @return Result code
 */
KEYMAGIC_API KeyMagicResult keymagic_engine_set_composition(
    EngineHandle* handle, 
    const char* text
);

/* ============================================================================
 * Memory Management
 * ============================================================================ */

/**
 * Free a string returned by the library
 * 
 * @param s String to free
 */
KEYMAGIC_API void keymagic_free_string(char* s);

/* ============================================================================
 * KM2 File Metadata Access
 * ============================================================================ */

/**
 * Load a KM2 file for metadata access (does not activate keyboard)
 * 
 * @param path Path to .km2 file
 * @return KM2 file handle or NULL on error
 */
KEYMAGIC_API Km2FileHandle* keymagic_km2_load(const char* path);

/**
 * Free a loaded KM2 file handle
 * 
 * @param handle KM2 file handle
 */
KEYMAGIC_API void keymagic_km2_free(Km2FileHandle* handle);

/**
 * Get keyboard name from KM2 file
 * 
 * @param handle KM2 file handle
 * @return Keyboard name (caller must free) or NULL if not defined
 */
KEYMAGIC_API char* keymagic_km2_get_name(Km2FileHandle* handle);

/**
 * Get keyboard description from KM2 file
 * 
 * @param handle KM2 file handle
 * @return Description (caller must free) or NULL if not defined
 */
KEYMAGIC_API char* keymagic_km2_get_description(Km2FileHandle* handle);

/**
 * Get hotkey string from KM2 file
 * 
 * @param handle KM2 file handle
 * @return Hotkey string (caller must free) or NULL if not defined
 */
KEYMAGIC_API char* keymagic_km2_get_hotkey(Km2FileHandle* handle);

/**
 * Get icon data from KM2 file
 * 
 * @param handle KM2 file handle
 * @param buffer Buffer to receive icon data (NULL to query size)
 * @param buffer_size Size of buffer
 * @return Required buffer size or actual bytes copied
 */
KEYMAGIC_API size_t keymagic_km2_get_icon_data(
    Km2FileHandle* handle, 
    uint8_t* buffer, 
    size_t buffer_size
);

/* ============================================================================
 * Utility Functions
 * ============================================================================ */

/**
 * Parse a hotkey string (e.g., "Ctrl+Shift+M")
 * 
 * @param hotkey_str Hotkey string to parse
 * @param info Output hotkey information structure
 * @return 1 on success, 0 on failure
 */
KEYMAGIC_API int keymagic_parse_hotkey(const char* hotkey_str, HotkeyInfo* info);

/**
 * Convert VirtualKey enum value to display string
 * 
 * @param key_code VirtualKey enum value
 * @return Display string (caller must free) or NULL if invalid
 */
KEYMAGIC_API char* keymagic_virtual_key_to_string(KeyMagicVirtualKey key_code);

/**
 * Get library version string
 * 
 * @return Version string (do not free)
 */
KEYMAGIC_API const char* keymagic_get_version(void);

#ifdef __cplusplus
}
#endif

#endif /* KEYMAGIC_CORE_H */