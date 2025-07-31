#pragma once

#include <windows.h>
#include <string>
#include <mutex>
#include "../../shared/include/TrayProtocol.h"

// TrayClient is used by the TIP to communicate with the tray manager
class TrayClient {
public:
    TrayClient();
    ~TrayClient();
    
    // Connect to tray manager
    bool Connect();
    
    // Disconnect from tray manager
    void Disconnect();
    
    // Send message to tray
    bool SendMessage(const TrayMessage& msg);
    
    // Notify focus gained
    void NotifyFocusGained(const std::wstring& keyboardId);
    
    // Notify focus lost
    void NotifyFocusLost();
    
    // Notify keyboard changed
    void NotifyKeyboardChanged(const std::wstring& keyboardId);
    
    // Notify TIP started
    void NotifyTipStarted();
    
    // Notify TIP stopped
    void NotifyTipStopped();
    
    // Check if connected
    bool IsConnected() const { return m_connected; }

private:
    // Attempt to connect with retry
    bool TryConnect();
    
    // Get current process name
    std::wstring GetProcessName();

private:
    HANDLE m_hPipe;
    bool m_connected;
    DWORD m_reconnectDelay;
    std::wstring m_pipeName;
    std::mutex m_mutex;
};