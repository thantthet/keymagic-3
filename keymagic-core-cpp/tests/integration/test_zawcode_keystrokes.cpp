#include <gtest/gtest.h>
#include <keymagic/keymagic.h>
#include <keymagic/engine.h>
#include <keymagic/types.h>
#include <keymagic/virtual_keys.h>
#include "../common/test_utils.h"
#include <iostream>
#include <fstream>

using namespace keymagic;

class ZawCodeKeystrokeTest : public ::testing::Test {
protected:
    void SetUp() override {
        // Use the test utility to find the ZawCode keyboard
        auto keyboardPath = keymagic_test::KeyboardFinder::findKeyboardFile("ZawCode.km2");
        
        if (!keyboardPath) {
            // Provide helpful debugging information
            std::string helpMessage = keymagic_test::getKeyboardLoadingHelp();
            FAIL() << "Could not find ZawCode.km2 keyboard file\n\n" << helpMessage;
        }
        
        // Load the keyboard
        Result result = engine.loadKeyboard(keyboardPath->string());
        ASSERT_EQ(result, Result::Success) 
            << "Failed to load ZawCode keyboard from: " << keyboardPath->string() 
            << "\n\nTroubleshooting info:\n" << keymagic_test::getKeyboardLoadingHelp();
        
        // Verify keyboard is loaded
        ASSERT_TRUE(engine.hasKeyboard()) << "Keyboard loaded but hasKeyboard() returns false";
        
        // Print keyboard info for debugging
        std::cout << "\n=== Keyboard Loaded Successfully ===" << std::endl;
        std::cout << "Path: " << keyboardPath->string() << std::endl;
        std::cout << "Name: '" << engine.getKeyboardName() << "'" << std::endl;
        std::cout << "Description: '" << engine.getKeyboardDescription() << "'" << std::endl;
        std::cout << "=================================" << std::endl;
    }
    
    void TearDown() override {
        engine.reset();
    }
    
    KeyMagicEngine engine;
};

TEST_F(ZawCodeKeystrokeTest, KeyboardMetadata) {
    // Test all keyboard metadata fields
    
    // Basic metadata should be present
    std::string name = engine.getKeyboardName();
    std::string description = engine.getKeyboardDescription();
    
    EXPECT_FALSE(name.empty()) << "Keyboard name should not be empty";
    EXPECT_FALSE(description.empty()) << "Keyboard description should not be empty";
    
    // Verify expected values for ZawCode keyboard
    EXPECT_EQ(name, "ဇော်ကုဒ်") << "Expected keyboard name 'ဇော်ကုဒ်'";
    EXPECT_EQ(description, "A Unicode Keyboard Layout that can be type the same sequence just like normal ZawGyi layout but the output is Unicode.") 
        << "Expected correct description";
    
    // Create a separate Engine instance to access internal details
    keymagic::Engine internalEngine;
    auto keyboardPath = keymagic_test::KeyboardFinder::findKeyboardFile("ZawCode.km2");
    if (keyboardPath) {
        internalEngine.loadKeyboardFromPath(keyboardPath->string());
    }
    
    const keymagic::KM2File* km2 = internalEngine.getKeyboard();
    ASSERT_NE(km2, nullptr) << "KM2 file should be loaded";
    
    // Check version information
    EXPECT_EQ(km2->getMajorVersion(), 1) << "Expected major version 1";
    EXPECT_EQ(km2->getMinorVersion(), 5) << "Expected minor version 5";
    
    // Verify info section exists (v1.4+)
    EXPECT_TRUE(km2->hasInfoSection()) << "Info section should exist for v1.4+";
    
    // Check font family metadata
    std::string fontFamily = km2->metadata.getFontFamily();
    EXPECT_EQ(fontFamily, "TharLon") << "Expected TharLon font";
    
    // Check for icon data
    const auto* iconData = km2->metadata.getIcon();
    if (iconData && !iconData->empty()) {
        std::cout << "Icon data present: " << iconData->size() << " bytes" << std::endl;
    }
    
    // Test layout options
    const auto& layoutOptions = km2->getLayoutOptions();
    
    // Print layout options for debugging
    std::cout << "\n=== Layout Options ===" << std::endl;
    std::cout << "Track CAPSLOCK: " << (layoutOptions.getTrackCaps() ? "Yes" : "No") << std::endl;
    std::cout << "Smart Backspace: " << (layoutOptions.getAutoBksp() ? "Yes" : "No") << std::endl;
    std::cout << "Eat unused keys: " << (layoutOptions.getEat() ? "Yes" : "No") << std::endl;
    std::cout << "US layout based: " << (layoutOptions.getPosBased() ? "Yes" : "No") << std::endl;
    std::cout << "Treat Ctrl+Alt as AltGr: " << (layoutOptions.getRightAlt() ? "Yes" : "No") << std::endl;
    
    // Verify expected layout options for ZawCode
    // Based on the KM2 file header values
    EXPECT_FALSE(layoutOptions.getTrackCaps()) << "ZawCode does not track CAPSLOCK";
    EXPECT_TRUE(layoutOptions.getAutoBksp()) << "ZawCode uses smart backspace";
    EXPECT_FALSE(layoutOptions.getEat()) << "ZawCode does not eat unused keys";
    EXPECT_TRUE(layoutOptions.getPosBased()) << "ZawCode is US layout based";
    EXPECT_TRUE(layoutOptions.getRightAlt()) << "ZawCode treats Ctrl+Alt as AltGr";
    
    // Display summary
    std::cout << "\n=== Keyboard Summary ===" << std::endl;
    std::cout << "Name: " << name << std::endl;
    std::cout << "Description: " << description << std::endl;
    std::cout << "Version: " << static_cast<int>(km2->getMajorVersion()) 
              << "." << static_cast<int>(km2->getMinorVersion()) << std::endl;
    std::cout << "Number of rules: " << km2->rules.size() << std::endl;
    std::cout << "Number of strings: " << km2->strings.size() << std::endl;
    std::cout << "=======================" << std::endl;
}

