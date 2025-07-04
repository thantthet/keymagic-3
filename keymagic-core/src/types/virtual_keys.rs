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

impl VirtualKey {
    /// Convert this VirtualKey to Windows Virtual Key code
    pub fn to_win_vk(&self) -> u16 {
        match self {
            // Special values
            VirtualKey::Null => 0x00,
            
            // Control keys
            VirtualKey::Back => 0x08,
            VirtualKey::Tab => 0x09,
            VirtualKey::Return => 0x0D,
            VirtualKey::Shift => 0x10,
            VirtualKey::Control => 0x11,
            VirtualKey::Menu => 0x12,
            VirtualKey::Pause => 0x13,
            VirtualKey::Capital => 0x14,
            VirtualKey::Kanji => 0x19,
            VirtualKey::Escape => 0x1B,
            VirtualKey::Space => 0x20,
            VirtualKey::Prior => 0x21,
            VirtualKey::Next => 0x22,
            VirtualKey::Delete => 0x2E,
            
            // Number keys
            VirtualKey::Key0 => 0x30,
            VirtualKey::Key1 => 0x31,
            VirtualKey::Key2 => 0x32,
            VirtualKey::Key3 => 0x33,
            VirtualKey::Key4 => 0x34,
            VirtualKey::Key5 => 0x35,
            VirtualKey::Key6 => 0x36,
            VirtualKey::Key7 => 0x37,
            VirtualKey::Key8 => 0x38,
            VirtualKey::Key9 => 0x39,
            
            // Letter keys
            VirtualKey::KeyA => 0x41,
            VirtualKey::KeyB => 0x42,
            VirtualKey::KeyC => 0x43,
            VirtualKey::KeyD => 0x44,
            VirtualKey::KeyE => 0x45,
            VirtualKey::KeyF => 0x46,
            VirtualKey::KeyG => 0x47,
            VirtualKey::KeyH => 0x48,
            VirtualKey::KeyI => 0x49,
            VirtualKey::KeyJ => 0x4A,
            VirtualKey::KeyK => 0x4B,
            VirtualKey::KeyL => 0x4C,
            VirtualKey::KeyM => 0x4D,
            VirtualKey::KeyN => 0x4E,
            VirtualKey::KeyO => 0x4F,
            VirtualKey::KeyP => 0x50,
            VirtualKey::KeyQ => 0x51,
            VirtualKey::KeyR => 0x52,
            VirtualKey::KeyS => 0x53,
            VirtualKey::KeyT => 0x54,
            VirtualKey::KeyU => 0x55,
            VirtualKey::KeyV => 0x56,
            VirtualKey::KeyW => 0x57,
            VirtualKey::KeyX => 0x58,
            VirtualKey::KeyY => 0x59,
            VirtualKey::KeyZ => 0x5A,
            
            // Numpad keys
            VirtualKey::Numpad0 => 0x60,
            VirtualKey::Numpad1 => 0x61,
            VirtualKey::Numpad2 => 0x62,
            VirtualKey::Numpad3 => 0x63,
            VirtualKey::Numpad4 => 0x64,
            VirtualKey::Numpad5 => 0x65,
            VirtualKey::Numpad6 => 0x66,
            VirtualKey::Numpad7 => 0x67,
            VirtualKey::Numpad8 => 0x68,
            VirtualKey::Numpad9 => 0x69,
            
            // Numpad operators
            VirtualKey::Multiply => 0x6A,
            VirtualKey::Add => 0x6B,
            VirtualKey::Separator => 0x6C,
            VirtualKey::Subtract => 0x6D,
            VirtualKey::Decimal => 0x6E,
            VirtualKey::Divide => 0x6F,
            
            // Function keys
            VirtualKey::F1 => 0x70,
            VirtualKey::F2 => 0x71,
            VirtualKey::F3 => 0x72,
            VirtualKey::F4 => 0x73,
            VirtualKey::F5 => 0x74,
            VirtualKey::F6 => 0x75,
            VirtualKey::F7 => 0x76,
            VirtualKey::F8 => 0x77,
            VirtualKey::F9 => 0x78,
            VirtualKey::F10 => 0x79,
            VirtualKey::F11 => 0x7A,
            VirtualKey::F12 => 0x7B,
            
            // Modifier keys (left/right variants)
            VirtualKey::LShift => 0xA0,
            VirtualKey::RShift => 0xA1,
            VirtualKey::LControl => 0xA2,
            VirtualKey::RControl => 0xA3,
            VirtualKey::LMenu => 0xA4,
            VirtualKey::RMenu => 0xA5,
            
            // OEM keys
            VirtualKey::Oem1 => 0xBA,
            VirtualKey::OemPlus => 0xBB,
            VirtualKey::OemComma => 0xBC,
            VirtualKey::OemMinus => 0xBD,
            VirtualKey::OemPeriod => 0xBE,
            VirtualKey::Oem2 => 0xBF,
            VirtualKey::Oem3 => 0xC0,
            VirtualKey::Oem4 => 0xDB,
            VirtualKey::Oem5 => 0xDC,
            VirtualKey::Oem6 => 0xDD,
            VirtualKey::Oem7 => 0xDE,
            VirtualKey::Oem8 => 0xDF,
            VirtualKey::OemAx => 0xE1,
            VirtualKey::Oem102 => 0xE2,
            VirtualKey::IcoHelp => 0xE3,
            VirtualKey::Ico00 => 0xE4,
        }
    }
    
