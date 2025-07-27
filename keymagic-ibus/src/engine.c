#include "engine.h"
#include "config.h"
#include "ffi_bridge.h"
#include "keycode_map.h"
#include <string.h>
#include <glib/gstdio.h>

/* Logging tag */
#define LOG_TAG "KeyMagicEngine"

/* Conditional logging for sensitive information */
#ifdef NDEBUG
    /* Release build - redact sensitive key information */
    #define LOG_KEY_EVENT(keyval, keycode, modifiers) \
        g_debug("%s: Processing key event - [REDACTED]", LOG_TAG)
    #define LOG_COMPOSING_TEXT(text) \
        g_debug("%s: Engine composing text: [REDACTED]", LOG_TAG)
#else
    /* Debug build - show full information */
    #define LOG_KEY_EVENT(keyval, keycode, modifiers) \
        g_debug("%s: Processing key event - keyval=%u, keycode=%u, modifiers=%u", \
                LOG_TAG, keyval, keycode, modifiers)
    #define LOG_COMPOSING_TEXT(text) \
        g_debug("%s: Engine composing text: %s", LOG_TAG, text)
#endif

/* File monitor event callback */
static void config_file_changed_cb(GFileMonitor* monitor, GFile* file, GFile* other_file,
                                   GFileMonitorEvent event_type, gpointer user_data);

/* Hotkey handling */
static gpointer create_hotkey_hash(guint modifiers, guint keyval);

/* Timeout callback for hiding auxiliary text */
static gboolean aux_text_timeout_cb(gpointer user_data);

/* Engine method implementations */
static void keymagic_engine_class_init(KeyMagicEngineClass* klass);
static void keymagic_engine_init(KeyMagicEngine* engine);
static void keymagic_engine_finalize(GObject* object);

/* IBus engine virtual methods */
static gboolean keymagic_engine_process_key_event(IBusEngine* engine, guint keyval, 
                                                   guint keycode, guint modifiers);
static void keymagic_engine_focus_in(IBusEngine* engine);
static void keymagic_engine_focus_out(IBusEngine* engine);
static void keymagic_engine_reset(IBusEngine* engine);
static void keymagic_engine_enable(IBusEngine* engine);
static void keymagic_engine_disable(IBusEngine* engine);
static void keymagic_engine_property_activate(IBusEngine* engine, const gchar* prop_name,
                                              guint prop_state);

/* Type registration */
G_DEFINE_TYPE(KeyMagicEngine, keymagic_engine, IBUS_TYPE_ENGINE)

/**
 * Initialize KeyMagicEngine class
 */
static void
keymagic_engine_class_init(KeyMagicEngineClass* klass)
{
    GObjectClass* object_class = G_OBJECT_CLASS(klass);
    IBusEngineClass* engine_class = IBUS_ENGINE_CLASS(klass);
    
    /* GObject methods */
    object_class->finalize = keymagic_engine_finalize;
    
    /* IBusEngine virtual methods */
    engine_class->process_key_event = keymagic_engine_process_key_event;
    engine_class->focus_in = keymagic_engine_focus_in;
    engine_class->focus_out = keymagic_engine_focus_out;
    engine_class->reset = keymagic_engine_reset;
    engine_class->enable = keymagic_engine_enable;
    engine_class->disable = keymagic_engine_disable;
    engine_class->property_activate = keymagic_engine_property_activate;
}

/**
 * Initialize KeyMagicEngine instance
 */
static void
keymagic_engine_init(KeyMagicEngine* engine)
{
    g_debug("%s: Initializing KeyMagic 3 engine", LOG_TAG);
    
    /* Initialize fields */
    engine->km_engine = NULL;
    engine->active_keyboard_id = NULL;
    engine->keyboard_path = NULL;
    engine->config_path = NULL;
    engine->config_monitor = NULL;
    engine->keyboard_load_failed = FALSE;
    engine->keyboard_changed = FALSE;
    engine->preedit_text = NULL;
    engine->preedit_visible = FALSE;
    engine->preedit_cursor_pos = 0;
    
    /* Initialize property management */
    engine->prop_list = NULL;
    engine->keyboard_properties = g_hash_table_new_full(g_str_hash, g_str_equal,
                                                        g_free, g_free);
    engine->keyboard_hotkeys = g_hash_table_new_full(g_direct_hash, g_direct_equal,
                                                     NULL, g_free);
    
    /* Initialize timeout management */
    engine->aux_text_timeout_id = 0;
    
    /* Set up configuration path and monitoring */
    engine->config_path = keymagic_config_get_default_path();
    if (engine->config_path) {
        GFile* config_file = g_file_new_for_path(engine->config_path);
        GError* error = NULL;
        
        engine->config_monitor = g_file_monitor_file(config_file, 
                                                     G_FILE_MONITOR_NONE, 
                                                     NULL, &error);
        if (engine->config_monitor) {
            g_signal_connect(engine->config_monitor, "changed",
                           G_CALLBACK(config_file_changed_cb), engine);
            g_debug("%s: Config file monitoring enabled: %s", LOG_TAG, engine->config_path);
        } else {
            g_warning("%s: Failed to monitor config file: %s", LOG_TAG, 
                     error ? error->message : "Unknown error");
            if (error) g_error_free(error);
        }
        
        g_object_unref(config_file);
    }
    
    /* Load initial configuration */
    keymagic_engine_load_config(engine);
}

/**
 * Finalize KeyMagicEngine instance
 */
