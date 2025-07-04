#ifndef KEYMAGIC_FFI_H
#define KEYMAGIC_FFI_H

#ifdef __cplusplus
extern "C" {
#endif

#include <stdint.h>
#include <stddef.h>

// Opaque handle to the engine
typedef struct EngineHandle EngineHandle;

// Result codes
typedef enum {
    KeyMagicResult_Success = 0,
    KeyMagicResult_ErrorInvalidHandle = -1,
    KeyMagicResult_ErrorInvalidParameter = -2,
    KeyMagicResult_ErrorEngineFailure = -3,
    KeyMagicResult_ErrorUtf8Conversion = -4,
    KeyMagicResult_ErrorNoKeyboard = -5,
} KeyMagicResult;

// Output structure from key processing
typedef struct {
    int action_type;      // 0=None, 1=Insert, 2=BackspaceDelete, 3=BackspaceDeleteAndInsert
    char* text;           // UTF-8 encoded, null-terminated (needs to be freed)
    int delete_count;     // Number of characters to delete
    char* composing_text; // UTF-8 encoded, null-terminated (needs to be freed)
    int is_processed;     // 0=false, 1=true
} ProcessKeyOutput;

// Engine management
EngineHandle* keymagic_engine_new(void);
void keymagic_engine_free(EngineHandle* handle);

// Keyboard loading
KeyMagicResult keymagic_engine_load_keyboard(EngineHandle* handle, const char* km2_path);
KeyMagicResult keymagic_engine_load_keyboard_from_memory(
    EngineHandle* handle, 
    const uint8_t* km2_data, 
    size_t data_len
);

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

// Memory management
void keymagic_free_string(char* s);

// Engine control
KeyMagicResult keymagic_engine_reset(EngineHandle* handle);
char* keymagic_engine_get_composition(EngineHandle* handle);

// Version info
const char* keymagic_get_version(void);

// Windows-specific key processing with VK codes
KeyMagicResult keymagic_engine_process_key_win(
    EngineHandle* handle,
    int vk_code,          // Windows VK code (e.g., 0x41 for VK_A)
    char character,
    int shift,
    int ctrl,
    int alt,
    int caps_lock,
    ProcessKeyOutput* output
);

#ifdef __cplusplus
}
#endif

#endif // KEYMAGIC_FFI_H