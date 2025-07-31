#include <windows.h>
#include <objbase.h>  // For COM types
#include <gdiplus.h>
#include "KeyboardPreviewWindow.h"
#include <usp10.h>  // For Uniscribe (complex script support)
#include <algorithm>
#include <fstream>
#include <sstream>

#pragma comment(lib, "gdiplus.lib")
#pragma comment(lib, "usp10.lib")  // Uniscribe library

// KeyMagic FFI declarations
extern "C" {
    // Engine handle management
    void* keymagic_engine_new();
    void keymagic_engine_free(void* handle);
    int keymagic_engine_load_keyboard(void* handle, const char* km2_path);
    void keymagic_engine_reset(void* handle);
    
    // Process key output structure
    struct ProcessKeyOutput {
        int action_type;
        char* text;
        int delete_count;
        char* composing_text;
        int is_processed;
    };
    
    // Key processing
    int keymagic_engine_process_key_test_win(
        void* handle,
        int vk_code,
        char character,
        int shift,
        int ctrl,
        int alt,
        int caps_lock,
        ProcessKeyOutput* output
    );
    
    // String management
    void keymagic_free_string(char* s);
    
    // KM2 file access
    void* keymagic_km2_load(const char* path);
    void keymagic_km2_free(void* handle);
    char* keymagic_km2_get_name(void* handle);
    void keymagic_free_string(char* str);
}

KeyboardPreviewWindow* KeyboardPreviewWindow::s_instance = nullptr;
const wchar_t* KeyboardPreviewWindow::WINDOW_CLASS_NAME = L"KeyMagicPreviewWindow";

KeyboardPreviewWindow::KeyboardPreviewWindow()
    : m_hInstance(nullptr)
    , m_hWnd(nullptr)
    , m_visible(false)
    , m_gdiplusToken(0)
    , m_hideTimer(0)
    , m_engineHandle(nullptr)
    , m_scale(0.7f)  // Default to 0.7 scale for compact view
    , m_keyboardXOffset(0) {
    s_instance = this;
}

KeyboardPreviewWindow::~KeyboardPreviewWindow() {
    Hide();
    
    if (m_hWnd) {
        DestroyWindow(m_hWnd);
        m_hWnd = nullptr;
    }
    
    // Free KeyMagic engine
    if (m_engineHandle) {
        keymagic_engine_free(m_engineHandle);
        m_engineHandle = nullptr;
    }
    
    // Shutdown GDI+
    if (m_gdiplusToken) {
        Gdiplus::GdiplusShutdown(m_gdiplusToken);
    }
    
    s_instance = nullptr;
}

bool KeyboardPreviewWindow::Initialize(HINSTANCE hInstance) {
    m_hInstance = hInstance;
    
    // Initialize GDI+
    Gdiplus::GdiplusStartupInput gdiplusStartupInput;
    if (Gdiplus::GdiplusStartup(&m_gdiplusToken, &gdiplusStartupInput, nullptr) != Gdiplus::Ok) {
        return false;
    }
    
    // Register window class
    if (!RegisterWindowClass()) {
        return false;
    }
    
    // Create window
    if (!CreatePreviewWindow()) {
        return false;
    }
    
    // Create fonts for rendering with scaling
    // Use a font that supports complex scripts (e.g., Segoe UI, Arial Unicode MS, or Noto Sans)
    m_fontNormal = std::make_unique<Gdiplus::Font>(L"Segoe UI", 14.0f * m_scale, Gdiplus::FontStyleRegular);
    m_fontSmall = std::make_unique<Gdiplus::Font>(L"Segoe UI", 10.0f * m_scale, Gdiplus::FontStyleRegular);
    m_fontLabel = std::make_unique<Gdiplus::Font>(L"Segoe UI", 9.0f * m_scale, Gdiplus::FontStyleBold);
    
    // If Segoe UI doesn't support the script, try fallback fonts
    if (m_fontNormal->GetLastStatus() != Gdiplus::Ok) {
        m_fontNormal = std::make_unique<Gdiplus::Font>(L"Arial Unicode MS", 14.0f * m_scale, Gdiplus::FontStyleRegular);
        m_fontSmall = std::make_unique<Gdiplus::Font>(L"Arial Unicode MS", 10.0f * m_scale, Gdiplus::FontStyleRegular);
    }
    
    // Create KeyMagic engine for simulation
    m_engineHandle = keymagic_engine_new();
    if (!m_engineHandle) {
        return false;
    }
    
    return true;
}