TEST_F(ZawCodeKeystrokeTest, BasicConsonantMapping) {
    // Test basic consonant mappings
    // 'u' should produce 'က' (U+1000)
    // ZawCode uses VK codes, so we need to pass VK_KEY_U
    Input input(VirtualKey::KeyU, static_cast<char32_t>('u'), Modifiers());
    auto result = engine.processKey(input);
    EXPECT_TRUE(result.isProcessed);
    EXPECT_EQ(result.composingText, std::string("က"));
    EXPECT_EQ(result.action, ActionType::Insert);
}

TEST_F(ZawCodeKeystrokeTest, EngineReset) {
    // Test engine reset functionality
    Input input_u(VirtualKey::KeyU, static_cast<char32_t>('u'), Modifiers());
    engine.processKey(input_u);
    EXPECT_FALSE(engine.getComposition().empty());
    
    engine.reset();
    EXPECT_TRUE(engine.getComposition().empty());
}

// Test specific keystroke sequences for ZawCode keyboard

TEST_F(ZawCodeKeystrokeTest, TestKyaungWord) {
    // Test "ကြောင်း" (kyaung) - keystroke sequence: ajumif;
    engine.reset();
    
    // Type 'a' - VK_KEY_A
    Input input_a(VirtualKey::KeyA, static_cast<char32_t>('a'), Modifiers());
    auto result_a = engine.processKey(input_a);
    EXPECT_TRUE(result_a.isProcessed);
    std::cout << "After 'a': output='" << result_a.composingText << "', composition='" << engine.getComposition() << "'" << std::endl;
    
    // Type 'j' - VK_KEY_J
    Input input_j(VirtualKey::KeyJ, static_cast<char32_t>('j'), Modifiers());
    auto result_j = engine.processKey(input_j);
    EXPECT_TRUE(result_j.isProcessed);
    std::cout << "After 'j': output='" << result_j.composingText << "', composition='" << engine.getComposition() << "'" << std::endl;
    
    // Type 'u' - VK_KEY_U
    Input input_u(VirtualKey::KeyU, static_cast<char32_t>('u'), Modifiers());
    auto result_u = engine.processKey(input_u);
    EXPECT_TRUE(result_u.isProcessed);
    std::cout << "After 'u': output='" << result_u.composingText << "', composition='" << engine.getComposition() << "'" << std::endl;
    
    // Type 'm' - VK_KEY_M
    Input input_m(VirtualKey::KeyM, static_cast<char32_t>('m'), Modifiers());
    auto result_m = engine.processKey(input_m);
    EXPECT_TRUE(result_m.isProcessed);
    std::cout << "After 'm': output='" << result_m.composingText << "', composition='" << engine.getComposition() << "'" << std::endl;
    
    // Type 'i' - VK_KEY_I
    Input input_i(VirtualKey::KeyI, static_cast<char32_t>('i'), Modifiers());
    auto result_i = engine.processKey(input_i);
    EXPECT_TRUE(result_i.isProcessed);
    std::cout << "After 'i': output='" << result_i.composingText << "', composition='" << engine.getComposition() << "'" << std::endl;
    
    // Type 'f' - VK_KEY_F
    Input input_f(VirtualKey::KeyF, static_cast<char32_t>('f'), Modifiers());
    auto result_f = engine.processKey(input_f);
    EXPECT_TRUE(result_f.isProcessed);
    std::cout << "After 'f': output='" << result_f.composingText << "', composition='" << engine.getComposition() << "'" << std::endl;
    
    // Type ';' - VK_OEM_1 (semicolon/colon key)
    Input input_semicolon(VirtualKey::Oem1, static_cast<char32_t>(';'), Modifiers());
    auto result_semicolon = engine.processKey(input_semicolon);
    EXPECT_TRUE(result_semicolon.isProcessed);
    std::cout << "After ';': output='" << result_semicolon.composingText << "', composition='" << engine.getComposition() << "'" << std::endl;
    
    // Final result should be "ကြောင်း"
    // For ZawCode, we need to check the internal composition, not just the output
    EXPECT_EQ(engine.getComposition(), std::string("ကြောင်း")) 
        << "Expected 'ကြောင်း', got: " << engine.getComposition();
}

