#include <gtest/gtest.h>
#include <keymagic/km2_format.h>
#include "../../src/km2/loader.h"
#include <cstring>
#include <vector>

using namespace keymagic;

TEST(KM2LoaderIntegrationTest, MinimalKM2) {
    // Create minimal valid KM2 file in memory
    std::vector<uint8_t> data;
    
    // Header
    FileHeader header;
    std::memcpy(header.magicCode, KM2_MAGIC_CODE, 4);
    header.majorVersion = 1;
    header.minorVersion = 5;
    header.stringCount = 0;  // No strings for minimal test
    header.infoCount = 0;    // No info
    header.ruleCount = 0;    // No rules
    header.layoutOptions = KM2LayoutOptions();
    
    // Write header
    data.insert(data.end(), reinterpret_cast<uint8_t*>(&header), 
                reinterpret_cast<uint8_t*>(&header) + sizeof(header));
    
    // Load and validate
    auto km2 = KM2Loader::loadFromMemory(data.data(), data.size());
    ASSERT_NE(km2, nullptr);
    EXPECT_TRUE(km2->isValid());
    EXPECT_EQ(km2->getMajorVersion(), 1);
    EXPECT_EQ(km2->getMinorVersion(), 5);
    EXPECT_EQ(km2->strings.size(), 0);
    EXPECT_EQ(km2->rules.size(), 0);
}

TEST(KM2LoaderIntegrationTest, WithOneRule) {
    std::vector<uint8_t> data;
    
    // Header
    FileHeader header;
    std::memcpy(header.magicCode, KM2_MAGIC_CODE, 4);
    header.majorVersion = 1;
    header.minorVersion = 5;
    header.stringCount = 0;
    header.infoCount = 0;
    header.ruleCount = 1;  // One rule
    header.layoutOptions = KM2LayoutOptions();
    
    // Write header
    data.insert(data.end(), reinterpret_cast<uint8_t*>(&header), 
                reinterpret_cast<uint8_t*>(&header) + sizeof(header));
    
    // Rule: "k" => "က" (using opSTRING)
    // LHS length (in bytes)
    uint16_t lhsLen = 6;  // 3 uint16_t values * 2 bytes each
    data.insert(data.end(), reinterpret_cast<uint8_t*>(&lhsLen), 
                reinterpret_cast<uint8_t*>(&lhsLen) + sizeof(lhsLen));
    
    // LHS opcodes
    uint16_t lhsOpcodes[] = {OP_STRING, 1, 'k'};
    data.insert(data.end(), reinterpret_cast<uint8_t*>(&lhsOpcodes), 
                reinterpret_cast<uint8_t*>(&lhsOpcodes) + sizeof(lhsOpcodes));
    
    // RHS length (in bytes)
    uint16_t rhsLen = 6;  // 3 uint16_t values * 2 bytes each
    data.insert(data.end(), reinterpret_cast<uint8_t*>(&rhsLen), 
                reinterpret_cast<uint8_t*>(&rhsLen) + sizeof(rhsLen));
    
    // RHS opcodes
    uint16_t rhsOpcodes[] = {OP_STRING, 1, 0x1000};  // က = U+1000
    data.insert(data.end(), reinterpret_cast<uint8_t*>(&rhsOpcodes), 
                reinterpret_cast<uint8_t*>(&rhsOpcodes) + sizeof(rhsOpcodes));
    
    // Load and validate
    auto km2 = KM2Loader::loadFromMemory(data.data(), data.size());
    ASSERT_NE(km2, nullptr);
    EXPECT_TRUE(km2->isValid());
    EXPECT_EQ(km2->rules.size(), 1);
    
    // Check rule content
    const auto& rule = km2->rules[0];
    EXPECT_EQ(rule.lhs.size(), 3);
    EXPECT_EQ(rule.lhs[0], OP_STRING);
    EXPECT_EQ(rule.lhs[1], 1);
    EXPECT_EQ(rule.lhs[2], 'k');
    
    EXPECT_EQ(rule.rhs.size(), 3);
    EXPECT_EQ(rule.rhs[0], OP_STRING);
    EXPECT_EQ(rule.rhs[1], 1);
    EXPECT_EQ(rule.rhs[2], 0x1000);
}