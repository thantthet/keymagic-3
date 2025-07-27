#ifndef KEYCODE_MAP_H
#define KEYCODE_MAP_H

#include <glib.h>

/**
 * Map IBus keyval to KeyMagic VirtualKey code
 * 
 * @param keyval IBus key value (X11 keysym)
 * @return KeyMagic VirtualKey code, or 0 if no mapping exists
 */
guint16 keymagic_map_ibus_keyval(guint keyval);

/**
 * Map KeyMagic VirtualKey code to IBus keyval
 * 
 * @param vk_code KeyMagic VirtualKey code
 * @return IBus key value (X11 keysym), or 0 if no mapping exists
 */
guint keymagic_map_virtual_key_to_ibus(guint16 vk_code);

#endif /* KEYCODE_MAP_H */