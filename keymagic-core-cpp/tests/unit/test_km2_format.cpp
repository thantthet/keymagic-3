#include <gtest/gtest.h>
#include <keymagic/km2_format.h>
#include <cstring>

using namespace keymagic;

TEST(KM2FormatTest, FileHeaderValidation) {
    FileHeader header;
    std::memcpy(header.magicCode, KM2_MAGIC_CODE, 4);
    header.majorVersion = 1;
    header.minorVersion = 5;
    
    EXPECT_TRUE(header.isValid());
    EXPECT_TRUE(header.isCompatibleVersion());
    
    // Test invalid magic code
    header.magicCode[0] = 'X';
    EXPECT_FALSE(header.isValid());
    
    // Test invalid version
    std::memcpy(header.magicCode, KM2_MAGIC_CODE, 4);
    header.majorVersion = 2;
    EXPECT_TRUE(header.isValid());
    EXPECT_FALSE(header.isCompatibleVersion());
}

TEST(KM2FormatTest, LayoutOptions) {
    KM2LayoutOptions opts;
    
    // Test defaults
    EXPECT_TRUE(opts.getTrackCaps());
    EXPECT_FALSE(opts.getAutoBksp());
    EXPECT_FALSE(opts.getEat());
    EXPECT_FALSE(opts.getPosBased());
    EXPECT_TRUE(opts.getRightAlt());
    
    // Test setting values
    opts.trackCaps = 0;
    opts.autoBksp = 1;
    EXPECT_FALSE(opts.getTrackCaps());
    EXPECT_TRUE(opts.getAutoBksp());
}

TEST(KM2FormatTest, InfoEntryTypes) {
    InfoEntry nameEntry(INFO_NAME, {});
    EXPECT_TRUE(nameEntry.isName());
    EXPECT_FALSE(nameEntry.isDescription());
    
    InfoEntry descEntry(INFO_DESC, {});
    EXPECT_TRUE(descEntry.isDescription());
    EXPECT_FALSE(descEntry.isName());
}

TEST(KM2FormatTest, BinaryOpcodes) {
    // Test opcode values match expected format
    EXPECT_EQ(0x00F0, OP_STRING);
    EXPECT_EQ(0x00F1, OP_VARIABLE);
    EXPECT_EQ(0x00F2, OP_REFERENCE);
    EXPECT_EQ(0x00F3, OP_PREDEFINED);
    EXPECT_EQ(0x00F4, OP_MODIFIER);
    EXPECT_EQ(0x00F6, OP_AND);
    EXPECT_EQ(0x00F8, OP_ANY);
    EXPECT_EQ(0x00F9, OP_SWITCH);
    
    // Test modifier flags
    EXPECT_EQ(0x00F5, FLAG_ANYOF);
    EXPECT_EQ(0x00F7, FLAG_NANYOF);
}