#pragma once

#include <windows.h>
#include <shlobj.h>
#include <sddl.h>
#include <string>
#include <memory>

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

} // namespace KeyMagicUtils