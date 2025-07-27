#ifndef KEYMAGIC_FFI_BRIDGE_H
#define KEYMAGIC_FFI_BRIDGE_H

#include <glib.h>

G_BEGIN_DECLS

/**
 * FFI Bridge to keymagic-core (Rust library)
 * 
 * This module provides a C interface to the Rust keymagic-core library,
 * handling the conversion between C types and Rust FFI types.
 */

/* Opaque handle to Rust EngineHandle */
typedef void* EngineHandle;

/**
 * Key processing result from engine
 * Matches the ProcessKeyOutput structure from keymagic-core FFI
 */
typedef struct {
    gchar* text;                    /* Output text (may be NULL) */
    gchar* composing_text;          /* Current composing text (may be NULL) */
    gboolean is_processed;          /* TRUE if engine handled the key */
    gint action_type;               /* Action type (Insert, Backspace, etc.) */
    gint delete_count;              /* Number of characters to delete */
} KeyProcessingResult;

/**
 * Engine result codes
 */
typedef enum {
    KEYMAGIC_RESULT_SUCCESS = 0,
    KEYMAGIC_RESULT_ERROR = 1,
    KEYMAGIC_RESULT_INVALID_ENGINE = 2,
    KEYMAGIC_RESULT_INVALID_KEYBOARD = 3
} KeyMagicResult;

/**
 * Load a keyboard layout from .km2 file
 * 
 * @param km2_file_path Path to .km2 keyboard file
 * @return Engine handle or NULL on failure
 */
EngineHandle* keymagic_ffi_load_keyboard(const gchar* km2_file_path);

/**
 * Free/destroy an engine handle
 * 
 * @param engine Engine handle to destroy
 */
void keymagic_ffi_destroy_engine(EngineHandle* engine);

/**
 * Process a key event
 * 
 * @param engine Engine handle
 * @param keyval Key value (GDK keyval)
 * @param keycode Hardware keycode
 * @param modifiers Modifier state
 * @param result Output result structure (caller must free)
 * @return Result code
 */
KeyMagicResult keymagic_ffi_process_key(EngineHandle* engine,
                                        guint keyval,
                                        guint keycode, 
                                        guint modifiers,
                                        KeyProcessingResult* result);

/**
 * Reset engine state
 * 
 * @param engine Engine handle
 * @return Result code
 */
KeyMagicResult keymagic_ffi_reset_engine(EngineHandle* engine);

/**
 * Get current composing text from engine
 * 
 * @param engine Engine handle
 * @return Current composing text (caller must free) or NULL
 */
gchar* keymagic_ffi_get_composing_text(EngineHandle* engine);

/**
 * Set composing text in engine (for sync purposes)
 * 
 * @param engine Engine handle
 * @param text Text to set as composing text
 * @return Result code
 */
KeyMagicResult keymagic_ffi_set_composing_text(EngineHandle* engine, const gchar* text);

/**
 * Free a string returned by the FFI layer
 * 
 * @param str String to free
 */
void keymagic_ffi_free_string(gchar* str);

/**
 * Free a KeyProcessingResult structure
 * 
 * @param result Result structure to free
 */
void keymagic_ffi_free_result(KeyProcessingResult* result);

/**
 * Load KM2 file for metadata access
 * 
 * @param km2_path Path to .km2 file
 * @return Handle to KM2 file or NULL on error
 */
void* keymagic_ffi_km2_load(const gchar* km2_path);

/**
 * Free KM2 file handle
 * 
 * @param handle KM2 file handle
 */
void keymagic_ffi_km2_free(void* handle);

/**
 * Get keyboard name from KM2 file
 * 
 * @param handle KM2 file handle  
 * @return Keyboard name (caller must free) or NULL
 */
gchar* keymagic_ffi_km2_get_name(void* handle);

/**
 * Get keyboard description from KM2 file
 * 
 * @param handle KM2 file handle
 * @return Description (caller must free) or NULL  
 */
gchar* keymagic_ffi_km2_get_description(void* handle);

/**
 * Get keyboard hotkey from KM2 file
 * 
 * @param handle KM2 file handle
 * @return Hotkey string (caller must free) or NULL
 */
gchar* keymagic_ffi_km2_get_hotkey(void* handle);

G_END_DECLS

#endif /* KEYMAGIC_FFI_BRIDGE_H */