void KeyboardPreviewWindow::Show(const POINT& anchorPoint, const std::wstring& keyboardId, const std::wstring& keyboardPath) {
    if (!m_hWnd) return;
    
    m_keyboardPath = keyboardPath;
    m_lastTrayIconPos = anchorPoint;
    
    // Load keyboard layout
    if (!LoadKeyboardLayout(keyboardPath)) {
        // Use default layout if loading fails
        InitializeDefaultLayout();
    } else {
        // Simulate keyboard layout using engine
        SimulateKeyboardLayout();
    }
    
    // Generate visual layout
    GenerateKeyboardLayout();
    
    // Position window near anchor point (usually tray icon)
    PositionWindow(anchorPoint);
    
    // Show window
    ShowWindow(m_hWnd, SW_SHOWNOACTIVATE);
    InvalidateRect(m_hWnd, nullptr, TRUE);
    m_visible = true;
    
    // Set auto-hide timer (shorter delay for hover mode)
    if (m_hideTimer) {
        KillTimer(m_hWnd, m_hideTimer);
    }
    m_hideTimer = SetTimer(m_hWnd, TIMER_AUTO_HIDE, 2000, nullptr);  // 2 seconds initial delay
}

void KeyboardPreviewWindow::Hide() {
    if (m_hWnd && m_visible) {
        ShowWindow(m_hWnd, SW_HIDE);
        m_visible = false;
        
        if (m_hideTimer) {
            KillTimer(m_hWnd, m_hideTimer);
            m_hideTimer = 0;
        }
    }
}

void KeyboardPreviewWindow::SetScale(float scale) {
    m_scale = scale;
    
    // Update window size if already created
    if (m_hWnd) {
        SetWindowPos(m_hWnd, nullptr, 0, 0, 
                     Scale(BASE_WINDOW_WIDTH), Scale(BASE_WINDOW_HEIGHT), 
                     SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE);
        
        // Recreate fonts with scaled sizes
        m_fontNormal = std::make_unique<Gdiplus::Font>(L"Segoe UI", 14.0f * m_scale, Gdiplus::FontStyleRegular);
        m_fontSmall = std::make_unique<Gdiplus::Font>(L"Segoe UI", 10.0f * m_scale, Gdiplus::FontStyleRegular);
        m_fontLabel = std::make_unique<Gdiplus::Font>(L"Segoe UI", 9.0f * m_scale, Gdiplus::FontStyleBold);
        
        InvalidateRect(m_hWnd, nullptr, TRUE);
    }
}

bool KeyboardPreviewWindow::RegisterWindowClass() {
    WNDCLASSEXW wcex = {};
    wcex.cbSize = sizeof(WNDCLASSEXW);
    wcex.style = CS_HREDRAW | CS_VREDRAW;
    wcex.lpfnWndProc = WindowProc;
    wcex.hInstance = m_hInstance;
    wcex.hCursor = LoadCursor(nullptr, IDC_ARROW);
    wcex.hbrBackground = (HBRUSH)(COLOR_WINDOW + 1);
    wcex.lpszClassName = WINDOW_CLASS_NAME;
    
    return RegisterClassExW(&wcex) != 0;
}

bool KeyboardPreviewWindow::CreatePreviewWindow() {
    // Create a layered window for smooth rendering
    m_hWnd = CreateWindowExW(
        WS_EX_TOOLWINDOW | WS_EX_TOPMOST | WS_EX_NOACTIVATE,
        WINDOW_CLASS_NAME,
        L"KeyMagic Preview",
        WS_POPUP,
        0, 0, Scale(BASE_WINDOW_WIDTH), Scale(BASE_WINDOW_HEIGHT),
        nullptr,
        nullptr,
        m_hInstance,
        nullptr
    );
    
    return m_hWnd != nullptr;
}

