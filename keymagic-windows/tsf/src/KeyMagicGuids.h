#ifndef KEYMAGIC_GUIDS_H
#define KEYMAGIC_GUIDS_H

#include <windows.h>

// GUID declarations - defined in DllMain.cpp
extern "C" const GUID CLSID_KeyMagicTextService;
extern "C" const GUID GUID_KeyMagicProfile;

// Global instance handle
extern HINSTANCE g_hInst;
extern LONG g_cRefDll;

#endif // KEYMAGIC_GUIDS_H