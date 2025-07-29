#include "IconVisibilityManager.h"
#include <algorithm>
#include <cctype>
#include <shlobj.h>
#include <knownfolders.h>

void IconVisibilityManager::EnsureIconVisible() {
    // Get the path to the current executable
    WCHAR exePath[MAX_PATH];
    if (GetModuleFileNameW(nullptr, exePath, MAX_PATH) == 0) {
        OutputDebugStringW(L"IconVisibilityManager: Failed to get executable path\n");
        return;
    }
    
    // Use the appropriate method based on Windows version
    PromoteIcon(exePath);
}

void IconVisibilityManager::PromoteIcon(const std::wstring& exePath) {
    PromotionResult result = PromotionResult::NotFound;
    
    if (IsWindows11()) {
        OutputDebugStringW(L"IconVisibilityManager: Detected Windows 11\n");
        result = PromoteIconWindows11(exePath);
        
        // Only fall back to Windows 10 method if the icon wasn't found
        if (result == PromotionResult::NotFound) {
            OutputDebugStringW(L"IconVisibilityManager: Icon not found in Windows 11 registry, trying Windows 10 method\n");
            result = PromoteIconWindows10(exePath);
        }
    } else {
        OutputDebugStringW(L"IconVisibilityManager: Using Windows 10 method\n");
        result = PromoteIconWindows10(exePath);
    }
    
    // Only refresh the notification area if we successfully promoted the icon
    if (result == PromotionResult::Promoted) {
        RefreshNotificationArea();
    }
}

bool IconVisibilityManager::IsWindows11() {
    // Check if NotifyIconSettings key exists (Windows 11 feature)
    HKEY hKey;
    LPCWSTR notifyIconPath = L"Control Panel\\NotifyIconSettings";
    
    if (RegOpenKeyExW(HKEY_CURRENT_USER, notifyIconPath, 0, KEY_READ, &hKey) == ERROR_SUCCESS) {
        RegCloseKey(hKey);
        return true;
    }
    
    return false;
}

IconVisibilityManager::PromotionResult IconVisibilityManager::PromoteIconWindows11(const std::wstring& exePath) {
    HKEY hNotifyIconSettings;
    LPCWSTR notifyIconPath = L"Control Panel\\NotifyIconSettings";
    
    if (RegOpenKeyExW(HKEY_CURRENT_USER, notifyIconPath, 0, KEY_READ, &hNotifyIconSettings) != ERROR_SUCCESS) {
        return PromotionResult::NotFound;
    }
    
    // Enumerate all subkeys (GUIDs)
    WCHAR guidKeyName[256];
    DWORD index = 0;
    PromotionResult result = PromotionResult::NotFound;
    
    while (RegEnumKeyW(hNotifyIconSettings, index++, guidKeyName, 256) == ERROR_SUCCESS) {
        // Open each GUID subkey
        HKEY hGuidKey;
        if (RegOpenKeyExW(hNotifyIconSettings, guidKeyName, 0, KEY_READ | KEY_WRITE, &hGuidKey) == ERROR_SUCCESS) {
            // Read the ExecutablePath value
            WCHAR executablePath[MAX_PATH];
            DWORD pathSize = sizeof(executablePath);
            DWORD valueType;
            
            if (RegQueryValueExW(hGuidKey, L"ExecutablePath", nullptr, &valueType, 
                               reinterpret_cast<LPBYTE>(executablePath), &pathSize) == ERROR_SUCCESS) {
                // Resolve the path if it contains a KNOWNFOLDERID
                std::wstring regPath = ResolveKnownFolderPath(executablePath);
                
                // Debug output
                if (regPath != executablePath) {
                    OutputDebugStringW((L"  Original registry path: " + std::wstring(executablePath) + L"\n").c_str());
                    OutputDebugStringW((L"  Resolved registry path: " + regPath + L"\n").c_str());
                }
                
                // Compare paths (case-insensitive)
                if (_wcsicmp(regPath.c_str(), exePath.c_str()) == 0) {
                    // Found our icon entry!
                    OutputDebugStringW(L"IconVisibilityManager: Found our icon entry in NotifyIconSettings\n");
                    OutputDebugStringW((L"  Registry path: " + regPath + L"\n").c_str());
                    OutputDebugStringW((L"  Our path: " + exePath + L"\n").c_str());
                    
                    // Check current IsPromoted value
                    DWORD currentIsPromoted = 0;
                    DWORD dataSize = sizeof(DWORD);
                    RegQueryValueExW(hGuidKey, L"IsPromoted", nullptr, nullptr, 
                                   reinterpret_cast<LPBYTE>(&currentIsPromoted), &dataSize);
                    
                    if (currentIsPromoted == 1) {
                        OutputDebugStringW(L"IconVisibilityManager: Icon is already promoted (IsPromoted = 1)\n");
                        result = PromotionResult::AlreadyPromoted;
                    } else {
                        // Set IsPromoted to 1
                        DWORD isPromoted = 1;
                        if (RegSetValueExW(hGuidKey, L"IsPromoted", 0, REG_DWORD, 
                                         reinterpret_cast<const BYTE*>(&isPromoted), sizeof(DWORD)) == ERROR_SUCCESS) {
                            OutputDebugStringW(L"IconVisibilityManager: Successfully set IsPromoted to 1\n");
                            result = PromotionResult::Promoted;
                        } else {
                            OutputDebugStringW(L"IconVisibilityManager: Failed to set IsPromoted value\n");
                            result = PromotionResult::Failed;
                        }
                    }
                }
            }
            
            RegCloseKey(hGuidKey);
            
            if (result != PromotionResult::NotFound) {
                break;  // Found our icon, stop searching
            }
        }
    }
    
    RegCloseKey(hNotifyIconSettings);
    
    if (result == PromotionResult::NotFound) {
        OutputDebugStringW(L"IconVisibilityManager: Icon entry not found in NotifyIconSettings\n");
    }
    
    return result;
}