void KeyboardPreviewWindow::PositionWindow(const POINT& anchorPoint) {
    // Get monitor info
    HMONITOR hMonitor = MonitorFromPoint(anchorPoint, MONITOR_DEFAULTTONEAREST);
    MONITORINFO mi = {};
    mi.cbSize = sizeof(MONITORINFO);
    GetMonitorInfo(hMonitor, &mi);
    
    // Calculate position - try to show above the anchor point
    int x = anchorPoint.x - Scale(BASE_WINDOW_WIDTH) / 2;
    int y = anchorPoint.y - Scale(BASE_WINDOW_HEIGHT) - 10;
    
    // Adjust if window goes off screen
    if (x < mi.rcWork.left) {
        x = mi.rcWork.left + 10;
    } else if (x + Scale(BASE_WINDOW_WIDTH) > mi.rcWork.right) {
        x = mi.rcWork.right - Scale(BASE_WINDOW_WIDTH) - 10;
    }
    
    if (y < mi.rcWork.top) {
        // Show below if not enough space above
        y = anchorPoint.y + 10;
    }
    
    SetWindowPos(m_hWnd, HWND_TOPMOST, x, y, 0, 0, SWP_NOSIZE | SWP_NOACTIVATE);
}

bool KeyboardPreviewWindow::LoadKeyboardLayout(const std::wstring& keyboardPath) {
    if (!m_engineHandle || keyboardPath.empty()) {
        return false;
    }
    
    // Convert wide string to narrow string for FFI
    std::string narrowPath;
    int size_needed = WideCharToMultiByte(CP_UTF8, 0, keyboardPath.c_str(), -1, nullptr, 0, nullptr, nullptr);
    if (size_needed > 0) {
        narrowPath.resize(size_needed - 1);
        WideCharToMultiByte(CP_UTF8, 0, keyboardPath.c_str(), -1, &narrowPath[0], size_needed, nullptr, nullptr);
    }
    
    // Load keyboard into engine
    if (keymagic_engine_load_keyboard(m_engineHandle, narrowPath.c_str()) != 0) {
        return false;
    }
    
    // Load KM2 file to get metadata
    void* km2Handle = keymagic_km2_load(narrowPath.c_str());
    if (km2Handle) {
        char* name = keymagic_km2_get_name(km2Handle);
        if (name) {
            // Convert UTF-8 to wide string
            int wsize = MultiByteToWideChar(CP_UTF8, 0, name, -1, nullptr, 0);
            if (wsize > 0) {
                m_keyboardName.resize(wsize - 1);
                MultiByteToWideChar(CP_UTF8, 0, name, -1, &m_keyboardName[0], wsize);
            }
            keymagic_free_string(name);
        } else {
            m_keyboardName = L"KeyMagic Keyboard";
        }
        keymagic_km2_free(km2Handle);
    }
    
    return true;
}