TEST_F(ZawCodeKeystrokeTest, TestVowelReorderingWithDelete) {
    // Test "ေြ" - keystroke sequence: aju [delete]
    // This tests vowel reordering behavior followed by deletion
    engine.reset();
    
    // Type 'a' - VK_KEY_A
    Input input_a(VirtualKey::KeyA, static_cast<char32_t>('a'), Modifiers());
    auto result_a = engine.processKey(input_a);
    EXPECT_TRUE(result_a.isProcessed);
    
    // Type 'j' - VK_KEY_J
    Input input_j(VirtualKey::KeyJ, static_cast<char32_t>('j'), Modifiers());
    auto result_j = engine.processKey(input_j);
    EXPECT_TRUE(result_j.isProcessed);
    
    // Type 'u' - VK_KEY_U
    Input input_u(VirtualKey::KeyU, static_cast<char32_t>('u'), Modifiers());
    auto result_u = engine.processKey(input_u);
    EXPECT_TRUE(result_u.isProcessed);
    
    // Now press backspace/delete
    Input input_delete(VirtualKey::Back, 0, Modifiers());
    auto result_delete = engine.processKey(input_delete);
    EXPECT_TRUE(result_delete.isProcessed);
    
    // Final result should be "ေြ" (vowel E + ra-yit medial)
    // Note: The exact characters are U+1031 (ေ) and U+103C (ြ)
    EXPECT_EQ(result_delete.composingText, std::string("ေြ")) 
        << "Expected 'ေြ', got: " << result_delete.composingText;
}