static void
keymagic_engine_finalize(GObject* object)
{
    KeyMagicEngine* engine = KEYMAGIC_ENGINE(object);
    
    g_debug("%s: Finalizing KeyMagic 3 engine", LOG_TAG);
    
    /* Cancel any pending timeout */
    if (engine->aux_text_timeout_id > 0) {
        g_source_remove(engine->aux_text_timeout_id);
        engine->aux_text_timeout_id = 0;
    }
    
    /* Cleanup engine */
    keymagic_engine_unload_keyboard(engine);
    
    /* Cleanup monitoring */
    if (engine->config_monitor) {
        g_object_unref(engine->config_monitor);
        engine->config_monitor = NULL;
    }
    
    /* Free strings */
    g_free(engine->active_keyboard_id);
    g_free(engine->keyboard_path);
    g_free(engine->config_path);
    
    /* Clear preedit */
    keymagic_engine_clear_preedit(engine);
    
    /* Cleanup property management */
    if (engine->prop_list) {
        engine->prop_list = NULL;
    }
    if (engine->keyboard_properties) {
        g_hash_table_destroy(engine->keyboard_properties);
        engine->keyboard_properties = NULL;
    }
    if (engine->keyboard_hotkeys) {
        g_hash_table_destroy(engine->keyboard_hotkeys);
        engine->keyboard_hotkeys = NULL;
    }
    
    /* Call parent finalize */
    G_OBJECT_CLASS(keymagic_engine_parent_class)->finalize(object);
}

/**
 * Create new KeyMagicEngine instance
 */
KeyMagicEngine*
keymagic_ibus_engine_new(void)
{
    return g_object_new(KEYMAGIC_TYPE_ENGINE, NULL);
}

/**
 * Load configuration from TOML file
 */
gboolean
keymagic_engine_load_config(KeyMagicEngine* engine)
{
    g_return_val_if_fail(KEYMAGIC_IS_ENGINE(engine), FALSE);
    
    if (!engine->config_path || !g_file_test(engine->config_path, G_FILE_TEST_EXISTS)) {
        g_debug("%s: Config file not found: %s", LOG_TAG, 
                engine->config_path ? engine->config_path : "(null)");
        return FALSE;
    }
    
    KeyMagicConfig* config = keymagic_config_load(engine->config_path);
    if (!config) {
        g_warning("%s: Failed to load config from: %s", LOG_TAG, engine->config_path);
        return FALSE;
    }
    
    /* Check if active keyboard changed */
    if (g_strcmp0(engine->active_keyboard_id, config->active_keyboard) != 0) {
        g_free(engine->active_keyboard_id);
        engine->active_keyboard_id = g_strdup(config->active_keyboard);
        engine->keyboard_changed = TRUE;
        
        g_debug("%s: Active keyboard changed to: %s", LOG_TAG, 
                engine->active_keyboard_id ? engine->active_keyboard_id : "(none)");
    }
    
    keymagic_config_free(config);
    return TRUE;
}

/**
 * Load keyboard by ID
 */
gboolean
keymagic_ibus_engine_load_keyboard(KeyMagicEngine* engine, const gchar* keyboard_id)
{
    g_return_val_if_fail(KEYMAGIC_IS_ENGINE(engine), FALSE);
    g_return_val_if_fail(keyboard_id != NULL, FALSE);
    
    /* Find keyboard file */
    gchar* keyboard_file = keymagic_config_find_keyboard_file(keyboard_id);
    if (!keyboard_file) {
        g_warning("%s: Keyboard file not found for ID: %s", LOG_TAG, keyboard_id);
        engine->keyboard_load_failed = TRUE;
        return FALSE;
    }
    
    /* Unload current keyboard if any */
    keymagic_engine_unload_keyboard(engine);
    
    /* Load new keyboard */
    engine->km_engine = keymagic_ffi_load_keyboard(keyboard_file);
    if (!engine->km_engine) {
        g_warning("%s: Failed to load keyboard: %s", LOG_TAG, keyboard_file);
        engine->keyboard_load_failed = TRUE;
        g_free(keyboard_file);
        return FALSE;
    }
    
    /* Update state */
    engine->keyboard_path = keyboard_file;
    engine->keyboard_load_failed = FALSE;
    engine->keyboard_changed = FALSE;
    
    g_debug("%s: Successfully loaded keyboard: %s (%s)", LOG_TAG, keyboard_id, keyboard_file);
    return TRUE;
}

/**
 * Unload current keyboard
 */
void
keymagic_engine_unload_keyboard(KeyMagicEngine* engine)
{
    g_return_if_fail(KEYMAGIC_IS_ENGINE(engine));
    
    if (engine->km_engine) {
        keymagic_ffi_destroy_engine(engine->km_engine);
        engine->km_engine = NULL;
        g_debug("%s: Keyboard unloaded", LOG_TAG);
    }
    
    g_free(engine->keyboard_path);
    engine->keyboard_path = NULL;
    
    engine->keyboard_load_failed = FALSE;
    engine->keyboard_changed = FALSE;
}

/**
 * Process key event - main input processing logic
 */