    /// Convert from raw enum value to VirtualKey
    pub fn from_raw(raw_value: u16) -> Option<Self> {
        match raw_value {
            1 => Some(VirtualKey::Null),
            2 => Some(VirtualKey::Back),
            3 => Some(VirtualKey::Tab),
            4 => Some(VirtualKey::Return),
            5 => Some(VirtualKey::Shift),
            6 => Some(VirtualKey::Control),
            7 => Some(VirtualKey::Menu),
            8 => Some(VirtualKey::Pause),
            9 => Some(VirtualKey::Capital),
            10 => Some(VirtualKey::Kanji),
            11 => Some(VirtualKey::Escape),
            12 => Some(VirtualKey::Space),
            13 => Some(VirtualKey::Prior),
            14 => Some(VirtualKey::Next),
            15 => Some(VirtualKey::Delete),
            16 => Some(VirtualKey::Key0),
            17 => Some(VirtualKey::Key1),
            18 => Some(VirtualKey::Key2),
            19 => Some(VirtualKey::Key3),
            20 => Some(VirtualKey::Key4),
            21 => Some(VirtualKey::Key5),
            22 => Some(VirtualKey::Key6),
            23 => Some(VirtualKey::Key7),
            24 => Some(VirtualKey::Key8),
            25 => Some(VirtualKey::Key9),
            26 => Some(VirtualKey::KeyA),
            27 => Some(VirtualKey::KeyB),
            28 => Some(VirtualKey::KeyC),
            29 => Some(VirtualKey::KeyD),
            30 => Some(VirtualKey::KeyE),
            31 => Some(VirtualKey::KeyF),
            32 => Some(VirtualKey::KeyG),
            33 => Some(VirtualKey::KeyH),
            34 => Some(VirtualKey::KeyI),
            35 => Some(VirtualKey::KeyJ),
            36 => Some(VirtualKey::KeyK),
            37 => Some(VirtualKey::KeyL),
            38 => Some(VirtualKey::KeyM),
            39 => Some(VirtualKey::KeyN),
            40 => Some(VirtualKey::KeyO),
            41 => Some(VirtualKey::KeyP),
            42 => Some(VirtualKey::KeyQ),
            43 => Some(VirtualKey::KeyR),
            44 => Some(VirtualKey::KeyS),
            45 => Some(VirtualKey::KeyT),
            46 => Some(VirtualKey::KeyU),
            47 => Some(VirtualKey::KeyV),
            48 => Some(VirtualKey::KeyW),
            49 => Some(VirtualKey::KeyX),
            50 => Some(VirtualKey::KeyY),
            51 => Some(VirtualKey::KeyZ),
            52 => Some(VirtualKey::Numpad0),
            53 => Some(VirtualKey::Numpad1),
            54 => Some(VirtualKey::Numpad2),
            55 => Some(VirtualKey::Numpad3),
            56 => Some(VirtualKey::Numpad4),
            57 => Some(VirtualKey::Numpad5),
            58 => Some(VirtualKey::Numpad6),
            59 => Some(VirtualKey::Numpad7),
            60 => Some(VirtualKey::Numpad8),
            61 => Some(VirtualKey::Numpad9),
            62 => Some(VirtualKey::Multiply),
            63 => Some(VirtualKey::Add),
            64 => Some(VirtualKey::Separator),
            65 => Some(VirtualKey::Subtract),
            66 => Some(VirtualKey::Decimal),
            67 => Some(VirtualKey::Divide),
            68 => Some(VirtualKey::F1),
            69 => Some(VirtualKey::F2),
            70 => Some(VirtualKey::F3),
            71 => Some(VirtualKey::F4),
            72 => Some(VirtualKey::F5),
            73 => Some(VirtualKey::F6),
            74 => Some(VirtualKey::F7),
            75 => Some(VirtualKey::F8),
            76 => Some(VirtualKey::F9),
            77 => Some(VirtualKey::F10),
            78 => Some(VirtualKey::F11),
            79 => Some(VirtualKey::F12),
            80 => Some(VirtualKey::LShift),
            81 => Some(VirtualKey::RShift),
            82 => Some(VirtualKey::LControl),
            83 => Some(VirtualKey::RControl),
            84 => Some(VirtualKey::LMenu),
            85 => Some(VirtualKey::RMenu),
            86 => Some(VirtualKey::Oem1),
            87 => Some(VirtualKey::OemPlus),
            88 => Some(VirtualKey::OemComma),
            89 => Some(VirtualKey::OemMinus),
            90 => Some(VirtualKey::OemPeriod),
            91 => Some(VirtualKey::Oem2),
            92 => Some(VirtualKey::Oem3),
            93 => Some(VirtualKey::Oem4),
            94 => Some(VirtualKey::Oem5),
            95 => Some(VirtualKey::Oem6),
            96 => Some(VirtualKey::Oem7),
            97 => Some(VirtualKey::Oem8),
            98 => Some(VirtualKey::OemAx),
            99 => Some(VirtualKey::Oem102),
            100 => Some(VirtualKey::IcoHelp),
            101 => Some(VirtualKey::Ico00),
            _ => None,
        }
    }
    