void KeyboardPreviewWindow::SimulateKeyboardLayout() {
    if (!m_engineHandle) {
        return;
    }
    
    m_keyMap.clear();
    
    // Key codes for standard US QWERTY layout
    struct KeyMapping {
        const wchar_t* keyCode;
        UINT vkCode;
        char unshifted;
        char shifted;
    } keyMappings[] = {
        // Number row
        {L"Backquote", VK_OEM_3, '`', '~'},
        {L"Digit1", '1', '1', '!'},
        {L"Digit2", '2', '2', '@'},
        {L"Digit3", '3', '3', '#'},
        {L"Digit4", '4', '4', '$'},
        {L"Digit5", '5', '5', '%'},
        {L"Digit6", '6', '6', '^'},
        {L"Digit7", '7', '7', '&'},
        {L"Digit8", '8', '8', '*'},
        {L"Digit9", '9', '9', '('},
        {L"Digit0", '0', '0', ')'},
        {L"Minus", VK_OEM_MINUS, '-', '_'},
        {L"Equal", VK_OEM_PLUS, '=', '+'},
        
        // Letters
        {L"KeyQ", 'Q', 'q', 'Q'},
        {L"KeyW", 'W', 'w', 'W'},
        {L"KeyE", 'E', 'e', 'E'},
        {L"KeyR", 'R', 'r', 'R'},
        {L"KeyT", 'T', 't', 'T'},
        {L"KeyY", 'Y', 'y', 'Y'},
        {L"KeyU", 'U', 'u', 'U'},
        {L"KeyI", 'I', 'i', 'I'},
        {L"KeyO", 'O', 'o', 'O'},
        {L"KeyP", 'P', 'p', 'P'},
        {L"KeyA", 'A', 'a', 'A'},
        {L"KeyS", 'S', 's', 'S'},
        {L"KeyD", 'D', 'd', 'D'},
        {L"KeyF", 'F', 'f', 'F'},
        {L"KeyG", 'G', 'g', 'G'},
        {L"KeyH", 'H', 'h', 'H'},
        {L"KeyJ", 'J', 'j', 'J'},
        {L"KeyK", 'K', 'k', 'K'},
        {L"KeyL", 'L', 'l', 'L'},
        {L"KeyZ", 'Z', 'z', 'Z'},
        {L"KeyX", 'X', 'x', 'X'},
        {L"KeyC", 'C', 'c', 'C'},
        {L"KeyV", 'V', 'v', 'V'},
        {L"KeyB", 'B', 'b', 'B'},
        {L"KeyN", 'N', 'n', 'N'},
        {L"KeyM", 'M', 'm', 'M'},
        
        // Punctuation
        {L"BracketLeft", VK_OEM_4, '[', '{'},
        {L"BracketRight", VK_OEM_6, ']', '}'},
        {L"Backslash", VK_OEM_5, '\\', '|'},
        {L"Semicolon", VK_OEM_1, ';', ':'},
        {L"Quote", VK_OEM_7, '\'', '"'},
        {L"Comma", VK_OEM_COMMA, ',', '<'},
        {L"Period", VK_OEM_PERIOD, '.', '>'},
        {L"Slash", VK_OEM_2, '/', '?'},
        {L"Space", VK_SPACE, ' ', ' '}
    };
    
    // Reset engine before simulation
    keymagic_engine_reset(m_engineHandle);
    
    // Process each key to get actual output from the keyboard layout
    for (const auto& mapping : keyMappings) {
        ProcessKeyOutput output;
        KeyInfo keyInfo;
        
        // Test unshifted key
        keymagic_engine_reset(m_engineHandle);
        int result = keymagic_engine_process_key_test_win(
            m_engineHandle,
            mapping.vkCode,
            mapping.unshifted,
            0,  // shift
            0,  // ctrl
            0,  // alt
            0,  // caps_lock
            &output
        );
        
        if (result == 0 && output.is_processed) {
            if (output.action_type == 1 && output.text) {  // Insert action
                // Convert UTF-8 to wide string
                int wsize = MultiByteToWideChar(CP_UTF8, 0, output.text, -1, nullptr, 0);
                if (wsize > 0) {
                    keyInfo.unshifted.resize(wsize - 1);
                    MultiByteToWideChar(CP_UTF8, 0, output.text, -1, &keyInfo.unshifted[0], wsize);
                }
            } else if (output.composing_text && strlen(output.composing_text) > 0) {
                // Use composing text if available
                int wsize = MultiByteToWideChar(CP_UTF8, 0, output.composing_text, -1, nullptr, 0);
                if (wsize > 0) {
                    keyInfo.unshifted.resize(wsize - 1);
                    MultiByteToWideChar(CP_UTF8, 0, output.composing_text, -1, &keyInfo.unshifted[0], wsize);
                }
            }
        } else {
            // Use default if not processed
            keyInfo.unshifted = std::wstring(1, mapping.unshifted);
        }
        
        // Free allocated strings
        if (output.text) keymagic_free_string(output.text);
        if (output.composing_text) keymagic_free_string(output.composing_text);
        
        // Test shifted key
        keymagic_engine_reset(m_engineHandle);
        result = keymagic_engine_process_key_test_win(
            m_engineHandle,
            mapping.vkCode,
            mapping.shifted,
            1,  // shift
            0,  // ctrl
            0,  // alt
            0,  // caps_lock
            &output
        );
        
        if (result == 0 && output.is_processed) {
            if (output.action_type == 1 && output.text) {  // Insert action
                int wsize = MultiByteToWideChar(CP_UTF8, 0, output.text, -1, nullptr, 0);
                if (wsize > 0) {
                    keyInfo.shifted.resize(wsize - 1);
                    MultiByteToWideChar(CP_UTF8, 0, output.text, -1, &keyInfo.shifted[0], wsize);
                }
            } else if (output.composing_text && strlen(output.composing_text) > 0) {
                int wsize = MultiByteToWideChar(CP_UTF8, 0, output.composing_text, -1, nullptr, 0);
                if (wsize > 0) {
                    keyInfo.shifted.resize(wsize - 1);
                    MultiByteToWideChar(CP_UTF8, 0, output.composing_text, -1, &keyInfo.shifted[0], wsize);
                }
            }
        } else {
            // Use default if not processed
            keyInfo.shifted = std::wstring(1, mapping.shifted);
        }
        
        // Free allocated strings
        if (output.text) keymagic_free_string(output.text);
        if (output.composing_text) keymagic_free_string(output.composing_text);
        
        // Store the key mapping
        m_keyMap[mapping.keyCode] = keyInfo;
    }
}

