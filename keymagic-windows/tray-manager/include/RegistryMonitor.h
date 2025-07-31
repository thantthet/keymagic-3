#pragma once

#include "Common.h"
#include "../../shared/include/KeyboardInfo.h"
#include "../../shared/include/RegistryUtils.h"

class RegistryMonitor {
public:
    using ChangeCallback = std::function<void()>;
    
    RegistryMonitor();
    ~RegistryMonitor();
    
    // Start monitoring registry changes
    bool Start(ChangeCallback callback);
    
    // Stop monitoring
    void Stop();
    
    // Get list of installed keyboards
    std::vector<KeyboardInfo> GetKeyboards();
    
    // Get current default keyboard
    std::wstring GetDefaultKeyboard();
    
    // Set default keyboard
    bool SetDefaultKeyboard(const std::wstring& keyboardId);
    
    // Get keyboard info by ID
    bool GetKeyboardInfo(const std::wstring& keyboardId, KeyboardInfo& info);
    
    // Notify all TIPs of registry change
    void NotifyTipsOfChange();

private:
    // Monitor thread
    void MonitorThread();
    
    // Ensure registry structure exists
    void EnsureRegistryStructure();

private:
    HKEY m_hKeyboardsKey;
    HKEY m_hSettingsKey;
    HANDLE m_hRegChangeEvent;
    HANDLE m_hGlobalUpdateEvent;
    HANDLE m_hStopEvent;
    std::thread m_monitorThread;
    std::atomic<bool> m_running;
    ChangeCallback m_callback;
};