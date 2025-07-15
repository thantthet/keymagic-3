#pragma once

#include <Windows.h>
#include <string>
#include <sstream>
#include "../include/keymagic_ffi.h"

// Key processing utilities shared between edit sessions
namespace KeyProcessingUtils
{
    // Input preparation result
    struct KeyInputData
    {
        char character;
        int shift;
        int ctrl;
        int alt;
        int capsLock;
        bool shouldSkip;  // True if key should be skipped (modifier/function keys)
    };

    // Prepares key input data for engine processing
    // Handles steps 2 (character mapping), 3 (modifier filtering), and 4 (state collection)
    KeyInputData PrepareKeyInput(WPARAM wParam, LPARAM lParam);

    // Helper to map virtual key to character
    char MapVirtualKeyToChar(WPARAM wParam, LPARAM lParam);

    // Helper to check if character is printable ASCII
    bool IsPrintableAscii(char c);

    // Helper to check if we should skip this key (modifiers, function keys)
    bool ShouldSkipKey(WPARAM wParam);
}