#pragma once

#include <string>

class ProcessDetector
{
public:
    /// Gets the effective process name for composition mode checking
    /// If the current process is msedgewebview2.exe, returns the parent process name
    /// Otherwise, returns the current process name
    static std::wstring GetEffectiveProcessName();

private:
    /// Gets the current process executable name
    static std::wstring GetCurrentProcessName();
    
    /// Gets the parent process name for the current process
    /// Returns empty string if unable to determine parent process
    static std::wstring GetParentProcessName();
    
    /// Converts a wide string to lowercase
    static std::wstring ToLowerCase(const std::wstring& input);
};