static gboolean
keymagic_engine_process_key_event(IBusEngine* ibus_engine, guint keyval, 
                                   guint keycode, guint modifiers)
{
    KeyMagicEngine* engine = KEYMAGIC_ENGINE(ibus_engine);
    
    LOG_KEY_EVENT(keyval, keycode, modifiers);
    
    /* Only process key press events, ignore key release */
    if (modifiers & IBUS_RELEASE_MASK) {
        g_debug("%s: Ignoring key release event", LOG_TAG);
        return FALSE;
    }
    
    /* Ignore standalone modifier key events */
    switch (keyval) {
        case IBUS_KEY_Shift_L:
        case IBUS_KEY_Shift_R:
        case IBUS_KEY_Control_L:
        case IBUS_KEY_Control_R:
        case IBUS_KEY_Alt_L:
        case IBUS_KEY_Alt_R:
        case IBUS_KEY_Meta_L:
        case IBUS_KEY_Meta_R:
        case IBUS_KEY_Super_L:
        case IBUS_KEY_Super_R:
        case IBUS_KEY_Hyper_L:
        case IBUS_KEY_Hyper_R:
        case IBUS_KEY_Caps_Lock:
        case IBUS_KEY_Num_Lock:
        case IBUS_KEY_Scroll_Lock:
            g_debug("%s: Ignoring modifier key: keyval=%u", LOG_TAG, keyval);
            return FALSE;
    }
    
    /* Check for hotkey match first (before loading keyboard) */
    if (engine->keyboard_hotkeys && g_hash_table_size(engine->keyboard_hotkeys) > 0) {
        /* Normalize keyval for hotkey matching */
        guint normalized_keyval = keyval;
        guint normalized_modifiers = modifiers;
        
        /* If Shift is pressed and we have an uppercase letter, convert to lowercase
         * and ensure Shift modifier is set for consistent hotkey matching */
        if (keyval >= IBUS_KEY_A && keyval <= IBUS_KEY_Z) {
            /* Convert uppercase to lowercase */
            normalized_keyval = keyval + (IBUS_KEY_a - IBUS_KEY_A);
            normalized_modifiers |= IBUS_SHIFT_MASK;
        }
        /* Handle shifted number keys */
        else if (modifiers & IBUS_SHIFT_MASK) {
            switch (keyval) {
                case IBUS_KEY_exclam:       normalized_keyval = IBUS_KEY_1; break;
                case IBUS_KEY_at:           normalized_keyval = IBUS_KEY_2; break;
                case IBUS_KEY_numbersign:   normalized_keyval = IBUS_KEY_3; break;
                case IBUS_KEY_dollar:       normalized_keyval = IBUS_KEY_4; break;
                case IBUS_KEY_percent:      normalized_keyval = IBUS_KEY_5; break;
                case IBUS_KEY_asciicircum:  normalized_keyval = IBUS_KEY_6; break;
                case IBUS_KEY_ampersand:    normalized_keyval = IBUS_KEY_7; break;
                case IBUS_KEY_asterisk:     normalized_keyval = IBUS_KEY_8; break;
                case IBUS_KEY_parenleft:    normalized_keyval = IBUS_KEY_9; break;
                case IBUS_KEY_parenright:   normalized_keyval = IBUS_KEY_0; break;
                /* Shifted OEM keys */
                case IBUS_KEY_colon:        normalized_keyval = IBUS_KEY_semicolon; break;
                case IBUS_KEY_plus:         normalized_keyval = IBUS_KEY_equal; break;
                case IBUS_KEY_less:         normalized_keyval = IBUS_KEY_comma; break;
                case IBUS_KEY_underscore:   normalized_keyval = IBUS_KEY_minus; break;
                case IBUS_KEY_greater:      normalized_keyval = IBUS_KEY_period; break;
                case IBUS_KEY_question:     normalized_keyval = IBUS_KEY_slash; break;
                case IBUS_KEY_asciitilde:   normalized_keyval = IBUS_KEY_grave; break;
                case IBUS_KEY_braceleft:    normalized_keyval = IBUS_KEY_bracketleft; break;
                case IBUS_KEY_bar:          normalized_keyval = IBUS_KEY_backslash; break;
                case IBUS_KEY_braceright:   normalized_keyval = IBUS_KEY_bracketright; break;
                case IBUS_KEY_quotedbl:     normalized_keyval = IBUS_KEY_apostrophe; break;
            }
        }
        
        /* Create hash from normalized key event */
        gpointer hotkey_hash = create_hotkey_hash(normalized_modifiers, normalized_keyval);
        
        /* Debug logging for hotkey matching */
        g_debug("%s: Checking hotkey - original keyval=0x%X (%u), normalized_keyval=0x%X (%u), modifiers=0x%X", 
                LOG_TAG, keyval, keyval, normalized_keyval, normalized_keyval, normalized_modifiers);
        
        const gchar* keyboard_id = g_hash_table_lookup(engine->keyboard_hotkeys, hotkey_hash);
        
        if (keyboard_id) {
            g_debug("%s: Hotkey matched for keyboard: %s", LOG_TAG, keyboard_id);
            
            /* Switch to the keyboard */
            if (g_strcmp0(keyboard_id, engine->active_keyboard_id) != 0) {
                /* Commit any pending preedit text before switching */
                if (engine->preedit_visible && engine->preedit_text) {
                    keymagic_engine_commit_preedit(engine);
                }
                
                /* Update active keyboard */
                g_free(engine->active_keyboard_id);
                engine->active_keyboard_id = g_strdup(keyboard_id);
                engine->keyboard_changed = TRUE;
                
                /* Load the keyboard immediately */
                if (keymagic_ibus_engine_load_keyboard(engine, keyboard_id)) {
                    
                    /* Show notification using auxiliary text */
                    gchar* message = NULL;
                    
                    /* Try to get display name from config */
                    KeyMagicConfig* config = keymagic_config_load(engine->config_path);
                    if (config) {
                        InstalledKeyboard* kb_info = keymagic_config_get_keyboard_info(config, keyboard_id);
                        if (kb_info && kb_info->name) {
                            message = g_strdup_printf("Switched to: %s", kb_info->name);
                        }
                        keymagic_config_free(config);
                    }
                    
                    /* Fallback to keyboard ID if no display name found */
                    if (!message) {
                        message = g_strdup_printf("Switched to: %s", keyboard_id);
                    }
                    
                    IBusText* text = ibus_text_new_from_string(message);
                    ibus_engine_update_auxiliary_text((IBusEngine*)engine, text, TRUE);
                    g_free(message);
                    
                    /* Cancel any existing timeout */
                    if (engine->aux_text_timeout_id > 0) {
                        g_source_remove(engine->aux_text_timeout_id);
                    }
                    
                    /* Hide notification after 2 seconds */
                    engine->aux_text_timeout_id = g_timeout_add_seconds(2, 
                        aux_text_timeout_cb, engine);
                    
                    /* Update configuration file */
                    if (config) {
                        g_free(config->active_keyboard);
                        config->active_keyboard = g_strdup(keyboard_id);
                        keymagic_config_save(engine->config_path, config);
                        keymagic_config_free(config);
                    }
                }
            }
            
            /* Consume the hotkey - don't pass it to the application */
            return TRUE;
        }
    }
    
    /* Load keyboard on-demand if needed */
    if (engine->keyboard_changed && engine->active_keyboard_id) {
        keymagic_ibus_engine_load_keyboard(engine, engine->active_keyboard_id);
    }
    
    /* Silent error handling - eat printable keys when no valid keyboard */
    if (!engine->km_engine || engine->keyboard_load_failed) {
        gboolean should_eat = keymagic_engine_is_printable_ascii(keyval);
        if (should_eat) {
            g_debug("%s: Eating key (no valid keyboard): keyval=%u", LOG_TAG, keyval);
        }
        return should_eat;
    }
    
    /* Process key with engine */
    KeyProcessingResult result = {0};
    KeyMagicResult status = keymagic_ffi_process_key(engine->km_engine, keyval, keycode, 
                                                     modifiers, &result);
    
    if (status != KEYMAGIC_RESULT_SUCCESS) {
        g_warning("%s: Engine process key failed, marking keyboard as failed", LOG_TAG);
        engine->keyboard_load_failed = TRUE;
        keymagic_ffi_free_result(&result);
        return keymagic_engine_is_printable_ascii(keyval);
    }
    
    /* Handle preedit based on engine output */
    if (result.composing_text && strlen(result.composing_text) > 0) {
        LOG_COMPOSING_TEXT(result.composing_text);
        
        /* Check if we should commit */
        if (keymagic_engine_should_commit(keyval, result.is_processed, result.composing_text)) {
            g_debug("%s: Committing composition", LOG_TAG);
            
            /* Update preedit with the final composing text before committing */
            keymagic_engine_update_preedit(engine, result.composing_text);
            
            /* Commit the composing text */
            keymagic_engine_commit_preedit(engine);
            
            /* Reset engine after commit */
            keymagic_ffi_reset_engine(engine->km_engine);
            
            /* For unprocessed space, commit space too */
            if (keyval == IBUS_KEY_space && !result.is_processed) {
                ibus_engine_commit_text(ibus_engine, ibus_text_new_from_string(" "));
            }
        } else {
            /* Update preedit display */
            keymagic_engine_update_preedit(engine, result.composing_text);
        }
    } else {
        /* Engine has no composing text - clear preedit */
        g_debug("%s: Engine has no composing text, clearing preedit", LOG_TAG);
        keymagic_engine_clear_preedit(engine);
        
        /* Reset engine for special keys */
        switch (keyval) {
            case IBUS_KEY_Escape:
            case IBUS_KEY_Return:
            case IBUS_KEY_Tab:
                keymagic_ffi_reset_engine(engine->km_engine);
                break;
        }
    }
    
    gboolean consumed = result.is_processed;
    keymagic_ffi_free_result(&result);
    
    g_debug("%s: Key processing complete - consumed=%s", LOG_TAG, consumed ? "TRUE" : "FALSE");
    return consumed;
}

