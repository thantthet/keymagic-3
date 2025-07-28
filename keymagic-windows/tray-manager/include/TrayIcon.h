#pragma once

#include "Common.h"
#include <shellapi.h>

// Forward declaration
struct KeyboardInfo;

class TrayIcon {
public:
    using MenuCallback = std::function<void(UINT)>;
    
    TrayIcon();
    ~TrayIcon();
    
    // Initialize tray icon
    bool Initialize(HWND hWnd);
    
    // Show/hide tray icon
    void Show();
    void Hide();
    
    // Update icon
    void SetIcon(HICON hIcon);
    
    // Update tooltip
    void SetTooltip(const std::wstring& tooltip);
    
    // Show context menu
    void ShowContextMenu(HWND hWnd, const std::vector<KeyboardInfo>& keyboards, 
                        const std::wstring& currentKeyboardId, MenuCallback callback);
    
    // Handle tray icon messages
    void HandleMessage(HWND hWnd, UINT message, WPARAM wParam, LPARAM lParam);
    
    // Check if icon is visible
    bool IsVisible() const { return m_visible; }

private:
    // Create default icon
    HICON CreateDefaultIcon();
    
    // Add or modify notification icon
    bool UpdateNotificationIcon(DWORD dwMessage);
    
    // Try to ensure icon is always visible
    void EnsureIconVisibility();

private:
    HWND m_hWnd;
    HICON m_hIcon;
    HICON m_hDefaultIcon;
    NOTIFYICONDATAW m_nid;
    bool m_visible;
    MenuCallback m_menuCallback;
    
    // Menu constants
    static constexpr UINT IDM_KEYBOARD_BASE = 1000;
    static constexpr UINT IDM_EXIT = 999;
    static constexpr UINT IDM_ABOUT = 998;
    static constexpr UINT IDM_SETTINGS = 997;
};