#include "KeyProcessingUtils.h"

namespace KeyProcessingUtils
{
    KeyInputData PrepareKeyInput(WPARAM wParam, LPARAM lParam)
    {
        KeyInputData data = {0};
        
        // Check if we should skip this key (modifier/function keys)
        if (ShouldSkipKey(wParam))
        {
            data.shouldSkip = true;
            return data;
        }
        
        // Map virtual key to character
        data.character = MapVirtualKeyToChar(wParam, lParam);
        
        // Get modifier states
        data.shift = (GetKeyState(VK_SHIFT) & 0x8000) ? 1 : 0;
        data.ctrl = (GetKeyState(VK_CONTROL) & 0x8000) ? 1 : 0;
        data.alt = (GetKeyState(VK_MENU) & 0x8000) ? 1 : 0;
        data.capsLock = (GetKeyState(VK_CAPITAL) & 0x0001) ? 1 : 0;
        
        // Only pass printable ASCII characters
        if (!IsPrintableAscii(data.character))
        {
            data.character = '\0';
        }
        
        
        return data;
    }
    
    char MapVirtualKeyToChar(WPARAM wParam, LPARAM lParam)
    {
        BYTE keyState[256];
        GetKeyboardState(keyState);
        
        WCHAR buffer[2] = {0};
        int result = ToUnicode(static_cast<UINT>(wParam), (lParam >> 16) & 0xFF, keyState, buffer, 2, 0);
        
        if (result == 1 && buffer[0] < 128)
        {
            return static_cast<char>(buffer[0]);
        }
        
        return '\0';
    }
    
    bool IsPrintableAscii(char c)
    {
        return c >= 0x20 && c <= 0x7E;
    }
    
    bool ShouldSkipKey(WPARAM wParam)
    {
        return wParam == VK_SHIFT || wParam == VK_CONTROL || wParam == VK_MENU ||
               wParam == VK_LWIN || wParam == VK_RWIN ||
               (wParam >= VK_F1 && wParam <= VK_F24);
    }
    
}