/**
 * Focus in - engine becomes active
 */
static void
keymagic_engine_focus_in(IBusEngine* ibus_engine)
{
    KeyMagicEngine* engine = KEYMAGIC_ENGINE(ibus_engine);
    g_debug("%s: Focus in", LOG_TAG);
    
    /* Reload config in case it changed while inactive */
    keymagic_engine_load_config(engine);
    
    /* Clear any stale state */
    keymagic_engine_clear_preedit(engine);
    if (engine->km_engine) {
        keymagic_ffi_reset_engine(engine->km_engine);
    }
    
    /* Call parent method */
    IBUS_ENGINE_CLASS(keymagic_engine_parent_class)->focus_in(ibus_engine);
}

/**
 * Focus out - engine becomes inactive
 */
static void
keymagic_engine_focus_out(IBusEngine* ibus_engine)
{
    KeyMagicEngine* engine = KEYMAGIC_ENGINE(ibus_engine);
    g_debug("%s: Focus out", LOG_TAG);
    
    /* Commit any pending preedit BEFORE parent focus_out */
    /* This ensures we can still send text to the client */
    keymagic_engine_commit_preedit(engine);
    
    /* Call parent method - this might disconnect from client */
    IBUS_ENGINE_CLASS(keymagic_engine_parent_class)->focus_out(ibus_engine);
    
    /* Reset engine state after parent processing */
    if (engine->km_engine) {
        keymagic_ffi_reset_engine(engine->km_engine);
    }
}

/**
 * Reset engine state
 */
static void
keymagic_engine_reset(IBusEngine* ibus_engine)
{
    KeyMagicEngine* engine = KEYMAGIC_ENGINE(ibus_engine);
    g_debug("%s: Reset", LOG_TAG);
    
    /* Always try to commit any pending preedit before reset */
    keymagic_engine_commit_preedit(engine);
    
    /* Reset core engine */
    if (engine->km_engine) {
        keymagic_ffi_reset_engine(engine->km_engine);
    }
    
    /* Call parent method */
    IBUS_ENGINE_CLASS(keymagic_engine_parent_class)->reset(ibus_engine);
}

/**
 * Enable engine
 */
