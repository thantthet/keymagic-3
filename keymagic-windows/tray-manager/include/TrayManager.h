#pragma once

#include "Common.h"
#include "NamedPipeServer.h"
#include "RegistryMonitor.h"
#include "IconCacheManager.h"
#include "ProfileMonitor.h"
#include "TrayIcon.h"

class TrayManager {
public:
    TrayManager();
    ~TrayManager();
    
    // Initialize and run the tray manager
    int Run();

private:
    // Window procedure
    static LRESULT CALLBACK WindowProc(HWND hWnd, UINT message, WPARAM wParam, LPARAM lParam);
    
    // Instance window procedure
    LRESULT HandleMessage(HWND hWnd, UINT message, WPARAM wParam, LPARAM lParam);
    
    // Initialize components
    bool Initialize();
    void Cleanup();
    
    // Create message window
    bool CreateMessageWindow();
    
    // Handle named pipe messages
    void OnPipeMessage(const TrayMessage& msg);
    
    // Handle registry changes
    void OnRegistryChange();
    
    // Handle tray menu selection
    void OnMenuCommand(UINT cmdId);
    
    // Update tray icon based on current state
    void UpdateTrayIcon();
    
    // Check if any TIP is active
    bool IsAnyTipActive() const;

private:
    // Core components
    std::unique_ptr<NamedPipeServer> m_pipeServer;
    std::unique_ptr<RegistryMonitor> m_registryMonitor;
    std::unique_ptr<IconCacheManager> m_iconCache;
    std::unique_ptr<ProfileMonitor> m_profileMonitor;
    std::unique_ptr<TrayIcon> m_trayIcon;
    
    // State
    std::set<DWORD> m_activeTipProcesses;
    std::wstring m_currentKeyboardId;
    bool m_hasFocus;
    std::mutex m_stateMutex;
    
    // Window handle
    HWND m_hWnd;
    
    // Singleton instance
    static TrayManager* s_instance;
};