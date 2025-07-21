#include "ffi_bridge.h"
#include "keycode_map.h"
#include <string.h>
#include <stdlib.h>
#include <ibus.h>

/* Logging tag */
#define LOG_TAG "KeyMagicFFI"

/* Conditional logging for sensitive information */
#ifdef NDEBUG
    /* Release build - redact sensitive information */
    #define LOG_FFI_PARAMS(km_keycode, keyval, character, shift, ctrl, alt, caps_lock) \
        g_debug("%s: Calling keymagic_engine_process_key with params: [REDACTED]", LOG_TAG)
    #define LOG_FFI_RESULT(text, composing_text, is_processed, action_type, delete_count) \
        g_debug("%s: Process key result - [REDACTED]", LOG_TAG)
#else
    /* Debug build - show full information */
    #define LOG_FFI_PARAMS(km_keycode, keyval, character, shift, ctrl, alt, caps_lock) \
        do { \
            g_debug("%s: Calling keymagic_engine_process_key with params:", LOG_TAG); \
            g_debug("%s:   - key_code: %u (mapped from keyval 0x%x)", LOG_TAG, km_keycode, keyval); \
            g_debug("%s:   - character: '%c' (0x%02x)", LOG_TAG, \
                    character >= 0x20 && character <= 0x7E ? character : '?', character); \
            g_debug("%s:   - shift: %d", LOG_TAG, shift ? 1 : 0); \
            g_debug("%s:   - ctrl: %d", LOG_TAG, ctrl ? 1 : 0); \
            g_debug("%s:   - alt: %d", LOG_TAG, alt ? 1 : 0); \
            g_debug("%s:   - caps_lock: %d", LOG_TAG, caps_lock ? 1 : 0); \
        } while(0)
    #define LOG_FFI_RESULT(text, composing_text, is_processed, action_type, delete_count) \
        g_debug("%s: Process key result - text=%s, composing=%s, processed=%s, action=%d, delete=%d", \
                LOG_TAG, \
                text ? text : "(null)", \
                composing_text ? composing_text : "(null)", \
                is_processed ? "TRUE" : "FALSE", \
                action_type, \
                delete_count)
#endif

/**
 * FFI Bridge Implementation
 * 
 * This module provides a C interface to the Rust keymagic-core library.
 * It handles type conversion between C and Rust FFI types and manages
 * memory allocation/deallocation across the FFI boundary.
 */

/* External FFI functions from keymagic-core (Rust) */
extern void* keymagic_engine_new();
extern void keymagic_engine_free(void* engine);
extern int keymagic_engine_load_keyboard(void* engine, const char* km2_path);
extern int keymagic_engine_process_key(void* engine, int key_code, char character,
                                       int shift, int ctrl, int alt, int caps_lock,
                                       void* output);
extern int keymagic_engine_reset(void* engine);
extern char* keymagic_engine_get_composition(void* engine);
extern int keymagic_engine_set_composition(void* engine, const char* text);
extern void keymagic_engine_free_string(char* str);

/* ProcessKeyOutput structure from Rust FFI */
typedef struct {
    int action_type;
    char* text;
    int delete_count;
    char* composing_text;
    int is_processed;
} RustProcessKeyOutput;

/**
 * Load keyboard from .km2 file
 */
EngineHandle*
keymagic_ffi_load_keyboard(const gchar* km2_file_path)
{
    g_return_val_if_fail(km2_file_path != NULL, NULL);
    
    g_debug("%s: Loading keyboard from: %s", LOG_TAG, km2_file_path);
    
    /* Check if file exists */
    if (!g_file_test(km2_file_path, G_FILE_TEST_EXISTS)) {
        g_warning("%s: Keyboard file not found: %s", LOG_TAG, km2_file_path);
        return NULL;
    }
    
    /* Create new engine */
    EngineHandle* handle = keymagic_engine_new();
    if (!handle) {
        g_warning("%s: Failed to create engine", LOG_TAG);
        return NULL;
    }
    
    /* Load keyboard into engine */
    int result = keymagic_engine_load_keyboard(handle, km2_file_path);
    if (result != 0) {
        g_warning("%s: Failed to load keyboard from file: %s", LOG_TAG, km2_file_path);
        keymagic_engine_free(handle);
        return NULL;
    }
    
    g_debug("%s: Successfully loaded keyboard: %s", LOG_TAG, km2_file_path);
    return handle;
}

/**
 * Destroy engine handle
 */
void
keymagic_ffi_destroy_engine(EngineHandle* engine)
{
    if (!engine) {
        return;
    }
    
    g_debug("%s: Destroying engine handle", LOG_TAG);
    keymagic_engine_free(engine);
}

/**
 * Process key event
 */
