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
        
        /* Shifted number keys (map to base number keys) */
        case IBUS_KEY_exclam:       return 17;  /* VirtualKey::Key1 (!) */
        case IBUS_KEY_at:           return 18;  /* VirtualKey::Key2 (@) */
        case IBUS_KEY_numbersign:   return 19;  /* VirtualKey::Key3 (#) */
        case IBUS_KEY_dollar:       return 20;  /* VirtualKey::Key4 ($) */
        case IBUS_KEY_percent:      return 21;  /* VirtualKey::Key5 (%) */
        case IBUS_KEY_asciicircum:  return 22;  /* VirtualKey::Key6 (^) */
        case IBUS_KEY_ampersand:    return 23;  /* VirtualKey::Key7 (&) */
        case IBUS_KEY_asterisk:     return 24;  /* VirtualKey::Key8 (*) */
        case IBUS_KEY_parenleft:    return 25;  /* VirtualKey::Key9 (() */
        case IBUS_KEY_parenright:   return 16;  /* VirtualKey::Key0 ()) */
        
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

/**
 * Map KeyMagic VirtualKey code to IBus keyval
 * 
 * @param vk_code KeyMagic VirtualKey code
 * @return IBus key value (X11 keysym), or 0 if no mapping exists
 */
guint
keymagic_map_virtual_key_to_ibus(guint16 vk_code)
{
    switch (vk_code) {
        /* Control keys */
        case 2:  return IBUS_KEY_BackSpace;     /* VirtualKey::Back */
        case 3:  return IBUS_KEY_Tab;           /* VirtualKey::Tab */
        case 4:  return IBUS_KEY_Return;        /* VirtualKey::Return */
        case 5:  return IBUS_KEY_Shift_L;       /* VirtualKey::Shift */
        case 6:  return IBUS_KEY_Control_L;     /* VirtualKey::Control */
        case 7:  return IBUS_KEY_Alt_L;         /* VirtualKey::Menu */
        case 8:  return IBUS_KEY_Pause;         /* VirtualKey::Pause */
        case 9:  return IBUS_KEY_Caps_Lock;     /* VirtualKey::Capital */
        case 11: return IBUS_KEY_Escape;        /* VirtualKey::Escape */
        case 12: return IBUS_KEY_space;         /* VirtualKey::Space */
        case 13: return IBUS_KEY_Prior;         /* VirtualKey::Prior (Page Up) */
        case 14: return IBUS_KEY_Next;          /* VirtualKey::Next (Page Down) */
        case 15: return IBUS_KEY_Delete;        /* VirtualKey::Delete */
        
        /* Number keys */
        case 16: return IBUS_KEY_0;             /* VirtualKey::Key0 */
        case 17: return IBUS_KEY_1;             /* VirtualKey::Key1 */
        case 18: return IBUS_KEY_2;             /* VirtualKey::Key2 */
        case 19: return IBUS_KEY_3;             /* VirtualKey::Key3 */
        case 20: return IBUS_KEY_4;             /* VirtualKey::Key4 */
        case 21: return IBUS_KEY_5;             /* VirtualKey::Key5 */
        case 22: return IBUS_KEY_6;             /* VirtualKey::Key6 */
        case 23: return IBUS_KEY_7;             /* VirtualKey::Key7 */
        case 24: return IBUS_KEY_8;             /* VirtualKey::Key8 */
        case 25: return IBUS_KEY_9;             /* VirtualKey::Key9 */
        
        /* Letter keys */
        case 26: return IBUS_KEY_a;             /* VirtualKey::KeyA */
        case 27: return IBUS_KEY_b;             /* VirtualKey::KeyB */
        case 28: return IBUS_KEY_c;             /* VirtualKey::KeyC */
        case 29: return IBUS_KEY_d;             /* VirtualKey::KeyD */
        case 30: return IBUS_KEY_e;             /* VirtualKey::KeyE */
        case 31: return IBUS_KEY_f;             /* VirtualKey::KeyF */
        case 32: return IBUS_KEY_g;             /* VirtualKey::KeyG */
        case 33: return IBUS_KEY_h;             /* VirtualKey::KeyH */
        case 34: return IBUS_KEY_i;             /* VirtualKey::KeyI */
        case 35: return IBUS_KEY_j;             /* VirtualKey::KeyJ */
        case 36: return IBUS_KEY_k;             /* VirtualKey::KeyK */
        case 37: return IBUS_KEY_l;             /* VirtualKey::KeyL */
        case 38: return IBUS_KEY_m;             /* VirtualKey::KeyM */
        case 39: return IBUS_KEY_n;             /* VirtualKey::KeyN */
        case 40: return IBUS_KEY_o;             /* VirtualKey::KeyO */
        case 41: return IBUS_KEY_p;             /* VirtualKey::KeyP */
        case 42: return IBUS_KEY_q;             /* VirtualKey::KeyQ */
        case 43: return IBUS_KEY_r;             /* VirtualKey::KeyR */
        case 44: return IBUS_KEY_s;             /* VirtualKey::KeyS */
        case 45: return IBUS_KEY_t;             /* VirtualKey::KeyT */
        case 46: return IBUS_KEY_u;             /* VirtualKey::KeyU */
        case 47: return IBUS_KEY_v;             /* VirtualKey::KeyV */
        case 48: return IBUS_KEY_w;             /* VirtualKey::KeyW */
        case 49: return IBUS_KEY_x;             /* VirtualKey::KeyX */
        case 50: return IBUS_KEY_y;             /* VirtualKey::KeyY */
        case 51: return IBUS_KEY_z;             /* VirtualKey::KeyZ */
        
        /* Numpad keys */
        case 52: return IBUS_KEY_KP_0;          /* VirtualKey::Numpad0 */
        case 53: return IBUS_KEY_KP_1;          /* VirtualKey::Numpad1 */
        case 54: return IBUS_KEY_KP_2;          /* VirtualKey::Numpad2 */
        case 55: return IBUS_KEY_KP_3;          /* VirtualKey::Numpad3 */
        case 56: return IBUS_KEY_KP_4;          /* VirtualKey::Numpad4 */
        case 57: return IBUS_KEY_KP_5;          /* VirtualKey::Numpad5 */
        case 58: return IBUS_KEY_KP_6;          /* VirtualKey::Numpad6 */
        case 59: return IBUS_KEY_KP_7;          /* VirtualKey::Numpad7 */
        case 60: return IBUS_KEY_KP_8;          /* VirtualKey::Numpad8 */
        case 61: return IBUS_KEY_KP_9;          /* VirtualKey::Numpad9 */
        
        /* Numpad operators */
        case 62: return IBUS_KEY_KP_Multiply;   /* VirtualKey::Multiply */
        case 63: return IBUS_KEY_KP_Add;        /* VirtualKey::Add */
        case 64: return IBUS_KEY_KP_Separator;  /* VirtualKey::Separator */
        case 65: return IBUS_KEY_KP_Subtract;   /* VirtualKey::Subtract */
        case 66: return IBUS_KEY_KP_Decimal;    /* VirtualKey::Decimal */
        case 67: return IBUS_KEY_KP_Divide;     /* VirtualKey::Divide */
        
        /* Function keys */
        case 68: return IBUS_KEY_F1;            /* VirtualKey::F1 */
        case 69: return IBUS_KEY_F2;            /* VirtualKey::F2 */
        case 70: return IBUS_KEY_F3;            /* VirtualKey::F3 */
        case 71: return IBUS_KEY_F4;            /* VirtualKey::F4 */
        case 72: return IBUS_KEY_F5;            /* VirtualKey::F5 */
        case 73: return IBUS_KEY_F6;            /* VirtualKey::F6 */
        case 74: return IBUS_KEY_F7;            /* VirtualKey::F7 */
        case 75: return IBUS_KEY_F8;            /* VirtualKey::F8 */
        case 76: return IBUS_KEY_F9;            /* VirtualKey::F9 */
        case 77: return IBUS_KEY_F10;           /* VirtualKey::F10 */
        case 78: return IBUS_KEY_F11;           /* VirtualKey::F11 */
        case 79: return IBUS_KEY_F12;           /* VirtualKey::F12 */
        
        /* Modifier keys (left/right variants) */
        case 80: return IBUS_KEY_Shift_L;       /* VirtualKey::LShift */
        case 81: return IBUS_KEY_Shift_R;       /* VirtualKey::RShift */
        case 82: return IBUS_KEY_Control_L;     /* VirtualKey::LControl */
        case 83: return IBUS_KEY_Control_R;     /* VirtualKey::RControl */
        case 84: return IBUS_KEY_Alt_L;         /* VirtualKey::LMenu */
        case 85: return IBUS_KEY_Alt_R;         /* VirtualKey::RMenu */
        
        /* OEM keys */
        case 86: return IBUS_KEY_semicolon;     /* VirtualKey::Oem1 (;:) */
        case 87: return IBUS_KEY_plus;          /* VirtualKey::OemPlus */
        case 88: return IBUS_KEY_comma;         /* VirtualKey::OemComma */
        case 89: return IBUS_KEY_minus;         /* VirtualKey::OemMinus */
        case 90: return IBUS_KEY_period;        /* VirtualKey::OemPeriod */
        case 91: return IBUS_KEY_slash;         /* VirtualKey::Oem2 (/?) */
        case 92: return IBUS_KEY_grave;         /* VirtualKey::Oem3 (`~) */
        case 93: return IBUS_KEY_bracketleft;   /* VirtualKey::Oem4 ([{) */
        case 94: return IBUS_KEY_backslash;     /* VirtualKey::Oem5 (\|) */
        case 95: return IBUS_KEY_bracketright;  /* VirtualKey::Oem6 (]}) */
        case 96: return IBUS_KEY_apostrophe;    /* VirtualKey::Oem7 ('") */
        
        /* Default: return 0 for unmapped keys */
        default:
            return 0;
    }
}