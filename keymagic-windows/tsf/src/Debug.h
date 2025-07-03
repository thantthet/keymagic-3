#ifndef KEYMAGIC_DEBUG_H
#define KEYMAGIC_DEBUG_H

#include <windows.h>
#include <stdio.h>
#include <time.h>

// Debug logging macros - Always enabled for troubleshooting
#if 1  // Always enable debug output for now
    #define DEBUG_OUTPUT(fmt, ...) \
        do { \
            char buffer[1024]; \
            sprintf_s(buffer, sizeof(buffer), "[KeyMagic] " fmt "\n", ##__VA_ARGS__); \
            OutputDebugStringA(buffer); \
        } while(0)

    #define DEBUG_FUNCTION_ENTER() DEBUG_OUTPUT("Enter: %s", __FUNCTION__)
    #define DEBUG_FUNCTION_EXIT() DEBUG_OUTPUT("Exit: %s", __FUNCTION__)
    #define DEBUG_KEY_EVENT(key, ch, shift, ctrl, alt) \
        DEBUG_OUTPUT("KeyEvent: vk=0x%X char='%c' shift=%d ctrl=%d alt=%d", \
                     key, (ch > 31 && ch < 127) ? ch : '?', shift, ctrl, alt)
#else
    #define DEBUG_OUTPUT(fmt, ...)
    #define DEBUG_FUNCTION_ENTER()
    #define DEBUG_FUNCTION_EXIT()
    #define DEBUG_KEY_EVENT(key, ch, shift, ctrl, alt)
#endif

#endif // KEYMAGIC_DEBUG_H