#include "HUD.h"
#include <algorithm>
#include <vector>

// RGB macro
#define RGB_MACRO(r,g,b) ((COLORREF)(((BYTE)(r)|((WORD)((BYTE)(g))<<8))|(((DWORD)(BYTE)(b))<<16)))

KeyMagicHUD& KeyMagicHUD::GetInstance()
{
    static KeyMagicHUD instance;
    return instance;
}

KeyMagicHUD::KeyMagicHUD() : m_hwnd(nullptr)
{
}

KeyMagicHUD::~KeyMagicHUD()
{
    Cleanup();
}

HRESULT KeyMagicHUD::Initialize()
{
    if (m_hwnd != nullptr)
        return S_OK; // Already initialized
        
    HINSTANCE hInstance = GetModuleHandle(nullptr);
    
    // Register window class
    WNDCLASSEXW wc = {};
    wc.cbSize = sizeof(WNDCLASSEXW);
    wc.style = CS_HREDRAW | CS_VREDRAW;
    wc.lpfnWndProc = WndProc;
    wc.cbWndExtra = sizeof(void*);
    wc.hInstance = hInstance;
    wc.hCursor = LoadCursor(nullptr, IDC_ARROW);
    wc.hbrBackground = (HBRUSH)(COLOR_WINDOW + 1);
    wc.lpszClassName = L"KeyMagicHUD";
    
    ATOM atom = RegisterClassExW(&wc);
    if (atom == 0 && GetLastError() != ERROR_CLASS_ALREADY_EXISTS)
    {
        return HRESULT_FROM_WIN32(GetLastError());
    }
    
    // Create window
    m_hwnd = CreateWindowExW(
        WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
        L"KeyMagicHUD",
        L"",
        WS_POPUP,
        0, 0, 0, 0,
        nullptr,
        nullptr,
        hInstance,
        this // Pass this pointer for WM_CREATE
    );
    
    if (!m_hwnd)
    {
        return HRESULT_FROM_WIN32(GetLastError());
    }
    
    return S_OK;
}

void KeyMagicHUD::ShowKeyboard(const std::wstring& keyboardName)
{
    if (!m_hwnd)
        return;
        
    // Allocate and copy string
    std::wstring* pText = new std::wstring(keyboardName);
    
    // Post message to show HUD
    PostMessage(m_hwnd, WM_SHOW_HUD, 0, reinterpret_cast<LPARAM>(pText));
}

void KeyMagicHUD::Cleanup()
{
    if (m_hwnd)
    {
        DestroyWindow(m_hwnd);
        m_hwnd = nullptr;
    }
}

LRESULT CALLBACK KeyMagicHUD::WndProc(HWND hwnd, UINT msg, WPARAM wParam, LPARAM lParam)
{
    KeyMagicHUD* pThis = nullptr;
    
    if (msg == WM_CREATE)
    {
        CREATESTRUCT* pCreate = reinterpret_cast<CREATESTRUCT*>(lParam);
        pThis = reinterpret_cast<KeyMagicHUD*>(pCreate->lpCreateParams);
        SetWindowLongPtr(hwnd, GWLP_USERDATA, reinterpret_cast<LONG_PTR>(pThis));
    }
    else
    {
        pThis = reinterpret_cast<KeyMagicHUD*>(GetWindowLongPtr(hwnd, GWLP_USERDATA));
    }
    
    if (pThis)
    {
        return pThis->HandleMessage(hwnd, msg, wParam, lParam);
    }
    
    return DefWindowProcW(hwnd, msg, wParam, lParam);
}

LRESULT KeyMagicHUD::HandleMessage(HWND hwnd, UINT msg, WPARAM wParam, LPARAM lParam)
{
    switch (msg)
    {
        case WM_SHOW_HUD:
        {
            std::wstring* pText = reinterpret_cast<std::wstring*>(lParam);
            if (pText)
            {
                ShowHudInternal(*pText);
                delete pText;
                
                // Set timer to hide
                SetTimer(hwnd, HUD_TIMER_ID, HUD_DISPLAY_TIME_MS, nullptr);
            }
            return 0;
        }
        
        case WM_TIMER:
        {
            if (wParam == HUD_TIMER_ID)
            {
                HideHud();
                KillTimer(hwnd, HUD_TIMER_ID);
            }
            return 0;
        }
        
        case WM_NCHITTEST:
            return HTNOWHERE; // Make window click-through
            
        default:
            return DefWindowProcW(hwnd, msg, wParam, lParam);
    }
}

