/**
 * IBus to KeyMagic keycode mapping
 * 
 * This module provides mapping between IBus key values and KeyMagic VirtualKey codes.
 * IBus uses X11 keysyms (keyvals) which need to be mapped to KeyMagic's internal 
 * VirtualKey representation.
 */

#include <ibus.h>

/**
 * Map IBus keyval to KeyMagic VirtualKey code
 * 
 * @param keyval IBus key value (X11 keysym)
 * @return KeyMagic VirtualKey code, or 0 if no mapping exists
 */
guint16
keymagic_map_ibus_keyval(guint keyval)
{
    switch (keyval) {
        /* Control keys */
        case IBUS_KEY_BackSpace:    return 2;   /* VirtualKey::Back */
        case IBUS_KEY_Tab:          return 3;   /* VirtualKey::Tab */
        case IBUS_KEY_Return:       return 4;   /* VirtualKey::Return */
        case IBUS_KEY_KP_Enter:     return 4;   /* VirtualKey::Return */
        case IBUS_KEY_Shift_L:      return 80;  /* VirtualKey::LShift */
        case IBUS_KEY_Shift_R:      return 81;  /* VirtualKey::RShift */
        case IBUS_KEY_Control_L:    return 82;  /* VirtualKey::LControl */
        case IBUS_KEY_Control_R:    return 83;  /* VirtualKey::RControl */
        case IBUS_KEY_Alt_L:        return 84;  /* VirtualKey::LAlt */
        case IBUS_KEY_Alt_R:        return 85;  /* VirtualKey::RAlt */
        case IBUS_KEY_Pause:        return 8;   /* VirtualKey::Pause */
        case IBUS_KEY_Caps_Lock:    return 9;   /* VirtualKey::Capital */
        case IBUS_KEY_Escape:       return 11;  /* VirtualKey::Escape */
        case IBUS_KEY_space:        return 12;  /* VirtualKey::Space */
        case IBUS_KEY_Page_Up:      return 13;  /* VirtualKey::Prior */
        case IBUS_KEY_Page_Down:    return 14;  /* VirtualKey::Next */
        case IBUS_KEY_Delete:       return 15;  /* VirtualKey::Delete */
        
        /* Number keys (0-9) */
        case IBUS_KEY_0:            return 16;  /* VirtualKey::Key0 */
        case IBUS_KEY_1:            return 17;  /* VirtualKey::Key1 */
        case IBUS_KEY_2:            return 18;  /* VirtualKey::Key2 */
        case IBUS_KEY_3:            return 19;  /* VirtualKey::Key3 */
        case IBUS_KEY_4:            return 20;  /* VirtualKey::Key4 */
        case IBUS_KEY_5:            return 21;  /* VirtualKey::Key5 */
        case IBUS_KEY_6:            return 22;  /* VirtualKey::Key6 */
        case IBUS_KEY_7:            return 23;  /* VirtualKey::Key7 */
        case IBUS_KEY_8:            return 24;  /* VirtualKey::Key8 */
        case IBUS_KEY_9:            return 25;  /* VirtualKey::Key9 */
        
        /* Letter keys (A-Z) - handle both lowercase and uppercase */
        case IBUS_KEY_a:
        case IBUS_KEY_A:            return 26;  /* VirtualKey::KeyA */
        case IBUS_KEY_b:
        case IBUS_KEY_B:            return 27;  /* VirtualKey::KeyB */
        case IBUS_KEY_c:
        case IBUS_KEY_C:            return 28;  /* VirtualKey::KeyC */
        case IBUS_KEY_d:
        case IBUS_KEY_D:            return 29;  /* VirtualKey::KeyD */
        case IBUS_KEY_e:
        case IBUS_KEY_E:            return 30;  /* VirtualKey::KeyE */
        case IBUS_KEY_f:
        case IBUS_KEY_F:            return 31;  /* VirtualKey::KeyF */
        case IBUS_KEY_g:
        case IBUS_KEY_G:            return 32;  /* VirtualKey::KeyG */
        case IBUS_KEY_h:
        case IBUS_KEY_H:            return 33;  /* VirtualKey::KeyH */
        case IBUS_KEY_i:
        case IBUS_KEY_I:            return 34;  /* VirtualKey::KeyI */
        case IBUS_KEY_j:
        case IBUS_KEY_J:            return 35;  /* VirtualKey::KeyJ */
        case IBUS_KEY_k:
        case IBUS_KEY_K:            return 36;  /* VirtualKey::KeyK */
        case IBUS_KEY_l:
        case IBUS_KEY_L:            return 37;  /* VirtualKey::KeyL */
        case IBUS_KEY_m:
        case IBUS_KEY_M:            return 38;  /* VirtualKey::KeyM */
        case IBUS_KEY_n:
        case IBUS_KEY_N:            return 39;  /* VirtualKey::KeyN */
        case IBUS_KEY_o:
        case IBUS_KEY_O:            return 40;  /* VirtualKey::KeyO */
        case IBUS_KEY_p:
        case IBUS_KEY_P:            return 41;  /* VirtualKey::KeyP */
        case IBUS_KEY_q:
        case IBUS_KEY_Q:            return 42;  /* VirtualKey::KeyQ */
        case IBUS_KEY_r:
        case IBUS_KEY_R:            return 43;  /* VirtualKey::KeyR */
        case IBUS_KEY_s:
        case IBUS_KEY_S:            return 44;  /* VirtualKey::KeyS */
        case IBUS_KEY_t:
        case IBUS_KEY_T:            return 45;  /* VirtualKey::KeyT */
        case IBUS_KEY_u:
        case IBUS_KEY_U:            return 46;  /* VirtualKey::KeyU */
        case IBUS_KEY_v:
        case IBUS_KEY_V:            return 47;  /* VirtualKey::KeyV */
        case IBUS_KEY_w:
        case IBUS_KEY_W:            return 48;  /* VirtualKey::KeyW */
        case IBUS_KEY_x:
        case IBUS_KEY_X:            return 49;  /* VirtualKey::KeyX */
        case IBUS_KEY_y:
        case IBUS_KEY_Y:            return 50;  /* VirtualKey::KeyY */
        case IBUS_KEY_z:
        case IBUS_KEY_Z:            return 51;  /* VirtualKey::KeyZ */
        
        /* Numpad keys */
        case IBUS_KEY_KP_0:         return 52;  /* VirtualKey::Numpad0 */
        case IBUS_KEY_KP_1:         return 53;  /* VirtualKey::Numpad1 */
        case IBUS_KEY_KP_2:         return 54;  /* VirtualKey::Numpad2 */
        case IBUS_KEY_KP_3:         return 55;  /* VirtualKey::Numpad3 */
        case IBUS_KEY_KP_4:         return 56;  /* VirtualKey::Numpad4 */
        case IBUS_KEY_KP_5:         return 57;  /* VirtualKey::Numpad5 */
        case IBUS_KEY_KP_6:         return 58;  /* VirtualKey::Numpad6 */
        case IBUS_KEY_KP_7:         return 59;  /* VirtualKey::Numpad7 */
        case IBUS_KEY_KP_8:         return 60;  /* VirtualKey::Numpad8 */
        case IBUS_KEY_KP_9:         return 61;  /* VirtualKey::Numpad9 */
        
        /* Numpad operators */
        case IBUS_KEY_KP_Multiply:  return 62;  /* VirtualKey::Multiply */
        case IBUS_KEY_KP_Add:       return 63;  /* VirtualKey::Add */
        case IBUS_KEY_KP_Separator: return 64;  /* VirtualKey::Separator */
        case IBUS_KEY_KP_Subtract:  return 65;  /* VirtualKey::Subtract */
        case IBUS_KEY_KP_Decimal:   return 66;  /* VirtualKey::Decimal */
        case IBUS_KEY_KP_Divide:    return 67;  /* VirtualKey::Divide */
        
        /* Function keys */
        case IBUS_KEY_F1:           return 68;  /* VirtualKey::F1 */
        case IBUS_KEY_F2:           return 69;  /* VirtualKey::F2 */
        case IBUS_KEY_F3:           return 70;  /* VirtualKey::F3 */
        case IBUS_KEY_F4:           return 71;  /* VirtualKey::F4 */
        case IBUS_KEY_F5:           return 72;  /* VirtualKey::F5 */
        case IBUS_KEY_F6:           return 73;  /* VirtualKey::F6 */
        case IBUS_KEY_F7:           return 74;  /* VirtualKey::F7 */
        case IBUS_KEY_F8:           return 75;  /* VirtualKey::F8 */
        case IBUS_KEY_F9:           return 76;  /* VirtualKey::F9 */
        case IBUS_KEY_F10:          return 77;  /* VirtualKey::F10 */
        case IBUS_KEY_F11:          return 78;  /* VirtualKey::F11 */
        case IBUS_KEY_F12:          return 79;  /* VirtualKey::F12 */
        
        /* OEM keys */
        case IBUS_KEY_semicolon:
        case IBUS_KEY_colon:        return 86;  /* VirtualKey::Oem1 */
        case IBUS_KEY_equal:
        case IBUS_KEY_plus:         return 87;  /* VirtualKey::OemPlus */
        case IBUS_KEY_comma:
        case IBUS_KEY_less:         return 88;  /* VirtualKey::OemComma */
        case IBUS_KEY_minus:
        case IBUS_KEY_underscore:   return 89;  /* VirtualKey::OemMinus */
        case IBUS_KEY_period:
        case IBUS_KEY_greater:      return 90;  /* VirtualKey::OemPeriod */
        case IBUS_KEY_slash:
        case IBUS_KEY_question:     return 91;  /* VirtualKey::Oem2 */
        case IBUS_KEY_grave:
        case IBUS_KEY_asciitilde:   return 92;  /* VirtualKey::Oem3 */
        case IBUS_KEY_bracketleft:
        case IBUS_KEY_braceleft:    return 93;  /* VirtualKey::Oem4 */
        case IBUS_KEY_backslash:
        case IBUS_KEY_bar:          return 94;  /* VirtualKey::Oem5 */
        case IBUS_KEY_bracketright:
        case IBUS_KEY_braceright:   return 95;  /* VirtualKey::Oem6 */
        case IBUS_KEY_apostrophe:
        case IBUS_KEY_quotedbl:     return 96;  /* VirtualKey::Oem7 */
        
        /* Default: return 0 for unmapped keys */
        default:
            return 0;
    }
}