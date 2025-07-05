#pragma once

#include <windows.h>
#include <string>
#include <sstream>
#include <iomanip>
#include <chrono>
#include "../include/keymagic_ffi.h"

// Debug logging macros and functions for KeyMagic TSF

// Always enable logging for both Debug and Release builds
#define DEBUG_LOG(msg) DebugLog(msg)
#define DEBUG_LOG_W(msg) DebugLog(msg)
#define DEBUG_LOG_FUNC() DebugLog(__FUNCTIONW__)
#define DEBUG_LOG_KEY(context, wParam, lParam, character) DebugLogKeyEvent(context, wParam, lParam, character)
#define DEBUG_LOG_ENGINE(output) DebugLogEngineOutput(output)
#define DEBUG_LOG_HR(context, hr) DebugLogHR(context, hr)

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

// Helper to log key events with character
inline void DebugLogKeyEvent(const std::wstring& context, WPARAM wParam, LPARAM lParam, char character)
{
    std::wostringstream oss;
    oss << context << L" - VK: 0x" << std::hex << wParam 
        << L" (" << std::dec << wParam << L")";
    
    if (character != '\0' && character >= 0x20 && character <= 0x7E) {
        oss << L", Char: '" << (wchar_t)character << L"'";
    } else if (character != '\0') {
        oss << L", Char: 0x" << std::hex << (int)(unsigned char)character;
    }
    
    oss << L", Scan: 0x" << std::hex << ((lParam >> 16) & 0xFF);
    
    // Add modifier states
    if (GetKeyState(VK_SHIFT) & 0x8000) oss << L" [SHIFT]";
    if (GetKeyState(VK_CONTROL) & 0x8000) oss << L" [CTRL]";
    if (GetKeyState(VK_MENU) & 0x8000) oss << L" [ALT]";
    if (GetKeyState(VK_CAPITAL) & 0x0001) oss << L" [CAPS]";
    
    DebugLog(oss.str());
}

// Helper to log engine output
inline void DebugLogEngineOutput(const ProcessKeyOutput& output)
{
    std::wostringstream oss;
    oss << L"Engine Output - Processed: " << (output.is_processed ? L"YES" : L"NO");
    oss << L", Action: ";
    
    switch (output.action_type) {
        case 0: oss << L"None"; break;
        case 1: oss << L"Insert"; break;
        case 2: oss << L"Delete(" << output.delete_count << L")"; break;
        case 3: oss << L"DeleteAndInsert(" << output.delete_count << L")"; break;
        default: oss << L"Unknown(" << output.action_type << L")"; break;
    }
    
    if (output.text) {
        std::string text(output.text);
        std::wstring wtext(text.begin(), text.end());
        oss << L", Text: \"" << wtext << L"\"";
    }
    
    if (output.composing_text) {
        std::string comp(output.composing_text);
        std::wstring wcomp(comp.begin(), comp.end());
        oss << L", Composing: \"" << wcomp << L"\"";
    }
    
    DebugLog(oss.str());
}