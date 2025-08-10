#include <gtest/gtest.h>
#include <keymagic/keymagic.h>

TEST(HotkeyParsingTest, SimpleHotkey) {
    KeyMagicHotkeyInfo info;
    
    // Test simple hotkey
    EXPECT_EQ(keymagic_parse_hotkey("ctrl+a", &info), 1);
    EXPECT_EQ(info.key_code, KeyMagic_VK_KeyA);
    EXPECT_EQ(info.ctrl, 1);
    EXPECT_EQ(info.alt, 0);
    EXPECT_EQ(info.shift, 0);
    EXPECT_EQ(info.meta, 0);
}

TEST(HotkeyParsingTest, MultipleModifiers) {
    KeyMagicHotkeyInfo info;
    
    // Test multiple modifiers
    EXPECT_EQ(keymagic_parse_hotkey("CTRL+SHIFT+ALT+K", &info), 1);
    EXPECT_EQ(info.key_code, KeyMagic_VK_KeyK);
    EXPECT_EQ(info.ctrl, 1);
    EXPECT_EQ(info.alt, 1);
    EXPECT_EQ(info.shift, 1);
    EXPECT_EQ(info.meta, 0);
}

TEST(HotkeyParsingTest, SpaceSeparated) {
    KeyMagicHotkeyInfo info;
    
    // Test space-separated
    EXPECT_EQ(keymagic_parse_hotkey("ctrl shift k", &info), 1);
    EXPECT_EQ(info.key_code, KeyMagic_VK_KeyK);
    EXPECT_EQ(info.ctrl, 1);
    EXPECT_EQ(info.shift, 1);
}

TEST(HotkeyParsingTest, SpecialKeys) {
    KeyMagicHotkeyInfo info;
    
    // Test special keys
    EXPECT_EQ(keymagic_parse_hotkey("ctrl+space", &info), 1);
    EXPECT_EQ(info.key_code, KeyMagic_VK_Space);
    
    EXPECT_EQ(keymagic_parse_hotkey("ctrl+enter", &info), 1);
    EXPECT_EQ(info.key_code, KeyMagic_VK_Return);
    
    EXPECT_EQ(keymagic_parse_hotkey("ctrl+f1", &info), 1);
    EXPECT_EQ(info.key_code, KeyMagic_VK_F1);
}

TEST(HotkeyParsingTest, DeleteVsBackspace) {
    KeyMagicHotkeyInfo info;
    
    // Test DELETE vs BACKSPACE distinction
    EXPECT_EQ(keymagic_parse_hotkey("DELETE", &info), 1);
    EXPECT_EQ(info.key_code, KeyMagic_VK_Delete);
    
    EXPECT_EQ(keymagic_parse_hotkey("BACKSPACE", &info), 1);
    EXPECT_EQ(info.key_code, KeyMagic_VK_Back);
    
    EXPECT_EQ(keymagic_parse_hotkey("BACK", &info), 1);
    EXPECT_EQ(info.key_code, KeyMagic_VK_Back);
}

TEST(HotkeyParsingTest, NavigationKeys) {
    KeyMagicHotkeyInfo info;
    
    // Test navigation keys
    EXPECT_EQ(keymagic_parse_hotkey("HOME", &info), 1);
    EXPECT_EQ(info.key_code, KeyMagic_VK_Home);
    
    EXPECT_EQ(keymagic_parse_hotkey("END", &info), 1);
    EXPECT_EQ(info.key_code, KeyMagic_VK_End);
    
    EXPECT_EQ(keymagic_parse_hotkey("LEFT", &info), 1);
    EXPECT_EQ(info.key_code, KeyMagic_VK_Left);
    
    EXPECT_EQ(keymagic_parse_hotkey("UP", &info), 1);
    EXPECT_EQ(info.key_code, KeyMagic_VK_Up);
    
    EXPECT_EQ(keymagic_parse_hotkey("RIGHT", &info), 1);
    EXPECT_EQ(info.key_code, KeyMagic_VK_Right);
    
    EXPECT_EQ(keymagic_parse_hotkey("DOWN", &info), 1);
    EXPECT_EQ(info.key_code, KeyMagic_VK_Down);
}

TEST(HotkeyParsingTest, OemKeys) {
    KeyMagicHotkeyInfo info;
    
    // Test OEM keys
    EXPECT_EQ(keymagic_parse_hotkey("CTRL+=", &info), 1);
    EXPECT_EQ(info.key_code, KeyMagic_VK_OemPlus);
    
    EXPECT_EQ(keymagic_parse_hotkey("CTRL+-", &info), 1);
    EXPECT_EQ(info.key_code, KeyMagic_VK_OemMinus);
    
    EXPECT_EQ(keymagic_parse_hotkey("CTRL+[", &info), 1);
    EXPECT_EQ(info.key_code, KeyMagic_VK_Oem4);
    
    EXPECT_EQ(keymagic_parse_hotkey("CTRL+]", &info), 1);
    EXPECT_EQ(info.key_code, KeyMagic_VK_Oem6);
    
    EXPECT_EQ(keymagic_parse_hotkey("CTRL+'", &info), 1);
    EXPECT_EQ(info.key_code, KeyMagic_VK_Oem7);
}

TEST(HotkeyParsingTest, MetaVariants) {
    KeyMagicHotkeyInfo info;
    
    // Test meta variants
    EXPECT_EQ(keymagic_parse_hotkey("meta+k", &info), 1);
    EXPECT_EQ(info.key_code, KeyMagic_VK_KeyK);
    EXPECT_EQ(info.meta, 1);
    
    EXPECT_EQ(keymagic_parse_hotkey("cmd+k", &info), 1);
    EXPECT_EQ(info.meta, 1);
    
    EXPECT_EQ(keymagic_parse_hotkey("win+k", &info), 1);
    EXPECT_EQ(info.meta, 1);
}

TEST(HotkeyParsingTest, ErrorCases) {
    KeyMagicHotkeyInfo info;
    
    // Test errors
    EXPECT_EQ(keymagic_parse_hotkey("", &info), 0);
    EXPECT_EQ(keymagic_parse_hotkey("ctrl+", &info), 0);
    EXPECT_EQ(keymagic_parse_hotkey("ctrl+shift", &info), 0);
    EXPECT_EQ(keymagic_parse_hotkey("ctrl+unknown", &info), 0);
    EXPECT_EQ(keymagic_parse_hotkey("ctrl+a+b", &info), 0);
    
    // Test function keys (only F1-F12 supported)
    EXPECT_EQ(keymagic_parse_hotkey("F12", &info), 1);
    EXPECT_EQ(info.key_code, KeyMagic_VK_F12);
    
    EXPECT_EQ(keymagic_parse_hotkey("F13", &info), 0); // Not supported
}