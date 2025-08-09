#include <gtest/gtest.h>
#include <keymagic/keymagic.h>
#include <keymagic/engine.h>
#include <keymagic/types.h>
#include <keymagic/virtual_keys.h>
#include "../common/test_utils.h"
#include <iostream>
#include <fstream>

using namespace keymagic;

class PyidaungsuKeystrokeTest : public ::testing::Test {
protected:
    void SetUp() override {
        // Use the test utility to find the Pyidaungsu keyboard
        auto keyboardPath = keymagic_test::KeyboardFinder::findKeyboardFile("Pyidaungsu MM.km2");
        
        if (!keyboardPath) {
            // Provide helpful debugging information
            std::string helpMessage = keymagic_test::getKeyboardLoadingHelp();
            FAIL() << "Could not find Pyidaungsu MM.km2 keyboard file\n\n" << helpMessage;
        }
        
        // Load the keyboard
        Result result = engine.loadKeyboard(keyboardPath->string());
        ASSERT_EQ(result, Result::Success) 
            << "Failed to load Pyidaungsu keyboard from: " << keyboardPath->string() 
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

TEST_F(PyidaungsuKeystrokeTest, BasicConsonantMapping) {
    // Test basic consonant mappings
    // 'u' should produce 'က' (U+1000)
    Input input(0, static_cast<char32_t>('u'), Modifiers());
    auto result = engine.processKey(input);
    EXPECT_TRUE(result.isProcessed);
    EXPECT_EQ(result.composingText, std::string("က"));
    EXPECT_EQ(result.action, ActionType::Insert);
}

TEST_F(PyidaungsuKeystrokeTest, VowelEReordering) {
    // Test the famous Rule 39 - vowel E reordering
    // Type 'a' (vowel E), then 'u' (က consonant)
    // Should reorder to produce 'ကေ' (consonant + vowel E)
    
    // Reset engine first
    engine.reset();
    
    // Type 'a' - should produce filler + vowel E
    Input input_a(0, static_cast<char32_t>('a'), Modifiers());
    auto result1 = engine.processKey(input_a);
    EXPECT_TRUE(result1.isProcessed);
    // Should contain vowel E (U+1031) and possibly a filler character
    EXPECT_TRUE(result1.composingText.find("\u1031") != std::string::npos);
    
    // Type 'u' - should reorder to consonant + vowel E
    Input input_u(0, static_cast<char32_t>('u'), Modifiers());
    auto result2 = engine.processKey(input_u);
    EXPECT_TRUE(result2.isProcessed);
    
    // Final result should be 'ကေ' (Ka + E)
    EXPECT_EQ(result2.composingText, std::string("ကေ")) << "Expected Ka+E, got: " << result2.composingText;
    EXPECT_EQ(result2.action, ActionType::BackspaceDeleteAndInsert);
}

TEST_F(PyidaungsuKeystrokeTest, MultipleConsonants) {
    // Test typing multiple consonants
    engine.reset();
    
    // Type 'u' for က
    Input input_u(0, static_cast<char32_t>('u'), Modifiers());
    auto result1 = engine.processKey(input_u);
    EXPECT_EQ(result1.composingText, std::string("က"));
    
    // Type 'i' for င  
    Input input_i(0, static_cast<char32_t>('i'), Modifiers());
    auto result2 = engine.processKey(input_i);
    EXPECT_EQ(result2.composingText, std::string("ကင"));
    EXPECT_EQ(result2.action, ActionType::Insert);
}

TEST_F(PyidaungsuKeystrokeTest, VowelAfterConsonant) {
    // Test vowel after consonant
    engine.reset();
    
    // Type 'u' for က
    Input input_u(0, static_cast<char32_t>('u'), Modifiers());
    auto result1 = engine.processKey(input_u);
    EXPECT_EQ(result1.composingText, std::string("က"));
    
    // Type 'k' for ု vowel  
    Input input_k(0, static_cast<char32_t>('k'), Modifiers());
    auto result2 = engine.processKey(input_k);
    EXPECT_EQ(result2.composingText, std::string("ကု"));
    EXPECT_EQ(result2.action, ActionType::Insert);
}

TEST_F(PyidaungsuKeystrokeTest, ComplexVowelEReordering) {
    // Test more complex vowel E reordering scenarios
    engine.reset();
    
    // Type 'a' (vowel E), then 'u' (က), then 's' (medial ya-yit)
    Input input_a(0, static_cast<char32_t>('a'), Modifiers());
    auto result1 = engine.processKey(input_a);
    EXPECT_TRUE(result1.isProcessed);
    
    Input input_u(0, static_cast<char32_t>('u'), Modifiers());
    auto result2 = engine.processKey(input_u);
    EXPECT_TRUE(result2.isProcessed);
    EXPECT_EQ(result2.composingText, std::string("ကေ"));
    
    Input input_s(0, static_cast<char32_t>('s'), Modifiers());
    auto result3 = engine.processKey(input_s);
    EXPECT_TRUE(result3.isProcessed);
    // Should handle medial characters properly with vowel E
}

TEST_F(PyidaungsuKeystrokeTest, EngineReset) {
    // Test engine reset functionality
    Input input_u(0, static_cast<char32_t>('u'), Modifiers());
    engine.processKey(input_u);
    EXPECT_FALSE(engine.getComposition().empty());
    
    engine.reset();
    EXPECT_TRUE(engine.getComposition().empty());
}

TEST_F(PyidaungsuKeystrokeTest, DocumentationExamples_KyuWord) {
    // Test "ကျူ" (kyu) - keystroke sequence: u s l
    engine.reset();
    
    // Type 'u' for က
    Input input_u(0, static_cast<char32_t>('u'), Modifiers());
    auto result1 = engine.processKey(input_u);
    EXPECT_EQ(result1.composingText, std::string("က"));
    
    // Type 's' for ya-yit medial (ျ)
    Input input_s(0, static_cast<char32_t>('s'), Modifiers());
    auto result2 = engine.processKey(input_s);
    EXPECT_TRUE(result2.composingText.find("ျ") != std::string::npos);
    
    // Type 'l' for ူ vowel
    Input input_l(0, static_cast<char32_t>('l'), Modifiers());
    auto result3 = engine.processKey(input_l);
    EXPECT_EQ(result3.composingText, std::string("ကျူ"));
}

TEST_F(PyidaungsuKeystrokeTest, DocumentationExamples_KyaungWord) {
    // Test "ကျောင်း" (kyaung) - keystroke sequence: a u s m i f ;
    engine.reset();
    
    // Type 'a' (vowel E first)
    Input input_a(0, static_cast<char32_t>('a'), Modifiers());
    auto result1 = engine.processKey(input_a);
    EXPECT_TRUE(result1.isProcessed);
    
    // Type 'u' (က consonant) - should reorder
    Input input_u(0, static_cast<char32_t>('u'), Modifiers());
    auto result2 = engine.processKey(input_u);
    EXPECT_EQ(result2.composingText, std::string("ကေ"));
    
    // Continue with 's' for ya-yit
    Input input_s(0, static_cast<char32_t>('s'), Modifiers());
    auto result3 = engine.processKey(input_s);
    
    // Type 'm' for ါ vowel
    Input input_m(0, static_cast<char32_t>('m'), Modifiers());
    auto result4 = engine.processKey(input_m);
    
    // The sequence should build towards "ကျောင်း"
    EXPECT_TRUE(result4.isProcessed);
}

TEST_F(PyidaungsuKeystrokeTest, DocumentationExamples_MyoWord) {
    // Test "မြို့" (myo) - keystroke sequence: r j d k h  
    engine.reset();
    
    // Type 'r' for မ
    Input input_r(0, static_cast<char32_t>('r'), Modifiers());
    auto result1 = engine.processKey(input_r);
    EXPECT_EQ(result1.composingText, std::string("မ"));
    
    // Type 'j' for ra-yit medial (ြ)
    Input input_j(0, static_cast<char32_t>('j'), Modifiers());
    auto result2 = engine.processKey(input_j);
    EXPECT_TRUE(result2.composingText.find("ြ") != std::string::npos);
    
    // Continue building the word
    Input input_d(0, static_cast<char32_t>('d'), Modifiers());
    auto result3 = engine.processKey(input_d);
    
    Input input_k(0, static_cast<char32_t>('k'), Modifiers());
    auto result4 = engine.processKey(input_k);
    
    Input input_h(0, static_cast<char32_t>('h'), Modifiers());
    auto result5 = engine.processKey(input_h);
    
    // Final result should be "မြို့"
    EXPECT_EQ(result5.composingText, std::string("မြို့"));
}

TEST_F(PyidaungsuKeystrokeTest, DocumentationExamples_StackedConsonants) {
    // Test "စက္කူ" (sak-ku) with stacked consonants - keystroke: p u F u l
    engine.reset();
    
    // Type 'p' for စ
    Input input_p(0, static_cast<char32_t>('p'), Modifiers());
    auto result1 = engine.processKey(input_p);
    EXPECT_EQ(result1.composingText, std::string("စ"));
    
    // Type 'u' for က
    Input input_u1(0, static_cast<char32_t>('u'), Modifiers());
    auto result2 = engine.processKey(input_u1);
    EXPECT_EQ(result2.composingText, std::string("စက"));
    
    // Type 'F' for stacking virama (္)
    Input input_F(0, static_cast<char32_t>('F'), Modifiers());
    auto result3 = engine.processKey(input_F);
    EXPECT_TRUE(result3.composingText.find("္") != std::string::npos);
    
    // Type 'u' for က again (stacked)
    Input input_u2(0, static_cast<char32_t>('u'), Modifiers());
    auto result4 = engine.processKey(input_u2);
    
    // Type 'l' for ူ vowel
    Input input_l(0, static_cast<char32_t>('l'), Modifiers());
    auto result5 = engine.processKey(input_l);
    
    // Should produce "စက္ကူ
    EXPECT_EQ(result5.composingText, std::string("စက္ကူ"));
}

TEST_F(PyidaungsuKeystrokeTest, KeyboardMetadata) {
    // Test all keyboard metadata fields
    
    // Basic metadata should be present
    std::string name = engine.getKeyboardName();
    std::string description = engine.getKeyboardDescription();
    
    EXPECT_FALSE(name.empty()) << "Keyboard name should not be empty";
    EXPECT_FALSE(description.empty()) << "Keyboard description should not be empty";
    
    // Verify expected values for Pyidaungsu keyboard
    EXPECT_EQ(name, "Pyidaungsu MM") << "Expected keyboard name 'Pyidaungsu MM'";
    EXPECT_EQ(description, "Burmese Layout") << "Expected description 'Burmese Layout'";
    
    // Note: The internal Engine class doesn't have getKeyboardHotkey() 
    // We can test it through KeyMagicEngine wrapper in a separate test
    
    // Access raw KM2 file for additional metadata
    // Note: getKeyboard() is an internal method in Engine class
    // For production code, this would not be exposed
    const keymagic::KM2File* km2 = nullptr;
    
    // Create a separate Engine instance to access internal details
    keymagic::Engine internalEngine;
    auto keyboardPath = keymagic_test::KeyboardFinder::findKeyboardFile("Pyidaungsu MM.km2");
    if (keyboardPath) {
        internalEngine.loadKeyboardFromPath(keyboardPath->string());
        km2 = internalEngine.getKeyboard();
    }
    
    ASSERT_NE(km2, nullptr) << "KM2 file should be loaded";
    
    // Check version information
    EXPECT_EQ(km2->getMajorVersion(), 1) << "Expected major version 1";
    EXPECT_GE(km2->getMinorVersion(), 4) << "Expected minor version >= 4";
    
    // Verify info section exists (v1.4+)
    EXPECT_TRUE(km2->hasInfoSection()) << "Info section should exist for v1.4+";
    
    // Check font family metadata
    std::string fontFamily = km2->metadata.getFontFamily();
    if (!fontFamily.empty()) {
        std::cout << "Recommended font: " << fontFamily << std::endl;
    }
    
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
    
    // Verify expected layout options for Pyidaungsu
    // Based on actual KM2 file settings:
    EXPECT_FALSE(layoutOptions.getTrackCaps()) << "Pyidaungsu does not track CAPSLOCK";
    EXPECT_FALSE(layoutOptions.getAutoBksp()) << "Pyidaungsu does not use smart backspace";
    EXPECT_FALSE(layoutOptions.getEat()) << "Pyidaungsu does not eat unused keys";
    EXPECT_TRUE(layoutOptions.getPosBased()) << "Pyidaungsu is US layout based";
    EXPECT_TRUE(layoutOptions.getRightAlt()) << "Pyidaungsu treats Ctrl+Alt as AltGr";
    
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

TEST_F(PyidaungsuKeystrokeTest, KeyboardLoadError) {
    // Test loading a non-existent keyboard
    keymagic::Engine errorEngine;
    
    auto result = errorEngine.loadKeyboardFromPath("non_existent_keyboard.km2");
    EXPECT_NE(result, keymagic::Result::Success) << "Loading non-existent file should fail";
    
    // Engine should handle operations gracefully without a keyboard
    EXPECT_TRUE(errorEngine.getKeyboardName().empty()) << "Name should be empty without keyboard";
    EXPECT_TRUE(errorEngine.getKeyboardDescription().empty()) << "Description should be empty without keyboard";
    EXPECT_FALSE(errorEngine.hasKeyboard()) << "hasKeyboard() should return false";
    
    // Processing keys without a keyboard should return None output
    Input input('a', static_cast<char32_t>('a'), Modifiers());
    auto output = errorEngine.processKey(input);
    EXPECT_EQ(output.action, ActionType::None) << "Should return None action without keyboard";
}

TEST_F(PyidaungsuKeystrokeTest, KeyMagicEngineAPIMetadata) {
    // Test the public KeyMagicEngine API (wrapper class)
    // This tests the high-level API that external applications would use
    
    KeyMagicEngine publicEngine;
    
    // Load keyboard using public API
    auto keyboardPath = keymagic_test::KeyboardFinder::findKeyboardFile("Pyidaungsu MM.km2");
    ASSERT_TRUE(keyboardPath.has_value()) << "Could not find Pyidaungsu keyboard";
    
    Result loadResult = publicEngine.loadKeyboard(keyboardPath->string());
    ASSERT_EQ(loadResult, Result::Success) << "Failed to load keyboard via public API";
    
    // Test hasKeyboard method
    EXPECT_TRUE(publicEngine.hasKeyboard()) << "Public API hasKeyboard() should return true";
    
    // Test metadata retrieval via public API
    std::string name = publicEngine.getKeyboardName();
    std::string description = publicEngine.getKeyboardDescription();
    
    EXPECT_EQ(name, "Pyidaungsu MM") << "Public API should return correct keyboard name";
    EXPECT_EQ(description, "Burmese Layout") << "Public API should return correct description";
    
    // Test version string
    std::string version = KeyMagicEngine::getVersion();
    EXPECT_FALSE(version.empty()) << "Version string should not be empty";
    EXPECT_EQ(version, "1.0.0") << "Expected version 1.0.0";
    
    // Test composition management
    publicEngine.setComposition("test");
    EXPECT_EQ(publicEngine.getComposition(), "test") << "Composition should be set correctly";
    
    publicEngine.reset();
    EXPECT_EQ(publicEngine.getComposition(), "") << "Composition should be empty after reset";
    
    // Test key processing via public API
    Output output = publicEngine.processWindowsKey(
        0,  // VK code
        'k', // character
        Modifiers(false, false, false) // no modifiers
    );
    
    EXPECT_EQ(output.action, ActionType::Insert) << "Should insert character";
    EXPECT_FALSE(output.text.empty()) << "Should produce output text";
    
    std::cout << "\n=== Public API Test Summary ===" << std::endl;
    std::cout << "API Version: " << version << std::endl;
    std::cout << "Keyboard: " << name << std::endl;
    std::cout << "Description: " << description << std::endl;
    std::cout << "Has Keyboard: " << (publicEngine.hasKeyboard() ? "Yes" : "No") << std::endl;
    std::cout << "==============================" << std::endl;
}