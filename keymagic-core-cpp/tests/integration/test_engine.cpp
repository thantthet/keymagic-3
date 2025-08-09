#include <gtest/gtest.h>
#include <keymagic/keymagic.h>
#include <keymagic/km2_format.h>
#include "../../src/km2/loader.h"
#include <cstring>

using namespace keymagic;

// Create a minimal KM2 file in memory for testing
class TestKM2Builder {
public:
    static std::vector<uint8_t> buildMinimalKM2() {
        std::vector<uint8_t> data;
        
        // Header
        FileHeader header;
        std::memcpy(header.magicCode, KM2_MAGIC_CODE, 4);
        header.majorVersion = 1;
        header.minorVersion = 5;
        header.stringCount = 2;  // Will add 2 strings
        header.infoCount = 1;    // Will add keyboard name
        header.ruleCount = 1;    // Will add one simple rule
        header.layoutOptions = KM2LayoutOptions();
        
        // Write header
        appendBytes(data, &header, sizeof(header));
        
        // String section - add two test strings
        // String 1: "ka"
        uint16_t len1 = 2;
        appendBytes(data, &len1, sizeof(len1));
        char16_t str1[] = u"ka";
        appendBytes(data, str1, len1 * sizeof(char16_t));
        
        // String 2: "က"
        uint16_t len2 = 1;
        appendBytes(data, &len2, sizeof(len2));
        char16_t str2[] = u"က";
        appendBytes(data, str2, len2 * sizeof(char16_t));
        
        // Info section - keyboard name
        appendBytes(data, INFO_NAME, 4);
        uint16_t nameLen = 10 * 2; // "Test Keyboard" in UTF-16LE bytes
        appendBytes(data, &nameLen, sizeof(nameLen));
        char16_t name[] = u"Test Keyboard";
        appendBytes(data, name, 10 * sizeof(char16_t));
        
        // Rules section - one rule: "ka" => "က"
        // LHS: opSTRING, length=2, "ka"
        uint16_t lhsLen = 8;  // 4 uint16_t values * 2 bytes each
        appendBytes(data, &lhsLen, sizeof(lhsLen));
        uint16_t lhs[] = {OP_STRING, 2, 'k', 'a'};
        appendBytes(data, lhs, 4 * sizeof(uint16_t));
        
        // RHS: opSTRING, length=1, "က"
        uint16_t rhsLen = 6;  // 3 uint16_t values * 2 bytes each
        appendBytes(data, &rhsLen, sizeof(rhsLen));
        uint16_t rhs[] = {OP_STRING, 1, 0x1000}; // က = U+1000
        appendBytes(data, rhs, 3 * sizeof(uint16_t));
        
        return data;
    }
    
private:
    static void appendBytes(std::vector<uint8_t>& data, const void* src, size_t len) {
        const uint8_t* bytes = static_cast<const uint8_t*>(src);
        data.insert(data.end(), bytes, bytes + len);
    }
};

TEST(EngineIntegrationTest, BasicKeyProcessing) {
    KeyMagicEngine engine;
    
    // Create test KM2 data
    auto km2Data = TestKM2Builder::buildMinimalKM2();
    
    // Load keyboard
    Result result = engine.loadKeyboardFromMemory(km2Data.data(), km2Data.size());
    EXPECT_EQ(result, Result::Success);
    EXPECT_TRUE(engine.hasKeyboard());
    
    // Test processing 'k' key
    Input inputK(0, 'k', Modifiers());
    Output outputK = engine.processKey(inputK);
    EXPECT_EQ(outputK.action, ActionType::Insert);
    EXPECT_EQ(outputK.text, "k");
    EXPECT_EQ(outputK.composingText, "k");
    
    // Test processing 'a' key (should trigger rule "ka" => "က")
    Input inputA(0, 'a', Modifiers());
    Output outputA = engine.processKey(inputA);
    EXPECT_EQ(outputA.action, ActionType::BackspaceDeleteAndInsert);
    EXPECT_EQ(outputA.deleteCount, 1);
    EXPECT_EQ(outputA.text, "က");
    EXPECT_EQ(outputA.composingText, "က");
    
    // Reset and test again
    engine.reset();
    EXPECT_EQ(engine.getComposition(), "");
    
    // Set composition manually
    engine.setComposition("test");
    EXPECT_EQ(engine.getComposition(), "test");
}

TEST(EngineIntegrationTest, NoKeyboardLoaded) {
    KeyMagicEngine engine;
    
    EXPECT_FALSE(engine.hasKeyboard());
    EXPECT_EQ(engine.getKeyboardName(), "");
    EXPECT_EQ(engine.getKeyboardDescription(), "");
    
    // Process key without keyboard
    Input input(0, 'a', Modifiers());
    Output output = engine.processKey(input);
    EXPECT_FALSE(output.isProcessed);
}

TEST(EngineIntegrationTest, InvalidKM2Data) {
    KeyMagicEngine engine;
    
    // Try to load invalid data
    uint8_t invalidData[] = {0x00, 0x01, 0x02, 0x03};
    Result result = engine.loadKeyboardFromMemory(invalidData, sizeof(invalidData));
    EXPECT_EQ(result, Result::ErrorInvalidFormat);
    EXPECT_FALSE(engine.hasKeyboard());
}