static void
keymagic_engine_enable(IBusEngine* ibus_engine)
{
    KeyMagicEngine* engine = KEYMAGIC_ENGINE(ibus_engine);
    g_debug("%s: Enable", LOG_TAG);
    
    /* Load config and keyboard if needed */
    keymagic_engine_load_config(engine);
    
    /* Update keyboard properties for menu/hotkeys */
    keymagic_engine_update_properties(engine);
    
    /* Call parent method */
    IBUS_ENGINE_CLASS(keymagic_engine_parent_class)->enable(ibus_engine);
}

/**
 * Disable engine
 */
static void
keymagic_engine_disable(IBusEngine* ibus_engine)
{
    KeyMagicEngine* engine = KEYMAGIC_ENGINE(ibus_engine);
    g_debug("%s: Disable", LOG_TAG);
    
    /* Commit any pending preedit */
    keymagic_engine_commit_preedit(engine);
    
    /* Reset engine state */
    if (engine->km_engine) {
        keymagic_ffi_reset_engine(engine->km_engine);
    }
    
    /* Call parent method */
    IBUS_ENGINE_CLASS(keymagic_engine_parent_class)->disable(ibus_engine);
}

/**
 * Update preedit text display
 */
void
keymagic_engine_update_preedit(KeyMagicEngine* engine, const gchar* text)
{
    g_return_if_fail(KEYMAGIC_IS_ENGINE(engine));
    
    IBusEngine* ibus_engine = IBUS_ENGINE(engine);
    
    if (!text || strlen(text) == 0) {
        keymagic_engine_clear_preedit(engine);
        return;
    }
    
    /* Create preedit text with underline attribute */
    if (engine->preedit_text) {
        g_object_unref(engine->preedit_text);
        engine->preedit_text = NULL;
    }
    
    IBusText* preedit = ibus_text_new_from_string(text);
    ibus_text_append_attribute(preedit, IBUS_ATTR_TYPE_UNDERLINE, 
                              IBUS_ATTR_UNDERLINE_SINGLE, 0, -1);
    
    /* Update cursor position to end of text */
    engine->preedit_cursor_pos = g_utf8_strlen(text, -1);
    engine->preedit_visible = TRUE;
    
    /* Update IBus preedit - IBus takes ownership of the text object */
    ibus_engine_update_preedit_text(ibus_engine, preedit, 
                                   engine->preedit_cursor_pos, TRUE);
    
    /* Store a copy for our reference */
    engine->preedit_text = ibus_text_new_from_string(text);
    
    g_debug("%s: Updated preedit text: %s (cursor at %u)", LOG_TAG, text, engine->preedit_cursor_pos);
}

/**
 * Commit current preedit text
 */
void
keymagic_engine_commit_preedit(KeyMagicEngine* engine)
{
    g_return_if_fail(KEYMAGIC_IS_ENGINE(engine));
    
    if (engine->preedit_visible && engine->preedit_text) {
        IBusEngine* ibus_engine = IBUS_ENGINE(engine);
        
        /* Get text to commit */
        const gchar* text = ibus_text_get_text(engine->preedit_text);
        if (text && strlen(text) > 0) {
            g_debug("%s: Attempting to commit preedit text: %s", LOG_TAG, text);
            
            /* Hide preedit first to ensure it's not shown as underlined */
            ibus_engine_hide_preedit_text(ibus_engine);
            
            /* Create a new text object for commit - IBus takes ownership */
            IBusText* commit_text = ibus_text_new_from_string(text);
            ibus_engine_commit_text(ibus_engine, commit_text);
            g_debug("%s: Committed preedit text: %s", LOG_TAG, text);
        }
    }
    
    /* Clear preedit after commit */
    keymagic_engine_clear_preedit(engine);
}

/**
 * Clear preedit text display
 */
void
keymagic_engine_clear_preedit(KeyMagicEngine* engine)
{
    g_return_if_fail(KEYMAGIC_IS_ENGINE(engine));
    
    if (engine->preedit_visible) {
        IBusEngine* ibus_engine = IBUS_ENGINE(engine);
        
        /* Hide preedit */
        ibus_engine_hide_preedit_text(ibus_engine);
        engine->preedit_visible = FALSE;
        
        g_debug("%s: Cleared preedit text", LOG_TAG);
    }
    
    /* Free preedit text object */
    if (engine->preedit_text) {
        g_object_unref(engine->preedit_text);
        engine->preedit_text = NULL;
    }
    
    engine->preedit_cursor_pos = 0;
}

/**
 * Check if key is printable ASCII
 */
gboolean
keymagic_engine_is_printable_ascii(guint keyval)
{
    /* Printable ASCII range: ! to ~ (0x21 to 0x7E) */
    /* Excludes space (0x20) as per KeyMagic ANY keyword behavior */
    return (keyval >= 0x21 && keyval <= 0x7E);
}

/**
 * Determine if composition should be committed
 */
gboolean
keymagic_engine_should_commit(guint keyval, gboolean is_processed, const gchar* composing_text)
{
    /* If engine didn't process the key, commit */
    if (!is_processed) {
        return TRUE;
    }
    
    /* Check special keys that trigger commit */
    switch (keyval) {
        case IBUS_KEY_space:
            /* Commit if composing text ends with space */
            if (composing_text && strlen(composing_text) > 0) {
                return composing_text[strlen(composing_text) - 1] == ' ';
            }
            return FALSE;
            
        case IBUS_KEY_Return:
        case IBUS_KEY_Tab:
        case IBUS_KEY_Escape:
            /* Always commit for these keys */
            return TRUE;
            
        default:
            /* Don't commit for other keys */
            return FALSE;
    }
}

/**
 * Timeout callback for hiding auxiliary text
 */
