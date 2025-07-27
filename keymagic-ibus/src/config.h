#ifndef KEYMAGIC_CONFIG_H
#define KEYMAGIC_CONFIG_H

#include <glib.h>

G_BEGIN_DECLS

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
    
    /* Keyboard settings */
    gchar* active_keyboard;             /* keyboards.active - ID of current keyboard */
    gchar** last_used;                  /* keyboards.last_used - NULL-terminated array */
    
    /* Composition mode settings */
    gchar** enabled_processes;          /* NULL-terminated array of process names */
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

G_END_DECLS

#endif /* KEYMAGIC_CONFIG_H */