#include "RegistryMonitor.h"
#include "SecurityUtils.h"
#include "../../shared/include/keymagic_ffi.h"

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
    return RegistryUtils::GetInstalledKeyboards();
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
    return RegistryUtils::GetKeyboardInfoById(keyboardId, info);
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

bool RegistryMonitor::ReadKeyboardInfo(HKEY hKey, const std::wstring& keyboardId, KeyboardInfo& info) {
    HKEY hSubKey;
    if (RegOpenKeyExW(hKey, keyboardId.c_str(), 0, KEY_READ, &hSubKey) != ERROR_SUCCESS) {
        return false;
    }
    
    auto readString = [&](const WCHAR* valueName, std::wstring& value) {
        WCHAR buffer[1024];
        DWORD bufferSize = sizeof(buffer);
        DWORD type;
        
        if (RegQueryValueExW(hSubKey, valueName, nullptr, &type,
                            reinterpret_cast<LPBYTE>(buffer), &bufferSize) == ERROR_SUCCESS) {
            if (type == REG_SZ) {
                value = buffer;
                return true;
            }
        }
        return false;
    };
    
    readString(L"Name", info.name);
    
    // Try to read the new FileName value first, fall back to Path for backward compatibility
    std::wstring filename;
    if (readString(L"FileName", filename) && !filename.empty()) {
        // Construct full path from filename
        // Get keyboards directory from settings or use default
        std::wstring keyboardsDir = RegistryUtils::GetKeyboardsPath();
        
        // Construct full path
        info.path = keyboardsDir + L"\\" + filename;
        OutputDebugStringW((L"RegistryMonitor: Using keyboard filename: " + filename + L" with full path: " + info.path + L"\n").c_str());
    } else {
        // Fall back to old Path value for backward compatibility
        if (!readString(L"Path", info.path)) {
            info.path.clear();
        } else {
            OutputDebugStringW((L"RegistryMonitor: Using legacy Path value: " + info.path + L"\n").c_str());
        }
    }
    
    // Read hotkey with TSF-style logic
    DWORD queryResult = ERROR_SUCCESS;
    WCHAR hotkeyBuffer[256] = {0};
    DWORD hotkeySize = sizeof(hotkeyBuffer);
    DWORD hotkeyType;
    
    queryResult = RegQueryValueExW(hSubKey, L"Hotkey", nullptr, &hotkeyType,
                                  reinterpret_cast<LPBYTE>(hotkeyBuffer), &hotkeySize);
    
    if (queryResult == ERROR_SUCCESS && hotkeyType == REG_SZ && hotkeyBuffer[0] != L'\0') {
        // Hotkey explicitly set in config (non-empty string)
        info.hotkey = hotkeyBuffer;
        OutputDebugStringW((L"RegistryMonitor: Using hotkey from registry for keyboard " + keyboardId + L": " + info.hotkey + L"\n").c_str());
    } else if (queryResult == ERROR_FILE_NOT_FOUND && !info.path.empty()) {
        // Hotkey not set in config - try to get from KM2 file
        OutputDebugStringW((L"RegistryMonitor: No registry hotkey for keyboard " + keyboardId + L", checking KM2 file\n").c_str());
        
        // Convert wide string path to UTF-8
        int utf8Len = WideCharToMultiByte(CP_UTF8, 0, info.path.c_str(), -1, nullptr, 0, nullptr, nullptr);
        if (utf8Len > 0) {
            std::vector<char> utf8Path(utf8Len);
            WideCharToMultiByte(CP_UTF8, 0, info.path.c_str(), -1, utf8Path.data(), utf8Len, nullptr, nullptr);
            
            // Load KM2 file to get hotkey
            Km2FileHandle* km2Handle = keymagic_km2_load(utf8Path.data());
            if (km2Handle) {
                char* hotkeyStr = keymagic_km2_get_hotkey(km2Handle);
                if (hotkeyStr && hotkeyStr[0] != '\0') {
                    // Convert UTF-8 hotkey to wide string
                    int wideLen = MultiByteToWideChar(CP_UTF8, 0, hotkeyStr, -1, nullptr, 0);
                    if (wideLen > 0) {
                        std::vector<WCHAR> wideHotkey(wideLen);
                        MultiByteToWideChar(CP_UTF8, 0, hotkeyStr, -1, wideHotkey.data(), wideLen);
                        info.hotkey = wideHotkey.data();
                        OutputDebugStringW((L"RegistryMonitor: Got hotkey from KM2 file for keyboard " + keyboardId + L": " + info.hotkey + L"\n").c_str());
                    }
                }
                if (hotkeyStr) keymagic_free_string(hotkeyStr);
                keymagic_km2_free(km2Handle);
            } else {
                OutputDebugStringW((L"RegistryMonitor: Failed to load KM2 file: " + info.path + L"\n").c_str());
            }
        }
    } else if (queryResult == ERROR_SUCCESS && hotkeyType == REG_SZ && hotkeyBuffer[0] == L'\0') {
        // Empty string means user explicitly disabled hotkey
        info.hotkey.clear();
        OutputDebugStringW((L"RegistryMonitor: Hotkey explicitly disabled for keyboard " + keyboardId + L"\n").c_str());
    }
    
    RegCloseKey(hSubKey);
    return true;
}

void RegistryMonitor::EnsureRegistryStructure() {
    // Use shared utility function
    RegistryUtils::EnsureRegistryStructure();
}