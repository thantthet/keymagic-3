#pragma once

#include <windows.h>
#include <string>
#include <sstream>
#include <iomanip>
#include <chrono>
#include "../../shared/include/keymagic_ffi.h"

// Debug logging macros and functions for KeyMagic TSF

// Always enable logging for both Debug and Release builds
#define DEBUG_LOG(msg) DebugLog(msg)
#define DEBUG_LOG_W(msg) DebugLog(msg)
#define DEBUG_LOG_FUNC() DebugLog(__FUNCTIONW__)

// Censor sensitive data in Release builds
#ifdef _DEBUG
    #define DEBUG_LOG_KEY(context, wParam, lParam, character) DebugLogKeyEvent(context, wParam, lParam, character)
    #define DEBUG_LOG_ENGINE(output) DebugLogEngineOutput(output)
    #define DEBUG_LOG_TEXT(context, text) DebugLog(std::wstring(context) + L": \"" + text + L"\"")
    #define DEBUG_LOG_SYNC_MISMATCH(engineText, documentText) \
        DebugLog(L"Composition text mismatch - Engine: \"" + ConvertUtf8ToUtf16(engineText) + \
                 L"\", Document: \"" + documentText + L"\" - continuing with sync")
#else
    // Release build - censor key events, engine output, and text content
    #define DEBUG_LOG_KEY(context, wParam, lParam, character) DebugLogKeyCensored(context, wParam)
    #define DEBUG_LOG_ENGINE(output) DebugLogEngineCensored(output)
    #define DEBUG_LOG_TEXT(context, text) DebugLog(std::wstring(context) + L": [REDACTED]")
    #define DEBUG_LOG_SYNC_MISMATCH(engineText, documentText) \
        DebugLog(L"Composition text mismatch - Engine: [REDACTED], Document: [REDACTED] - continuing with sync")
#endif

#define DEBUG_LOG_HR(context, hr) DebugLogHR(context, hr)
// DEBUG_LOG_UTF8 is deprecated - all DEBUG_LOG calls now automatically format Unicode

// Helper to format wide string with Unicode notation for non-ASCII characters
inline std::wstring FormatWideStringWithUnicodeNotation(const std::wstring& wideStr)
{
    std::wostringstream oss;
    
    // Format each character
    for (wchar_t ch : wideStr) {
        if (ch >= 0x20 && ch <= 0x7E) {
            // ASCII printable characters
            oss << ch;
        } else if (ch == L'\n') {
            oss << L"\\n";
        } else if (ch == L'\r') {
            oss << L"\\r";
        } else if (ch == L'\t') {
            oss << L"\\t";
        } else {
            // Non-ASCII or control characters - show Unicode notation
            oss << L"\\u" << std::hex << std::setfill(L'0') << std::setw(4) << (unsigned int)ch;
        }
    }
    
    return oss.str();
}

// Core debug logging function
inline void DebugLog(const std::wstring& message)
{
    // Add timestamp
    auto now = std::chrono::system_clock::now();
    auto time_t = std::chrono::system_clock::to_time_t(now);
    auto ms = std::chrono::duration_cast<std::chrono::milliseconds>(now.time_since_epoch()) % 1000;
    
    std::tm tm;
    localtime_s(&tm, &time_t);
    
    // Format the message with Unicode notation
    std::wstring formattedMessage = FormatWideStringWithUnicodeNotation(message);
    
    std::wostringstream oss;
    oss << L"[KeyMagicTSF] "
        << std::put_time(&tm, L"%H:%M:%S") 
        << L"." << std::setfill(L'0') << std::setw(3) << ms.count() 
        << L" [" << GetCurrentThreadId() << L"] "
        << formattedMessage << L"\n";
    
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
    
    if (character != '\0') {
        if (character >= 0x20 && character <= 0x7E) {
            oss << L", Char: '" << (wchar_t)character << L"' (0x" << std::hex << (int)(unsigned char)character << L")";
        } else {
            oss << L", Char: 0x" << std::hex << (int)(unsigned char)character;
        }
    }
    
    oss << L", Scan: 0x" << std::hex << ((lParam >> 16) & 0xFF);
    
    // Add modifier states
    if (GetKeyState(VK_SHIFT) & 0x8000) oss << L" [SHIFT]";
    if (GetKeyState(VK_CONTROL) & 0x8000) oss << L" [CTRL]";
    if (GetKeyState(VK_MENU) & 0x8000) oss << L" [ALT]";
    if (GetKeyState(VK_CAPITAL) & 0x0001) oss << L" [CAPS]";
    
    DebugLog(oss.str());
}