TEST_F(ZawCodeKeystrokeTest, TestSmartBackspaceAfterRepeatedWord) {
    // Test smart backspace behavior after typing the same word multiple times
    // Type "ကျောင်း" (kyaung) 10 times, then backspace should delete only last "း"
    engine.reset();
    
    // Helper lambda to type "ကျောင်း" once
    auto typeKyaung = [this]() {
        // Keystroke sequence: ajumif;
        engine.processKey(Input(VirtualKey::KeyA, static_cast<char32_t>('a'), Modifiers()));
        engine.processKey(Input(VirtualKey::KeyJ, static_cast<char32_t>('j'), Modifiers()));
        engine.processKey(Input(VirtualKey::KeyU, static_cast<char32_t>('u'), Modifiers()));
        engine.processKey(Input(VirtualKey::KeyM, static_cast<char32_t>('m'), Modifiers()));
        engine.processKey(Input(VirtualKey::KeyI, static_cast<char32_t>('i'), Modifiers()));
        engine.processKey(Input(VirtualKey::KeyF, static_cast<char32_t>('f'), Modifiers()));
        engine.processKey(Input(VirtualKey::Oem1, static_cast<char32_t>(';'), Modifiers()));
    };
    
    // Type "ကျောင်း" 10 times
    for (int i = 0; i < 10; i++) {
        typeKyaung();
    }
    
    // The composition should have "ကြောင်း" repeated 10 times
    // Note: ZawCode produces the correct Unicode form with "ြ" (U+103C) not "ျ" (U+103B)
    std::string expected = "";
    for (int i = 0; i < 10; i++) {
        expected += "ကြောင်း";
    }
    EXPECT_EQ(engine.getComposition(), expected) 
        << "Should have 'ကြောင်း' repeated 10 times";
    
    // Now press backspace
    Input input_backspace(VirtualKey::Back, 0, Modifiers());
    auto result = engine.processKey(input_backspace);
    
    // Should delete only the last "း" character
    std::string expectedAfterBackspace = "";
    for (int i = 0; i < 10; i++) {
        if (i == 9) {
            expectedAfterBackspace += "ကြောင်";  // Last word without "း"
        } else {
            expectedAfterBackspace += "ကြောင်း";
        }
    }
    
    EXPECT_EQ(engine.getComposition(), expectedAfterBackspace)
        << "Backspace should delete only the last 'း' character";
    EXPECT_TRUE(result.isProcessed);
    
    // Verify the action type - should be simple BackspaceDelete(1) 
    // since we're only deleting the last character
    EXPECT_EQ(result.action, ActionType::BackspaceDelete)
        << "Should be a simple backspace delete action";
}

TEST_F(ZawCodeKeystrokeTest, TestComplexWordSequence) {
    // Additional test for complex word formation
    // Test "မြန်မာ" (Myanmar) - keystroke sequence: rjef rm
    engine.reset();
    
    // Type 'r' for မ - VK_KEY_R
    Input input_r(VirtualKey::KeyR, static_cast<char32_t>('r'), Modifiers());
    auto result1 = engine.processKey(input_r);
    EXPECT_EQ(result1.composingText, std::string("မ"));
    
    // Type 'j' for ra-yit medial (ြ) - VK_KEY_J
    Input input_j(VirtualKey::KeyJ, static_cast<char32_t>('j'), Modifiers());
    auto result2 = engine.processKey(input_j);
    EXPECT_TRUE(result2.composingText.find("ြ") != std::string::npos);
    
    // Type 'e' for န - VK_KEY_E
    Input input_e(VirtualKey::KeyE, static_cast<char32_t>('e'), Modifiers());
    auto result3 = engine.processKey(input_e);
    EXPECT_TRUE(result3.isProcessed);
    
    // Type 'f' for ် - VK_KEY_F
    Input input_f(VirtualKey::KeyF, static_cast<char32_t>('f'), Modifiers());
    auto result4 = engine.processKey(input_f);
    EXPECT_TRUE(result4.isProcessed);
    
    // Type space - VK_SPACE
    Input input_space(VirtualKey::Space, static_cast<char32_t>(' '), Modifiers());
    auto result5 = engine.processKey(input_space);
    
    // Type 'r' for မ - VK_KEY_R
    Input input_r2(VirtualKey::KeyR, static_cast<char32_t>('r'), Modifiers());
    auto result6 = engine.processKey(input_r2);
    
    // Type 'm' for ာ - VK_KEY_M
    Input input_m(VirtualKey::KeyM, static_cast<char32_t>('m'), Modifiers());
    auto result7 = engine.processKey(input_m);
    
    // Should produce "မြန်မာ"
    EXPECT_EQ(result7.composingText, std::string("မြန်မာ"));
}