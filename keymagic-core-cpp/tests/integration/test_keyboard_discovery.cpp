#include <gtest/gtest.h>
#include "../common/test_utils.h"
#include <iostream>

class KeyboardDiscoveryTest : public ::testing::Test {
protected:
    void SetUp() override {
        // This test verifies our keyboard discovery utilities work correctly
    }
};

TEST_F(KeyboardDiscoveryTest, CanFindKeyboardsDirectory) {
    // Test that we can find the keyboards/bundled directory
    auto keyboardsDir = keymagic_test::KeyboardFinder::findKeyboardsDirectory();
    
    if (keyboardsDir) {
        std::cout << "Found keyboards directory: " << keyboardsDir->string() << std::endl;
        EXPECT_TRUE(std::filesystem::exists(*keyboardsDir));
        EXPECT_TRUE(std::filesystem::is_directory(*keyboardsDir));
    } else {
        // Print helpful debugging info if not found
        std::cout << keymagic_test::getKeyboardLoadingHelp() << std::endl;
        FAIL() << "Could not find keyboards/bundled directory";
    }
}

TEST_F(KeyboardDiscoveryTest, CanListKeyboardFiles) {
    // Test that we can find keyboard files
    auto keyboards = keymagic_test::KeyboardFinder::getAllKeyboardFiles();
    
    std::cout << "Found " << keyboards.size() << " keyboard files:" << std::endl;
    for (const auto& keyboard : keyboards) {
        std::cout << "  - " << keyboard.filename().string() << std::endl;
        EXPECT_TRUE(std::filesystem::exists(keyboard));
        EXPECT_EQ(keyboard.extension().string(), ".km2");
    }
    
    // We expect at least the Pyidaungsu keyboard
    EXPECT_GT(keyboards.size(), 0) << "No keyboard files found";
}

TEST_F(KeyboardDiscoveryTest, CanFindPyidaungsuKeyboard) {
    // Test that we can specifically find the Pyidaungsu keyboard
    auto pyidaungsuPath = keymagic_test::KeyboardFinder::findKeyboardFile("Pyidaungsu MM.km2");
    
    if (pyidaungsuPath) {
        std::cout << "Found Pyidaungsu keyboard: " << pyidaungsuPath->string() << std::endl;
        EXPECT_TRUE(std::filesystem::exists(*pyidaungsuPath));
        EXPECT_TRUE(std::filesystem::is_regular_file(*pyidaungsuPath));
        EXPECT_EQ(pyidaungsuPath->filename().string(), "Pyidaungsu MM.km2");
    } else {
        std::cout << keymagic_test::getKeyboardLoadingHelp() << std::endl;
        FAIL() << "Could not find Pyidaungsu MM.km2 keyboard file";
    }
}