// Helper to format string with Unicode notation for non-ASCII characters
inline std::wstring FormatStringWithUnicodeNotation(const std::string& utf8Str)
{
    std::wostringstream oss;
    
    // Convert UTF-8 to UTF-16
    int wideSize = MultiByteToWideChar(CP_UTF8, 0, utf8Str.c_str(), -1, nullptr, 0);
    if (wideSize == 0) return L"[Invalid UTF-8]";
    
    std::wstring wideStr(wideSize - 1, L'\0');
    MultiByteToWideChar(CP_UTF8, 0, utf8Str.c_str(), -1, &wideStr[0], wideSize);
    
    // Format each character
    for (wchar_t ch : wideStr) {
        if (ch >= 0x20 && ch <= 0x7E) {
            // ASCII printable characters
            oss << ch;
        } else {
            // Non-ASCII or control characters - show Unicode notation
            oss << L"\\u" << std::hex << std::setfill(L'0') << std::setw(4) << (unsigned int)ch;
        }
    }
    
    return oss.str();
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
        oss << L", Text: \"" << FormatStringWithUnicodeNotation(text) << L"\"";
    }
    
    if (output.composing_text) {
        std::string comp(output.composing_text);
        oss << L", Composing: \"" << FormatStringWithUnicodeNotation(comp) << L"\"";
    }
    
    DebugLog(oss.str());
}

// Censored version for Release builds - only logs key type without actual values
inline void DebugLogKeyCensored(const std::wstring& context, WPARAM wParam)
{
    std::wostringstream oss;
    oss << context << L" - ";
    
    // Only log key categories, not actual keys
    if (wParam >= 'A' && wParam <= 'Z') {
        oss << L"[LETTER KEY]";
    } else if (wParam >= '0' && wParam <= '9') {
        oss << L"[DIGIT KEY]";
    } else if (wParam == VK_SPACE) {
        oss << L"[SPACE]";
    } else if (wParam == VK_RETURN) {
        oss << L"[ENTER]";
    } else if (wParam == VK_BACK) {
        oss << L"[BACKSPACE]";
    } else if (wParam == VK_TAB) {
        oss << L"[TAB]";
    } else if (wParam == VK_ESCAPE) {
        oss << L"[ESCAPE]";
    } else if (wParam >= VK_F1 && wParam <= VK_F12) {
        oss << L"[FUNCTION KEY]";
    } else {
        oss << L"[OTHER KEY]";
    }
    
    // Show modifiers since they're not sensitive
    if (GetKeyState(VK_SHIFT) & 0x8000) oss << L" [SHIFT]";
    if (GetKeyState(VK_CONTROL) & 0x8000) oss << L" [CTRL]";
    if (GetKeyState(VK_MENU) & 0x8000) oss << L" [ALT]";
    if (GetKeyState(VK_CAPITAL) & 0x0001) oss << L" [CAPS]";
    
    DebugLog(oss.str());
}

// Censored engine output for Release builds - no text content
inline void DebugLogEngineCensored(const ProcessKeyOutput& output)
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
    
    // Don't log actual text content in release builds
    if (output.text) {
        oss << L", Text: [REDACTED]";
    }
    
    if (output.composing_text) {
        oss << L", Composing: [REDACTED]";
    }
    
    DebugLog(oss.str());
}