KeyMagicResult
keymagic_ffi_process_key(EngineHandle* engine, guint keyval, guint keycode, 
                         guint modifiers, KeyProcessingResult* result)
{
    g_return_val_if_fail(engine != NULL, KEYMAGIC_RESULT_INVALID_ENGINE);
    g_return_val_if_fail(result != NULL, KEYMAGIC_RESULT_ERROR);
    
    /* Initialize result structure */
    memset(result, 0, sizeof(KeyProcessingResult));
    
    /* Convert IBus modifiers to individual flags */
    gboolean shift = (modifiers & IBUS_SHIFT_MASK) != 0;
    gboolean ctrl = (modifiers & IBUS_CONTROL_MASK) != 0;
    gboolean alt = (modifiers & IBUS_MOD1_MASK) != 0;
    gboolean caps_lock = (modifiers & IBUS_LOCK_MASK) != 0;
    
    /* Convert keyval to character - for ASCII printable chars */
    /* Only pass character when no modifiers (except Shift) are pressed */
    char character = 0;
    if ((keyval >= 0x20 && keyval <= 0x7E) && !ctrl && !alt) {
        character = (char)keyval;
    }
    
    /* Map IBus keyval to KeyMagic VirtualKey code */
    guint16 km_keycode = keymagic_map_ibus_keyval(keyval);
    
    /* If no mapping found, try to use the raw keycode as fallback */
    if (km_keycode == 0) {
        g_debug("%s: No mapping for keyval 0x%x, using raw keycode %u", LOG_TAG, keyval, keycode);
        km_keycode = keycode;
    } else {
        g_debug("%s: Mapped keyval 0x%x to VirtualKey %u", LOG_TAG, keyval, km_keycode);
    }
    
    /* Log parameters before calling Rust FFI */
    LOG_FFI_PARAMS(km_keycode, keyval, character, shift, ctrl, alt, caps_lock);
    
    /* Call Rust FFI function with mapped keycode */
    RustProcessKeyOutput rust_output = {0};
    int rust_result = keymagic_engine_process_key(engine, km_keycode, character,
                                                  shift ? 1 : 0, ctrl ? 1 : 0, 
                                                  alt ? 1 : 0, caps_lock ? 1 : 0,
                                                  &rust_output);
    
    /* Convert Rust result to our enum */
    if (rust_result != 0) {
        g_warning("%s: Engine process key failed with code: %d", LOG_TAG, rust_result);
        return KEYMAGIC_RESULT_ERROR;
    }
    
    /* Copy results to C structure */
    result->text = rust_output.text ? g_strdup(rust_output.text) : NULL;
    result->composing_text = rust_output.composing_text ? g_strdup(rust_output.composing_text) : NULL;
    result->is_processed = rust_output.is_processed ? TRUE : FALSE;
    result->action_type = rust_output.action_type;
    result->delete_count = rust_output.delete_count;
    
    /* Free Rust-allocated strings */
    if (rust_output.text) keymagic_engine_free_string(rust_output.text);
    if (rust_output.composing_text) keymagic_engine_free_string(rust_output.composing_text);
    
    LOG_FFI_RESULT(result->text, result->composing_text, result->is_processed,
                   result->action_type, result->delete_count);
    
    return KEYMAGIC_RESULT_SUCCESS;
}

/**
 * Reset engine state
 */
KeyMagicResult
keymagic_ffi_reset_engine(EngineHandle* engine)
{
    g_return_val_if_fail(engine != NULL, KEYMAGIC_RESULT_INVALID_ENGINE);
    
    g_debug("%s: Resetting engine", LOG_TAG);
    
    int result = keymagic_engine_reset(engine);
    if (result != 0) {
        g_warning("%s: Engine reset failed with code: %d", LOG_TAG, result);
        return KEYMAGIC_RESULT_ERROR;
    }
    
    return KEYMAGIC_RESULT_SUCCESS;
}

/**
 * Get current composing text
 */
gchar*
keymagic_ffi_get_composing_text(EngineHandle* engine)
{
    g_return_val_if_fail(engine != NULL, NULL);
    
    char* rust_text = keymagic_engine_get_composition(engine);
    if (!rust_text) {
        return NULL;
    }
    
    /* Copy to GLib-allocated string */
    gchar* c_text = g_strdup(rust_text);
    
    /* Free Rust-allocated string */
    keymagic_engine_free_string(rust_text);
    
    return c_text;
}

/**
 * Set composing text (for sync purposes)
 */
KeyMagicResult
keymagic_ffi_set_composing_text(EngineHandle* engine, const gchar* text)
{
    g_return_val_if_fail(engine != NULL, KEYMAGIC_RESULT_INVALID_ENGINE);
    
    const char* text_to_set = text ? text : "";
    
    g_debug("%s: Setting composing text: %s", LOG_TAG, text_to_set);
    
    int result = keymagic_engine_set_composition(engine, text_to_set);
    if (result != 0) {
        g_warning("%s: Set composing text failed with code: %d", LOG_TAG, result);
        return KEYMAGIC_RESULT_ERROR;
    }
    
    return KEYMAGIC_RESULT_SUCCESS;
}

/**
 * Free FFI-allocated string
 */
void
keymagic_ffi_free_string(gchar* str)
{
    g_free(str);
}

/**
 * Free KeyProcessingResult structure
 */
void
keymagic_ffi_free_result(KeyProcessingResult* result)
{
    if (!result) {
        return;
    }
    
    g_free(result->text);
    g_free(result->composing_text);
    
    /* Clear structure */
    memset(result, 0, sizeof(KeyProcessingResult));
}