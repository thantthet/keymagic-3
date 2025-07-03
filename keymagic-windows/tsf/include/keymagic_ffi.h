#ifndef KEYMAGIC_FFI_H
#define KEYMAGIC_FFI_H

#ifdef __cplusplus
extern "C" {
#endif

#include <stdint.h>

// Opaque handle to the KeyMagic engine
typedef struct EngineHandle EngineHandle;

// Result codes
typedef enum {
    KEYMAGIC_SUCCESS = 0,
    KEYMAGIC_ERROR_INVALID_HANDLE = -1,
    KEYMAGIC_ERROR_INVALID_PARAMETER = -2,
    KEYMAGIC_ERROR_ENGINE_FAILURE = -3,
    KEYMAGIC_ERROR_UTF8_CONVERSION = -4,
    KEYMAGIC_ERROR_NO_KEYBOARD = -5,
} KeyMagicResult;

// Output from key processing
typedef struct {
    // Action type: 0=None, 1=Insert, 2=BackspaceDelete, 3=BackspaceDeleteAndInsert
    int action_type;
    // UTF-8 encoded text to insert (null-terminated)
    char* text;
    // Number of characters to delete
    int delete_count;
    // Current composing text (UTF-8, null-terminated)
    char* composing_text;
    // Whether the key was consumed (0=false, 1=true)
    int consumed;
} ProcessKeyOutput;

// Engine lifecycle functions
EngineHandle* keymagic_engine_new(void);
void keymagic_engine_free(EngineHandle* handle);

// Keyboard loading
KeyMagicResult keymagic_engine_load_keyboard(EngineHandle* handle, const char* km2_path);

// Key processing
KeyMagicResult keymagic_engine_process_key(
    EngineHandle* handle,
    int key_code,
    char character,
    int shift,
    int ctrl,
    int alt,
    int caps_lock,
    ProcessKeyOutput* output
);

// State management
KeyMagicResult keymagic_engine_reset(EngineHandle* handle);
char* keymagic_engine_get_composition(EngineHandle* handle);

// Memory management
void keymagic_free_string(char* s);

// Version information
const char* keymagic_windows_version(void);

#ifdef __cplusplus
}
#endif

#endif // KEYMAGIC_FFI_H