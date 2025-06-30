use std::collections::HashMap;

#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VirtualKey {
    // Special values
    Null = 1,              // pdNULL

    // Control keys
    Back = 2,              // pdVK_BACK (Backspace)
    Tab = 3,               // pdVK_TAB
    Return = 4,            // pdVK_RETURN (Enter)
    Shift = 5,             // pdVK_SHIFT
    Control = 6,           // pdVK_CONTROL
    Menu = 7,              // pdVK_MENU (Alt)
    Pause = 8,             // pdVK_PAUSE
    Capital = 9,           // pdVK_CAPITAL (Caps Lock)
    Kanji = 10,            // pdVK_KANJI
    Escape = 11,           // pdVK_ESCAPE
    Space = 12,            // pdVK_SPACE
    Prior = 13,            // pdVK_PRIOR (Page Up)
    Next = 14,             // pdVK_NEXT (Page Down)
    Delete = 15,           // pdVK_DELETE

    // Number keys (0-9)
    Key0 = 16,             // pdVK_KEY_0
    Key1 = 17,             // pdVK_KEY_1
    Key2 = 18,             // pdVK_KEY_2
    Key3 = 19,             // pdVK_KEY_3
    Key4 = 20,             // pdVK_KEY_4
    Key5 = 21,             // pdVK_KEY_5
    Key6 = 22,             // pdVK_KEY_6
    Key7 = 23,             // pdVK_KEY_7
    Key8 = 24,             // pdVK_KEY_8
    Key9 = 25,             // pdVK_KEY_9

    // Letter keys (A-Z)
    KeyA = 26,             // pdVK_KEY_A
    KeyB = 27,             // pdVK_KEY_B
    KeyC = 28,             // pdVK_KEY_C
    KeyD = 29,             // pdVK_KEY_D
    KeyE = 30,             // pdVK_KEY_E
    KeyF = 31,             // pdVK_KEY_F
    KeyG = 32,             // pdVK_KEY_G
    KeyH = 33,             // pdVK_KEY_H
    KeyI = 34,             // pdVK_KEY_I
    KeyJ = 35,             // pdVK_KEY_J
    KeyK = 36,             // pdVK_KEY_K
    KeyL = 37,             // pdVK_KEY_L
    KeyM = 38,             // pdVK_KEY_M
    KeyN = 39,             // pdVK_KEY_N
    KeyO = 40,             // pdVK_KEY_O
    KeyP = 41,             // pdVK_KEY_P
    KeyQ = 42,             // pdVK_KEY_Q
    KeyR = 43,             // pdVK_KEY_R
    KeyS = 44,             // pdVK_KEY_S
    KeyT = 45,             // pdVK_KEY_T
    KeyU = 46,             // pdVK_KEY_U
    KeyV = 47,             // pdVK_KEY_V
    KeyW = 48,             // pdVK_KEY_W
    KeyX = 49,             // pdVK_KEY_X
    KeyY = 50,             // pdVK_KEY_Y
    KeyZ = 51,             // pdVK_KEY_Z

    // Numpad keys
    Numpad0 = 52,          // pdVK_NUMPAD0
    Numpad1 = 53,          // pdVK_NUMPAD1
    Numpad2 = 54,          // pdVK_NUMPAD2
    Numpad3 = 55,          // pdVK_NUMPAD3
    Numpad4 = 56,          // pdVK_NUMPAD4
    Numpad5 = 57,          // pdVK_NUMPAD5
    Numpad6 = 58,          // pdVK_NUMPAD6
    Numpad7 = 59,          // pdVK_NUMPAD7
    Numpad8 = 60,          // pdVK_NUMPAD8
    Numpad9 = 61,          // pdVK_NUMPAD9

    // Numpad operators
    Multiply = 62,         // pdVK_MULTIPLY
    Add = 63,              // pdVK_ADD
    Separator = 64,        // pdVK_SEPARATOR
    Subtract = 65,         // pdVK_SUBTRACT
    Decimal = 66,          // pdVK_DECIMAL
    Divide = 67,           // pdVK_DIVIDE

    // Function keys
    F1 = 68,               // pdVK_F1
    F2 = 69,               // pdVK_F2
    F3 = 70,               // pdVK_F3
    F4 = 71,               // pdVK_F4
    F5 = 72,               // pdVK_F5
    F6 = 73,               // pdVK_F6
    F7 = 74,               // pdVK_F7
    F8 = 75,               // pdVK_F8
    F9 = 76,               // pdVK_F9
    F10 = 77,              // pdVK_F10
    F11 = 78,              // pdVK_F11
    F12 = 79,              // pdVK_F12

    // Modifier keys (left/right variants)
    LShift = 80,           // pdVK_LSHIFT
    RShift = 81,           // pdVK_RSHIFT
    LControl = 82,         // pdVK_LCONTROL
    RControl = 83,         // pdVK_RCONTROL
    LMenu = 84,            // pdVK_LMENU (Left Alt)
    RMenu = 85,            // pdVK_RMENU (Right Alt/AltGr)

    // OEM keys (keyboard-specific)
    Oem1 = 86,             // pdVK_OEM_1 (;: for US)
    OemPlus = 87,          // pdVK_OEM_PLUS (+ key)
    OemComma = 88,         // pdVK_OEM_COMMA
    OemMinus = 89,         // pdVK_OEM_MINUS
    OemPeriod = 90,        // pdVK_OEM_PERIOD
    Oem2 = 91,             // pdVK_OEM_2 (/? for US)
    Oem3 = 92,             // pdVK_OEM_3 (`~ for US)
    Oem4 = 93,             // pdVK_OEM_4 ([{ for US)
    Oem5 = 94,             // pdVK_OEM_5 (\| for US)
    Oem6 = 95,             // pdVK_OEM_6 (]} for US)
    Oem7 = 96,             // pdVK_OEM_7 ('" for US)
    Oem8 = 97,             // pdVK_OEM_8
    OemAx = 98,            // pdVK_OEM_AX
    Oem102 = 99,           // pdVK_OEM_102 (<> or \| on 102-key keyboard)
    IcoHelp = 100,         // pdVK_ICO_HELP
    Ico00 = 101,           // pdVK_ICO_00
}

