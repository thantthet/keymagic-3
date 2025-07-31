#pragma once

#include <windows.h>

// Message types for tray communication
enum TrayMessageType {
    MSG_FOCUS_GAINED = 1,
    MSG_FOCUS_LOST = 2,
    MSG_KEYBOARD_CHANGE = 3,
    MSG_TIP_STARTED = 4,
    MSG_TIP_STOPPED = 5
};

// Message structure for IPC
struct TrayMessage {
    DWORD messageType;
    DWORD processId;
    WCHAR keyboardId[256];
    WCHAR processName[256];
};