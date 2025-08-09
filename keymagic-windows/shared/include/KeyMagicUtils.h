#pragma once

#include <windows.h>
#include <shlobj.h>
#include <sddl.h>
#include <string>
#include <memory>
#include <vector>
#include "keymagic_ffi.h"

namespace KeyMagicUtils {

// Get the current user's SID as a string
inline std::wstring GetUserSID() {
    HANDLE hToken;
    if (!OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &hToken)) {
        return L"";
    }

    DWORD dwBufferSize = 0;
    GetTokenInformation(hToken, TokenUser, nullptr, 0, &dwBufferSize);
    
    auto buffer = std::make_unique<BYTE[]>(dwBufferSize);
    if (!GetTokenInformation(hToken, TokenUser, buffer.get(), dwBufferSize, &dwBufferSize)) {
        CloseHandle(hToken);
        return L"";
    }

    TOKEN_USER* pTokenUser = reinterpret_cast<TOKEN_USER*>(buffer.get());
    
    LPWSTR stringSid = nullptr;
    if (!ConvertSidToStringSidW(pTokenUser->User.Sid, &stringSid)) {
        CloseHandle(hToken);
        return L"";
    }

    std::wstring result(stringSid);
    LocalFree(stringSid);
    CloseHandle(hToken);
    
    return result;
}

// Get the named pipe name for the current user
inline std::wstring GetPipeName() {
    return L"\\\\.\\pipe\\KeyMagicTray-" + GetUserSID();
}

// Get the local app data path for KeyMagic
inline std::wstring GetLocalAppDataPath() {
    wchar_t path[MAX_PATH];
    if (SUCCEEDED(SHGetFolderPathW(nullptr, CSIDL_LOCAL_APPDATA, nullptr, 0, path))) {
        return std::wstring(path) + L"\\KeyMagic";
    }
    return L"";
}

// Get the icon cache path
inline std::wstring GetIconCachePath() {
    return GetLocalAppDataPath() + L"\\IconCache";
}

// Convert UTF-8 string to UTF-16 string
inline std::wstring ConvertUtf8ToUtf16(const std::string& utf8)
{
    if (utf8.empty())
        return std::wstring();
        
    int size_needed = MultiByteToWideChar(CP_UTF8, 0, utf8.c_str(), static_cast<int>(utf8.length()), NULL, 0);
    std::wstring utf16(size_needed, 0);
    MultiByteToWideChar(CP_UTF8, 0, utf8.c_str(), static_cast<int>(utf8.length()), &utf16[0], size_needed);
    return utf16;
}

// Convert UTF-16 string to UTF-8 string
inline std::string ConvertUtf16ToUtf8(const std::wstring& utf16)
{
    if (utf16.empty())
        return std::string();
        
    int size_needed = WideCharToMultiByte(CP_UTF8, 0, utf16.c_str(), static_cast<int>(utf16.length()), NULL, 0, NULL, NULL);
    std::string utf8(size_needed, 0);
    WideCharToMultiByte(CP_UTF8, 0, utf16.c_str(), static_cast<int>(utf16.length()), &utf8[0], size_needed, NULL, NULL);
    return utf8;
}

// Load hotkey from KM2 file
// Returns empty string if no hotkey is defined or on error
inline std::wstring LoadHotkeyFromKm2(const std::wstring& km2Path)
{
    if (km2Path.empty())
        return L"";
    
    // Convert path to UTF-8
    std::string utf8Path = ConvertUtf16ToUtf8(km2Path);
    if (utf8Path.empty())
        return L"";
    
    // Load KM2 file
    Km2FileHandle* km2Handle = keymagic_km2_load(utf8Path.c_str());
    if (!km2Handle)
        return L"";
    
    std::wstring result;
    
    // Get hotkey from KM2 file
    char* hotkeyStr = keymagic_km2_get_hotkey(km2Handle);
    if (hotkeyStr && hotkeyStr[0] != '\0')
    {
        // Convert UTF-8 hotkey to wide string
        result = ConvertUtf8ToUtf16(hotkeyStr);
    }
    
    // Clean up
    if (hotkeyStr)
        keymagic_free_string(hotkeyStr);
    keymagic_km2_free(km2Handle);
    
    return result;
}

// Normalize hotkey string for display
// Converts hotkey string to a consistent display format
// Example: "ctrl+shift+a" -> "Ctrl+Shift+A"
inline std::wstring NormalizeHotkeyForDisplay(const std::wstring& hotkey)
{
    if (hotkey.empty())
        return L"";
    
    // Convert to UTF-8 for parsing
    std::string utf8Hotkey = ConvertUtf16ToUtf8(hotkey);
    if (utf8Hotkey.empty())
        return L"";
    
    // Parse hotkey using FFI
    KeyMagicHotkeyInfo info = {0};
    if (keymagic_parse_hotkey(utf8Hotkey.c_str(), &info) == 0)
    {
        // Failed to parse, return original
        return hotkey;
    }
    
    // Build normalized display string
    std::wstring normalized;
    
    // Add modifiers in consistent order: Ctrl, Alt, Shift
    if (info.ctrl)
    {
        normalized += L"Ctrl";
    }
    
    if (info.alt)
    {
        if (!normalized.empty())
            normalized += L"+";
        normalized += L"Alt";
    }
    
    if (info.shift)
    {
        if (!normalized.empty())
            normalized += L"+";
        normalized += L"Shift";
    }
    
    // Add the main key
    if (info.key_code != 0)
    {
        if (!normalized.empty())
            normalized += L"+";
        
        // Use FFI function to get key display name
        char* keyNameStr = keymagic_virtual_key_to_string(info.key_code);
        if (keyNameStr)
        {
            // Convert UTF-8 to UTF-16
            std::wstring keyName = ConvertUtf8ToUtf16(keyNameStr);
            normalized += keyName;
            
            // Free the allocated string
            keymagic_free_string(keyNameStr);
        }
        else
        {
            // Fallback for unknown keys
            wchar_t fallback[32];
            swprintf_s(fallback, L"Key%d", info.key_code);
            normalized += fallback;
        }
    }
    
    return normalized;
}

} // namespace KeyMagicUtils