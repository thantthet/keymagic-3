#pragma once

#include <windows.h>

// KeyMagic Registry Paths
constexpr const wchar_t* KEYMAGIC_REGISTRY_PATH = L"Software\\KeyMagic";
constexpr const wchar_t* KEYMAGIC_KEYBOARDS_PATH = L"Software\\KeyMagic\\Keyboards";
constexpr const wchar_t* KEYMAGIC_SETTINGS_PATH = L"Software\\KeyMagic\\Settings";

// KeyMagic TIP CLSID
constexpr const wchar_t* KEYMAGIC_TIP_CLSID = L"{B9F5A039-9008-4D0F-97F5-26AA6D3C5F06}";

// Named objects
constexpr const wchar_t* KEYMAGIC_MUTEX_NAME = L"Global\\KeyMagicTrayManager";
constexpr const wchar_t* KEYMAGIC_REGISTRY_UPDATE_EVENT = L"Global\\KeyMagicRegistryUpdate";

// Pipe configuration
constexpr DWORD PIPE_BUFFER_SIZE = 4096;