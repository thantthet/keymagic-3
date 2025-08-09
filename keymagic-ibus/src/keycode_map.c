/**
 * IBus to KeyMagic keycode mapping
 * 
 * This module provides mapping between IBus key values and KeyMagic VirtualKey codes.
 * IBus uses X11 keysyms (keyvals) which need to be mapped to KeyMagic's internal 
 * VirtualKey representation.
 */

#include <ibus.h>
#include "keycode_map.h"

/**
 * Map IBus keyval to KeyMagic VirtualKey code
 * 
 * @param keyval IBus key value (X11 keysym)
 * @return KeyMagic VirtualKey enum value, or 0 if no mapping exists
 */
KeyMagicVirtualKey
keymagic_map_ibus_keyval(guint keyval)
{
    switch (keyval) {
        /* Control keys */
        case IBUS_KEY_BackSpace:    return KeyMagic_VK_Back;
        case IBUS_KEY_Tab:          return KeyMagic_VK_Tab;
        case IBUS_KEY_Return:       return KeyMagic_VK_Return;
        case IBUS_KEY_KP_Enter:     return KeyMagic_VK_Return;
        case IBUS_KEY_Shift_L:      return KeyMagic_VK_LShift;
        case IBUS_KEY_Shift_R:      return KeyMagic_VK_RShift;
        case IBUS_KEY_Control_L:    return KeyMagic_VK_LControl;
        case IBUS_KEY_Control_R:    return KeyMagic_VK_RControl;
        case IBUS_KEY_Alt_L:        return KeyMagic_VK_LAlt;
        case IBUS_KEY_Alt_R:        return KeyMagic_VK_RAlt;
        case IBUS_KEY_Pause:        return KeyMagic_VK_Pause;
        case IBUS_KEY_Caps_Lock:    return KeyMagic_VK_Capital;
        case IBUS_KEY_Escape:       return KeyMagic_VK_Escape;
        case IBUS_KEY_space:        return KeyMagic_VK_Space;
        case IBUS_KEY_Page_Up:      return KeyMagic_VK_Prior;
        case IBUS_KEY_Page_Down:    return KeyMagic_VK_Next;
        case IBUS_KEY_Delete:       return KeyMagic_VK_Delete;
        
        /* Number keys (0-9) */
        case IBUS_KEY_0:            return KeyMagic_VK_Key0;
        case IBUS_KEY_1:            return KeyMagic_VK_Key1;
        case IBUS_KEY_2:            return KeyMagic_VK_Key2;
        case IBUS_KEY_3:            return KeyMagic_VK_Key3;
        case IBUS_KEY_4:            return KeyMagic_VK_Key4;
        case IBUS_KEY_5:            return KeyMagic_VK_Key5;
        case IBUS_KEY_6:            return KeyMagic_VK_Key6;
        case IBUS_KEY_7:            return KeyMagic_VK_Key7;
        case IBUS_KEY_8:            return KeyMagic_VK_Key8;
        case IBUS_KEY_9:            return KeyMagic_VK_Key9;
        
        /* Letter keys (A-Z) - handle both lowercase and uppercase */
        case IBUS_KEY_a:
        case IBUS_KEY_A:            return KeyMagic_VK_KeyA;
        case IBUS_KEY_b:
        case IBUS_KEY_B:            return KeyMagic_VK_KeyB;
        case IBUS_KEY_c:
        case IBUS_KEY_C:            return KeyMagic_VK_KeyC;
        case IBUS_KEY_d:
        case IBUS_KEY_D:            return KeyMagic_VK_KeyD;
        case IBUS_KEY_e:
        case IBUS_KEY_E:            return KeyMagic_VK_KeyE;
        case IBUS_KEY_f:
        case IBUS_KEY_F:            return KeyMagic_VK_KeyF;
        case IBUS_KEY_g:
        case IBUS_KEY_G:            return KeyMagic_VK_KeyG;
        case IBUS_KEY_h:
        case IBUS_KEY_H:            return KeyMagic_VK_KeyH;
        case IBUS_KEY_i:
        case IBUS_KEY_I:            return KeyMagic_VK_KeyI;
        case IBUS_KEY_j:
        case IBUS_KEY_J:            return KeyMagic_VK_KeyJ;
        case IBUS_KEY_k:
        case IBUS_KEY_K:            return KeyMagic_VK_KeyK;
        case IBUS_KEY_l:
        case IBUS_KEY_L:            return KeyMagic_VK_KeyL;
        case IBUS_KEY_m:
        case IBUS_KEY_M:            return KeyMagic_VK_KeyM;
        case IBUS_KEY_n:
        case IBUS_KEY_N:            return KeyMagic_VK_KeyN;
        case IBUS_KEY_o:
        case IBUS_KEY_O:            return KeyMagic_VK_KeyO;
        case IBUS_KEY_p:
        case IBUS_KEY_P:            return KeyMagic_VK_KeyP;
        case IBUS_KEY_q:
        case IBUS_KEY_Q:            return KeyMagic_VK_KeyQ;
        case IBUS_KEY_r:
        case IBUS_KEY_R:            return KeyMagic_VK_KeyR;
        case IBUS_KEY_s:
        case IBUS_KEY_S:            return KeyMagic_VK_KeyS;
        case IBUS_KEY_t:
        case IBUS_KEY_T:            return KeyMagic_VK_KeyT;
        case IBUS_KEY_u:
        case IBUS_KEY_U:            return KeyMagic_VK_KeyU;
        case IBUS_KEY_v:
        case IBUS_KEY_V:            return KeyMagic_VK_KeyV;
        case IBUS_KEY_w:
        case IBUS_KEY_W:            return KeyMagic_VK_KeyW;
        case IBUS_KEY_x:
        case IBUS_KEY_X:            return KeyMagic_VK_KeyX;
        case IBUS_KEY_y:
        case IBUS_KEY_Y:            return KeyMagic_VK_KeyY;
        case IBUS_KEY_z:
        case IBUS_KEY_Z:            return KeyMagic_VK_KeyZ;
        
        /* Numpad keys */
        case IBUS_KEY_KP_0:         return KeyMagic_VK_Numpad0;
        case IBUS_KEY_KP_1:         return KeyMagic_VK_Numpad1;
        case IBUS_KEY_KP_2:         return KeyMagic_VK_Numpad2;
        case IBUS_KEY_KP_3:         return KeyMagic_VK_Numpad3;
        case IBUS_KEY_KP_4:         return KeyMagic_VK_Numpad4;
        case IBUS_KEY_KP_5:         return KeyMagic_VK_Numpad5;
        case IBUS_KEY_KP_6:         return KeyMagic_VK_Numpad6;
        case IBUS_KEY_KP_7:         return KeyMagic_VK_Numpad7;
        case IBUS_KEY_KP_8:         return KeyMagic_VK_Numpad8;
        case IBUS_KEY_KP_9:         return KeyMagic_VK_Numpad9;
        
        /* Numpad operators */
        case IBUS_KEY_KP_Multiply:  return KeyMagic_VK_Multiply;
        case IBUS_KEY_KP_Add:       return KeyMagic_VK_Add;
        case IBUS_KEY_KP_Separator: return KeyMagic_VK_Separator;
        case IBUS_KEY_KP_Subtract:  return KeyMagic_VK_Subtract;
        case IBUS_KEY_KP_Decimal:   return KeyMagic_VK_Decimal;
        case IBUS_KEY_KP_Divide:    return KeyMagic_VK_Divide;
        
        /* Function keys */
        case IBUS_KEY_F1:           return KeyMagic_VK_F1;
        case IBUS_KEY_F2:           return KeyMagic_VK_F2;
        case IBUS_KEY_F3:           return KeyMagic_VK_F3;
        case IBUS_KEY_F4:           return KeyMagic_VK_F4;
        case IBUS_KEY_F5:           return KeyMagic_VK_F5;
        case IBUS_KEY_F6:           return KeyMagic_VK_F6;
        case IBUS_KEY_F7:           return KeyMagic_VK_F7;
        case IBUS_KEY_F8:           return KeyMagic_VK_F8;
        case IBUS_KEY_F9:           return KeyMagic_VK_F9;
        case IBUS_KEY_F10:          return KeyMagic_VK_F10;
        case IBUS_KEY_F11:          return KeyMagic_VK_F11;
        case IBUS_KEY_F12:          return KeyMagic_VK_F12;
        
        /* Shifted number keys (map to base number keys) */
        case IBUS_KEY_exclam:       return KeyMagic_VK_Key1;
        case IBUS_KEY_at:           return KeyMagic_VK_Key2;
        case IBUS_KEY_numbersign:   return KeyMagic_VK_Key3;
        case IBUS_KEY_dollar:       return KeyMagic_VK_Key4;
        case IBUS_KEY_percent:      return KeyMagic_VK_Key5;
        case IBUS_KEY_asciicircum:  return KeyMagic_VK_Key6;
        case IBUS_KEY_ampersand:    return KeyMagic_VK_Key7;
        case IBUS_KEY_asterisk:     return KeyMagic_VK_Key8;  /* VirtualKey::Key8 (*) */
        case IBUS_KEY_parenleft:    return KeyMagic_VK_Key9;
        case IBUS_KEY_parenright:   return KeyMagic_VK_Key0;
        
        /* OEM keys */
        case IBUS_KEY_semicolon:
        case IBUS_KEY_colon:        return KeyMagic_VK_Oem1;
        case IBUS_KEY_equal:
        case IBUS_KEY_plus:         return KeyMagic_VK_OemPlus;
        case IBUS_KEY_comma:
        case IBUS_KEY_less:         return KeyMagic_VK_OemComma;
        case IBUS_KEY_minus:
        case IBUS_KEY_underscore:   return KeyMagic_VK_OemMinus;
        case IBUS_KEY_period:
        case IBUS_KEY_greater:      return KeyMagic_VK_OemPeriod;
        case IBUS_KEY_slash:
        case IBUS_KEY_question:     return KeyMagic_VK_Oem2;
        case IBUS_KEY_grave:
        case IBUS_KEY_asciitilde:   return KeyMagic_VK_Oem3;
        case IBUS_KEY_bracketleft:
        case IBUS_KEY_braceleft:    return KeyMagic_VK_Oem4;
        case IBUS_KEY_backslash:
        case IBUS_KEY_bar:          return KeyMagic_VK_Oem5;
        case IBUS_KEY_bracketright:
        case IBUS_KEY_braceright:   return KeyMagic_VK_Oem6;
        case IBUS_KEY_apostrophe:
        case IBUS_KEY_quotedbl:     return KeyMagic_VK_Oem7;
        
        /* Default: return 0 for unmapped keys */
        default:
            return 0;
    }
}

