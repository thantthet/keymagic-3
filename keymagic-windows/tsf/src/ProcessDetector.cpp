#include "ProcessDetector.h"
#include "Debug.h"
#include <windows.h>
#include <tlhelp32.h>
#include <algorithm>

std::wstring ProcessDetector::GetEffectiveProcessName()
{
    std::wstring currentProcessName = GetCurrentProcessName();
    
    // If current process is msedgewebview2.exe, check parent process instead
    if (currentProcessName == L"msedgewebview2.exe")
    {
        DEBUG_LOG(L"Current process is msedgewebview2.exe, checking parent process");
        
        std::wstring parentProcessName = GetParentProcessName();
        if (!parentProcessName.empty())
        {
            DEBUG_LOG(L"Parent process found: " + parentProcessName);
            return parentProcessName;
        }
        else
        {
            DEBUG_LOG(L"Failed to get parent process, using current process: " + currentProcessName);
            return currentProcessName;
        }
    }
    
    return currentProcessName;
}

std::wstring ProcessDetector::GetCurrentProcessName()
{
    wchar_t processPath[MAX_PATH];
    if (GetModuleFileNameW(NULL, processPath, MAX_PATH) == 0)
    {
        DEBUG_LOG(L"Failed to get process path");
        return L"unknown";
    }
    
    // Extract just the executable name from the full path
    std::wstring fullPath(processPath);
    size_t lastSlash = fullPath.find_last_of(L"\\");
    std::wstring exeName = (lastSlash != std::wstring::npos) ? fullPath.substr(lastSlash + 1) : fullPath;
    
    return ToLowerCase(exeName);
}

std::wstring ProcessDetector::GetParentProcessName()
{
    DWORD currentPid = GetCurrentProcessId();
    HANDLE hSnapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
    
    if (hSnapshot == INVALID_HANDLE_VALUE)
    {
        DEBUG_LOG(L"Failed to create process snapshot");
        return L"";
    }
    
    PROCESSENTRY32W pe32;
    pe32.dwSize = sizeof(PROCESSENTRY32W);
    
    std::wstring parentProcessName;
    
    if (Process32FirstW(hSnapshot, &pe32))
    {
        do
        {
            if (pe32.th32ProcessID == currentPid)
            {
                // Found current process, now get parent process info
                DWORD parentPid = pe32.th32ParentProcessID;
                
                // Find parent process name
                HANDLE hParentSnapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
                if (hParentSnapshot != INVALID_HANDLE_VALUE)
                {
                    PROCESSENTRY32W parentPe32;
                    parentPe32.dwSize = sizeof(PROCESSENTRY32W);
                    
                    if (Process32FirstW(hParentSnapshot, &parentPe32))
                    {
                        do
                        {
                            if (parentPe32.th32ProcessID == parentPid)
                            {
                                parentProcessName = ToLowerCase(std::wstring(parentPe32.szExeFile));
                                break;
                            }
                        } while (Process32NextW(hParentSnapshot, &parentPe32));
                    }
                    CloseHandle(hParentSnapshot);
                }
                break;
            }
        } while (Process32NextW(hSnapshot, &pe32));
    }
    
    CloseHandle(hSnapshot);
    return parentProcessName;
}

std::wstring ProcessDetector::ToLowerCase(const std::wstring& input)
{
    std::wstring result = input;
    std::transform(result.begin(), result.end(), result.begin(), ::towlower);
    return result;
}