IconVisibilityManager::PromotionResult IconVisibilityManager::PromoteIconWindows10(const std::wstring& exePath) {
    // Try to update IconStreams for targeted visibility control
    return UpdateIconStreamsVisibility(exePath);
}

IconVisibilityManager::PromotionResult IconVisibilityManager::UpdateIconStreamsVisibility(const std::wstring& exePath) {
    HKEY hKey;
    LPCWSTR keyPath = L"Software\\Classes\\Local Settings\\Software\\Microsoft\\Windows\\CurrentVersion\\TrayNotify";
    
    if (RegOpenKeyExW(HKEY_CURRENT_USER, keyPath, 0, KEY_READ | KEY_WRITE, &hKey) != ERROR_SUCCESS) {
        OutputDebugStringW(L"IconVisibilityManager: Failed to open TrayNotify registry key\n");
        return PromotionResult::NotFound;
    }
    
    // Read IconStreams data
    DWORD dataSize = 0;
    if (RegQueryValueExW(hKey, L"IconStreams", nullptr, nullptr, nullptr, &dataSize) != ERROR_SUCCESS) {
        RegCloseKey(hKey);
        OutputDebugStringW(L"IconVisibilityManager: Failed to query IconStreams size\n");
        return PromotionResult::NotFound;
    }
    
    std::vector<BYTE> iconStreams(dataSize);
    if (RegQueryValueExW(hKey, L"IconStreams", nullptr, nullptr, iconStreams.data(), &dataSize) != ERROR_SUCCESS) {
        RegCloseKey(hKey);
        OutputDebugStringW(L"IconVisibilityManager: Failed to read IconStreams data\n");
        return PromotionResult::NotFound;
    }
    
    // Create backup
    RegSetValueExW(hKey, L"IconStreams_Backup", 0, REG_BINARY, iconStreams.data(), dataSize);
    
    // Constants from documentation
    const int HEADER_SIZE = 20;
    const int RECORD_SIZE = 1640;
    const int PATH_OFFSET = 0;
    const int VISIBILITY_OFFSET = 528;
    const int PATH_SIZE = 528;
    
    // Encode our executable path for comparison
    std::wstring encodedExePath = Rot13Encode(exePath);
    std::transform(encodedExePath.begin(), encodedExePath.end(), encodedExePath.begin(), ::towlower);
    
    bool found = false;
    
    // Skip header and process records
    for (size_t i = HEADER_SIZE; i + RECORD_SIZE <= iconStreams.size(); i += RECORD_SIZE) {
        // Extract path (Unicode, ROT-13 encoded)
        std::wstring recordPath;
        for (int j = 0; j < PATH_SIZE && j + 1 < PATH_SIZE; j += 2) {
            BYTE low = iconStreams[i + j];
            BYTE high = iconStreams[i + j + 1];
            
            if (low == 0 && high == 0) break; // Null terminator
            
            wchar_t ch = (high << 8) | low;
            recordPath += ch;
        }
        
        // Convert to lowercase for case-insensitive comparison
        std::transform(recordPath.begin(), recordPath.end(), recordPath.begin(), ::towlower);
        
        // Check if this is our application
        if (recordPath.find(encodedExePath) != std::wstring::npos) {
            OutputDebugStringW(L"IconVisibilityManager: Found our icon in IconStreams\n");
            
            // Get current visibility value
            BYTE currentVisibility = iconStreams[i + VISIBILITY_OFFSET];
            
            if (currentVisibility == 2) {
                OutputDebugStringW(L"IconVisibilityManager: Icon visibility is already set to 2 (Always show)\n");
                RegCloseKey(hKey);
                return PromotionResult::AlreadyPromoted;
            } else {
                // Update visibility flag to 2 (Always show)
                iconStreams[i + VISIBILITY_OFFSET] = 2;
                found = true;
                
                // Debug: Show visibility change
                std::wstring debugMsg = L"IconVisibilityManager: Updated visibility from " + 
                                       std::to_wstring(currentVisibility) + 
                                       L" to 2\n";
                OutputDebugStringW(debugMsg.c_str());
            }
            break;
        }
    }
    
    if (found) {
        // Write modified data back
        if (RegSetValueExW(hKey, L"IconStreams", 0, REG_BINARY, iconStreams.data(), dataSize) == ERROR_SUCCESS) {
            OutputDebugStringW(L"IconVisibilityManager: Successfully updated IconStreams\n");
            
            // Clear PastIconsStream to force refresh
            RegDeleteValueW(hKey, L"PastIconsStream");
            
            RegCloseKey(hKey);
            
            // Note: Explorer restart would be required for immediate effect
            OutputDebugStringW(L"IconVisibilityManager: Note - Changes will take effect on next Explorer restart\n");
            
            return PromotionResult::Promoted;
        } else {
            OutputDebugStringW(L"IconVisibilityManager: Failed to write IconStreams\n");
            RegCloseKey(hKey);
            return PromotionResult::Failed;
        }
    } else {
        OutputDebugStringW(L"IconVisibilityManager: Icon not found in IconStreams\n");
    }
    
    RegCloseKey(hKey);
    return PromotionResult::NotFound;
}


