//
//  KeyMagic-Bridging-Header.h
//  KeyMagic
//
//  Bridging header to use keymagic-core FFI directly from Swift
//

#ifndef KeyMagic_Bridging_Header_h
#define KeyMagic_Bridging_Header_h

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

// Opaque handle to the KeyMagic engine
typedef void* EngineHandle;

// Result codes for FFI functions
typedef enum {
    KeyMagicResult_Success = 0,
    KeyMagicResult_ErrorInvalidHandle = -1,
    KeyMagicResult_ErrorInvalidParameter = -2,
    KeyMagicResult_ErrorEngineFailure = -3,
    KeyMagicResult_ErrorUtf8Conversion = -4,
    KeyMagicResult_ErrorNoKeyboard = -5,
} KeyMagicResult;

// Output from processing a key event
typedef struct {
    int action_type;
    char* text;
    int delete_count;
    char* composing_text;
    int is_processed;
} ProcessKeyOutput;

// FFI functions from keymagic-core
extern EngineHandle* keymagic_engine_new(void);
extern void keymagic_engine_free(EngineHandle* engine);
extern KeyMagicResult keymagic_engine_load_keyboard(EngineHandle* engine, const char* km2_path);
extern KeyMagicResult keymagic_engine_process_key(EngineHandle* engine, int key_code, char character,
                                                   int shift, int ctrl, int alt, int caps_lock,
                                                   ProcessKeyOutput* output);
extern KeyMagicResult keymagic_engine_reset(EngineHandle* engine);
extern char* keymagic_engine_get_composition(EngineHandle* engine);
extern KeyMagicResult keymagic_engine_set_composition(EngineHandle* engine, const char* text);
extern void keymagic_free_string(char* str);

#endif /* KeyMagic_Bridging_Header_h */