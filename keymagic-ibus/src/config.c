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
    config->installed_keyboards = NULL;
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
        
        /* Parse installed array */
        toml_array_t* installed = toml_array_in(keyboards, "installed");
        if (installed) {
            int n = toml_array_nelem(installed);
            
            for (int i = 0; i < n; i++) {
                toml_table_t* kb_table = toml_table_at(installed, i);
                if (kb_table) {
                    InstalledKeyboard* kb = g_new0(InstalledKeyboard, 1);
                    
                    /* Parse keyboard fields */
                    datum = toml_string_in(kb_table, "id");
                    if (datum.ok) {
                        kb->id = g_strdup(datum.u.s);
                        free(datum.u.s);
                    }
                    
                    datum = toml_string_in(kb_table, "name");
                    if (datum.ok) {
                        kb->name = g_strdup(datum.u.s);
                        free(datum.u.s);
                    }
                    
                    datum = toml_string_in(kb_table, "filename");
                    if (datum.ok) {
                        kb->filename = g_strdup(datum.u.s);
                        free(datum.u.s);
                    }
                    
                    datum = toml_string_in(kb_table, "hotkey");
                    if (datum.ok) {
                        kb->hotkey = g_strdup(datum.u.s);
                        free(datum.u.s);
                    }
                    /* Note: kb->hotkey will be:
                     * - NULL if not present in TOML (use default from KM2)
                     * - Empty string "" if explicitly set to empty (hotkey disabled)
                     * - Non-empty string if hotkey is set
                     */
                    
                    datum = toml_string_in(kb_table, "hash");
                    if (datum.ok) {
                        kb->hash = g_strdup(datum.u.s);
                        free(datum.u.s);
                    }
                    
                    /* Add to list if we have at least an ID */
                    if (kb->id) {
                        config->installed_keyboards = g_list_append(config->installed_keyboards, kb);
                        g_debug("%s: Loaded installed keyboard: %s (%s)", LOG_TAG, 
                                kb->id, kb->name ? kb->name : "unnamed");
                    } else {
                        keymagic_config_free_keyboard(kb);
                    }
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
 * Free an InstalledKeyboard structure
 */
void
keymagic_config_free_keyboard(InstalledKeyboard* keyboard)
{
    if (!keyboard) {
        return;
    }
    
    g_free(keyboard->id);
    g_free(keyboard->name);
    g_free(keyboard->filename);
    g_free(keyboard->hotkey);
    g_free(keyboard->hash);
    g_free(keyboard);
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
    
    /* Free installed keyboards list */
    if (config->installed_keyboards) {
        g_list_free_full(config->installed_keyboards, (GDestroyNotify)keymagic_config_free_keyboard);
    }
    
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
 * Find keyboard file by ID or filename
 */
gchar*
keymagic_config_find_keyboard_file(const gchar* keyboard_id)
{
    g_return_val_if_fail(keyboard_id != NULL, NULL);
    
    /* First check if we have this keyboard in config with a filename */
    gchar* config_path = keymagic_config_get_default_path();
    if (config_path && g_file_test(config_path, G_FILE_TEST_EXISTS)) {
        KeyMagicConfig* config = keymagic_config_load(config_path);
        if (config) {
            InstalledKeyboard* kb_info = keymagic_config_get_keyboard_info(config, keyboard_id);
            if (kb_info && kb_info->filename) {
                gchar* keyboards_dir = keymagic_config_get_keyboards_dir();
                if (keyboards_dir) {
                    gchar* filepath = g_build_filename(keyboards_dir, kb_info->filename, NULL);
                    g_free(keyboards_dir);
                    
                    if (g_file_test(filepath, G_FILE_TEST_EXISTS)) {
                        keymagic_config_free(config);
                        g_free(config_path);
                        g_debug("%s: Found keyboard file from config for ID '%s': %s", 
                                LOG_TAG, keyboard_id, filepath);
                        return filepath;
                    }
                    g_free(filepath);
                }
            }
            keymagic_config_free(config);
        }
        g_free(config_path);
    }
    
    /* Fallback to directory scan */
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

/**
 * Get installed keyboard info by ID
 */
InstalledKeyboard*
keymagic_config_get_keyboard_info(KeyMagicConfig* config, const gchar* keyboard_id)
{
    g_return_val_if_fail(config != NULL, NULL);
    g_return_val_if_fail(keyboard_id != NULL, NULL);
    
    GList* iter;
    for (iter = config->installed_keyboards; iter != NULL; iter = iter->next) {
        InstalledKeyboard* kb = (InstalledKeyboard*)iter->data;
        if (kb && kb->id && g_strcmp0(kb->id, keyboard_id) == 0) {
            return kb;
        }
    }
    
    return NULL;
}

/**
 * Update active keyboard in configuration file
 */
gboolean
keymagic_config_update_active_keyboard(const gchar* config_path, const gchar* keyboard_id)
{
    g_return_val_if_fail(config_path != NULL, FALSE);
    g_return_val_if_fail(keyboard_id != NULL, FALSE);
    
    /* Load current configuration */
    KeyMagicConfig* config = keymagic_config_load(config_path);
    if (!config) {
        g_warning("%s: Failed to load config for update: %s", LOG_TAG, config_path);
        return FALSE;
    }
    
    /* Build TOML string */
    GString* toml_str = g_string_new("");
    
    /* General section */
    g_string_append(toml_str, "[general]\n");
    g_string_append_printf(toml_str, "start_with_system = %s\n", 
                          config->start_with_system ? "true" : "false");
    g_string_append_printf(toml_str, "check_for_updates = %s\n", 
                          config->check_for_updates ? "true" : "false");
    if (config->last_update_check) {
        g_string_append_printf(toml_str, "last_update_check = \"%s\"\n", 
                              config->last_update_check);
    }
    g_string_append(toml_str, "\n");
    
    /* Keyboards section */
    g_string_append(toml_str, "[keyboards]\n");
    
    /* Update active keyboard to the new value */
    g_string_append_printf(toml_str, "active = \"%s\"\n", keyboard_id);
    
    /* Add last_used array if present */
    if (config->last_used && config->last_used[0]) {
        g_string_append(toml_str, "last_used = [");
        for (gint i = 0; config->last_used[i] != NULL; i++) {
            if (i > 0) g_string_append(toml_str, ", ");
            g_string_append_printf(toml_str, "\"%s\"", config->last_used[i]);
        }
        g_string_append(toml_str, "]\n");
    }
    
    /* Add installed keyboards array */
    if (config->installed_keyboards) {
        g_string_append(toml_str, "\n");
        GList* iter;
        for (iter = config->installed_keyboards; iter != NULL; iter = iter->next) {
            InstalledKeyboard* kb = (InstalledKeyboard*)iter->data;
            if (!kb || !kb->id) continue;
            
            g_string_append(toml_str, "[[keyboards.installed]]\n");
            g_string_append_printf(toml_str, "id = \"%s\"\n", kb->id);
            if (kb->name) 
                g_string_append_printf(toml_str, "name = \"%s\"\n", kb->name);
            if (kb->filename) 
                g_string_append_printf(toml_str, "filename = \"%s\"\n", kb->filename);
            if (kb->hash) 
                g_string_append_printf(toml_str, "hash = \"%s\"\n", kb->hash);
            if (kb->hotkey) 
                g_string_append_printf(toml_str, "hotkey = \"%s\"\n", kb->hotkey);
            g_string_append(toml_str, "\n");
        }
    }
    
    /* Add composition_mode section if enabled_processes exists */
    if (config->enabled_processes && config->enabled_processes[0]) {
        g_string_append(toml_str, "[composition_mode]\n");
        g_string_append(toml_str, "enabled_processes = [");
        for (gint i = 0; config->enabled_processes[i] != NULL; i++) {
            if (i > 0) g_string_append(toml_str, ", ");
            g_string_append_printf(toml_str, "\"%s\"", config->enabled_processes[i]);
        }
        g_string_append(toml_str, "]\n");
    }
    
    /* Write to file */
    GError* error = NULL;
    gboolean success = g_file_set_contents(config_path, toml_str->str, -1, &error);
    
    if (!success) {
        g_warning("%s: Failed to write config file: %s", LOG_TAG, 
                  error ? error->message : "Unknown error");
        if (error) g_error_free(error);
    } else {
        g_debug("%s: Successfully updated active keyboard to: %s", LOG_TAG, keyboard_id);
    }
    
    g_string_free(toml_str, TRUE);
    keymagic_config_free(config);
    
    return success;
}