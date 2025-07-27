#include "config.h"
#include "toml.h"
#include <stdio.h>
#include <string.h>
#include <stdlib.h>

/* Logging tag */
#define LOG_TAG "KeyMagicConfig"

/**
 * Load configuration from TOML file using tomlc99
 */
KeyMagicConfig*
keymagic_config_load(const gchar* config_path)
{
    g_return_val_if_fail(config_path != NULL, NULL);
    
    if (!g_file_test(config_path, G_FILE_TEST_EXISTS)) {
        g_debug("%s: Config file does not exist: %s", LOG_TAG, config_path);
        return NULL;
    }
    
    /* Open and parse TOML file */
    FILE* fp = fopen(config_path, "r");
    if (!fp) {
        g_warning("%s: Cannot open config file: %s", LOG_TAG, config_path);
        return NULL;
    }
    
    char errbuf[256];
    toml_table_t* conf = toml_parse_file(fp, errbuf, sizeof(errbuf));
    fclose(fp);
    
    if (!conf) {
        g_warning("%s: Cannot parse config file %s: %s", LOG_TAG, config_path, errbuf);
        return NULL;
    }
    
    KeyMagicConfig* config = g_new0(KeyMagicConfig, 1);
    
    /* Set default values */
    config->start_with_system = FALSE;
    config->check_for_updates = TRUE;
    config->last_update_check = NULL;
    config->active_keyboard = NULL;
    config->last_used = NULL;
    config->enabled_processes = NULL;
    
    /* Parse [general] section */
    toml_table_t* general = toml_table_in(conf, "general");
    if (general) {
        toml_datum_t datum;
        
        datum = toml_bool_in(general, "start_with_system");
        if (datum.ok) {
            config->start_with_system = datum.u.b ? TRUE : FALSE;
        }
        
        datum = toml_bool_in(general, "check_for_updates");
        if (datum.ok) {
            config->check_for_updates = datum.u.b ? TRUE : FALSE;
        }
        
        datum = toml_string_in(general, "last_update_check");
        if (datum.ok) {
            config->last_update_check = g_strdup(datum.u.s);
            free(datum.u.s);
        }
    }
    
    /* Parse [keyboards] section */
    toml_table_t* keyboards = toml_table_in(conf, "keyboards");
    if (keyboards) {
        toml_datum_t datum;
        
        datum = toml_string_in(keyboards, "active");
        if (datum.ok) {
            config->active_keyboard = g_strdup(datum.u.s);
            free(datum.u.s);
        }
        
        /* Parse last_used array */
        toml_array_t* last_used = toml_array_in(keyboards, "last_used");
        if (last_used) {
            int n = toml_array_nelem(last_used);
            config->last_used = g_new0(gchar*, n + 1);
            
            for (int i = 0; i < n; i++) {
                datum = toml_string_at(last_used, i);
                if (datum.ok) {
                    config->last_used[i] = g_strdup(datum.u.s);
                    free(datum.u.s);
                }
            }
        }
    }
    
    /* Parse [composition_mode] section */
    toml_table_t* comp_mode = toml_table_in(conf, "composition_mode");
    if (comp_mode) {
        toml_array_t* enabled_procs = toml_array_in(comp_mode, "enabled_processes");
        if (enabled_procs) {
            int n = toml_array_nelem(enabled_procs);
            config->enabled_processes = g_new0(gchar*, n + 1);
            
            for (int i = 0; i < n; i++) {
                toml_datum_t datum = toml_string_at(enabled_procs, i);
                if (datum.ok) {
                    config->enabled_processes[i] = g_strdup(datum.u.s);
                    free(datum.u.s);
                }
            }
        }
    }
    
    toml_free(conf);
    
    g_debug("%s: Successfully loaded config from: %s", LOG_TAG, config_path);
    g_debug("%s: Active keyboard: %s", LOG_TAG, 
            config->active_keyboard ? config->active_keyboard : "(none)");
    
    return config;
}

/**
 * Free configuration structure
 */
void
keymagic_config_free(KeyMagicConfig* config)
{
    if (!config) {
        return;
    }
    
    g_free(config->last_update_check);
    g_free(config->active_keyboard);
    g_strfreev(config->last_used);
    g_strfreev(config->enabled_processes);
    
    g_free(config);
}

/**
 * Get default configuration path
 */
gchar*
keymagic_config_get_default_path(void)
{
    const gchar* config_dir = g_get_user_config_dir();
    if (!config_dir) {
        g_warning("%s: Failed to get user config directory", LOG_TAG);
        return NULL;
    }
    
    return g_build_filename(config_dir, "keymagic3", "config.toml", NULL);
}

/**
 * Get keyboards directory path
 */
gchar*
keymagic_config_get_keyboards_dir(void)
{
    const gchar* data_dir = g_get_user_data_dir();
    if (!data_dir) {
        g_warning("%s: Failed to get user data directory", LOG_TAG);
        return NULL;
    }
    
    return g_build_filename(data_dir, "keymagic3", "keyboards", NULL);
}

/**
 * Find keyboard file by ID
 */
gchar*
keymagic_config_find_keyboard_file(const gchar* keyboard_id)
{
    g_return_val_if_fail(keyboard_id != NULL, NULL);
    
    gchar* keyboards_dir = keymagic_config_get_keyboards_dir();
    if (!keyboards_dir) {
        return NULL;
    }
    
    /* Try direct match first: keyboard_id.km2 */
    gchar* filename = g_strdup_printf("%s.km2", keyboard_id);
    gchar* filepath = g_build_filename(keyboards_dir, filename, NULL);
    g_free(filename);
    
    if (g_file_test(filepath, G_FILE_TEST_EXISTS)) {
        g_free(keyboards_dir);
        return filepath;
    }
    g_free(filepath);
    
    /* Search through all .km2 files in directory */
    GDir* dir = g_dir_open(keyboards_dir, 0, NULL);
    if (!dir) {
        g_free(keyboards_dir);
        return NULL;
    }
    
    const gchar* entry;
    gchar* found_path = NULL;
    
    while ((entry = g_dir_read_name(dir)) != NULL) {
        if (g_str_has_suffix(entry, ".km2")) {
            /* Check if filename (without extension) matches keyboard_id */
            gchar* basename = g_strndup(entry, strlen(entry) - 4);  /* Remove .km2 */
            if (g_strcmp0(basename, keyboard_id) == 0) {
                found_path = g_build_filename(keyboards_dir, entry, NULL);
                g_free(basename);
                break;
            }
            g_free(basename);
        }
    }
    
    g_dir_close(dir);
    g_free(keyboards_dir);
    
    if (found_path) {
        g_debug("%s: Found keyboard file for ID '%s': %s", LOG_TAG, keyboard_id, found_path);
    } else {
        g_debug("%s: No keyboard file found for ID: %s", LOG_TAG, keyboard_id);
    }
    
    return found_path;
}