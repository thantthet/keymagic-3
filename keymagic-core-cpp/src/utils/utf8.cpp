#include "utf8.h"
#include <codecvt>
#include <locale>
#include <algorithm>

#ifdef _WIN32
#include <windows.h>
#endif

namespace keymagic {
namespace utils {

// UTF-8 to UTF-16 conversion
std::u16string utf8ToUtf16(const std::string& utf8) {
    if (utf8.empty()) {
        return std::u16string();
    }
    
#ifdef _WIN32
    // Use Windows API for better compatibility
    int requiredSize = MultiByteToWideChar(CP_UTF8, 0, utf8.c_str(), -1, nullptr, 0);
    if (requiredSize <= 0) {
        return std::u16string();
    }
    
    std::wstring wideStr(requiredSize - 1, L'\0');
    MultiByteToWideChar(CP_UTF8, 0, utf8.c_str(), -1, &wideStr[0], requiredSize);
    
    // Convert wstring to u16string (on Windows, wchar_t is 16-bit)
    return std::u16string(wideStr.begin(), wideStr.end());
#else
    // Use standard C++ conversion for other platforms
    try {
        std::wstring_convert<std::codecvt_utf8_utf16<char16_t>, char16_t> converter;
        return converter.from_bytes(utf8);
    } catch (...) {
        return std::u16string();
    }
#endif
}

// UTF-16 to UTF-8 conversion
std::string utf16ToUtf8(const std::u16string& utf16) {
    if (utf16.empty()) {
        return std::string();
    }
    
#ifdef _WIN32
    // Convert u16string to wstring (on Windows, wchar_t is 16-bit)
    std::wstring wideStr(utf16.begin(), utf16.end());
    
    // Use Windows API for conversion
    int requiredSize = WideCharToMultiByte(CP_UTF8, 0, wideStr.c_str(), -1, 
                                          nullptr, 0, nullptr, nullptr);
    if (requiredSize <= 0) {
        return std::string();
    }
    
    std::string utf8(requiredSize - 1, '\0');
    WideCharToMultiByte(CP_UTF8, 0, wideStr.c_str(), -1, 
                       &utf8[0], requiredSize, nullptr, nullptr);
    return utf8;
#else
    // Use standard C++ conversion for other platforms
    try {
        std::wstring_convert<std::codecvt_utf8_utf16<char16_t>, char16_t> converter;
        return converter.to_bytes(utf16);
    } catch (...) {
        return std::string();
    }
#endif
}

// UTF-16LE to UTF-16 conversion (byte order conversion)
std::u16string utf16leToUtf16(const uint8_t* data, size_t byteLength) {
    if (!data || byteLength == 0 || byteLength % 2 != 0) {
        return std::u16string();
    }
    
    size_t charCount = byteLength / 2;
    std::u16string result;
    result.reserve(charCount);
    
    for (size_t i = 0; i < byteLength; i += 2) {
        // Little-endian: low byte first, high byte second
        char16_t ch = static_cast<char16_t>(data[i]) | 
                     (static_cast<char16_t>(data[i + 1]) << 8);
        result.push_back(ch);
    }
    
    return result;
}

// UTF-16 to UTF-16LE conversion
std::vector<uint8_t> utf16ToUtf16le(const std::u16string& utf16) {
    std::vector<uint8_t> result;
    result.reserve(utf16.size() * 2);
    
    for (char16_t ch : utf16) {
        // Little-endian: low byte first, high byte second
        result.push_back(static_cast<uint8_t>(ch & 0xFF));
        result.push_back(static_cast<uint8_t>((ch >> 8) & 0xFF));
    }
    
    return result;
}

// UTF-32 to UTF-8 conversion (for char32_t)
std::string utf32ToUtf8(char32_t codepoint) {
    std::string result;
    
    if (codepoint <= 0x7F) {
        // 1-byte sequence
        result.push_back(static_cast<char>(codepoint));
    }
    else if (codepoint <= 0x7FF) {
        // 2-byte sequence
        result.push_back(static_cast<char>(0xC0 | (codepoint >> 6)));
        result.push_back(static_cast<char>(0x80 | (codepoint & 0x3F)));
    }
    else if (codepoint <= 0xFFFF) {
        // 3-byte sequence
        result.push_back(static_cast<char>(0xE0 | (codepoint >> 12)));
        result.push_back(static_cast<char>(0x80 | ((codepoint >> 6) & 0x3F)));
        result.push_back(static_cast<char>(0x80 | (codepoint & 0x3F)));
    }
    else if (codepoint <= 0x10FFFF) {
        // 4-byte sequence
        result.push_back(static_cast<char>(0xF0 | (codepoint >> 18)));
        result.push_back(static_cast<char>(0x80 | ((codepoint >> 12) & 0x3F)));
        result.push_back(static_cast<char>(0x80 | ((codepoint >> 6) & 0x3F)));
        result.push_back(static_cast<char>(0x80 | (codepoint & 0x3F)));
    }
    
    return result;
}

// Count UTF-8 characters (code points)
size_t utf8CharCount(const std::string& utf8) {
    size_t count = 0;
    size_t i = 0;
    
    while (i < utf8.size()) {
        uint8_t byte = static_cast<uint8_t>(utf8[i]);
        
        if (byte <= 0x7F) {
            // 1-byte character
            i += 1;
        }
        else if ((byte & 0xE0) == 0xC0) {
            // 2-byte character
            i += 2;
        }
        else if ((byte & 0xF0) == 0xE0) {
            // 3-byte character
            i += 3;
        }
        else if ((byte & 0xF8) == 0xF0) {
            // 4-byte character
            i += 4;
        }
        else {
            // Invalid UTF-8 sequence, skip byte
            i += 1;
        }
        
        count++;
    }
    
    return count;
}

// Get substring by character count (not byte count)
std::string utf8Substring(const std::string& utf8, size_t start, size_t length) {
    size_t byteStart = 0;
    size_t charIndex = 0;
    
    // Find byte position of start
    while (byteStart < utf8.size() && charIndex < start) {
        uint8_t byte = static_cast<uint8_t>(utf8[byteStart]);
        
        if (byte <= 0x7F) {
            byteStart += 1;
        }
        else if ((byte & 0xE0) == 0xC0) {
            byteStart += 2;
        }
        else if ((byte & 0xF0) == 0xE0) {
            byteStart += 3;
        }
        else if ((byte & 0xF8) == 0xF0) {
            byteStart += 4;
        }
        else {
            byteStart += 1;
        }
        
        charIndex++;
    }
    
    if (byteStart >= utf8.size()) {
        return std::string();
    }
    
    // Find byte length
    size_t byteEnd = byteStart;
    size_t charCount = 0;
    
    while (byteEnd < utf8.size() && charCount < length) {
        uint8_t byte = static_cast<uint8_t>(utf8[byteEnd]);
        
        if (byte <= 0x7F) {
            byteEnd += 1;
        }
        else if ((byte & 0xE0) == 0xC0) {
            byteEnd += 2;
        }
        else if ((byte & 0xF0) == 0xE0) {
            byteEnd += 3;
        }
        else if ((byte & 0xF8) == 0xF0) {
            byteEnd += 4;
        }
        else {
            byteEnd += 1;
        }
        
        charCount++;
    }
    
    return utf8.substr(byteStart, byteEnd - byteStart);
}

// Check if string is valid UTF-8
bool isValidUtf8(const std::string& utf8) {
    size_t i = 0;
    
    while (i < utf8.size()) {
        uint8_t byte = static_cast<uint8_t>(utf8[i]);
        size_t sequenceLength = 0;
        
        if (byte <= 0x7F) {
            sequenceLength = 1;
        }
        else if ((byte & 0xE0) == 0xC0) {
            sequenceLength = 2;
        }
        else if ((byte & 0xF0) == 0xE0) {
            sequenceLength = 3;
        }
        else if ((byte & 0xF8) == 0xF0) {
            sequenceLength = 4;
        }
        else {
            return false;  // Invalid first byte
        }
        
        // Check if we have enough bytes
        if (i + sequenceLength > utf8.size()) {
            return false;
        }
        
        // Check continuation bytes
        for (size_t j = 1; j < sequenceLength; j++) {
            uint8_t contByte = static_cast<uint8_t>(utf8[i + j]);
            if ((contByte & 0xC0) != 0x80) {
                return false;  // Invalid continuation byte
            }
        }
        
        i += sequenceLength;
    }
    
    return true;
}

// Convert single UTF-8 character to char32_t
char32_t utf8ToChar32(const std::string& utf8, size_t& bytesConsumed) {
    bytesConsumed = 0;
    
    if (utf8.empty()) {
        return 0;
    }
    
    uint8_t byte1 = static_cast<uint8_t>(utf8[0]);
    
    if (byte1 <= 0x7F) {
        // 1-byte sequence
        bytesConsumed = 1;
        return byte1;
    }
    else if ((byte1 & 0xE0) == 0xC0 && utf8.size() >= 2) {
        // 2-byte sequence
        uint8_t byte2 = static_cast<uint8_t>(utf8[1]);
        if ((byte2 & 0xC0) == 0x80) {
            bytesConsumed = 2;
            return ((byte1 & 0x1F) << 6) | (byte2 & 0x3F);
        }
    }
    else if ((byte1 & 0xF0) == 0xE0 && utf8.size() >= 3) {
        // 3-byte sequence
        uint8_t byte2 = static_cast<uint8_t>(utf8[1]);
        uint8_t byte3 = static_cast<uint8_t>(utf8[2]);
        if ((byte2 & 0xC0) == 0x80 && (byte3 & 0xC0) == 0x80) {
            bytesConsumed = 3;
            return ((byte1 & 0x0F) << 12) | ((byte2 & 0x3F) << 6) | (byte3 & 0x3F);
        }
    }
    else if ((byte1 & 0xF8) == 0xF0 && utf8.size() >= 4) {
        // 4-byte sequence
        uint8_t byte2 = static_cast<uint8_t>(utf8[1]);
        uint8_t byte3 = static_cast<uint8_t>(utf8[2]);
        uint8_t byte4 = static_cast<uint8_t>(utf8[3]);
        if ((byte2 & 0xC0) == 0x80 && (byte3 & 0xC0) == 0x80 && (byte4 & 0xC0) == 0x80) {
            bytesConsumed = 4;
            return ((byte1 & 0x07) << 18) | ((byte2 & 0x3F) << 12) | 
                   ((byte3 & 0x3F) << 6) | (byte4 & 0x3F);
        }
    }
    
    // Invalid UTF-8 sequence
    bytesConsumed = 1;  // Skip the invalid byte
    return 0xFFFD;  // Replacement character
}

} // namespace utils
} // namespace keymagic