static gboolean
aux_text_timeout_cb(gpointer user_data)
{
    KeyMagicEngine* engine = KEYMAGIC_ENGINE(user_data);
    
    /* Check if engine is still valid */
    if (!KEYMAGIC_IS_ENGINE(engine)) {
        return G_SOURCE_REMOVE;
    }
    
    /* Hide auxiliary text */
    ibus_engine_hide_auxiliary_text(IBUS_ENGINE(engine));
    
    /* Clear timeout ID */
    engine->aux_text_timeout_id = 0;
    
    /* Remove this timeout */
    return G_SOURCE_REMOVE;
}

/**
 * Config file change callback
 */
static void
config_file_changed_cb(GFileMonitor* monitor G_GNUC_UNUSED, GFile* file, 
                       GFile* other_file G_GNUC_UNUSED,
                       GFileMonitorEvent event_type, gpointer user_data)
{
    KeyMagicEngine* engine = KEYMAGIC_ENGINE(user_data);
    
    /* Only handle changes to the file content */
    if (event_type == G_FILE_MONITOR_EVENT_CHANGED || 
        event_type == G_FILE_MONITOR_EVENT_CREATED) {
        
        gchar* file_path = g_file_get_path(file);
        g_debug("%s: Config file changed: %s", LOG_TAG, file_path ? file_path : "(unknown)");
        g_free(file_path);
        
        /* Reload configuration */
        keymagic_engine_load_config(engine);
    }
}

/**
 * Parse hotkey string into modifiers and keyval
 * 
 * Uses the Rust FFI to parse hotkey strings like "Ctrl+Shift+M" into IBus modifiers and keyval.
 * 
 * @param hotkey_str Hotkey string (e.g., "Ctrl+Shift+M")
 * @param modifiers_out Output for modifier flags
 * @param keyval_out Output for key value
 * @return TRUE if parsing succeeded, FALSE otherwise
 */
static gboolean
parse_hotkey_string(const gchar* hotkey_str, guint* modifiers_out, guint* keyval_out)
{
    g_return_val_if_fail(hotkey_str != NULL, FALSE);
    g_return_val_if_fail(modifiers_out != NULL, FALSE);
    g_return_val_if_fail(keyval_out != NULL, FALSE);
    
    *modifiers_out = 0;
    *keyval_out = 0;
    
    /* Parse using Rust FFI */
    gint key_code;
    gboolean ctrl, alt, shift, meta;
    
    if (!keymagic_ffi_parse_hotkey(hotkey_str, &key_code, &ctrl, &alt, &shift, &meta)) {
        return FALSE;
    }
    
    /* Convert VirtualKey to IBus keyval */
    guint keyval = keymagic_map_virtual_key_to_ibus((guint16)key_code);
    if (keyval == 0) {
        return FALSE;
    }

    /* Build modifier mask */
    guint modifiers = 0;
    if (ctrl) modifiers |= IBUS_CONTROL_MASK;
    if (shift) modifiers |= IBUS_SHIFT_MASK;
    if (alt) modifiers |= IBUS_MOD1_MASK;
    if (meta) modifiers |= IBUS_SUPER_MASK;
    
    /* Must have at least one modifier */
    if (modifiers == 0) {
        return FALSE;
    }
    
    *modifiers_out = modifiers;
    *keyval_out = keyval;
    return TRUE;
}

/**
 * Create hotkey hash value from modifiers and keyval
 * 
 * Combines modifiers and keyval into a single value for hash table lookup.
 * Uses upper 16 bits for modifiers, lower 16 bits for keyval.
 */
static gpointer
create_hotkey_hash(guint modifiers, guint keyval)
{
    /* Mask out irrelevant modifier bits and combine with keyval */
    guint relevant_modifiers = modifiers & (IBUS_CONTROL_MASK | IBUS_SHIFT_MASK | 
                                           IBUS_MOD1_MASK | IBUS_SUPER_MASK);
    return GUINT_TO_POINTER((relevant_modifiers << 16) | (keyval & 0xFFFF));
}

/**
 * Create an IBus property for a keyboard
 * 
 * @param kb Keyboard information
 * @param prop_key Property key string
 * @param keyboards_dir Path to keyboards directory (for loading KM2 metadata)
 * @param is_active Whether this keyboard is currently active
 * @return New IBusProperty or NULL on error
 */
static IBusProperty*
create_keyboard_property(InstalledKeyboard* kb, const gchar* prop_key, 
                        const gchar* keyboards_dir, gboolean is_active)
{
    g_return_val_if_fail(kb != NULL, NULL);
    g_return_val_if_fail(prop_key != NULL, NULL);
    
    /* Use name from config or fallback to ID */
    const gchar* display_name = kb->name ? kb->name : kb->id;
    
    /* Determine hotkey to use */
    gchar* hotkey_to_use = NULL;
    gboolean hotkey_from_km2 = FALSE;
    
    if (kb->hotkey == NULL) {
        /* Hotkey not set in config - try to get from KM2 file */
        gchar* km2_path = NULL;
        if (kb->filename && keyboards_dir) {
            km2_path = g_build_filename(keyboards_dir, kb->filename, NULL);
        }
        
        if (km2_path && g_file_test(km2_path, G_FILE_TEST_EXISTS)) {
            void* km2_handle = keymagic_ffi_km2_load(km2_path);
            if (km2_handle) {
                hotkey_to_use = keymagic_ffi_km2_get_hotkey(km2_handle);
                hotkey_from_km2 = TRUE;
                keymagic_ffi_km2_free(km2_handle);
            }
        }
        g_free(km2_path);
    } else if (strlen(kb->hotkey) > 0) {
        /* Hotkey explicitly set in config */
        hotkey_to_use = g_strdup(kb->hotkey);
    }
    /* else: kb->hotkey is empty string - user disabled hotkey */
    
    /* Create IBus property */
    IBusText* label = ibus_text_new_from_string(display_name);
    IBusText* tooltip = NULL;
    
    if (hotkey_to_use && strlen(hotkey_to_use) > 0) {
        gchar* tooltip_str = g_strdup_printf("%s (%s)", display_name, hotkey_to_use);
        tooltip = ibus_text_new_from_string(tooltip_str);
        g_free(tooltip_str);
    } else {
        tooltip = ibus_text_new_from_string(display_name);
    }
    
    /* Determine property state (checked if active) */
    IBusPropState state = is_active ? PROP_STATE_CHECKED : PROP_STATE_UNCHECKED;
    
    /* Create property with radio button type for exclusive selection */
    IBusProperty* property = ibus_property_new(prop_key,
                                              PROP_TYPE_RADIO,
                                              label,
                                              NULL,  /* icon */
                                              tooltip,
                                              TRUE,  /* sensitive */
                                              TRUE,  /* visible */
                                              state,
                                              NULL); /* sub_props */
    
    g_debug("%s: Created keyboard property: %s (hotkey: %s%s)", LOG_TAG, display_name,
            hotkey_to_use ? hotkey_to_use : "none",
            hotkey_from_km2 ? " [from KM2]" : "");
    
    g_free(hotkey_to_use);
    
    return property;
}

