#pragma once

#include <windows.h>
#include <shlobj.h>
#include <string>
#include <vector>
#include <memory>
#include "KeyMagicConstants.h"
#include "KeyboardInfo.h"

namespace RegistryUtils {

// Helper function to read a string value from registry
inline bool ReadRegistryString(HKEY hKey, const std::wstring& valueName, std::wstring& value) {
    WCHAR buffer[512];
    DWORD bufferSize = sizeof(buffer);
    DWORD type;
    
    if (RegQueryValueExW(hKey, valueName.c_str(), nullptr, &type,
                         reinterpret_cast<LPBYTE>(buffer), &bufferSize) == ERROR_SUCCESS) {
        if (type == REG_SZ) {
            value = buffer;
            return true;
        }
    }
    return false;
}

// Helper function to write a string value to registry
inline bool WriteRegistryString(HKEY hKey, const std::wstring& valueName, const std::wstring& value) {
    return RegSetValueExW(hKey, valueName.c_str(), 0, REG_SZ,
                          reinterpret_cast<const BYTE*>(value.c_str()),
                          static_cast<DWORD>((value.length() + 1) * sizeof(WCHAR))) == ERROR_SUCCESS;
}

// Helper function to read keyboard info from a registry key
inline bool ReadKeyboardInfo(HKEY hKeyboardsKey, const std::wstring& keyboardId, KeyboardInfo& info) {
    HKEY hSubKey;
    if (RegOpenKeyExW(hKeyboardsKey, keyboardId.c_str(), 0, KEY_READ, &hSubKey) != ERROR_SUCCESS) {
        return false;
    }
    
    info.id = keyboardId;
    
    // Read keyboard properties
    ReadRegistryString(hSubKey, L"Name", info.name);
    
    // Handle path - try FileName first, then fall back to Path
    std::wstring filename;
    if (ReadRegistryString(hSubKey, L"FileName", filename) && !filename.empty()) {
        // Get keyboards directory
        std::wstring keyboardsDir;
        HKEY hSettingsKey;
        if (RegOpenKeyExW(HKEY_CURRENT_USER, KEYMAGIC_SETTINGS_PATH, 0, KEY_READ, &hSettingsKey) == ERROR_SUCCESS) {
            ReadRegistryString(hSettingsKey, L"KeyboardsPath", keyboardsDir);
            RegCloseKey(hSettingsKey);
        }
        
        // Use default if not found
        if (keyboardsDir.empty()) {
            wchar_t localAppData[MAX_PATH];
            if (SUCCEEDED(SHGetFolderPathW(NULL, CSIDL_LOCAL_APPDATA, NULL, 0, localAppData))) {
                keyboardsDir = std::wstring(localAppData) + L"\\KeyMagic\\Keyboards";
            }
        }
        
        if (!keyboardsDir.empty()) {
            info.path = keyboardsDir + L"\\" + filename;
        }
    } else {
        // Fall back to old Path value
        ReadRegistryString(hSubKey, L"Path", info.path);
    }
    
    // Read hotkey - note: the special logic for reading from KM2 file
    // should be handled by the caller if needed
    ReadRegistryString(hSubKey, L"Hotkey", info.hotkey);
    
    // Read enabled state (default to true if not present)
    DWORD enabled = 1;
    DWORD dataSize = sizeof(enabled);
    DWORD type;
    if (RegQueryValueExW(hSubKey, L"Enabled", nullptr, &type,
                         reinterpret_cast<LPBYTE>(&enabled), &dataSize) == ERROR_SUCCESS) {
        if (type == REG_DWORD) {
            info.enabled = (enabled != 0);
        }
    }
    
    RegCloseKey(hSubKey);
    return true;
}

// Get keyboard info by ID from HKEY_CURRENT_USER
inline bool GetKeyboardInfoById(const std::wstring& keyboardId, KeyboardInfo& info) {
    HKEY hKeyboardsKey;
    if (RegOpenKeyExW(HKEY_CURRENT_USER, KEYMAGIC_KEYBOARDS_PATH, 0, KEY_READ, &hKeyboardsKey) != ERROR_SUCCESS) {
        return false;
    }
    
    bool result = ReadKeyboardInfo(hKeyboardsKey, keyboardId, info);
    RegCloseKey(hKeyboardsKey);
    return result;
}

// Get list of all installed keyboards
inline std::vector<KeyboardInfo> GetInstalledKeyboards() {
    std::vector<KeyboardInfo> keyboards;
    HKEY hKeyboardsKey;
    
    if (RegOpenKeyExW(HKEY_CURRENT_USER, KEYMAGIC_KEYBOARDS_PATH, 0, KEY_READ, &hKeyboardsKey) != ERROR_SUCCESS) {
        return keyboards;
    }
    
    WCHAR keyName[256];
    DWORD keyNameSize;
    DWORD index = 0;
    
    while (true) {
        keyNameSize = 256;
        if (RegEnumKeyExW(hKeyboardsKey, index, keyName, &keyNameSize, 
                          nullptr, nullptr, nullptr, nullptr) != ERROR_SUCCESS) {
            break;
        }
        
        KeyboardInfo info;
        if (ReadKeyboardInfo(hKeyboardsKey, keyName, info)) {
            keyboards.push_back(info);
        }
        
        index++;
    }
    
    RegCloseKey(hKeyboardsKey);
    return keyboards;
}

// Get current default keyboard ID
inline std::wstring GetDefaultKeyboardId() {
    HKEY hSettingsKey;
    if (RegOpenKeyExW(HKEY_CURRENT_USER, KEYMAGIC_SETTINGS_PATH, 0, KEY_READ, &hSettingsKey) != ERROR_SUCCESS) {
        return L"";
    }
    
    std::wstring defaultKeyboard;
    ReadRegistryString(hSettingsKey, L"DefaultKeyboard", defaultKeyboard);
    
    RegCloseKey(hSettingsKey);
    return defaultKeyboard;
}

// Set default keyboard ID
inline bool SetDefaultKeyboardId(const std::wstring& keyboardId) {
    HKEY hSettingsKey;
    if (RegOpenKeyExW(HKEY_CURRENT_USER, KEYMAGIC_SETTINGS_PATH, 0, KEY_WRITE, &hSettingsKey) != ERROR_SUCCESS) {
        return false;
    }
    
    bool result = WriteRegistryString(hSettingsKey, L"DefaultKeyboard", keyboardId);
    RegCloseKey(hSettingsKey);
    return result;
}

// Ensure KeyMagic registry structure exists
inline void EnsureRegistryStructure() {
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

// Read enabled languages from registry (returns language codes like "en", "my")
inline std::vector<std::wstring> GetEnabledLanguages() {
    std::vector<std::wstring> languages;
    HKEY hSettingsKey;
    
    if (RegOpenKeyExW(HKEY_CURRENT_USER, KEYMAGIC_SETTINGS_PATH, 0, KEY_READ, &hSettingsKey) != ERROR_SUCCESS) {
        return languages;
    }
    
    DWORD dwType = REG_MULTI_SZ;
    DWORD dwSize = 0;
    
    // Get size first
    if (RegQueryValueExW(hSettingsKey, L"EnabledLanguages", nullptr, &dwType, nullptr, &dwSize) == ERROR_SUCCESS && dwSize > 0) {
        // Allocate buffer and read data
        std::vector<WCHAR> buffer(dwSize / sizeof(WCHAR));
        if (RegQueryValueExW(hSettingsKey, L"EnabledLanguages", nullptr, &dwType, 
                           reinterpret_cast<LPBYTE>(buffer.data()), &dwSize) == ERROR_SUCCESS) {
            // Parse multi-string data
            LPCWSTR pszCurrent = buffer.data();
            while (*pszCurrent) {
                languages.push_back(pszCurrent);
                pszCurrent += wcslen(pszCurrent) + 1;
            }
        }
    }
    
    RegCloseKey(hSettingsKey);
    return languages;
}

} // namespace RegistryUtils