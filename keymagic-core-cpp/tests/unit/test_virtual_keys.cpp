#include <gtest/gtest.h>
#include <keymagic/virtual_keys.h>

using namespace keymagic;

TEST(VirtualKeysTest, KeyCategories) {
    EXPECT_TRUE(isLetterKey(VirtualKey::KeyA));
    EXPECT_TRUE(isLetterKey(VirtualKey::KeyZ));
    EXPECT_FALSE(isLetterKey(VirtualKey::Key0));
    
    EXPECT_TRUE(isNumberKey(VirtualKey::Key0));
    EXPECT_TRUE(isNumberKey(VirtualKey::Key9));
    EXPECT_FALSE(isNumberKey(VirtualKey::KeyA));
    
    EXPECT_TRUE(isNumpadKey(VirtualKey::Numpad0));
    EXPECT_TRUE(isNumpadKey(VirtualKey::Divide));
    EXPECT_FALSE(isNumpadKey(VirtualKey::Key0));
    
    EXPECT_TRUE(isFunctionKey(VirtualKey::F1));
    EXPECT_TRUE(isFunctionKey(VirtualKey::F12));
    EXPECT_FALSE(isFunctionKey(VirtualKey::Escape));
    
    EXPECT_TRUE(isModifierKey(VirtualKey::Shift));
    EXPECT_TRUE(isModifierKey(VirtualKey::Control));
    EXPECT_TRUE(isModifierKey(VirtualKey::Alt));
    EXPECT_FALSE(isModifierKey(VirtualKey::KeyA));
}

TEST(VirtualKeysTest, Validation) {
    EXPECT_TRUE(VirtualKeyHelper::isValid(static_cast<uint16_t>(VirtualKey::Null)));
    EXPECT_TRUE(VirtualKeyHelper::isValid(static_cast<uint16_t>(VirtualKey::KeyA)));
    EXPECT_TRUE(VirtualKeyHelper::isValid(static_cast<uint16_t>(VirtualKey::MaxValue)));
    
    EXPECT_FALSE(VirtualKeyHelper::isValid(0));
    EXPECT_FALSE(VirtualKeyHelper::isValid(1000));
}