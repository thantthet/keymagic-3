#pragma once

#include <string>

// Common structure for keyboard information
struct KeyboardInfo {
    std::wstring id;
    std::wstring name;
    std::wstring path;
    std::wstring hotkey;
    bool enabled = true;  // Default to enabled if not specified
};