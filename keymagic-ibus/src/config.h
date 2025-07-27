#ifndef KEYMAGIC_CONFIG_H
#define KEYMAGIC_CONFIG_H

#include <glib.h>

G_BEGIN_DECLS

/**
 * Installed Keyboard Information
 * 
 * Represents a keyboard entry in the keyboards.installed array
 */
typedef struct {
    gchar* id;                          /* Keyboard ID */
    gchar* name;                        /* Display name */
    gchar* filename;                    /* Filename (not full path) */
    gchar* hotkey;                      /* Hotkey string or NULL */
    gchar* hash;                        /* File hash */
} InstalledKeyboard;

/**
 * KeyMagic Configuration Structure
 * 
 * Represents the parsed TOML configuration from ~/.config/keymagic/config.toml
 * Matches the structure used by the cross-platform GUI.
 */
typedef struct {
    /* General settings */
    gboolean start_with_system;
    gboolean check_for_updates;
    gchar* last_update_check;           /* ISO 8601 timestamp or NULL */
    gchar* last_scanned_version;        /* Last scanned app version or NULL */
    gchar* update_remind_after;         /* ISO 8601 timestamp or NULL */
    
    /* Keyboard settings */
    gchar* active_keyboard;             /* keyboards.active - ID of current keyboard */
    gchar** last_used;                  /* keyboards.last_used - NULL-terminated array */
    GList* installed_keyboards;         /* keyboards.installed - List of InstalledKeyboard* */
    
    /* Composition mode settings */
    gchar** composition_mode_hosts;     /* NULL-terminated array of host names/processes */
    
    /* Direct mode settings */
    gchar** direct_mode_hosts;          /* NULL-terminated array of host names */
} KeyMagicConfig;

/**
 * Load configuration from TOML file
 * 
 * @param config_path Path to config.toml file
 * @return Parsed configuration or NULL on error
 */
KeyMagicConfig* keymagic_config_load(const gchar* config_path);

/**
 * Free configuration structure
 * 
 * @param config Configuration to free
 */
void keymagic_config_free(KeyMagicConfig* config);

/**
 * Get default configuration path
 * 
 * @return Path to ~/.config/keymagic/config.toml (caller must free)
 */
gchar* keymagic_config_get_default_path(void);

/**
 * Get keyboards directory path
 * 
 * @return Path to ~/.local/share/keymagic/keyboards (caller must free)
 */
gchar* keymagic_config_get_keyboards_dir(void);

/**
 * Find keyboard file by ID
 * 
 * @param keyboard_id Keyboard ID to search for
 * @return Full path to .km2 file or NULL if not found (caller must free)
 */
gchar* keymagic_config_find_keyboard_file(const gchar* keyboard_id);

/**
 * Get installed keyboard info by ID
 * 
 * @param config Configuration structure
 * @param keyboard_id Keyboard ID to search for
 * @return InstalledKeyboard info or NULL if not found (do not free - owned by config)
 */
InstalledKeyboard* keymagic_config_get_keyboard_info(KeyMagicConfig* config, const gchar* keyboard_id);

/**
 * Free an InstalledKeyboard structure
 * 
 * @param keyboard InstalledKeyboard to free
 */
void keymagic_config_free_keyboard(InstalledKeyboard* keyboard);

/**
 * Save configuration to TOML file
 * 
 * @param config_path Path to config.toml file
 * @param config Configuration to save
 * @return TRUE on success, FALSE on error
 */
gboolean keymagic_config_save(const gchar* config_path, const KeyMagicConfig* config);

G_END_DECLS

#endif /* KEYMAGIC_CONFIG_H */