void KeyboardPreviewWindow::InitializeDefaultLayout() {
    // Initialize with default QWERTY layout
    m_keyboardName = L"Default QWERTY";
    m_keyMap.clear();
    
    // Number row
    m_keyMap[L"Backquote"] = {L"`", L"~"};
    m_keyMap[L"Digit1"] = {L"1", L"!"};
    m_keyMap[L"Digit2"] = {L"2", L"@"};
    m_keyMap[L"Digit3"] = {L"3", L"#"};
    m_keyMap[L"Digit4"] = {L"4", L"$"};
    m_keyMap[L"Digit5"] = {L"5", L"%"};
    m_keyMap[L"Digit6"] = {L"6", L"^"};
    m_keyMap[L"Digit7"] = {L"7", L"&"};
    m_keyMap[L"Digit8"] = {L"8", L"*"};
    m_keyMap[L"Digit9"] = {L"9", L"("};
    m_keyMap[L"Digit0"] = {L"0", L")"};
    m_keyMap[L"Minus"] = {L"-", L"_"};
    m_keyMap[L"Equal"] = {L"=", L"+"};
    
    // Letters (showing example with complex scripts)
    // In a real implementation, these would come from the KM2 file
    m_keyMap[L"KeyQ"] = {L"q", L"Q"};
    m_keyMap[L"KeyW"] = {L"w", L"W"};
    m_keyMap[L"KeyE"] = {L"e", L"E"};
    // ... etc
}

void KeyboardPreviewWindow::GenerateKeyboardLayout() {
    m_visualKeys.clear();
    
    // Calculate total keyboard width (30 half-key units)
    const int totalColumns = 30;
    const int keyboardWidth = totalColumns * Scale(BASE_KEY_SIZE + BASE_KEY_GAP) / 2 - Scale(BASE_KEY_GAP);
    const int windowInnerWidth = Scale(BASE_WINDOW_WIDTH) - 2 * Scale(BASE_MARGIN);
    m_keyboardXOffset = (windowInnerWidth - keyboardWidth) / 2;  // Center the keyboard
    
    // Define keyboard rows with their keys
    // Row 0: Number row
    AddKey(0, 0, 2, L"`", L"Backquote");
    for (int i = 1; i <= 9; i++) {
        AddKey(0, i * 2, 2, std::to_wstring(i), L"Digit" + std::to_wstring(i));
    }
    AddKey(0, 20, 2, L"0", L"Digit0");
    AddKey(0, 22, 2, L"-", L"Minus");
    AddKey(0, 24, 2, L"=", L"Equal");
    AddKey(0, 26, 4, L"BACK", L"Backspace", true);
    
    // Row 1: QWERTY row
    AddKey(1, 0, 3, L"TAB", L"Tab", true);
    const wchar_t* row1[] = {L"Q", L"W", L"E", L"R", L"T", L"Y", L"U", L"I", L"O", L"P"};
    for (int i = 0; i < 10; i++) {
        AddKey(1, 3 + i * 2, 2, row1[i], std::wstring(L"Key") + row1[i]);
    }
    AddKey(1, 23, 2, L"[", L"BracketLeft");
    AddKey(1, 25, 2, L"]", L"BracketRight");
    AddKey(1, 27, 3, L"\\", L"Backslash");
    
    // Row 2: ASDF row
    AddKey(2, 0, 4, L"CAPS", L"CapsLock", true);
    const wchar_t* row2[] = {L"A", L"S", L"D", L"F", L"G", L"H", L"J", L"K", L"L"};
    for (int i = 0; i < 9; i++) {
        AddKey(2, 4 + i * 2, 2, row2[i], std::wstring(L"Key") + row2[i]);
    }
    AddKey(2, 22, 2, L";", L"Semicolon");
    AddKey(2, 24, 2, L"'", L"Quote");
    AddKey(2, 26, 4, L"ENTER", L"Enter", true);
    
    // Row 3: ZXCV row
    AddKey(3, 0, 5, L"SHIFT", L"ShiftLeft", true);
    const wchar_t* row3[] = {L"Z", L"X", L"C", L"V", L"B", L"N", L"M"};
    for (int i = 0; i < 7; i++) {
        AddKey(3, 5 + i * 2, 2, row3[i], std::wstring(L"Key") + row3[i]);
    }
    AddKey(3, 19, 2, L",", L"Comma");
    AddKey(3, 21, 2, L".", L"Period");
    AddKey(3, 23, 2, L"/", L"Slash");
    AddKey(3, 25, 5, L"SHIFT", L"ShiftRight", true);
    
    // Row 4: Space row
    AddKey(4, 0, 3, L"CTRL", L"ControlLeft", true);
    AddKey(4, 3, 3, L"WIN", L"MetaLeft", true);
    AddKey(4, 6, 3, L"ALT", L"AltLeft", true);
    AddKey(4, 9, 12, L"SPACE", L"Space");
    AddKey(4, 21, 3, L"ALT", L"AltRight", true);
    AddKey(4, 24, 3, L"WIN", L"MetaRight", true);
    AddKey(4, 27, 3, L"CTRL", L"ControlRight", true);
}

