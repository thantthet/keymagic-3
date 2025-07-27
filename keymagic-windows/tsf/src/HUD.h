#ifndef KEYMAGIC_HUD_H
#define KEYMAGIC_HUD_H

#include <windows.h>
#include <string>

class KeyMagicHUD
{
public:
    static KeyMagicHUD& GetInstance();
    
    // Initialize the HUD window
    HRESULT Initialize();
    
    // Show keyboard name in HUD
    void ShowKeyboard(const std::wstring& keyboardName);
    
    // Cleanup
    void Cleanup();
    
private:
    KeyMagicHUD();
    ~KeyMagicHUD();
    
    // Prevent copying
    KeyMagicHUD(const KeyMagicHUD&) = delete;
    KeyMagicHUD& operator=(const KeyMagicHUD&) = delete;
    
    // Window procedure
    static LRESULT CALLBACK WndProc(HWND hwnd, UINT msg, WPARAM wParam, LPARAM lParam);
    LRESULT HandleMessage(HWND hwnd, UINT msg, WPARAM wParam, LPARAM lParam);
    
    // Internal methods
    void ShowHudInternal(const std::wstring& text);
    void HideHud();
    void UpdateLayeredWindow(HDC memDC, int width, int height);
    void SetBitmapAlpha(HDC hdc, HBITMAP bitmap, COLORREF transparentColor, COLORREF textColor);
    
    HWND m_hwnd;
    static const UINT WM_SHOW_HUD = WM_USER + 1;
    static const UINT HUD_TIMER_ID = 1;
    static const UINT HUD_DISPLAY_TIME_MS = 1500;
};

#endif // KEYMAGIC_HUD_H