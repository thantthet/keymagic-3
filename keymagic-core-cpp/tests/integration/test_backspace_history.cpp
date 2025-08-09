//! Tests for backspace history functionality (ported from Rust tests)

#include <gtest/gtest.h>
#include <keymagic/engine.h>
#include <keymagic/virtual_keys.h>
#include <keymagic/km2_format.h>
#include "../common/test_utils.h"

using namespace keymagic;
using namespace keymagic_test;

class BackspaceHistoryTest : public ::testing::Test {
protected:
    void SetUp() override {
        // Create basic engine for testing
        engine = std::make_unique<Engine>();
    }

    std::unique_ptr<Engine> engine;
    
    // Helper to create input for backspace
    Input createBackspaceInput() {
        return Input(static_cast<int>(VirtualKey::Back), 0, Modifiers());
    }
    
    // Helper to create input for character
    Input createCharInput(char ch) {
        return Input(0, static_cast<char32_t>(ch), Modifiers());
    }
};

TEST_F(BackspaceHistoryTest, BackspaceHistoryWithAutoBksp) {
    // Create a keyboard with auto_bksp enabled
    auto km2 = createBasicKM2WithOptions(true, false, false); // auto_bksp=true
    
    ASSERT_EQ(engine->loadKeyboard(std::move(km2)), Result::Success);
    
    // Type some characters
    auto result1 = engine->processKey(createCharInput('a'));
    EXPECT_EQ(result1.composingText, "a");
    
    auto result2 = engine->processKey(createCharInput('b'));  
    EXPECT_EQ(result2.composingText, "ab");
    
    auto result3 = engine->processKey(createCharInput('c'));
    EXPECT_EQ(result3.composingText, "abc");
    
    // Backspace should restore previous state
    auto backspaceResult1 = engine->processKey(createBackspaceInput());
    EXPECT_EQ(backspaceResult1.composingText, "ab");
    EXPECT_TRUE(backspaceResult1.isProcessed);
    
    // Another backspace
    auto backspaceResult2 = engine->processKey(createBackspaceInput());
    EXPECT_EQ(backspaceResult2.composingText, "a");
    EXPECT_TRUE(backspaceResult2.isProcessed);
    
    // Another backspace
    auto backspaceResult3 = engine->processKey(createBackspaceInput());
    EXPECT_EQ(backspaceResult3.composingText, "");
    EXPECT_TRUE(backspaceResult3.isProcessed);
}

TEST_F(BackspaceHistoryTest, BackspaceWithoutAutoBksp) {
    // Create a keyboard with auto_bksp disabled (default)
    auto km2 = createBasicKM2WithOptions(false, false, false); // auto_bksp=false
    
    ASSERT_EQ(engine->loadKeyboard(std::move(km2)), Result::Success);
    
    // Type some characters
    engine->processKey(createCharInput('a'));
    engine->processKey(createCharInput('b'));
    engine->processKey(createCharInput('c'));
    EXPECT_EQ(engine->getComposingTextUtf8(), "abc");
    
    // Backspace with auto_bksp disabled should simply delete last character
    auto backspaceResult1 = engine->processKey(createBackspaceInput());
    EXPECT_EQ(backspaceResult1.composingText, "ab"); // Should delete 'c'
    EXPECT_TRUE(backspaceResult1.isProcessed);
    EXPECT_EQ(backspaceResult1.action, ActionType::BackspaceDelete);
    
    // Another backspace
    auto backspaceResult2 = engine->processKey(createBackspaceInput());
    EXPECT_EQ(backspaceResult2.composingText, "a"); // Should delete 'b'
    EXPECT_TRUE(backspaceResult2.isProcessed);
}

TEST_F(BackspaceHistoryTest, ProcessKeyTestWithBackspaceHistory) {
    // Create a keyboard with auto_bksp enabled
    auto km2 = createBasicKM2WithOptions(true, false, false); // auto_bksp=true
    
    ASSERT_EQ(engine->loadKeyboard(std::move(km2)), Result::Success);
    
    // Type some characters
    engine->processKey(createCharInput('a'));
    engine->processKey(createCharInput('b'));
    EXPECT_EQ(engine->getComposingTextUtf8(), "ab");
    
    // Test mode backspace should work with history
    auto testOutput = engine->testProcessKey(createBackspaceInput());
    EXPECT_EQ(testOutput.composingText, "a");
    EXPECT_TRUE(testOutput.isProcessed);
    
    // Original state should still be "ab"
    EXPECT_EQ(engine->getComposingTextUtf8(), "ab");
}