void KeyMagicHUD::ShowHudInternal(const std::wstring& text)
{
    // Show window
    ShowWindow(m_hwnd, SW_SHOWNOACTIVATE);
    
    HDC hdcScreen = GetDC(nullptr);
    HDC memDC = CreateCompatibleDC(hdcScreen);
    
    // Create font
    int fontSize = -MulDiv(20, GetDeviceCaps(hdcScreen, LOGPIXELSY), 72);
    HFONT font = CreateFontW(
        fontSize, 0, 0, 0,
        FW_NORMAL, FALSE, FALSE, FALSE,
        DEFAULT_CHARSET, OUT_DEFAULT_PRECIS,
        CLIP_DEFAULT_PRECIS, DEFAULT_QUALITY,
        DEFAULT_PITCH | FF_SWISS,
        L"Segoe UI"
    );
    
    HFONT oldFont = (HFONT)SelectObject(memDC, font);
    
    // Measure text
    SIZE textSize;
    GetTextExtentPoint32W(memDC, text.c_str(), (int)text.length(), &textSize);
    
    const int padding = 20;
    int width = textSize.cx + (padding * 2);
    int height = textSize.cy + (padding * 2);
    
    // Create bitmap
    HBITMAP bitmap = CreateCompatibleBitmap(hdcScreen, width, height);
    HBITMAP oldBitmap = (HBITMAP)SelectObject(memDC, bitmap);
    
    // Colors
    COLORREF transparentColor = RGB_MACRO(255, 0, 255); // Magenta
    COLORREF bgColor = RGB_MACRO(0, 0, 0); // Black
    COLORREF textColor = RGB_MACRO(255, 255, 255); // White
    
    // Fill with transparent color
    HBRUSH transparentBrush = CreateSolidBrush(transparentColor);
    RECT fillRect = {0, 0, width, height};
    FillRect(memDC, &fillRect, transparentBrush);
    DeleteObject(transparentBrush);
    
    // Draw rounded rectangle background
    HBRUSH bgBrush = CreateSolidBrush(bgColor);
    HBRUSH oldBrush = (HBRUSH)SelectObject(memDC, bgBrush);
    HPEN bgPen = CreatePen(PS_SOLID, 0, bgColor);
    HPEN oldPen = (HPEN)SelectObject(memDC, bgPen);
    
    RoundRect(memDC, 0, 0, width, height, 22, 22);
    
    SelectObject(memDC, oldBrush);
    SelectObject(memDC, oldPen);
    DeleteObject(bgBrush);
    DeleteObject(bgPen);
    
    // Draw text
    SetBkMode(memDC, TRANSPARENT);
    SetTextColor(memDC, textColor);
    RECT textRect = {0, 0, width, height};
    DrawTextW(memDC, text.c_str(), (int)text.length(), &textRect, DT_CENTER | DT_SINGLELINE | DT_VCENTER);
    
    // Apply alpha channel
    SetBitmapAlpha(memDC, bitmap, transparentColor, textColor);
    
    // Update layered window
    UpdateLayeredWindow(memDC, width, height);
    
    // Cleanup
    SelectObject(memDC, oldFont);
    SelectObject(memDC, oldBitmap);
    DeleteObject(font);
    DeleteObject(bitmap);
    DeleteDC(memDC);
    ReleaseDC(nullptr, hdcScreen);
}

void KeyMagicHUD::HideHud()
{
    // Hide by moving off-screen
    SetWindowPos(m_hwnd, HWND_TOP, -1000, -1000, 0, 0, SWP_HIDEWINDOW | SWP_NOACTIVATE);
}

void KeyMagicHUD::UpdateLayeredWindow(HDC memDC, int width, int height)
{
    // Get monitor info
    HMONITOR monitor = MonitorFromWindow(m_hwnd, MONITOR_DEFAULTTOPRIMARY);
    MONITORINFO mi = {};
    mi.cbSize = sizeof(MONITORINFO);
    GetMonitorInfo(monitor, &mi);
    
    // Calculate position - bottom-right corner
    const int margin = 20;
    int x = mi.rcWork.right - width - margin;
    int y = mi.rcWork.bottom - height - margin - 50;
    
    SIZE size = {width, height};
    POINT srcPoint = {0, 0};
    POINT dstPoint = {x, y};
    
    BLENDFUNCTION blend = {};
    blend.BlendOp = AC_SRC_OVER;
    blend.BlendFlags = 0;
    blend.SourceConstantAlpha = 255;
    blend.AlphaFormat = AC_SRC_ALPHA;
    
    COLORREF transparentColor = RGB_MACRO(255, 0, 255);
    
    ::UpdateLayeredWindow(
        m_hwnd,
        nullptr,
        &dstPoint,
        &size,
        memDC,
        &srcPoint,
        transparentColor,
        &blend,
        ULW_ALPHA
    );
}

void KeyMagicHUD::SetBitmapAlpha(HDC hdc, HBITMAP bitmap, COLORREF transparentColor, COLORREF textColor)
{
    BITMAP bm;
    GetObject(bitmap, sizeof(BITMAP), &bm);
    
    BITMAPINFOHEADER bih = {};
    bih.biSize = sizeof(BITMAPINFOHEADER);
    bih.biWidth = bm.bmWidth;
    bih.biHeight = bm.bmHeight;
    bih.biPlanes = 1;
    bih.biBitCount = 32;
    bih.biCompression = BI_RGB;
    
    BITMAPINFO bi = {};
    bi.bmiHeader = bih;
    
    // Get pixel data
    int pixelCount = bm.bmWidth * bm.bmHeight;
    std::vector<BYTE> pixelData(pixelCount * 4);
    
    GetDIBits(hdc, bitmap, 0, bm.bmHeight, pixelData.data(), &bi, DIB_RGB_COLORS);
    
    // Process each pixel
    for (int i = 0; i < pixelCount; i++)
    {
        int offset = i * 4;
        BYTE b = pixelData[offset];
        BYTE g = pixelData[offset + 1];
        BYTE r = pixelData[offset + 2];
        COLORREF color = RGB_MACRO(r, g, b);
        
        BYTE alpha;
        if (color == transparentColor)
        {
            alpha = 0;
        }
        else if (color == textColor)
        {
            alpha = 255;
        }
        else
        {
            alpha = (BYTE)(255 * 0.8); // 80% opacity for background
        }
        
        pixelData[offset + 3] = alpha;
        // Premultiply alpha
        pixelData[offset] = (BYTE)(b * alpha / 255);
        pixelData[offset + 1] = (BYTE)(g * alpha / 255);
        pixelData[offset + 2] = (BYTE)(r * alpha / 255);
    }
    
    // Set the modified pixels back
    SetDIBits(hdc, bitmap, 0, bm.bmHeight, pixelData.data(), &bi, DIB_RGB_COLORS);
}