#include "TrayIcon.h"
#include "RegistryMonitor.h"
#include "IconVisibilityManager.h"
#include <strsafe.h>

TrayIcon::TrayIcon()
    : m_hWnd(nullptr)
    , m_hIcon(nullptr)
    , m_hDefaultIcon(nullptr)
    , m_visible(false) {
    ZeroMemory(&m_nid, sizeof(m_nid));
}

TrayIcon::~TrayIcon() {
    Hide();
    
    if (m_hDefaultIcon) {
        DestroyIcon(m_hDefaultIcon);
        m_hDefaultIcon = nullptr;
    }
}

bool TrayIcon::Initialize(HWND hWnd) {
    m_hWnd = hWnd;
    
    // Create default icon
    m_hDefaultIcon = CreateDefaultIcon();
    if (!m_hDefaultIcon) {
        // Load from application resources if available
        m_hDefaultIcon = LoadIcon(GetModuleHandle(nullptr), MAKEINTRESOURCE(1));
        if (!m_hDefaultIcon) {
            // Use system icon as last resort
            m_hDefaultIcon = LoadIcon(nullptr, IDI_APPLICATION);
        }
    }
    
    // Setup notification icon data
    m_nid.cbSize = sizeof(NOTIFYICONDATAW);
    m_nid.hWnd = m_hWnd;
    m_nid.uID = TRAY_ICON_ID;
    m_nid.uFlags = NIF_MESSAGE | NIF_ICON | NIF_TIP | NIF_SHOWTIP;
    m_nid.uCallbackMessage = WM_TRAYICON;
    m_nid.hIcon = m_hDefaultIcon;
    StringCchCopyW(m_nid.szTip, ARRAYSIZE(m_nid.szTip), L"KeyMagic");
    
    // Set version for Windows Vista+ features
    m_nid.uVersion = NOTIFYICON_VERSION_4;
    
    return true;
}

void TrayIcon::Show() {
    if (!m_visible) {
        m_visible = UpdateNotificationIcon(NIM_ADD);
        if (m_visible) {
            Shell_NotifyIconW(NIM_SETVERSION, &m_nid);
            
            // Try to ensure the icon is promoted to always visible
            // This creates a registry entry that Windows uses to remember icon visibility
            EnsureIconVisibility();
        }
    }
}

void TrayIcon::Hide() {
    if (m_visible) {
        UpdateNotificationIcon(NIM_DELETE);
        m_visible = false;
    }
}

void TrayIcon::SetIcon(HICON hIcon) {
    m_hIcon = hIcon ? hIcon : m_hDefaultIcon;
    m_nid.hIcon = m_hIcon;
    
    if (m_visible) {
        UpdateNotificationIcon(NIM_MODIFY);
    }
}

void TrayIcon::SetTooltip(const std::wstring& tooltip) {
    StringCchCopyW(m_nid.szTip, ARRAYSIZE(m_nid.szTip), tooltip.c_str());
    
    if (m_visible) {
        UpdateNotificationIcon(NIM_MODIFY);
    }
}

void TrayIcon::ShowContextMenu(HWND hWnd, const std::vector<KeyboardInfo>& keyboards,
                               const std::wstring& currentKeyboardId, MenuCallback callback) {
    m_menuCallback = callback;
    
    // Notify that menu is about to be shown (use SendMessage for immediate handling)
    SendMessage(hWnd, WM_MENU_SHOWN, 0, 0);
    
    // Create popup menu
    HMENU hMenu = CreatePopupMenu();
    if (!hMenu) {
        SendMessage(hWnd, WM_MENU_DISMISSED, 0, 0);
        return;
    }
    
    // Add keyboard items
    UINT menuId = IDM_KEYBOARD_BASE;
    for (const auto& keyboard : keyboards) {
        UINT flags = MF_STRING;
        if (keyboard.id == currentKeyboardId) {
            flags |= MF_CHECKED;
        }
        
        std::wstring menuText = keyboard.name;
        if (!keyboard.hotkey.empty()) {
            menuText += L"\t" + keyboard.hotkey;
        }
        
        AppendMenuW(hMenu, flags, menuId++, menuText.c_str());
    }
    
    // Add separator
    if (!keyboards.empty()) {
        AppendMenuW(hMenu, MF_SEPARATOR, 0, nullptr);
    }
    
    // Add standard items
    AppendMenuW(hMenu, MF_STRING, IDM_SETTINGS, L"Settings...");
    AppendMenuW(hMenu, MF_SEPARATOR, 0, nullptr);
    AppendMenuW(hMenu, MF_STRING, IDM_EXIT, L"Exit");
    
    // Get cursor position
    POINT pt;
    GetCursorPos(&pt);
    
    // Required to make menu disappear when clicking outside
    SetForegroundWindow(hWnd);
    
    // Show menu
    UINT cmd = TrackPopupMenuEx(hMenu, 
                                TPM_LEFTALIGN | TPM_BOTTOMALIGN | TPM_RETURNCMD | TPM_NONOTIFY,
                                pt.x, pt.y, hWnd, nullptr);
    
    // Clean up
    DestroyMenu(hMenu);
    
    // Handle command
    if (cmd != 0 && m_menuCallback) {
        m_menuCallback(cmd);
    }
    
    // Notify that menu has been dismissed (use SendMessage for immediate handling)
    SendMessage(hWnd, WM_MENU_DISMISSED, 0, 0);
}

