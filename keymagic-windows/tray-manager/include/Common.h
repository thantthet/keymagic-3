#pragma once

#include <windows.h>
#include <shlobj.h>
#include <sddl.h>
#include <string>
#include <memory>
#include <vector>
#include <set>
#include <map>
#include <mutex>
#include <thread>
#include <atomic>
#include <functional>

#include "../../shared/include/KeyMagicConstants.h"
#include "../../shared/include/KeyMagicUtils.h"

// Application constants
constexpr const wchar_t* KEYMAGIC_TRAY_CLASS = L"KeyMagicTrayWindow";

// Window messages
constexpr UINT WM_TRAYICON = WM_USER + 1;
constexpr UINT WM_PIPE_MESSAGE = WM_USER + 2;
constexpr UINT WM_MENU_SHOWN = WM_USER + 3;
constexpr UINT WM_MENU_DISMISSED = WM_USER + 4;

// Timer IDs
constexpr UINT TIMER_HIDE_DELAY = 1;

// Delays
constexpr DWORD HIDE_DELAY_MS = 50;  // Delay before hiding icon on focus lost (optimized based on timing)

// Tray icon ID
constexpr UINT TRAY_ICON_ID = 1;

// Default icon size
constexpr int DEFAULT_ICON_SIZE = 16;

// Using namespace for shared utilities
using namespace KeyMagicUtils;