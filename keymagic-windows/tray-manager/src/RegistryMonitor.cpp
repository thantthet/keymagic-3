#include "RegistryMonitor.h"
#include "SecurityUtils.h"

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
    std::vector<KeyboardInfo> keyboards;
    
    if (!m_hKeyboardsKey) {
        return keyboards;
    }
    
    DWORD index = 0;
    WCHAR keyName[256];
    DWORD keyNameSize = 256;
    
    while (RegEnumKeyExW(m_hKeyboardsKey, index, keyName, &keyNameSize,
                         nullptr, nullptr, nullptr, nullptr) == ERROR_SUCCESS) {
        KeyboardInfo info;
        info.id = keyName;
        
        if (ReadKeyboardInfo(m_hKeyboardsKey, keyName, info)) {
            keyboards.push_back(info);
        }
        
        keyNameSize = 256;
        index++;
    }
    
    return keyboards;
}

std::wstring RegistryMonitor::GetDefaultKeyboard() {
    if (!m_hSettingsKey) {
        return L"";
    }
    
    WCHAR buffer[256];
    DWORD bufferSize = sizeof(buffer);
    DWORD type;
    
    if (RegQueryValueExW(m_hSettingsKey, L"DefaultKeyboard", nullptr, &type,
                         reinterpret_cast<LPBYTE>(buffer), &bufferSize) == ERROR_SUCCESS) {
        if (type == REG_SZ) {
            return std::wstring(buffer);
        }
    }
    
    return L"";
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
    if (!m_hKeyboardsKey) {
        return false;
    }
    
    info.id = keyboardId;
    return ReadKeyboardInfo(m_hKeyboardsKey, keyboardId, info);
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
        std::wstring keyboardsDir;
        HKEY hSettingsKey;
        if (RegOpenKeyExW(HKEY_CURRENT_USER, KEYMAGIC_SETTINGS_PATH, 0, KEY_READ, &hSettingsKey) == ERROR_SUCCESS) {
            wchar_t dirPath[MAX_PATH] = {0};
            DWORD dirSize = sizeof(dirPath);
            if (RegQueryValueExW(hSettingsKey, L"KeyboardsPath", nullptr, nullptr, 
                               reinterpret_cast<LPBYTE>(dirPath), &dirSize) == ERROR_SUCCESS && dirPath[0] != L'\0') {
                keyboardsDir = dirPath;
            }
            RegCloseKey(hSettingsKey);
        }
        
        // Use default if not found in registry
        if (keyboardsDir.empty()) {
            keyboardsDir = GetLocalAppDataPath() + L"\\Keyboards";
        }
        
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
    
    readString(L"Hotkey", info.hotkey);
    readString(L"Description", info.description);
    readString(L"FontFamily", info.fontFamily);
    
    RegCloseKey(hSubKey);
    return true;
}

void RegistryMonitor::EnsureRegistryStructure() {
    HKEY hKey;
    
    // Create KeyMagic key
    if (RegCreateKeyExW(HKEY_CURRENT_USER, KEYMAGIC_REGISTRY_PATH, 0, nullptr,
                        REG_OPTION_NON_VOLATILE, KEY_WRITE, nullptr, &hKey, nullptr) == ERROR_SUCCESS) {
        RegCloseKey(hKey);
    }
    
    // Create Keyboards subkey
    if (RegCreateKeyExW(HKEY_CURRENT_USER, KEYMAGIC_KEYBOARDS_PATH, 0, nullptr,
                        REG_OPTION_NON_VOLATILE, KEY_WRITE, nullptr, &hKey, nullptr) == ERROR_SUCCESS) {
        RegCloseKey(hKey);
    }
    
    // Create Settings subkey
    if (RegCreateKeyExW(HKEY_CURRENT_USER, KEYMAGIC_SETTINGS_PATH, 0, nullptr,
                        REG_OPTION_NON_VOLATILE, KEY_WRITE, nullptr, &hKey, nullptr) == ERROR_SUCCESS) {
        RegCloseKey(hKey);
    }
}