TEST_F(BackspaceHistoryTest, BackspaceHistoryWithRules) {
    // Create a keyboard with a rule "ka" => "က" and auto_bksp enabled
    auto km2 = createKM2WithRule("ka", "က", true, false, false); // auto_bksp=true
    
    ASSERT_EQ(engine->loadKeyboard(std::move(km2)), Result::Success);
    
    // Type 'k'
    auto result1 = engine->processKey(createCharInput('k'));
    EXPECT_EQ(result1.composingText, "k");
    
    // Type 'a' -> should trigger rule "ka" => "က"
    auto result2 = engine->processKey(createCharInput('a'));
    EXPECT_EQ(result2.composingText, "က");
    
    // Backspace should restore to "k" (the state before the transformation)
    auto backspaceResult = engine->processKey(createBackspaceInput());
    EXPECT_EQ(backspaceResult.composingText, "k");
    EXPECT_TRUE(backspaceResult.isProcessed);
    
    // Another backspace should clear everything
    auto backspaceResult2 = engine->processKey(createBackspaceInput());
    EXPECT_EQ(backspaceResult2.composingText, "");
    EXPECT_TRUE(backspaceResult2.isProcessed);
}

TEST_F(BackspaceHistoryTest, BackspaceHistoryNotRecordedForBackspace) {
    // Create a keyboard with auto_bksp enabled
    auto km2 = createBasicKM2WithOptions(true, false, false); // auto_bksp=true
    
    ASSERT_EQ(engine->loadKeyboard(std::move(km2)), Result::Success);
    
    // Type characters
    engine->processKey(createCharInput('a'));
    engine->processKey(createCharInput('b'));  
    engine->processKey(createCharInput('c'));
    EXPECT_EQ(engine->getComposingTextUtf8(), "abc");
    
    // First backspace restores "ab"
    engine->processKey(createBackspaceInput());
    EXPECT_EQ(engine->getComposingTextUtf8(), "ab");
    
    // Type new character
    engine->processKey(createCharInput('d'));
    EXPECT_EQ(engine->getComposingTextUtf8(), "abd");
    
    // Backspace should restore to "ab" (not "abc" since that backspace wasn't recorded)
    auto backspaceResult = engine->processKey(createBackspaceInput());
    EXPECT_EQ(backspaceResult.composingText, "ab");
    EXPECT_TRUE(backspaceResult.isProcessed);
}

TEST_F(BackspaceHistoryTest, HistoryMaxSize) {
    // Create a keyboard with auto_bksp enabled
    auto km2 = createBasicKM2WithOptions(true, false, false); // auto_bksp=true
    
    ASSERT_EQ(engine->loadKeyboard(std::move(km2)), Result::Success);
    
    // Type more characters than max history size (assume 20 like in Rust)
    for (int i = 0; i < 25; i++) {
        char ch = 'a' + (i % 26);
        engine->processKey(createCharInput(ch));
    }
    
    size_t lengthAfterTyping = engine->getComposingTextUtf8().size();
    EXPECT_EQ(lengthAfterTyping, 25);
    
    // Should be able to backspace up to max_history_size times
    for (int i = 0; i < 20; i++) {
        engine->processKey(createBackspaceInput());
    }
    
    // After 20 backspaces, we should have 5 characters left (25 - 20)
    EXPECT_EQ(engine->getComposingTextUtf8().size(), 5);
    
    // Further backspaces should delete one character at a time
    engine->processKey(createBackspaceInput());
    EXPECT_EQ(engine->getComposingTextUtf8().size(), 4);
}

TEST_F(BackspaceHistoryTest, HistoryClearedOnSetComposingText) {
    // Create a keyboard with auto_bksp enabled  
    auto km2 = createBasicKM2WithOptions(true, false, false); // auto_bksp=true
    
    ASSERT_EQ(engine->loadKeyboard(std::move(km2)), Result::Success);
    
    // Type some characters
    engine->processKey(createCharInput('a'));
    engine->processKey(createCharInput('b'));
    EXPECT_EQ(engine->getComposingTextUtf8(), "ab");
    
    // Set composing text externally (clears history)  
    engine->setComposingText("xyz");
    EXPECT_EQ(engine->getComposingTextUtf8(), "xyz");
    
    // Backspace should delete one character, not restore history
    auto backspaceResult = engine->processKey(createBackspaceInput());
    EXPECT_EQ(backspaceResult.composingText, "xy");
    EXPECT_TRUE(backspaceResult.isProcessed);
}