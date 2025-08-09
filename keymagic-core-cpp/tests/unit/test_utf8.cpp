#include <gtest/gtest.h>
#include "../../src/utils/utf8.h"

using namespace keymagic::utils;

TEST(Utf8Test, BasicConversions) {
    // Test ASCII
    std::string ascii = "Hello";
    std::u16string utf16 = utf8ToUtf16(ascii);
    EXPECT_EQ(ascii, utf16ToUtf8(utf16));
    
    // Test empty string
    EXPECT_EQ(std::u16string(), utf8ToUtf16(""));
    EXPECT_EQ(std::string(), utf16ToUtf8(std::u16string()));
}

TEST(Utf8Test, CharacterCounting) {
    // ASCII
    EXPECT_EQ(5, utf8CharCount("Hello"));
    
    // Multi-byte UTF-8 (Myanmar text)
    std::string myanmar = u8"မြန်မာ";
    EXPECT_EQ(6, utf8CharCount(myanmar));
    
    // Empty string
    EXPECT_EQ(0, utf8CharCount(""));
}

TEST(Utf8Test, SingleAsciiPrintable) {
    EXPECT_TRUE(isSingleAsciiPrintable("a"));
    EXPECT_TRUE(isSingleAsciiPrintable("!"));
    EXPECT_TRUE(isSingleAsciiPrintable("~"));
    
    EXPECT_FALSE(isSingleAsciiPrintable(" "));  // Space is not included
    EXPECT_FALSE(isSingleAsciiPrintable(""));
    EXPECT_FALSE(isSingleAsciiPrintable("ab"));
    EXPECT_FALSE(isSingleAsciiPrintable(u8"မ"));
}

TEST(Utf8Test, AnyCharacterRange) {
    EXPECT_TRUE(isAnyCharacter('!'));
    EXPECT_TRUE(isAnyCharacter('a'));
    EXPECT_TRUE(isAnyCharacter('~'));
    
    EXPECT_FALSE(isAnyCharacter(' '));
    EXPECT_FALSE(isAnyCharacter('\n'));
    EXPECT_FALSE(isAnyCharacter(0x1000));  // Myanmar character
}