void KeyboardPreviewWindow::AddKey(int row, int col, int colSpan, const std::wstring& label, 
                                   const std::wstring& keyCode, bool isModifier) {
    VisualKey key;
    key.x = Scale(BASE_MARGIN) + m_keyboardXOffset + col * Scale(BASE_KEY_SIZE + BASE_KEY_GAP) / 2;
    key.y = Scale(BASE_TITLE_HEIGHT + BASE_MARGIN) + row * Scale(BASE_KEY_SIZE + BASE_KEY_GAP);
    key.width = Scale(BASE_KEY_SIZE + BASE_KEY_GAP) * colSpan / 2 - Scale(BASE_KEY_GAP);
    key.height = Scale(BASE_KEY_SIZE);
    key.label = label;
    key.isModifier = isModifier;
    
    // Get actual key mappings
    auto it = m_keyMap.find(keyCode);
    if (it != m_keyMap.end()) {
        key.unshifted = it->second.unshifted;
        key.shifted = it->second.shifted;
    } else if (!isModifier) {
        // Default to label if no mapping found
        key.unshifted = label;
        std::transform(key.unshifted.begin(), key.unshifted.end(), key.unshifted.begin(), ::tolower);
    }
    
    m_visualKeys.push_back(key);
}

LRESULT CALLBACK KeyboardPreviewWindow::WindowProc(HWND hWnd, UINT message, WPARAM wParam, LPARAM lParam) {
    if (s_instance) {
        return s_instance->HandleMessage(hWnd, message, wParam, lParam);
    }
    return DefWindowProc(hWnd, message, wParam, lParam);
}

LRESULT KeyboardPreviewWindow::HandleMessage(HWND hWnd, UINT message, WPARAM wParam, LPARAM lParam) {
    switch (message) {
        case WM_PAINT:
            {
                PAINTSTRUCT ps;
                HDC hdc = BeginPaint(hWnd, &ps);
                OnPaint(hdc);
                EndPaint(hWnd, &ps);
            }
            return 0;
            
        case WM_TIMER:
            if (wParam == TIMER_AUTO_HIDE) {
                KillTimer(hWnd, TIMER_AUTO_HIDE);
                m_hideTimer = 0;
                
                // Check if mouse is over the preview window
                POINT pt;
                GetCursorPos(&pt);
                RECT windowRect;
                GetWindowRect(m_hWnd, &windowRect);
                
                if (PtInRect(&windowRect, pt)) {
                    // Mouse is over the window, reset timer
                    m_hideTimer = SetTimer(hWnd, TIMER_AUTO_HIDE, 2000, nullptr);
                } else if (IsMouseNearTrayIcon()) {
                    // Mouse is near tray icon, reset timer with shorter delay
                    m_hideTimer = SetTimer(hWnd, TIMER_AUTO_HIDE, 500, nullptr);
                } else {
                    // Mouse is away from both, hide the window
                    Hide();
                }
            }
            return 0;
            
        case WM_MOUSEMOVE:
            // Track mouse for auto-hide
            {
                TRACKMOUSEEVENT tme = {};
                tme.cbSize = sizeof(TRACKMOUSEEVENT);
                tme.dwFlags = TME_LEAVE;
                tme.hwndTrack = hWnd;
                TrackMouseEvent(&tme);
                
                // Reset auto-hide timer when mouse is over the window
                if (m_hideTimer) {
                    KillTimer(hWnd, m_hideTimer);
                    m_hideTimer = SetTimer(hWnd, TIMER_AUTO_HIDE, AUTO_HIDE_DELAY, nullptr);
                }
            }
            return 0;
            
        case WM_MOUSELEAVE:
            // Start auto-hide timer when mouse leaves the window
            if (m_hideTimer) {
                KillTimer(hWnd, m_hideTimer);
            }
            m_hideTimer = SetTimer(hWnd, TIMER_AUTO_HIDE, 500, nullptr); // Check quickly after mouse leaves
            return 0;
            
        case WM_ERASEBKGND:
            return 1; // We'll handle background in WM_PAINT
    }
    
    return DefWindowProc(hWnd, message, wParam, lParam);
}

