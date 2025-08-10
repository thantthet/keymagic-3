#include <iostream>
#include <cassert>
#include <keymagic/keymagic.h>

// Test hotkey parsing
void test_hotkey_parsing() {
    KeyMagicHotkeyInfo info;
    
    // Test simple hotkey
    assert(keymagic_parse_hotkey("ctrl+a", &info) == 1);
    assert(info.key_code == KeyMagic_VK_KeyA);
    assert(info.ctrl == 1);
    assert(info.alt == 0);
    assert(info.shift == 0);
    assert(info.meta == 0);
    
    // Test multiple modifiers
    assert(keymagic_parse_hotkey("CTRL+SHIFT+ALT+K", &info) == 1);
    assert(info.key_code == KeyMagic_VK_KeyK);
    assert(info.ctrl == 1);
    assert(info.alt == 1);
    assert(info.shift == 1);
    assert(info.meta == 0);
    
    // Test space-separated
    assert(keymagic_parse_hotkey("ctrl shift k", &info) == 1);
    assert(info.key_code == KeyMagic_VK_KeyK);
    assert(info.ctrl == 1);
    assert(info.shift == 1);
    
    // Test special keys
    assert(keymagic_parse_hotkey("ctrl+space", &info) == 1);
    assert(info.key_code == KeyMagic_VK_Space);
    
    assert(keymagic_parse_hotkey("ctrl+enter", &info) == 1);
    assert(info.key_code == KeyMagic_VK_Return);
    
    assert(keymagic_parse_hotkey("ctrl+f1", &info) == 1);
    assert(info.key_code == KeyMagic_VK_F1);
    
    // Test DELETE vs BACKSPACE distinction
    assert(keymagic_parse_hotkey("DELETE", &info) == 1);
    assert(info.key_code == KeyMagic_VK_Delete);
    
    assert(keymagic_parse_hotkey("BACKSPACE", &info) == 1);
    assert(info.key_code == KeyMagic_VK_Back);
    
    assert(keymagic_parse_hotkey("BACK", &info) == 1);
    assert(info.key_code == KeyMagic_VK_Back);
    
    // Test navigation keys
    assert(keymagic_parse_hotkey("HOME", &info) == 1);
    assert(info.key_code == KeyMagic_VK_Home);
    
    assert(keymagic_parse_hotkey("END", &info) == 1);
    assert(info.key_code == KeyMagic_VK_End);
    
    assert(keymagic_parse_hotkey("LEFT", &info) == 1);
    assert(info.key_code == KeyMagic_VK_Left);
    
    assert(keymagic_parse_hotkey("UP", &info) == 1);
    assert(info.key_code == KeyMagic_VK_Up);
    
    assert(keymagic_parse_hotkey("RIGHT", &info) == 1);
    assert(info.key_code == KeyMagic_VK_Right);
    
    assert(keymagic_parse_hotkey("DOWN", &info) == 1);
    assert(info.key_code == KeyMagic_VK_Down);
    
    // Test OEM keys
    assert(keymagic_parse_hotkey("CTRL+=", &info) == 1);
    assert(info.key_code == KeyMagic_VK_OemPlus);
    
    assert(keymagic_parse_hotkey("CTRL+-", &info) == 1);
    assert(info.key_code == KeyMagic_VK_OemMinus);
    
    assert(keymagic_parse_hotkey("CTRL+[", &info) == 1);
    assert(info.key_code == KeyMagic_VK_Oem4);
    
    assert(keymagic_parse_hotkey("CTRL+]", &info) == 1);
    assert(info.key_code == KeyMagic_VK_Oem6);
    
    assert(keymagic_parse_hotkey("CTRL+'", &info) == 1);
    assert(info.key_code == KeyMagic_VK_Oem7);
    
    // Test meta variants
    assert(keymagic_parse_hotkey("meta+k", &info) == 1);
    assert(info.key_code == KeyMagic_VK_KeyK);
    assert(info.meta == 1);
    
    assert(keymagic_parse_hotkey("cmd+k", &info) == 1);
    assert(info.meta == 1);
    
    assert(keymagic_parse_hotkey("win+k", &info) == 1);
    assert(info.meta == 1);
    
    // Test errors
    assert(keymagic_parse_hotkey("", &info) == 0);
    assert(keymagic_parse_hotkey("ctrl+", &info) == 0);
    assert(keymagic_parse_hotkey("ctrl+shift", &info) == 0);
    assert(keymagic_parse_hotkey("ctrl+unknown", &info) == 0);
    assert(keymagic_parse_hotkey("ctrl+a+b", &info) == 0);
    
    // Test function keys (only F1-F12 supported)
    assert(keymagic_parse_hotkey("F12", &info) == 1);
    assert(info.key_code == KeyMagic_VK_F12);
    
    assert(keymagic_parse_hotkey("F13", &info) == 0); // Not supported
    
    std::cout << "All hotkey parsing tests passed!" << std::endl;
}

int main() {
    test_hotkey_parsing();
    return 0;
}