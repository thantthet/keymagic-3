#include "RegistryMonitor.h"
#include "SecurityUtils.h"
#include "../../shared/include/keymagic_ffi.h"
#include "../../shared/include/KeyMagicUtils.h"

RegistryMonitor::RegistryMonitor()
    : m_hKeyboardsKey(nullptr)
    , m_hSettingsKey(nullptr)
    , m_hRegChangeEvent(nullptr)
    , m_hGlobalUpdateEvent(nullptr)
    , m_hStopEvent(nullptr)
    , m_running(false) {
}

RegistryMonitor::~RegistryMonitor() {
    Stop();
}

bool RegistryMonitor::Start(ChangeCallback callback) {
    if (m_running) {
        return false;
    }
    
    m_callback = callback;
    
    // Ensure registry structure exists
    EnsureRegistryStructure();
    
    // Open registry keys
    if (RegOpenKeyExW(HKEY_CURRENT_USER, KEYMAGIC_KEYBOARDS_PATH, 0, 
                      KEY_READ | KEY_NOTIFY, &m_hKeyboardsKey) != ERROR_SUCCESS) {
        return false;
    }
    
    if (RegOpenKeyExW(HKEY_CURRENT_USER, KEYMAGIC_SETTINGS_PATH, 0,
                      KEY_READ | KEY_WRITE, &m_hSettingsKey) != ERROR_SUCCESS) {
        RegCloseKey(m_hKeyboardsKey);
        m_hKeyboardsKey = nullptr;
        return false;
    }
    
    // Create registry change event
    m_hRegChangeEvent = CreateEventW(nullptr, FALSE, FALSE, nullptr);
    if (!m_hRegChangeEvent) {
        RegCloseKey(m_hKeyboardsKey);
        RegCloseKey(m_hSettingsKey);
        m_hKeyboardsKey = nullptr;
        m_hSettingsKey = nullptr;
        return false;
    }
    
    // Create or open global update event
    m_hGlobalUpdateEvent = SecurityUtils::CreateGlobalEvent(KEYMAGIC_REGISTRY_UPDATE_EVENT);
    if (!m_hGlobalUpdateEvent) {
        m_hGlobalUpdateEvent = SecurityUtils::OpenGlobalEvent(KEYMAGIC_REGISTRY_UPDATE_EVENT);
    }
    
    // Create stop event
    m_hStopEvent = CreateEventW(nullptr, TRUE, FALSE, nullptr);
    if (!m_hStopEvent) {
        CloseHandle(m_hRegChangeEvent);
        if (m_hGlobalUpdateEvent) CloseHandle(m_hGlobalUpdateEvent);
        RegCloseKey(m_hKeyboardsKey);
        RegCloseKey(m_hSettingsKey);
        return false;
    }
    
    m_running = true;
    m_monitorThread = std::thread(&RegistryMonitor::MonitorThread, this);
    
    return true;
}

void RegistryMonitor::Stop() {
    if (!m_running) {
        return;
    }
    
    m_running = false;
    
    if (m_hStopEvent) {
        SetEvent(m_hStopEvent);
    }
    
    if (m_monitorThread.joinable()) {
        m_monitorThread.join();
    }
    
    if (m_hRegChangeEvent) {
        CloseHandle(m_hRegChangeEvent);
        m_hRegChangeEvent = nullptr;
    }
    
    if (m_hGlobalUpdateEvent) {
        CloseHandle(m_hGlobalUpdateEvent);
        m_hGlobalUpdateEvent = nullptr;
    }
    
    if (m_hStopEvent) {
        CloseHandle(m_hStopEvent);
        m_hStopEvent = nullptr;
    }
    
    if (m_hKeyboardsKey) {
        RegCloseKey(m_hKeyboardsKey);
        m_hKeyboardsKey = nullptr;
    }
    
    if (m_hSettingsKey) {
        RegCloseKey(m_hSettingsKey);
        m_hSettingsKey = nullptr;
    }
}

