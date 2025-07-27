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

#endif /* KEYCODE_MAP_H */