/**
 * Update IBus properties for all available keyboards
 */
void
keymagic_engine_update_properties(KeyMagicEngine* engine)
{
    g_return_if_fail(KEYMAGIC_IS_ENGINE(engine));
    
    g_debug("%s: Updating keyboard properties with hotkeys", LOG_TAG);
    
    /* Create new property list */
    engine->prop_list = ibus_prop_list_new();
    
    /* Clear keyboard properties mapping */
    g_hash_table_remove_all(engine->keyboard_properties);
    
    /* Clear hotkey mappings */
    g_hash_table_remove_all(engine->keyboard_hotkeys);
    
    /* Load configuration to get installed keyboards */
    KeyMagicConfig* config = keymagic_config_load(engine->config_path);
    if (!config) {
        g_warning("%s: Failed to load config for keyboard properties", LOG_TAG);
        return;
    }
    
    gint keyboard_count = 0;
    gchar* keyboards_dir = keymagic_config_get_keyboards_dir();
    
    /* First, try to use installed keyboards from config */
    if (config->installed_keyboards) {
        GList* iter;
        for (iter = config->installed_keyboards; iter != NULL; iter = iter->next) {
            InstalledKeyboard* kb = (InstalledKeyboard*)iter->data;
            if (!kb || !kb->id) continue;
            
            /* Create property key */
            gchar* prop_key = g_strdup_printf("keyboard.%s", kb->id);
            
            /* Check if this is the active keyboard */
            gboolean is_active = (engine->active_keyboard_id && 
                                 g_strcmp0(kb->id, engine->active_keyboard_id) == 0);
            
            /* Create property using shared function */
            IBusProperty* property = create_keyboard_property(kb, prop_key, keyboards_dir, is_active);
            if (property) {
                /* Add to property list */
                ibus_prop_list_append(engine->prop_list, property);
                
                /* Map property key to keyboard ID */
                g_hash_table_insert(engine->keyboard_properties, 
                                   g_strdup(prop_key), 
                                   g_strdup(kb->id));
                
                /* Register hotkey if available */
                gchar* hotkey_str = NULL;
                if (kb->hotkey == NULL) {
                    /* Try to get from KM2 file */
                    gchar* km2_path = NULL;
                    if (kb->filename && keyboards_dir) {
                        km2_path = g_build_filename(keyboards_dir, kb->filename, NULL);
                    }
                    
                    if (km2_path && g_file_test(km2_path, G_FILE_TEST_EXISTS)) {
                        void* km2_handle = keymagic_ffi_km2_load(km2_path);
                        if (km2_handle) {
                            hotkey_str = keymagic_ffi_km2_get_hotkey(km2_handle);
                            keymagic_ffi_km2_free(km2_handle);
                        }
                    }
                    g_free(km2_path);
                } else if (strlen(kb->hotkey) > 0) {
                    hotkey_str = g_strdup(kb->hotkey);
                }
                
                /* Parse and register hotkey */
                if (hotkey_str && strlen(hotkey_str) > 0) {
                    guint modifiers, keyval;
                    if (parse_hotkey_string(hotkey_str, &modifiers, &keyval)) {
                        gpointer hotkey_hash = create_hotkey_hash(modifiers, keyval);
                        g_hash_table_insert(engine->keyboard_hotkeys, 
                                           hotkey_hash, 
                                           g_strdup(kb->id));
                        g_debug("%s: Registered hotkey %s for keyboard %s - keyval=0x%X (%u), modifiers=0x%X, hash=%p", 
                                LOG_TAG, hotkey_str, kb->id, keyval, keyval, modifiers, hotkey_hash);
                    } else {
                        g_warning("%s: Failed to parse hotkey '%s' for keyboard %s", 
                                  LOG_TAG, hotkey_str, kb->id);
                    }
                }
                g_free(hotkey_str);
                
                keyboard_count++;
            }
            
            g_free(prop_key);
        }
    }
    
    g_free(keyboards_dir);
    keymagic_config_free(config);
    
    /* Add separator if we have keyboards */
    if (keyboard_count > 0) {
        IBusProperty* separator = ibus_property_new("separator",
                                                   PROP_TYPE_SEPARATOR,
                                                   NULL,
                                                   NULL,
                                                   NULL,
                                                   TRUE,
                                                   TRUE,
                                                   PROP_STATE_UNCHECKED,
                                                   NULL);
        ibus_prop_list_append(engine->prop_list, separator);
    }
    
    /* Add "Open Configurator" menu item */
    IBusText* configurator_label = ibus_text_new_from_string("Open KeyMagic...");
    IBusText* configurator_tooltip = ibus_text_new_from_string("Open KeyMagic window");
    
    IBusProperty* configurator_prop = ibus_property_new("open-configurator",
                                                       PROP_TYPE_NORMAL,
                                                       configurator_label,
                                                       NULL,  /* icon */
                                                       configurator_tooltip,
                                                       TRUE,  /* sensitive */
                                                       TRUE,  /* visible */
                                                       PROP_STATE_UNCHECKED,
                                                       NULL); /* sub_props */
    
    ibus_prop_list_append(engine->prop_list, configurator_prop);
    
    /* Register properties with IBus */
    ibus_engine_register_properties((IBusEngine*)engine, engine->prop_list);
    g_debug("%s: Registered %d keyboard properties plus configurator menu", LOG_TAG, keyboard_count);
}