    /// Convert Windows Virtual Key code to VirtualKey
    pub fn from_win_vk(vk_code: u16) -> Option<Self> {
        match vk_code {
            // Special values
            0x00 => Some(VirtualKey::Null),
            
            // Control keys
            0x08 => Some(VirtualKey::Back),
            0x09 => Some(VirtualKey::Tab),
            0x0D => Some(VirtualKey::Return),
            0x10 => Some(VirtualKey::Shift),
            0x11 => Some(VirtualKey::Control),
            0x12 => Some(VirtualKey::Menu),
            0x13 => Some(VirtualKey::Pause),
            0x14 => Some(VirtualKey::Capital),
            0x19 => Some(VirtualKey::Kanji),
            0x1B => Some(VirtualKey::Escape),
            0x20 => Some(VirtualKey::Space),
            0x21 => Some(VirtualKey::Prior),
            0x22 => Some(VirtualKey::Next),
            0x2E => Some(VirtualKey::Delete),
            
            // Number keys
            0x30 => Some(VirtualKey::Key0),
            0x31 => Some(VirtualKey::Key1),
            0x32 => Some(VirtualKey::Key2),
            0x33 => Some(VirtualKey::Key3),
            0x34 => Some(VirtualKey::Key4),
            0x35 => Some(VirtualKey::Key5),
            0x36 => Some(VirtualKey::Key6),
            0x37 => Some(VirtualKey::Key7),
            0x38 => Some(VirtualKey::Key8),
            0x39 => Some(VirtualKey::Key9),
            
            // Letter keys
            0x41 => Some(VirtualKey::KeyA),
            0x42 => Some(VirtualKey::KeyB),
            0x43 => Some(VirtualKey::KeyC),
            0x44 => Some(VirtualKey::KeyD),
            0x45 => Some(VirtualKey::KeyE),
            0x46 => Some(VirtualKey::KeyF),
            0x47 => Some(VirtualKey::KeyG),
            0x48 => Some(VirtualKey::KeyH),
            0x49 => Some(VirtualKey::KeyI),
            0x4A => Some(VirtualKey::KeyJ),
            0x4B => Some(VirtualKey::KeyK),
            0x4C => Some(VirtualKey::KeyL),
            0x4D => Some(VirtualKey::KeyM),
            0x4E => Some(VirtualKey::KeyN),
            0x4F => Some(VirtualKey::KeyO),
            0x50 => Some(VirtualKey::KeyP),
            0x51 => Some(VirtualKey::KeyQ),
            0x52 => Some(VirtualKey::KeyR),
            0x53 => Some(VirtualKey::KeyS),
            0x54 => Some(VirtualKey::KeyT),
            0x55 => Some(VirtualKey::KeyU),
            0x56 => Some(VirtualKey::KeyV),
            0x57 => Some(VirtualKey::KeyW),
            0x58 => Some(VirtualKey::KeyX),
            0x59 => Some(VirtualKey::KeyY),
            0x5A => Some(VirtualKey::KeyZ),
            
            // Numpad keys
            0x60 => Some(VirtualKey::Numpad0),
            0x61 => Some(VirtualKey::Numpad1),
            0x62 => Some(VirtualKey::Numpad2),
            0x63 => Some(VirtualKey::Numpad3),
            0x64 => Some(VirtualKey::Numpad4),
            0x65 => Some(VirtualKey::Numpad5),
            0x66 => Some(VirtualKey::Numpad6),
            0x67 => Some(VirtualKey::Numpad7),
            0x68 => Some(VirtualKey::Numpad8),
            0x69 => Some(VirtualKey::Numpad9),
            
            // Numpad operators
            0x6A => Some(VirtualKey::Multiply),
            0x6B => Some(VirtualKey::Add),
            0x6C => Some(VirtualKey::Separator),
            0x6D => Some(VirtualKey::Subtract),
            0x6E => Some(VirtualKey::Decimal),
            0x6F => Some(VirtualKey::Divide),
            
            // Function keys
            0x70 => Some(VirtualKey::F1),
            0x71 => Some(VirtualKey::F2),
            0x72 => Some(VirtualKey::F3),
            0x73 => Some(VirtualKey::F4),
            0x74 => Some(VirtualKey::F5),
            0x75 => Some(VirtualKey::F6),
            0x76 => Some(VirtualKey::F7),
            0x77 => Some(VirtualKey::F8),
            0x78 => Some(VirtualKey::F9),
            0x79 => Some(VirtualKey::F10),
            0x7A => Some(VirtualKey::F11),
            0x7B => Some(VirtualKey::F12),
            
            // Modifier keys (left/right variants)
            0xA0 => Some(VirtualKey::LShift),
            0xA1 => Some(VirtualKey::RShift),
            0xA2 => Some(VirtualKey::LControl),
            0xA3 => Some(VirtualKey::RControl),
            0xA4 => Some(VirtualKey::LMenu),
            0xA5 => Some(VirtualKey::RMenu),
            
            // OEM keys
            0xBA => Some(VirtualKey::Oem1),
            0xBB => Some(VirtualKey::OemPlus),
            0xBC => Some(VirtualKey::OemComma),
            0xBD => Some(VirtualKey::OemMinus),
            0xBE => Some(VirtualKey::OemPeriod),
            0xBF => Some(VirtualKey::Oem2),
            0xC0 => Some(VirtualKey::Oem3),
            0xDB => Some(VirtualKey::Oem4),
            0xDC => Some(VirtualKey::Oem5),
            0xDD => Some(VirtualKey::Oem6),
            0xDE => Some(VirtualKey::Oem7),
            0xDF => Some(VirtualKey::Oem8),
            0xE1 => Some(VirtualKey::OemAx),
            0xE2 => Some(VirtualKey::Oem102),
            0xE3 => Some(VirtualKey::IcoHelp),
            0xE4 => Some(VirtualKey::Ico00),
            
            _ => None,
        }
    }
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