void TrayIcon::HandleMessage(HWND hWnd, UINT message, WPARAM wParam, LPARAM lParam) {
    if (message != WM_TRAYICON) {
        return;
    }
    
    switch (LOWORD(lParam)) {
        case WM_LBUTTONUP:
            // Could show keyboard switcher UI here
            break;
            
        case WM_RBUTTONUP:
            // Context menu will be shown by TrayManager
            PostMessage(hWnd, WM_COMMAND, MAKEWPARAM(0, 0), 0);
            break;
            
        case WM_CONTEXTMENU:
            // Already handled by WM_RBUTTONUP
            break;
    }
}

HICON TrayIcon::CreateDefaultIcon() {
    // Create a simple 16x16 icon programmatically
    HDC hDC = GetDC(nullptr);
    HDC hMemDC = CreateCompatibleDC(hDC);
    
    BITMAPINFO bmi = {};
    bmi.bmiHeader.biSize = sizeof(BITMAPINFOHEADER);
    bmi.bmiHeader.biWidth = 16;
    bmi.bmiHeader.biHeight = 16;
    bmi.bmiHeader.biPlanes = 1;
    bmi.bmiHeader.biBitCount = 32;
    bmi.bmiHeader.biCompression = BI_RGB;
    
    void* pBits;
    HBITMAP hBitmap = CreateDIBSection(hMemDC, &bmi, DIB_RGB_COLORS, &pBits, nullptr, 0);
    if (!hBitmap) {
        DeleteDC(hMemDC);
        ReleaseDC(nullptr, hDC);
        return nullptr;
    }
    
    // Draw a simple "K" icon
    HBITMAP hOldBitmap = (HBITMAP)SelectObject(hMemDC, hBitmap);
    
    // Fill with transparent background
    RECT rc = {0, 0, 16, 16};
    HBRUSH hBrush = CreateSolidBrush(RGB(255, 255, 255));
    FillRect(hMemDC, &rc, hBrush);
    DeleteObject(hBrush);
    
    // Draw "K" letter
    SetBkMode(hMemDC, TRANSPARENT);
    SetTextColor(hMemDC, RGB(0, 0, 128));
    
    HFONT hFont = CreateFont(14, 0, 0, 0, FW_BOLD, FALSE, FALSE, FALSE,
                            DEFAULT_CHARSET, OUT_DEFAULT_PRECIS, CLIP_DEFAULT_PRECIS,
                            DEFAULT_QUALITY, DEFAULT_PITCH | FF_SWISS, L"Arial");
    HFONT hOldFont = (HFONT)SelectObject(hMemDC, hFont);
    
    DrawTextW(hMemDC, L"K", 1, &rc, DT_CENTER | DT_VCENTER | DT_SINGLELINE);
    
    SelectObject(hMemDC, hOldFont);
    DeleteObject(hFont);
    SelectObject(hMemDC, hOldBitmap);
    
    // Create icon
    ICONINFO iconInfo = {};
    iconInfo.fIcon = TRUE;
    iconInfo.hbmMask = hBitmap;
    iconInfo.hbmColor = hBitmap;
    
    HICON hIcon = CreateIconIndirect(&iconInfo);
    
    // Clean up
    DeleteObject(hBitmap);
    DeleteDC(hMemDC);
    ReleaseDC(nullptr, hDC);
    
    return hIcon;
}

bool TrayIcon::UpdateNotificationIcon(DWORD dwMessage) {
    return Shell_NotifyIconW(dwMessage, &m_nid) == TRUE;
}

void TrayIcon::EnsureIconVisibility() {
    // Use IconVisibilityManager to ensure the icon is visible
    IconVisibilityManager visibilityManager;
    visibilityManager.EnsureIconVisible();
}