std::wstring IconVisibilityManager::Rot13Encode(const std::wstring& input) {
    std::wstring result = input;
    for (wchar_t& ch : result) {
        if ((ch >= L'A' && ch <= L'Z')) {
            ch = ((ch - L'A' + 13) % 26) + L'A';
        } else if ((ch >= L'a' && ch <= L'z')) {
            ch = ((ch - L'a' + 13) % 26) + L'a';
        }
        // Numbers and special characters remain unchanged
    }
    return result;
}

std::wstring IconVisibilityManager::ResolveKnownFolderPath(const std::wstring& path) {
    // Check if the path starts with a CLSID pattern {xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx}
    if (path.length() > 38 && path[0] == L'{' && path[37] == L'}') {
        // Extract the GUID string
        std::wstring guidStr = path.substr(0, 38);
        
        // Try to convert it to a GUID
        GUID folderGuid;
        if (SUCCEEDED(CLSIDFromString(guidStr.c_str(), &folderGuid))) {
            // Try to get the known folder path
            PWSTR folderPath = nullptr;
            if (SUCCEEDED(SHGetKnownFolderPath(folderGuid, 0, nullptr, &folderPath))) {
                std::wstring resolvedPath(folderPath);
                
                // Append the rest of the path after the GUID
                if (path.length() > 38) {
                    resolvedPath += path.substr(38);
                }
                
                CoTaskMemFree(folderPath);
                return resolvedPath;
            }
        }
    }
    
    // Return original path if it's not a known folder path
    return path;
}

void IconVisibilityManager::RefreshNotificationArea() {
    // Notify explorer to refresh the notification area
    HWND hShellTrayWnd = FindWindowW(L"Shell_TrayWnd", nullptr);
    if (hShellTrayWnd) {
        SendMessage(hShellTrayWnd, WM_SETTINGCHANGE, 0, 0);
        
        // Also try sending a more specific notification
        HWND hTrayNotifyWnd = FindWindowExW(hShellTrayWnd, nullptr, L"TrayNotifyWnd", nullptr);
        if (hTrayNotifyWnd) {
            SendMessage(hTrayNotifyWnd, WM_SETTINGCHANGE, 0, 0);
        }
        
        // Send refresh command
        SendMessage(hShellTrayWnd, WM_COMMAND, 419, 0);
    }
}