/**
 * Map KeyMagic VirtualKey code to IBus keyval
 * 
 * @param vk_code KeyMagic VirtualKey enum value
 * @return IBus key value (X11 keysym), or 0 if no mapping exists
 */
guint
keymagic_map_virtual_key_to_ibus(KeyMagicVirtualKey vk_code)
{
    switch (vk_code) {
        /* Control keys */
        case KeyMagic_VK_Back:  return IBUS_KEY_BackSpace;
        case KeyMagic_VK_Tab:   return IBUS_KEY_Tab;
        case KeyMagic_VK_Return:  return IBUS_KEY_Return;        /* VirtualKey::Return */
        case KeyMagic_VK_Shift:  return IBUS_KEY_Shift_L;       /* VirtualKey::Shift */
        case KeyMagic_VK_Control:  return IBUS_KEY_Control_L;     /* VirtualKey::Control */
        case KeyMagic_VK_Menu:  return IBUS_KEY_Alt_L;         /* VirtualKey::Menu */
        case KeyMagic_VK_Pause:  return IBUS_KEY_Pause;         /* VirtualKey::Pause */
        case KeyMagic_VK_Capital:  return IBUS_KEY_Caps_Lock;     /* VirtualKey::Capital */
        case KeyMagic_VK_Escape: return IBUS_KEY_Escape;        /* VirtualKey::Escape */
        case KeyMagic_VK_Space: return IBUS_KEY_space;         /* VirtualKey::Space */
        case KeyMagic_VK_Prior: return IBUS_KEY_Prior;         /* VirtualKey::Prior (Page Up) */
        case KeyMagic_VK_Next: return IBUS_KEY_Next;          /* VirtualKey::Next (Page Down) */
        case KeyMagic_VK_Delete: return IBUS_KEY_Delete;        /* VirtualKey::Delete */
        
        /* Number keys */
        case KeyMagic_VK_Key0: return IBUS_KEY_0;             /* VirtualKey::Key0 */
        case KeyMagic_VK_Key1: return IBUS_KEY_1;             /* VirtualKey::Key1 */
        case KeyMagic_VK_Key2: return IBUS_KEY_2;             /* VirtualKey::Key2 */
        case KeyMagic_VK_Key3: return IBUS_KEY_3;             /* VirtualKey::Key3 */
        case KeyMagic_VK_Key4: return IBUS_KEY_4;             /* VirtualKey::Key4 */
        case KeyMagic_VK_Key5: return IBUS_KEY_5;             /* VirtualKey::Key5 */
        case KeyMagic_VK_Key6: return IBUS_KEY_6;             /* VirtualKey::Key6 */
        case KeyMagic_VK_Key7: return IBUS_KEY_7;             /* VirtualKey::Key7 */
        case KeyMagic_VK_Key8: return IBUS_KEY_8;             /* VirtualKey::Key8 */
        case KeyMagic_VK_Key9: return IBUS_KEY_9;             /* VirtualKey::Key9 */
        
        /* Letter keys */
        case KeyMagic_VK_KeyA: return IBUS_KEY_a;             /* VirtualKey::KeyA */
        case KeyMagic_VK_KeyB: return IBUS_KEY_b;             /* VirtualKey::KeyB */
        case KeyMagic_VK_KeyC: return IBUS_KEY_c;             /* VirtualKey::KeyC */
        case KeyMagic_VK_KeyD: return IBUS_KEY_d;             /* VirtualKey::KeyD */
        case KeyMagic_VK_KeyE: return IBUS_KEY_e;             /* VirtualKey::KeyE */
        case KeyMagic_VK_KeyF: return IBUS_KEY_f;             /* VirtualKey::KeyF */
        case KeyMagic_VK_KeyG: return IBUS_KEY_g;             /* VirtualKey::KeyG */
        case KeyMagic_VK_KeyH: return IBUS_KEY_h;             /* VirtualKey::KeyH */
        case KeyMagic_VK_KeyI: return IBUS_KEY_i;             /* VirtualKey::KeyI */
        case KeyMagic_VK_KeyJ: return IBUS_KEY_j;             /* VirtualKey::KeyJ */
        case KeyMagic_VK_KeyK: return IBUS_KEY_k;             /* VirtualKey::KeyK */
        case KeyMagic_VK_KeyL: return IBUS_KEY_l;             /* VirtualKey::KeyL */
        case KeyMagic_VK_KeyM: return IBUS_KEY_m;             /* VirtualKey::KeyM */
        case KeyMagic_VK_KeyN: return IBUS_KEY_n;             /* VirtualKey::KeyN */
        case KeyMagic_VK_KeyO: return IBUS_KEY_o;             /* VirtualKey::KeyO */
        case KeyMagic_VK_KeyP: return IBUS_KEY_p;             /* VirtualKey::KeyP */
        case KeyMagic_VK_KeyQ: return IBUS_KEY_q;             /* VirtualKey::KeyQ */
        case KeyMagic_VK_KeyR: return IBUS_KEY_r;             /* VirtualKey::KeyR */
        case KeyMagic_VK_KeyS: return IBUS_KEY_s;             /* VirtualKey::KeyS */
        case KeyMagic_VK_KeyT: return IBUS_KEY_t;             /* VirtualKey::KeyT */
        case KeyMagic_VK_KeyU: return IBUS_KEY_u;             /* VirtualKey::KeyU */
        case KeyMagic_VK_KeyV: return IBUS_KEY_v;             /* VirtualKey::KeyV */
        case KeyMagic_VK_KeyW: return IBUS_KEY_w;             /* VirtualKey::KeyW */
        case KeyMagic_VK_KeyX: return IBUS_KEY_x;             /* VirtualKey::KeyX */
        case KeyMagic_VK_KeyY: return IBUS_KEY_y;             /* VirtualKey::KeyY */
        case KeyMagic_VK_KeyZ: return IBUS_KEY_z;             /* VirtualKey::KeyZ */
        
        /* Numpad keys */
        case KeyMagic_VK_Numpad0: return IBUS_KEY_KP_0;          /* VirtualKey::Numpad0 */
        case KeyMagic_VK_Numpad1: return IBUS_KEY_KP_1;          /* VirtualKey::Numpad1 */
        case KeyMagic_VK_Numpad2: return IBUS_KEY_KP_2;          /* VirtualKey::Numpad2 */
        case KeyMagic_VK_Numpad3: return IBUS_KEY_KP_3;          /* VirtualKey::Numpad3 */
        case KeyMagic_VK_Numpad4: return IBUS_KEY_KP_4;          /* VirtualKey::Numpad4 */
        case KeyMagic_VK_Numpad5: return IBUS_KEY_KP_5;          /* VirtualKey::Numpad5 */
        case KeyMagic_VK_Numpad6: return IBUS_KEY_KP_6;          /* VirtualKey::Numpad6 */
        case KeyMagic_VK_Numpad7: return IBUS_KEY_KP_7;          /* VirtualKey::Numpad7 */
        case KeyMagic_VK_Numpad8: return IBUS_KEY_KP_8;          /* VirtualKey::Numpad8 */
        case KeyMagic_VK_Numpad9: return IBUS_KEY_KP_9;          /* VirtualKey::Numpad9 */
        
        /* Numpad operators */
        case KeyMagic_VK_Multiply: return IBUS_KEY_KP_Multiply;   /* VirtualKey::Multiply */
        case KeyMagic_VK_Add: return IBUS_KEY_KP_Add;        /* VirtualKey::Add */
        case KeyMagic_VK_Separator: return IBUS_KEY_KP_Separator;  /* VirtualKey::Separator */
        case KeyMagic_VK_Subtract: return IBUS_KEY_KP_Subtract;   /* VirtualKey::Subtract */
        case KeyMagic_VK_Decimal: return IBUS_KEY_KP_Decimal;    /* VirtualKey::Decimal */
        case KeyMagic_VK_Divide: return IBUS_KEY_KP_Divide;     /* VirtualKey::Divide */
        
        /* Function keys */
        case KeyMagic_VK_F1: return IBUS_KEY_F1;            /* VirtualKey::F1 */
        case KeyMagic_VK_F2: return IBUS_KEY_F2;            /* VirtualKey::F2 */
        case KeyMagic_VK_F3: return IBUS_KEY_F3;            /* VirtualKey::F3 */
        case KeyMagic_VK_F4: return IBUS_KEY_F4;            /* VirtualKey::F4 */
        case KeyMagic_VK_F5: return IBUS_KEY_F5;            /* VirtualKey::F5 */
        case KeyMagic_VK_F6: return IBUS_KEY_F6;            /* VirtualKey::F6 */
        case KeyMagic_VK_F7: return IBUS_KEY_F7;            /* VirtualKey::F7 */
        case KeyMagic_VK_F8: return IBUS_KEY_F8;            /* VirtualKey::F8 */
        case KeyMagic_VK_F9: return IBUS_KEY_F9;            /* VirtualKey::F9 */
        case KeyMagic_VK_F10: return IBUS_KEY_F10;           /* VirtualKey::F10 */
        case KeyMagic_VK_F11: return IBUS_KEY_F11;           /* VirtualKey::F11 */
        case KeyMagic_VK_F12: return IBUS_KEY_F12;           /* VirtualKey::F12 */
        
        /* Modifier keys (left/right variants) */
        case KeyMagic_VK_LShift: return IBUS_KEY_Shift_L;       /* VirtualKey::LShift */
        case KeyMagic_VK_RShift: return IBUS_KEY_Shift_R;       /* VirtualKey::RShift */
        case KeyMagic_VK_LControl: return IBUS_KEY_Control_L;     /* VirtualKey::LControl */
        case KeyMagic_VK_RControl: return IBUS_KEY_Control_R;     /* VirtualKey::RControl */
        case KeyMagic_VK_LAlt: return IBUS_KEY_Alt_L;         /* VirtualKey::LMenu */
        case KeyMagic_VK_RAlt: return IBUS_KEY_Alt_R;         /* VirtualKey::RMenu */
        
        /* OEM keys */
        case KeyMagic_VK_Oem1: return IBUS_KEY_semicolon;     /* VirtualKey::Oem1 (;:) */
        case KeyMagic_VK_OemPlus: return IBUS_KEY_plus;          /* VirtualKey::OemPlus */
        case KeyMagic_VK_OemComma: return IBUS_KEY_comma;         /* VirtualKey::OemComma */
        case KeyMagic_VK_OemMinus: return IBUS_KEY_minus;         /* VirtualKey::OemMinus */
        case KeyMagic_VK_OemPeriod: return IBUS_KEY_period;        /* VirtualKey::OemPeriod */
        case KeyMagic_VK_Oem2: return IBUS_KEY_slash;         /* VirtualKey::Oem2 (/?) */
        case KeyMagic_VK_Oem3: return IBUS_KEY_grave;         /* VirtualKey::Oem3 (`~) */
        case KeyMagic_VK_Oem4: return IBUS_KEY_bracketleft;   /* VirtualKey::Oem4 ([{) */
        case KeyMagic_VK_Oem5: return IBUS_KEY_backslash;     /* VirtualKey::Oem5 (\|) */
        case KeyMagic_VK_Oem6: return IBUS_KEY_bracketright;  /* VirtualKey::Oem6 (]}) */
        case KeyMagic_VK_Oem7: return IBUS_KEY_apostrophe;    /* VirtualKey::Oem7 ('") */
        
        /* Default: return 0 for unmapped keys */
        default:
            return 0;
    }
}