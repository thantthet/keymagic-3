#include <gtest/gtest.h>
#include <keymagic/keymagic.h>
#include <string>
#include <vector>
#include <filesystem>
#include <algorithm>
#include "../common/test_utils.h"

// Helper function to get a test KM2 file using KeyboardFinder
std::string getTestKM2File() {
    // Use KeyboardFinder to locate keyboards
    auto keyboards = keymagic_test::KeyboardFinder::getAllKeyboardFiles();
    
    // Filter out macOS metadata files (starting with ._)
    std::vector<std::filesystem::path> validKeyboards;
    for (const auto& kb : keyboards) {
        std::string filename = kb.filename().string();
        if (filename.substr(0, 2) != "._") {
            validKeyboards.push_back(kb);
        }
    }
    
    // Look for Pyidaungsu keyboard first (commonly available)
    for (const auto& kb : validKeyboards) {
        std::string filename = kb.filename().string();
        std::transform(filename.begin(), filename.end(), filename.begin(), ::tolower);
        if (filename.find("pyidaungsu") != std::string::npos) {
            return kb.string();
        }
    }
    
    // If Pyidaungsu not found, return the first available valid keyboard
    if (!validKeyboards.empty()) {
        return validKeyboards[0].string();
    }
    
    // No keyboards found
    return "";
}

TEST(KM2MetadataTest, LoadValidKM2File) {
    // Get a test KM2 file using KeyboardFinder
    std::string km2Path = getTestKM2File();
    
    if (km2Path.empty()) {
        GTEST_SKIP() << "No KM2 files found in standard locations";
    }
    
    Km2FileHandle* handle = keymagic_km2_load(km2Path.c_str());
    ASSERT_NE(handle, nullptr) << "Failed to load: " << km2Path;
    
    // Clean up
    keymagic_km2_free(handle);
}

TEST(KM2MetadataTest, LoadInvalidPath) {
    Km2FileHandle* handle = keymagic_km2_load("/invalid/path/file.km2");
    EXPECT_EQ(handle, nullptr);
}

TEST(KM2MetadataTest, LoadNullPath) {
    Km2FileHandle* handle = keymagic_km2_load(nullptr);
    EXPECT_EQ(handle, nullptr);
}

TEST(KM2MetadataTest, GetKeyboardName) {
    std::string km2Path = getTestKM2File();
    
    if (km2Path.empty()) {
        GTEST_SKIP() << "No KM2 files found in standard locations";
    }
    
    Km2FileHandle* handle = keymagic_km2_load(km2Path.c_str());
    ASSERT_NE(handle, nullptr) << "Failed to load: " << km2Path;
    
    char* name = keymagic_km2_get_name(handle);
    
    // Check if name exists (it might be NULL if not defined in the file)
    if (name) {
        EXPECT_GT(strlen(name), 0);
        // The actual name might vary, but it should contain "Pyidaungsu" or similar
        std::string nameStr(name);
        // Just check it's not empty
        EXPECT_FALSE(nameStr.empty());
        
        // Free the allocated string
        keymagic_free_string(name);
    }
    
    keymagic_km2_free(handle);
}

TEST(KM2MetadataTest, GetKeyboardDescription) {
    std::string km2Path = getTestKM2File();
    
    if (km2Path.empty()) {
        GTEST_SKIP() << "No KM2 files found in standard locations";
    }
    
    Km2FileHandle* handle = keymagic_km2_load(km2Path.c_str());
    ASSERT_NE(handle, nullptr) << "Failed to load: " << km2Path;
    
    char* description = keymagic_km2_get_description(handle);
    
    // Description might be NULL if not defined
    if (description) {
        EXPECT_GT(strlen(description), 0);
        keymagic_free_string(description);
    }
    
    keymagic_km2_free(handle);
}

