#ifndef KEYMAGIC_UTF8_H
#define KEYMAGIC_UTF8_H

#include <string>
#include <vector>
#include <cstdint>

namespace keymagic {
namespace utils {

// UTF-8 <-> UTF-16 conversions
std::u16string utf8ToUtf16(const std::string& utf8);
std::string utf16ToUtf8(const std::u16string& utf16);

// UTF-16LE <-> UTF-16 conversions (for binary file format)
std::u16string utf16leToUtf16(const uint8_t* data, size_t byteLength);
std::vector<uint8_t> utf16ToUtf16le(const std::u16string& utf16);

// UTF-32 <-> UTF-8 conversions (for char32_t)
std::string utf32ToUtf8(char32_t codepoint);
char32_t utf8ToChar32(const std::string& utf8, size_t& bytesConsumed);

// UTF-8 string utilities
size_t utf8CharCount(const std::string& utf8);
std::string utf8Substring(const std::string& utf8, size_t start, size_t length = std::string::npos);
bool isValidUtf8(const std::string& utf8);

// UTF-16 string utilities
std::u16string utf16Substring(const std::u16string& utf16, size_t start, size_t length = std::u16string::npos);
char32_t utf16ToChar32(const std::u16string& utf16, size_t& charsConsumed);
std::u16string utf32ToUtf16(char32_t codepoint);

// Helper to check if a character is a single ASCII printable (for recursion stopping)
inline bool isSingleAsciiPrintable(const std::string& str) {
    if (str.size() != 1) return false;
    char ch = str[0];
    return ch >= '!' && ch <= '~';  // ASCII printable range excluding space
}

// UTF-16 version
inline bool isSingleAsciiPrintable(const std::u16string& str) {
    if (str.size() != 1) return false;
    char16_t ch = str[0];
    return ch >= u'!' && ch <= u'~';  // ASCII printable range excluding space
}

// Helper to check if character is in ANY range (ASCII printable)
inline bool isAnyCharacter(char32_t ch) {
    return ch >= 0x21 && ch <= 0x7E;  // ! to ~
}

} // namespace utils
} // namespace keymagic

#endif // KEYMAGIC_UTF8_H