#pragma once

#include <windows.h>
#include <shlobj.h>
#include <sddl.h>
#include <string>
#include <memory>
#include <vector>
#include <set>
#include <map>
#include <mutex>
#include <thread>
#include <atomic>
#include <functional>

// Application constants
constexpr const wchar_t* KEYMAGIC_TRAY_CLASS = L"KeyMagicTrayWindow";
constexpr const wchar_t* KEYMAGIC_MUTEX_NAME = L"Global\\KeyMagicTrayManager";
constexpr const wchar_t* KEYMAGIC_REGISTRY_UPDATE_EVENT = L"Global\\KeyMagicRegistryUpdate";
constexpr const wchar_t* KEYMAGIC_REGISTRY_PATH = L"Software\\KeyMagic";
constexpr const wchar_t* KEYMAGIC_KEYBOARDS_PATH = L"Software\\KeyMagic\\Keyboards";
constexpr const wchar_t* KEYMAGIC_SETTINGS_PATH = L"Software\\KeyMagic\\Settings";
constexpr const wchar_t* KEYMAGIC_TIP_CLSID = L"{B9F5A039-9008-4D0F-97F5-26AA6D3C5F06}";

// Window messages
constexpr UINT WM_TRAYICON = WM_USER + 1;
constexpr UINT WM_PIPE_MESSAGE = WM_USER + 2;

// Tray icon ID
constexpr UINT TRAY_ICON_ID = 1;

// Default icon size
constexpr int DEFAULT_ICON_SIZE = 16;

// Pipe buffer size
constexpr DWORD PIPE_BUFFER_SIZE = 4096;

// Helper functions
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

inline std::wstring GetPipeName() {
    return L"\\\\.\\pipe\\KeyMagicTray-" + GetUserSID();
}

inline std::wstring GetLocalAppDataPath() {
    wchar_t path[MAX_PATH];
    if (SUCCEEDED(SHGetFolderPathW(nullptr, CSIDL_LOCAL_APPDATA, nullptr, 0, path))) {
        return std::wstring(path) + L"\\KeyMagic";
    }
    return L"";
}

inline std::wstring GetIconCachePath() {
    return GetLocalAppDataPath() + L"\\IconCache";
}