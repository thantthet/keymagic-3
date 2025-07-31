#pragma once

#include "Common.h"
#include <shellapi.h>
#include <memory>

// Forward declarations
struct KeyboardInfo;
class KeyboardPreviewWindow;

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
    
    // Set keyboard info for preview
    void SetKeyboardInfo(const std::wstring& keyboardId, const std::wstring& keyboardPath);

private:
    // Create default icon
    HICON CreateDefaultIcon();
    
    // Add or modify notification icon
    bool UpdateNotificationIcon(DWORD dwMessage);
    
    // Try to ensure icon is always visible
    void EnsureIconVisibility();
    
    // Update tooltip visibility based on preview window setting
    void UpdateTooltipVisibility();

private:
    HWND m_hWnd;
    HICON m_hIcon;
    HICON m_hDefaultIcon;
    NOTIFYICONDATAW m_nid;
    bool m_visible;
    MenuCallback m_menuCallback;
    
    // Keyboard preview window
    std::unique_ptr<KeyboardPreviewWindow> m_previewWindow;
    std::wstring m_currentKeyboardId;
    std::wstring m_currentKeyboardPath;
    bool m_isMenuShowing;
    
    // Menu constants
    static constexpr UINT IDM_KEYBOARD_BASE = 1000;
    static constexpr UINT IDM_EXIT = 999;
    static constexpr UINT IDM_ABOUT = 998;
    static constexpr UINT IDM_SETTINGS = 997;
};