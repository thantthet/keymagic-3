#include <gtest/gtest.h>
#include <keymagic/keymagic.h>
#include <keymagic/km2_format.h>
#include "../../src/km2/loader.h"
#include <cstring>
#include <iostream>

using namespace keymagic;

TEST(EngineDebugTest, RuleLoading) {
    // Create minimal KM2 with one rule
    std::vector<uint8_t> data;
    
    // Header
    FileHeader header;
    std::memcpy(header.magicCode, KM2_MAGIC_CODE, 4);
    header.majorVersion = 1;
    header.minorVersion = 5;
    header.stringCount = 0;
    header.infoCount = 0;
    header.ruleCount = 1;
    header.layoutOptions = KM2LayoutOptions();
    
    data.insert(data.end(), reinterpret_cast<uint8_t*>(&header), 
                reinterpret_cast<uint8_t*>(&header) + sizeof(header));
    
    // Rule: "ka" => "က"
    // LHS: opSTRING, 2, 'k', 'a'
    uint16_t lhsLen = 8;  // 4 uint16_t values * 2 bytes
    data.insert(data.end(), reinterpret_cast<uint8_t*>(&lhsLen), 
                reinterpret_cast<uint8_t*>(&lhsLen) + sizeof(lhsLen));
    uint16_t lhs[] = {OP_STRING, 2, 'k', 'a'};
    data.insert(data.end(), reinterpret_cast<uint8_t*>(&lhs), 
                reinterpret_cast<uint8_t*>(&lhs) + sizeof(lhs));
    
    // RHS: opSTRING, 1, 'က'
    uint16_t rhsLen = 6;  // 3 uint16_t values * 2 bytes
    data.insert(data.end(), reinterpret_cast<uint8_t*>(&rhsLen), 
                reinterpret_cast<uint8_t*>(&rhsLen) + sizeof(rhsLen));
    uint16_t rhs[] = {OP_STRING, 1, 0x1000};
    data.insert(data.end(), reinterpret_cast<uint8_t*>(&rhs), 
                reinterpret_cast<uint8_t*>(&rhs) + sizeof(rhs));
    
    // Load KM2
    auto km2 = KM2Loader::loadFromMemory(data.data(), data.size());
    ASSERT_NE(km2, nullptr);
    ASSERT_EQ(km2->rules.size(), 1);
    
    // Debug output
    const auto& rule = km2->rules[0];
    std::cout << "Rule LHS opcodes (" << rule.lhs.size() << "): ";
    for (auto op : rule.lhs) {
        std::cout << std::hex << "0x" << op << " ";
    }
    std::cout << std::endl;
    
    std::cout << "Rule RHS opcodes (" << rule.rhs.size() << "): ";
    for (auto op : rule.rhs) {
        std::cout << std::hex << "0x" << op << " ";
    }
    std::cout << std::endl;
    
    // Load into engine
    KeyMagicEngine engine;
    Result result = engine.loadKeyboardFromMemory(data.data(), data.size());
    EXPECT_EQ(result, Result::Success);
    
    // Check keyboard was loaded
    EXPECT_TRUE(engine.hasKeyboard());
    std::cout << "Keyboard loaded successfully" << std::endl;
    
    // Test "k"
    Input inputK(0, 'k', Modifiers());
    Output outputK = engine.processKey(inputK);
    std::cout << "After 'k': action=" << static_cast<int>(outputK.action) 
              << ", text='" << outputK.text << "'"
              << ", composing='" << outputK.composingText << "'" << std::endl;
    
    // Test "a"
    Input inputA(0, 'a', Modifiers());
    Output outputA = engine.processKey(inputA);
    std::cout << "After 'a': action=" << static_cast<int>(outputA.action) 
              << ", text='" << outputA.text << "'"
              << ", composing='" << outputA.composingText << "'"
              << ", deleteCount=" << outputA.deleteCount << std::endl;
    
    // Check what we got
    if (outputA.composingText != "က") {
        std::cout << "ERROR: Expected 'က' but got '" << outputA.composingText << "'" << std::endl;
        std::cout << "Hex dump of composing text: ";
        for (unsigned char c : outputA.composingText) {
            std::cout << std::hex << "0x" << (int)c << " ";
        }
        std::cout << std::endl;
    }
    
    // Check if rule matched
    EXPECT_EQ(outputA.composingText, "က");
}