void KeyboardPreviewWindow::OnPaint(HDC hdc) {
    // Create GDI+ graphics object
    Gdiplus::Graphics graphics(hdc);
    
    // Enable high-quality text rendering for complex scripts
    graphics.SetTextRenderingHint(Gdiplus::TextRenderingHintAntiAlias);
    graphics.SetSmoothingMode(Gdiplus::SmoothingModeHighQuality);
    
    // Clear background
    Gdiplus::SolidBrush backgroundBrush(Gdiplus::Color(245, 245, 245));
    graphics.FillRectangle(&backgroundBrush, 0, 0, Scale(BASE_WINDOW_WIDTH), Scale(BASE_WINDOW_HEIGHT));
    
    // Draw border
    Gdiplus::Pen borderPen(Gdiplus::Color(200, 200, 200), 1.0f * m_scale);
    graphics.DrawRectangle(&borderPen, 0, 0, Scale(BASE_WINDOW_WIDTH) - 1, Scale(BASE_WINDOW_HEIGHT) - 1);
    
    // Draw title
    Gdiplus::SolidBrush titleBrush(Gdiplus::Color(51, 51, 51));
    Gdiplus::Font titleFont(L"Segoe UI", 16.0f * m_scale, Gdiplus::FontStyleBold);
    Gdiplus::StringFormat titleFormat;
    titleFormat.SetAlignment(Gdiplus::StringAlignmentCenter);
    titleFormat.SetLineAlignment(Gdiplus::StringAlignmentCenter);
    titleFormat.SetFormatFlags(Gdiplus::StringFormatFlagsNoClip);  // Allow text overflow
    
    Gdiplus::RectF titleRect(0, 0, Scale(BASE_WINDOW_WIDTH), Scale(BASE_TITLE_HEIGHT));
    graphics.DrawString(m_keyboardName.c_str(), -1, &titleFont, titleRect, &titleFormat, &titleBrush);
    
    // Draw keyboard
    DrawKeyboard(graphics);
}

void KeyboardPreviewWindow::DrawKeyboard(Gdiplus::Graphics& graphics) {
    for (const auto& key : m_visualKeys) {
        DrawKey(graphics, key);
    }
}