TEST(KM2MetadataTest, GetKeyboardHotkey) {
    std::string km2Path = getTestKM2File();
    
    if (km2Path.empty()) {
        GTEST_SKIP() << "No KM2 files found in standard locations";
    }
    
    Km2FileHandle* handle = keymagic_km2_load(km2Path.c_str());
    ASSERT_NE(handle, nullptr) << "Failed to load: " << km2Path;
    
    char* hotkey = keymagic_km2_get_hotkey(handle);
    
    // Hotkey might be NULL if not defined
    if (hotkey) {
        EXPECT_GT(strlen(hotkey), 0);
        
        // If hotkey exists, verify it can be parsed
        KeyMagicHotkeyInfo info;
        int result = keymagic_parse_hotkey(hotkey, &info);
        EXPECT_EQ(result, 1) << "Hotkey string should be parseable: " << hotkey;
        
        keymagic_free_string(hotkey);
    }
    
    keymagic_km2_free(handle);
}

TEST(KM2MetadataTest, GetIconData) {
    std::string km2Path = getTestKM2File();
    
    if (km2Path.empty()) {
        GTEST_SKIP() << "No KM2 files found in standard locations";
    }
    
    Km2FileHandle* handle = keymagic_km2_load(km2Path.c_str());
    ASSERT_NE(handle, nullptr) << "Failed to load: " << km2Path;
    
    // First, query the size
    size_t iconSize = keymagic_km2_get_icon_data(handle, nullptr, 0);
    
    if (iconSize > 0) {
        // Icon exists, allocate buffer and get data
        std::vector<uint8_t> iconBuffer(iconSize);
        size_t actualSize = keymagic_km2_get_icon_data(handle, iconBuffer.data(), iconSize);
        
        EXPECT_EQ(actualSize, iconSize);
        
        // Verify it looks like an image file (BMP starts with "BM", PNG starts with "\x89P")
        if (iconSize >= 2) {
            bool isBMP = (iconBuffer[0] == 'B' && iconBuffer[1] == 'M');
            bool isPNG = (iconBuffer[0] == 0x89 && iconBuffer[1] == 'P');
            EXPECT_TRUE(isBMP || isPNG) << "Icon should be either BMP or PNG format";
        }
    }
    
    keymagic_km2_free(handle);
}

TEST(KM2MetadataTest, GetIconDataWithSmallBuffer) {
    std::string km2Path = getTestKM2File();
    
    if (km2Path.empty()) {
        GTEST_SKIP() << "No KM2 files found in standard locations";
    }
    
    Km2FileHandle* handle = keymagic_km2_load(km2Path.c_str());
    ASSERT_NE(handle, nullptr) << "Failed to load: " << km2Path;
    
    // Query the actual size
    size_t iconSize = keymagic_km2_get_icon_data(handle, nullptr, 0);
    
    if (iconSize > 10) {
        // Try with a buffer that's too small
        uint8_t smallBuffer[10];
        size_t result = keymagic_km2_get_icon_data(handle, smallBuffer, 10);
        
        // C++ API returns the number of bytes actually copied (min of buffer_size and icon_size)
        EXPECT_EQ(result, 10) << "Should return number of bytes copied when buffer is smaller";
        
        // Verify partial data was copied
        // The first bytes should match with full buffer
        std::vector<uint8_t> fullBuffer(iconSize);
        size_t fullResult = keymagic_km2_get_icon_data(handle, fullBuffer.data(), iconSize);
        EXPECT_EQ(fullResult, iconSize) << "Should return full size with adequate buffer";
        
        // Check that the partial data matches the beginning of the full data
        for (size_t i = 0; i < 10; ++i) {
            EXPECT_EQ(smallBuffer[i], fullBuffer[i]) << "Byte " << i << " should match";
        }
    }
    
    keymagic_km2_free(handle);
}

TEST(KM2MetadataTest, MetadataFromNullHandle) {
    // Test that functions handle NULL gracefully
    char* name = keymagic_km2_get_name(nullptr);
    EXPECT_EQ(name, nullptr);
    
    char* description = keymagic_km2_get_description(nullptr);
    EXPECT_EQ(description, nullptr);
    
    char* hotkey = keymagic_km2_get_hotkey(nullptr);
    EXPECT_EQ(hotkey, nullptr);
    
    size_t iconSize = keymagic_km2_get_icon_data(nullptr, nullptr, 0);
    EXPECT_EQ(iconSize, 0);
}