pub fn create_vk_map() -> HashMap<&'static str, VirtualKey> {
    let mut map = HashMap::new();

    // Special keys
    map.insert("NULL", VirtualKey::Null);
    map.insert("null", VirtualKey::Null);

    // Control keys
    map.insert("VK_BACK", VirtualKey::Back);
    map.insert("VK_TAB", VirtualKey::Tab);
    map.insert("VK_RETURN", VirtualKey::Return);
    map.insert("VK_ENTER", VirtualKey::Return);
    map.insert("VK_SHIFT", VirtualKey::Shift);
    map.insert("VK_CONTROL", VirtualKey::Control);
    map.insert("VK_CTRL", VirtualKey::Control);
    map.insert("VK_MENU", VirtualKey::Menu);
    map.insert("VK_ALT", VirtualKey::Menu);
    map.insert("VK_PAUSE", VirtualKey::Pause);
    map.insert("VK_CAPITAL", VirtualKey::Capital);
    map.insert("VK_CAPSLOCK", VirtualKey::Capital);
    map.insert("VK_KANJI", VirtualKey::Kanji);
    map.insert("VK_ESCAPE", VirtualKey::Escape);
    map.insert("VK_ESC", VirtualKey::Escape);
    map.insert("VK_SPACE", VirtualKey::Space);
    map.insert("VK_PRIOR", VirtualKey::Prior);
    map.insert("VK_NEXT", VirtualKey::Next);
    map.insert("VK_DELETE", VirtualKey::Delete);

    // Number keys
    map.insert("VK_KEY_0", VirtualKey::Key0);
    map.insert("VK_KEY_1", VirtualKey::Key1);
    map.insert("VK_KEY_2", VirtualKey::Key2);
    map.insert("VK_KEY_3", VirtualKey::Key3);
    map.insert("VK_KEY_4", VirtualKey::Key4);
    map.insert("VK_KEY_5", VirtualKey::Key5);
    map.insert("VK_KEY_6", VirtualKey::Key6);
    map.insert("VK_KEY_7", VirtualKey::Key7);
    map.insert("VK_KEY_8", VirtualKey::Key8);
    map.insert("VK_KEY_9", VirtualKey::Key9);

    // Letter keys
    map.insert("VK_KEY_A", VirtualKey::KeyA);
    map.insert("VK_KEY_B", VirtualKey::KeyB);
    map.insert("VK_KEY_C", VirtualKey::KeyC);
    map.insert("VK_KEY_D", VirtualKey::KeyD);
    map.insert("VK_KEY_E", VirtualKey::KeyE);
    map.insert("VK_KEY_F", VirtualKey::KeyF);
    map.insert("VK_KEY_G", VirtualKey::KeyG);
    map.insert("VK_KEY_H", VirtualKey::KeyH);
    map.insert("VK_KEY_I", VirtualKey::KeyI);
    map.insert("VK_KEY_J", VirtualKey::KeyJ);
    map.insert("VK_KEY_K", VirtualKey::KeyK);
    map.insert("VK_KEY_L", VirtualKey::KeyL);
    map.insert("VK_KEY_M", VirtualKey::KeyM);
    map.insert("VK_KEY_N", VirtualKey::KeyN);
    map.insert("VK_KEY_O", VirtualKey::KeyO);
    map.insert("VK_KEY_P", VirtualKey::KeyP);
    map.insert("VK_KEY_Q", VirtualKey::KeyQ);
    map.insert("VK_KEY_R", VirtualKey::KeyR);
    map.insert("VK_KEY_S", VirtualKey::KeyS);
    map.insert("VK_KEY_T", VirtualKey::KeyT);
    map.insert("VK_KEY_U", VirtualKey::KeyU);
    map.insert("VK_KEY_V", VirtualKey::KeyV);
    map.insert("VK_KEY_W", VirtualKey::KeyW);
    map.insert("VK_KEY_X", VirtualKey::KeyX);
    map.insert("VK_KEY_Y", VirtualKey::KeyY);
    map.insert("VK_KEY_Z", VirtualKey::KeyZ);

    // Numpad keys
    map.insert("VK_NUMPAD0", VirtualKey::Numpad0);
    map.insert("VK_NUMPAD1", VirtualKey::Numpad1);
    map.insert("VK_NUMPAD2", VirtualKey::Numpad2);
    map.insert("VK_NUMPAD3", VirtualKey::Numpad3);
    map.insert("VK_NUMPAD4", VirtualKey::Numpad4);
    map.insert("VK_NUMPAD5", VirtualKey::Numpad5);
    map.insert("VK_NUMPAD6", VirtualKey::Numpad6);
    map.insert("VK_NUMPAD7", VirtualKey::Numpad7);
    map.insert("VK_NUMPAD8", VirtualKey::Numpad8);
    map.insert("VK_NUMPAD9", VirtualKey::Numpad9);

    // Numpad operators
    map.insert("VK_MULTIPLY", VirtualKey::Multiply);
    map.insert("VK_ADD", VirtualKey::Add);
    map.insert("VK_SEPARATOR", VirtualKey::Separator);
    map.insert("VK_SUBTRACT", VirtualKey::Subtract);
    map.insert("VK_DECIMAL", VirtualKey::Decimal);
    map.insert("VK_DIVIDE", VirtualKey::Divide);

    // Function keys
    map.insert("VK_F1", VirtualKey::F1);
    map.insert("VK_F2", VirtualKey::F2);
    map.insert("VK_F3", VirtualKey::F3);
    map.insert("VK_F4", VirtualKey::F4);
    map.insert("VK_F5", VirtualKey::F5);
    map.insert("VK_F6", VirtualKey::F6);
    map.insert("VK_F7", VirtualKey::F7);
    map.insert("VK_F8", VirtualKey::F8);
    map.insert("VK_F9", VirtualKey::F9);
    map.insert("VK_F10", VirtualKey::F10);
    map.insert("VK_F11", VirtualKey::F11);
    map.insert("VK_F12", VirtualKey::F12);

    // Modifier variants
    map.insert("VK_LSHIFT", VirtualKey::LShift);
    map.insert("VK_RSHIFT", VirtualKey::RShift);
    map.insert("VK_LCONTROL", VirtualKey::LControl);
    map.insert("VK_LCTRL", VirtualKey::LControl);
    map.insert("VK_RCONTROL", VirtualKey::RControl);
    map.insert("VK_RCTRL", VirtualKey::RControl);
    map.insert("VK_LMENU", VirtualKey::LMenu);
    map.insert("VK_LALT", VirtualKey::LMenu);
    map.insert("VK_RMENU", VirtualKey::RMenu);
    map.insert("VK_RALT", VirtualKey::RMenu);
    map.insert("VK_ALT_GR", VirtualKey::RMenu);

    // OEM keys with aliases
    map.insert("VK_OEM_1", VirtualKey::Oem1);
    map.insert("VK_COLON", VirtualKey::Oem1);
    map.insert("VK_OEM_PLUS", VirtualKey::OemPlus);
    map.insert("VK_OEM_COMMA", VirtualKey::OemComma);
    map.insert("VK_OEM_MINUS", VirtualKey::OemMinus);
    map.insert("VK_OEM_PERIOD", VirtualKey::OemPeriod);
    map.insert("VK_OEM_2", VirtualKey::Oem2);
    map.insert("VK_QUESTION", VirtualKey::Oem2);
    map.insert("VK_OEM_3", VirtualKey::Oem3);
    map.insert("VK_CFLEX", VirtualKey::Oem3);
    map.insert("VK_OEM_4", VirtualKey::Oem4);
    map.insert("VK_LBRACKET", VirtualKey::Oem4);
    map.insert("VK_OEM_5", VirtualKey::Oem5);
    map.insert("VK_BACKSLASH", VirtualKey::Oem5);
    map.insert("VK_OEM_6", VirtualKey::Oem6);
    map.insert("VK_RBRACKET", VirtualKey::Oem6);
    map.insert("VK_OEM_7", VirtualKey::Oem7);
    map.insert("VK_QUOTE", VirtualKey::Oem7);
    map.insert("VK_OEM_8", VirtualKey::Oem8);
    map.insert("VK_EXCM", VirtualKey::Oem8);
    map.insert("VK_OEM_AX", VirtualKey::OemAx);
    map.insert("VK_OEM_102", VirtualKey::Oem102);
    map.insert("VK_LESSTHEN", VirtualKey::Oem102);
    map.insert("VK_ICO_HELP", VirtualKey::IcoHelp);
    map.insert("VK_ICO_00", VirtualKey::Ico00);

    map
}