void KeyboardPreviewWindow::DrawKey(Gdiplus::Graphics& graphics, const VisualKey& key) {
    // Draw key background
    Gdiplus::SolidBrush keyBrush(key.isModifier ? Gdiplus::Color(220, 220, 220) : Gdiplus::Color(250, 250, 250));
    Gdiplus::Pen keyPen(Gdiplus::Color(200, 200, 200), 1.0f * m_scale);
    
    Gdiplus::RectF keyRect(key.x, key.y, key.width, key.height);
    
    // Draw rounded rectangle for key
    float radius = 4.0f * m_scale;
    Gdiplus::GraphicsPath path;
    path.AddArc(keyRect.X, keyRect.Y, radius * 2, radius * 2, 180, 90);
    path.AddArc(keyRect.X + keyRect.Width - radius * 2, keyRect.Y, radius * 2, radius * 2, 270, 90);
    path.AddArc(keyRect.X + keyRect.Width - radius * 2, keyRect.Y + keyRect.Height - radius * 2, radius * 2, radius * 2, 0, 90);
    path.AddArc(keyRect.X, keyRect.Y + keyRect.Height - radius * 2, radius * 2, radius * 2, 90, 90);
    path.CloseFigure();
    
    graphics.FillPath(&keyBrush, &path);
    graphics.DrawPath(&keyPen, &path);
    
    // Draw key content
    if (key.isModifier) {
        // Draw modifier key label
        Gdiplus::StringFormat format;
        format.SetAlignment(Gdiplus::StringAlignmentCenter);
        format.SetLineAlignment(Gdiplus::StringAlignmentCenter);
        format.SetFormatFlags(Gdiplus::StringFormatFlagsNoClip);  // Allow text overflow
        
        Gdiplus::SolidBrush textBrush(Gdiplus::Color(100, 100, 100));
        DrawTextWithComplexScript(graphics, key.label, keyRect, format, textBrush, 9.0f * m_scale);
    } else {
        // Draw shifted character (top-left)
        if (!key.shifted.empty() && key.shifted != key.unshifted) {
            Gdiplus::RectF shiftedRect(key.x + Scale(4), key.y + Scale(3), key.width - Scale(8), key.height / 3);
            Gdiplus::StringFormat shiftedFormat;
            shiftedFormat.SetAlignment(Gdiplus::StringAlignmentNear);
            shiftedFormat.SetLineAlignment(Gdiplus::StringAlignmentNear);
            shiftedFormat.SetFormatFlags(Gdiplus::StringFormatFlagsNoClip);  // Allow text overflow
            
            Gdiplus::SolidBrush shiftedBrush(Gdiplus::Color(120, 120, 120));
            DrawTextWithComplexScript(graphics, key.shifted, shiftedRect, shiftedFormat, shiftedBrush, 10.0f * m_scale);
        }
        
        // Draw unshifted character (bottom-right)
        if (!key.unshifted.empty()) {
            Gdiplus::RectF unshiftedRect(key.x + key.width / 3, key.y + key.height / 2, key.width * 2 / 3 - Scale(4), key.height / 2 - Scale(4));
            Gdiplus::StringFormat unshiftedFormat;
            unshiftedFormat.SetAlignment(Gdiplus::StringAlignmentFar);
            unshiftedFormat.SetLineAlignment(Gdiplus::StringAlignmentFar);
            unshiftedFormat.SetFormatFlags(Gdiplus::StringFormatFlagsNoClip);  // Allow text overflow
            
            Gdiplus::SolidBrush unshiftedBrush(Gdiplus::Color(34, 34, 34));
            DrawTextWithComplexScript(graphics, key.unshifted, unshiftedRect, unshiftedFormat, unshiftedBrush, 16.0f * m_scale);
        }
    }
}

void KeyboardPreviewWindow::DrawTextWithComplexScript(Gdiplus::Graphics& graphics, const std::wstring& text,
                                                     const Gdiplus::RectF& rect, const Gdiplus::StringFormat& format,
                                                     const Gdiplus::Brush& brush, float fontSize) {
    // Try multiple fonts to ensure complex script support
    const wchar_t* fontFamilies[] = {
        L"Segoe UI",           // Default Windows font
        L"Arial Unicode MS",   // Broad Unicode support
        L"Microsoft Sans Serif",
        L"Nirmala UI",        // Good for Indic scripts
        L"Microsoft YaHei",    // Chinese
        L"Meiryo",            // Japanese
        L"Malgun Gothic",     // Korean
        L"Leelawadee UI",     // Thai
        L"Myanmar Text"        // Myanmar
    };
    
    // Try each font until we find one that can render the text
    for (const auto& fontFamily : fontFamilies) {
        Gdiplus::Font font(fontFamily, fontSize, Gdiplus::FontStyleRegular);
        if (font.GetLastStatus() == Gdiplus::Ok) {
            // Use DirectWrite/Uniscribe for complex script shaping
            graphics.DrawString(text.c_str(), -1, &font, rect, &format, &brush);
            
            // Check if the font actually rendered the characters
            // (In a real implementation, you might want to check if glyphs are missing)
            break;
        }
    }
}

bool KeyboardPreviewWindow::IsMouseNearTrayIcon() const {
    POINT currentPos;
    GetCursorPos(&currentPos);
    
    // Get the taskbar/system tray area
    HWND hTaskbar = FindWindowW(L"Shell_TrayWnd", nullptr);
    if (!hTaskbar) return false;
    
    HWND hTrayNotify = FindWindowExW(hTaskbar, nullptr, L"TrayNotifyWnd", nullptr);
    if (!hTrayNotify) return false;
    
    RECT trayRect;
    GetWindowRect(hTrayNotify, &trayRect);
    
    // Expand the rect a bit for tolerance
    InflateRect(&trayRect, 20, 20);
    
    // Check if mouse is in the tray area
    return PtInRect(&trayRect, currentPos) != FALSE;
}