std::vector<KeyboardInfo> RegistryMonitor::GetKeyboards() {
    // Use shared utility function
    std::vector<KeyboardInfo> keyboards = RegistryUtils::GetInstalledKeyboards();
    
    // For each keyboard, if hotkey is empty, try to load from KM2 file
    for (auto& keyboard : keyboards) {
        if (keyboard.hotkey.empty() && !keyboard.path.empty()) {
            std::wstring km2Hotkey = KeyMagicUtils::LoadHotkeyFromKm2(keyboard.path);
            if (!km2Hotkey.empty()) {
                keyboard.hotkey = km2Hotkey;
                OutputDebugStringW((L"RegistryMonitor: Got hotkey from KM2 file for keyboard " + keyboard.id + L": " + keyboard.hotkey + L"\n").c_str());
            }
        }
    }
    
    return keyboards;
}

std::wstring RegistryMonitor::GetDefaultKeyboard() {
    // Use shared utility function
    return RegistryUtils::GetDefaultKeyboardId();
}

bool RegistryMonitor::SetDefaultKeyboard(const std::wstring& keyboardId) {
    if (!m_hSettingsKey) {
        return false;
    }
    
    if (RegSetValueExW(m_hSettingsKey, L"DefaultKeyboard", 0, REG_SZ,
                       reinterpret_cast<const BYTE*>(keyboardId.c_str()),
                       static_cast<DWORD>((keyboardId.length() + 1) * sizeof(WCHAR))) != ERROR_SUCCESS) {
        return false;
    }
    
    NotifyTipsOfChange();
    return true;
}

bool RegistryMonitor::GetKeyboardInfo(const std::wstring& keyboardId, KeyboardInfo& info) {
    // Use shared utility function
    if (!RegistryUtils::GetKeyboardInfoById(keyboardId, info)) {
        return false;
    }
    
    // If hotkey is empty, try to load from KM2 file
    if (info.hotkey.empty() && !info.path.empty()) {
        std::wstring km2Hotkey = KeyMagicUtils::LoadHotkeyFromKm2(info.path);
        if (!km2Hotkey.empty()) {
            info.hotkey = km2Hotkey;
            OutputDebugStringW((L"RegistryMonitor: Got hotkey from KM2 file for keyboard " + info.id + L": " + info.hotkey + L"\n").c_str());
        }
    }
    
    return true;
}

void RegistryMonitor::NotifyTipsOfChange() {
    if (m_hGlobalUpdateEvent) {
        SetEvent(m_hGlobalUpdateEvent);
    }
}

void RegistryMonitor::MonitorThread() {
    // Initial notification setup
    RegNotifyChangeKeyValue(m_hKeyboardsKey, TRUE, 
                           REG_NOTIFY_CHANGE_NAME | REG_NOTIFY_CHANGE_LAST_SET,
                           m_hRegChangeEvent, TRUE);
    
    HANDLE waitHandles[] = { m_hStopEvent, m_hRegChangeEvent };
    
    while (m_running) {
        DWORD waitResult = WaitForMultipleObjects(2, waitHandles, FALSE, INFINITE);
        
        if (waitResult == WAIT_OBJECT_0) {
            // Stop event signaled
            break;
        } else if (waitResult == WAIT_OBJECT_0 + 1) {
            // Registry changed
            if (m_callback) {
                m_callback();
            }
            
            // Re-register for notifications
            RegNotifyChangeKeyValue(m_hKeyboardsKey, TRUE,
                                   REG_NOTIFY_CHANGE_NAME | REG_NOTIFY_CHANGE_LAST_SET,
                                   m_hRegChangeEvent, TRUE);
        }
    }
}

void RegistryMonitor::EnsureRegistryStructure() {
    // Use shared utility function
    RegistryUtils::EnsureRegistryStructure();
}