#ifndef KEYMAGIC_ENGINE_H
#define KEYMAGIC_ENGINE_H

#include <ibus.h>
#include <glib.h>
#include <gio/gio.h>

G_BEGIN_DECLS

#define KEYMAGIC_TYPE_ENGINE (keymagic_engine_get_type())
#define KEYMAGIC_ENGINE(o) (G_TYPE_CHECK_INSTANCE_CAST((o), KEYMAGIC_TYPE_ENGINE, KeyMagicEngine))
#define KEYMAGIC_IS_ENGINE(o) (G_TYPE_CHECK_INSTANCE_TYPE((o), KEYMAGIC_TYPE_ENGINE))

typedef struct _KeyMagicEngine KeyMagicEngine;
typedef struct _KeyMagicEngineClass KeyMagicEngineClass;

/**
 * KeyMagic IBus Engine Structure
 * 
 * Manages keyboard layouts and input processing for the KeyMagic input method.
 * Uses file-based configuration monitoring and on-demand keyboard loading.
 */
struct _KeyMagicEngine {
    IBusEngine parent;
    
    /* Core engine handle from keymagic-core (Rust FFI) */
    void* km_engine;                    /* EngineHandle* - NULL if no keyboard loaded */
    
    /* Configuration and keyboard management */
    gchar* active_keyboard_id;          /* Current keyboard ID from config */
    gchar* keyboard_path;               /* Path to current .km2 file */
    gchar* config_path;                 /* Path to config.toml file */
    GFileMonitor* config_monitor;       /* Monitor for config file changes */
    
    /* State management */
    gboolean keyboard_load_failed;      /* TRUE if current keyboard failed to load */
    gboolean keyboard_changed;          /* TRUE if config indicates keyboard change */
    
    /* Preedit text management */
    IBusText* preedit_text;             /* Current preedit text being composed */
    gboolean preedit_visible;           /* Whether preedit is currently shown */
    guint preedit_cursor_pos;           /* Cursor position in preedit text */
    
    /* Property management for keyboard switching */
    IBusPropList* prop_list;            /* List of properties (keyboards with hotkeys) */
    GHashTable* keyboard_properties;    /* Maps property key to keyboard ID */
};

struct _KeyMagicEngineClass {
    IBusEngineClass parent;
};

/* Type registration */
GType keymagic_engine_get_type(void) G_GNUC_CONST;

/* Engine lifecycle */
KeyMagicEngine* keymagic_ibus_engine_new(void);

/* Configuration management */
gboolean keymagic_engine_load_config(KeyMagicEngine* engine);
gboolean keymagic_ibus_engine_load_keyboard(KeyMagicEngine* engine, const gchar* keyboard_id);
void keymagic_engine_unload_keyboard(KeyMagicEngine* engine);

/* Preedit text management */
void keymagic_engine_update_preedit(KeyMagicEngine* engine, const gchar* text);
void keymagic_engine_commit_preedit(KeyMagicEngine* engine);
void keymagic_engine_clear_preedit(KeyMagicEngine* engine);

/* Utility functions */
gboolean keymagic_engine_is_printable_ascii(guint keyval);
gboolean keymagic_engine_should_commit(guint keyval, gboolean is_processed, const gchar* composing_text);

/* Property/hotkey management */
void keymagic_engine_update_properties(KeyMagicEngine* engine);
void keymagic_engine_activate_property(KeyMagicEngine* engine, const gchar* prop_name);

G_END_DECLS

#endif /* KEYMAGIC_ENGINE_H */