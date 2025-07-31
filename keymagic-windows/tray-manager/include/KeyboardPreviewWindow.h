#pragma once

#include <windows.h>
#include <objbase.h>
#include <gdiplus.h>
#include <string>
#include <vector>
#include <memory>
#include <unordered_map>

// Forward declaration
struct KeyboardInfo;

// Key information structure
struct KeyInfo {
    std::wstring unshifted;
    std::wstring shifted;
};

// Visual key structure for rendering
struct VisualKey {
    int x, y, width, height;
    std::wstring label;
    std::wstring unshifted;
    std::wstring shifted;
    bool isModifier;
};

class KeyboardPreviewWindow {
public:
    KeyboardPreviewWindow();
    ~KeyboardPreviewWindow();
    
    bool Initialize(HINSTANCE hInstance);
    void Show(const POINT& anchorPoint, const std::wstring& keyboardId, const std::wstring& keyboardPath);
    void Hide();
    bool IsVisible() const { return m_visible; }
    
    // Window procedure
    static LRESULT CALLBACK WindowProc(HWND hWnd, UINT message, WPARAM wParam, LPARAM lParam);
    
private:
    // Window creation and management
    bool RegisterWindowClass();
    bool CreatePreviewWindow();
    void PositionWindow(const POINT& anchorPoint);
    
    // Keyboard layout parsing
    bool LoadKeyboardLayout(const std::wstring& keyboardPath);
    void SimulateKeyboardLayout();
    void InitializeDefaultLayout();
    
    // Rendering
    void OnPaint(HDC hdc);
    void DrawKeyboard(Gdiplus::Graphics& graphics);
    void DrawKey(Gdiplus::Graphics& graphics, const VisualKey& key);
    void DrawTextWithComplexScript(Gdiplus::Graphics& graphics, const std::wstring& text, 
                                   const Gdiplus::RectF& rect, const Gdiplus::StringFormat& format,
                                   const Gdiplus::Brush& brush, float fontSize);
    
    // Layout generation
    void GenerateKeyboardLayout();
    void AddKey(int row, int col, int colSpan, const std::wstring& label, 
                const std::wstring& keyCode, bool isModifier = false);
    
    // Message handling
    LRESULT HandleMessage(HWND hWnd, UINT message, WPARAM wParam, LPARAM lParam);
    
private:
    static KeyboardPreviewWindow* s_instance;
    static const wchar_t* WINDOW_CLASS_NAME;
    
    HINSTANCE m_hInstance;
    HWND m_hWnd;
    bool m_visible;
    
    // GDI+ resources
    ULONG_PTR m_gdiplusToken;
    std::unique_ptr<Gdiplus::Font> m_fontNormal;
    std::unique_ptr<Gdiplus::Font> m_fontSmall;
    std::unique_ptr<Gdiplus::Font> m_fontLabel;
    
    // Keyboard data
    std::wstring m_keyboardName;
    std::wstring m_keyboardPath;
    std::unordered_map<std::wstring, KeyInfo> m_keyMap;
    std::vector<VisualKey> m_visualKeys;
    
    // KeyMagic engine handle for simulation
    void* m_engineHandle;  // EngineHandle from FFI
    
    // Layout constants
    static const int WINDOW_WIDTH = 800;
    static const int WINDOW_HEIGHT = 320;
    static const int KEY_SIZE = 48;
    static const int KEY_GAP = 4;
    static const int MARGIN = 20;
    static const int TITLE_HEIGHT = 40;
    
    // Timer for auto-hide
    UINT_PTR m_hideTimer;
    static const UINT TIMER_AUTO_HIDE = 1;
    static const UINT AUTO_HIDE_DELAY = 5000; // 5 seconds
    
    // Tracking for smart auto-hide
    POINT m_lastTrayIconPos;
    bool IsMouseNearTrayIcon() const;
};