/**
 * Handle property activation (keyboard switching)
 */
void
keymagic_engine_activate_property(KeyMagicEngine* engine, const gchar* prop_name)
{
    g_return_if_fail(KEYMAGIC_IS_ENGINE(engine));
    g_return_if_fail(prop_name != NULL);
    
    g_debug("%s: Property activated: %s", LOG_TAG, prop_name);
    
    if (!engine->keyboard_properties)
        return;
        
    /* Look up keyboard ID from property name */
    const gchar* keyboard_id = g_hash_table_lookup(engine->keyboard_properties, prop_name);
    if (!keyboard_id) {
        g_debug("%s: Unknown property: %s", LOG_TAG, prop_name);
        return;
    }
    
    g_debug("%s: Switching to keyboard: %s", LOG_TAG, keyboard_id);
    
    /* Commit any pending preedit text before switching */
    if (engine->preedit_visible && engine->preedit_text) {
        keymagic_engine_commit_preedit(engine);
    }
    
    /* Update active keyboard in configuration */
    g_free(engine->active_keyboard_id);
    engine->active_keyboard_id = g_strdup(keyboard_id);
    engine->keyboard_changed = TRUE;
    
    /* Load the keyboard immediately */
    if (keymagic_ibus_engine_load_keyboard(engine, keyboard_id)) {
        /* Load config to update properties and save active keyboard */
        KeyMagicConfig* config = keymagic_config_load(engine->config_path);
        if (config) {
            /* Update property states to reflect new selection */
            if (engine->prop_list) {
                gchar* keyboards_dir = keymagic_config_get_keyboards_dir();
                
                /* IBusPropList is opaque, so we iterate through our hash table instead */
                GHashTableIter iter;
                gpointer key, value;
                g_hash_table_iter_init(&iter, engine->keyboard_properties);
                
                while (g_hash_table_iter_next(&iter, &key, &value)) {
                    const gchar* prop_key = (const gchar*)key;
                    const gchar* kb_id = (const gchar*)value;
                    
                    /* Find keyboard info from config */
                    InstalledKeyboard* kb_info = keymagic_config_get_keyboard_info(config, kb_id);
                    if (kb_info) {
                        /* Check if this keyboard is now active */
                        gboolean is_active = (g_strcmp0(prop_key, prop_name) == 0);
                        
                        /* Create property using shared function */
                        IBusProperty* prop = create_keyboard_property(kb_info, prop_key, 
                                                                     keyboards_dir, is_active);
                        if (prop) {
                            /* Update this property */
                            ibus_engine_update_property((IBusEngine*)engine, prop);
                            g_object_unref(prop);
                        }
                    }
                }
                
                g_free(keyboards_dir);
            }
            
            /* Update active keyboard and save configuration */
            g_free(config->active_keyboard);
            config->active_keyboard = g_strdup(keyboard_id);
            if (!keymagic_config_save(engine->config_path, config)) {
                g_warning("%s: Failed to persist keyboard selection to config", LOG_TAG);
            }
            
            keymagic_config_free(config);
        } else {
            g_warning("%s: Failed to load config for keyboard update", LOG_TAG);
        }
    }
}

/**
 * Handle property activation callback
 */
static void
keymagic_engine_property_activate(IBusEngine* ibus_engine,
                                 const gchar* prop_name,
                                 guint prop_state)
{
    KeyMagicEngine* engine = KEYMAGIC_ENGINE(ibus_engine);
    
    g_debug("%s: property_activate called - prop_name: %s, prop_state: %u", 
            LOG_TAG, prop_name, prop_state);
    
    /* Handle special menu items */
    if (g_strcmp0(prop_name, "open-configurator") == 0) {
        g_debug("%s: Opening KeyMagic configurator", LOG_TAG);
        
        /* Launch the KeyMagic configurator */
        GError* error = NULL;
        gchar* argv[] = { (gchar*)"keymagic3-gui", NULL };
        
        g_debug("%s: Attempting to spawn: %s", LOG_TAG, "keymagic3-gui");
        
        if (!g_spawn_async(NULL,           /* working directory */
                          argv,            /* argv */
                          NULL,            /* envp */
                          G_SPAWN_SEARCH_PATH | G_SPAWN_STDOUT_TO_DEV_NULL | G_SPAWN_STDERR_TO_DEV_NULL,
                          NULL,            /* child_setup */
                          NULL,            /* user_data */
                          NULL,            /* child_pid */
                          &error)) {
            g_warning("%s: Failed to launch KeyMagic configurator: %s", 
                     LOG_TAG, error ? error->message : "Unknown error");
            if (error) g_error_free(error);
        } else {
            g_debug("%s: Successfully spawned KeyMagic configurator", LOG_TAG);
        }
    }
    /* Handle radio button properties (keyboard selection) */
    else if (prop_state == PROP_STATE_CHECKED) {
        keymagic_engine_activate_property(engine, prop_name);
    }
    
    /* Call parent class method */
    IBUS_ENGINE_CLASS(keymagic_engine_parent_class)->property_activate(ibus_engine,
                                                                       prop_name,
                                                                       prop_state);
}