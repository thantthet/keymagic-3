#include "engine.h"
#include "config.h"
#include "ffi_bridge.h"
#include <string.h>
#include <glib/gstdio.h>

/* Logging tag */
#define LOG_TAG "KeyMagicEngine"

/* File monitor event callback */
static void config_file_changed_cb(GFileMonitor* monitor, GFile* file, GFile* other_file,
                                   GFileMonitorEvent event_type, gpointer user_data);

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
}

/**
 * Initialize KeyMagicEngine instance
 */
static void
keymagic_engine_init(KeyMagicEngine* engine)
{
    g_debug("%s: Initializing KeyMagic engine", LOG_TAG);
    
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
    
    g_debug("%s: Finalizing KeyMagic engine", LOG_TAG);
    
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
    gboolean keyboard_changed = FALSE;
    if (g_strcmp0(engine->active_keyboard_id, config->active_keyboard) != 0) {
        keyboard_changed = TRUE;
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
    
    g_debug("%s: Processing key event - keyval=%u, keycode=%u, modifiers=%u", 
            LOG_TAG, keyval, keycode, modifiers);
    
    /* Only process key press events, ignore key release */
    if (modifiers & IBUS_RELEASE_MASK) {
        g_debug("%s: Ignoring key release event", LOG_TAG);
        return FALSE;
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
        g_debug("%s: Engine composing text: %s", LOG_TAG, result.composing_text);
        
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
    
    /* Commit any pending preedit */
    keymagic_engine_commit_preedit(engine);
    
    /* Reset engine state */
    if (engine->km_engine) {
        keymagic_ffi_reset_engine(engine->km_engine);
    }
    
    /* Call parent method */
    IBUS_ENGINE_CLASS(keymagic_engine_parent_class)->focus_out(ibus_engine);
}

/**
 * Reset engine state
 */
static void
keymagic_engine_reset(IBusEngine* ibus_engine)
{
    KeyMagicEngine* engine = KEYMAGIC_ENGINE(ibus_engine);
    g_debug("%s: Reset", LOG_TAG);
    
    /* Clear preedit */
    keymagic_engine_clear_preedit(engine);
    
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
 * Config file change callback
 */
static void
config_file_changed_cb(GFileMonitor* monitor, GFile* file, GFile* other_file,
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