TEST(KM2MetadataTest, FreeNullHandle) {
    // Should not crash
    keymagic_km2_free(nullptr);
    SUCCEED();
}

TEST(KM2MetadataTest, MultipleMetadataAccess) {
    // Test that we can access metadata multiple times from the same handle
    std::string km2Path = getTestKM2File();
    
    if (km2Path.empty()) {
        GTEST_SKIP() << "No KM2 files found in standard locations";
    }
    
    Km2FileHandle* handle = keymagic_km2_load(km2Path.c_str());
    ASSERT_NE(handle, nullptr) << "Failed to load: " << km2Path;
    
    // Access name multiple times
    char* name1 = keymagic_km2_get_name(handle);
    char* name2 = keymagic_km2_get_name(handle);
    
    if (name1 && name2) {
        // Should return the same content
        EXPECT_STREQ(name1, name2);
        
        // But different allocations
        EXPECT_NE(name1, name2);
        
        keymagic_free_string(name1);
        keymagic_free_string(name2);
    }
    
    keymagic_km2_free(handle);
}

TEST(KM2MetadataTest, ParseHotkeyFromMetadata) {
    std::string km2Path = getTestKM2File();
    
    if (km2Path.empty()) {
        GTEST_SKIP() << "No KM2 files found in standard locations";
    }
    
    Km2FileHandle* handle = keymagic_km2_load(km2Path.c_str());
    ASSERT_NE(handle, nullptr) << "Failed to load: " << km2Path;
    
    char* hotkey = keymagic_km2_get_hotkey(handle);
    
    if (hotkey) {
        // Parse the hotkey
        KeyMagicHotkeyInfo info;
        int result = keymagic_parse_hotkey(hotkey, &info);
        
        EXPECT_EQ(result, 1);
        
        if (result == 1) {
            // Verify the parsed info makes sense
            EXPECT_NE(info.key_code, KeyMagic_VK_Null);
            
            // At least one modifier or a valid key
            bool hasModifier = (info.ctrl || info.alt || info.shift || info.meta);
            bool hasValidKey = (info.key_code >= KeyMagic_VK_Key0 && info.key_code <= KeyMagic_VK_KeyZ) ||
                              (info.key_code >= KeyMagic_VK_F1 && info.key_code <= KeyMagic_VK_F12);
            
            EXPECT_TRUE(hasModifier || hasValidKey);
        }
        
        keymagic_free_string(hotkey);
    }
    
    keymagic_km2_free(handle);
}

// Test loading multiple KM2 files if available
TEST(KM2MetadataTest, LoadMultipleFiles) {
    // Use KeyboardFinder to get all available keyboards
    auto keyboards = keymagic_test::KeyboardFinder::getAllKeyboardFiles();
    
    // Filter out macOS metadata files
    std::vector<std::filesystem::path> validKeyboards;
    for (const auto& kb : keyboards) {
        std::string filename = kb.filename().string();
        if (filename.substr(0, 2) != "._") {
            validKeyboards.push_back(kb);
        }
    }
    
    if (validKeyboards.empty()) {
        GTEST_SKIP() << "No KM2 files found in standard locations";
    }
    
    std::vector<Km2FileHandle*> handles;
    
    // Load all available keyboards
    for (const auto& kbPath : validKeyboards) {
        Km2FileHandle* handle = keymagic_km2_load(kbPath.string().c_str());
        
        if (handle) {
            handles.push_back(handle);
            
            // Get and verify name
            char* name = keymagic_km2_get_name(handle);
            if (name) {
                EXPECT_GT(strlen(name), 0) << "File: " << kbPath;
                keymagic_free_string(name);
            }
        }
    }
    
    // Should have loaded at least one
    EXPECT_GT(handles.size(), 0);
    
    // Clean up all handles
    for (auto handle : handles) {
        keymagic_km2_free(handle);
    }
}