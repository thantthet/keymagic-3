#pragma once

#include <windows.h>
#include <string>
#include <sstream>
#include <iomanip>
#include <chrono>

// Debug logging macros and functions for KeyMagic TSF

// Enable debug logging in debug builds
#ifdef _DEBUG
    #define DEBUG_LOG(msg) DebugLog(msg)
    #define DEBUG_LOG_W(msg) DebugLog(msg)
#else
    #define DEBUG_LOG(msg) ((void)0)
    #define DEBUG_LOG_W(msg) ((void)0)
#endif

// Core debug logging function
inline void DebugLog(const std::wstring& message)
{
    // Add timestamp
    auto now = std::chrono::system_clock::now();
    auto time_t = std::chrono::system_clock::to_time_t(now);
    auto ms = std::chrono::duration_cast<std::chrono::milliseconds>(now.time_since_epoch()) % 1000;
    
    std::tm tm;
    localtime_s(&tm, &time_t);
    
    std::wostringstream oss;
    oss << L"[KeyMagicTSF] "
        << std::put_time(&tm, L"%H:%M:%S") 
        << L"." << std::setfill(L'0') << std::setw(3) << ms.count() 
        << L" [" << GetCurrentThreadId() << L"] "
        << message << L"\n";
    
    OutputDebugStringW(oss.str().c_str());
}

// Overload for ANSI strings
inline void DebugLog(const std::string& message)
{
    // Convert to wide string
    std::wstring wmsg(message.begin(), message.end());
    DebugLog(wmsg);
}

// Helper to log HRESULT values
inline void DebugLogHR(const std::wstring& context, HRESULT hr)
{
    std::wostringstream oss;
    oss << context << L" - HRESULT: 0x" << std::hex << hr;
    if (FAILED(hr))
    {
        // Get error message
        LPWSTR messageBuffer = nullptr;
        FormatMessageW(FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_IGNORE_INSERTS,
                       NULL, hr, MAKELANGID(LANG_NEUTRAL, SUBLANG_DEFAULT),
                       (LPWSTR)&messageBuffer, 0, NULL);
        
        if (messageBuffer)
        {
            oss << L" - " << messageBuffer;
            LocalFree(messageBuffer);
        }
    }
    DebugLog(oss.str());
}

// Helper to log key events
inline void DebugLogKey(const std::wstring& context, WPARAM wParam, LPARAM lParam)
{
    std::wostringstream oss;
    oss << context << L" - VK: 0x" << std::hex << wParam 
        << L" (" << std::dec << wParam << L")"
        << L", Scan: 0x" << std::hex << ((lParam >> 16) & 0xFF)
        << L", Flags: 0x" << std::hex << (lParam >> 24);
    DebugLog(oss.str());
}