#ifndef GLOBALS_H
#define GLOBALS_H

#include <windows.h>
#include <atomic>

// Global instance handle
extern HINSTANCE g_hInst;

// Global DLL reference count
extern std::atomic<LONG> g_cRefDll;

// Helper functions
inline void DllAddRef() { g_cRefDll++; }
inline void DllRelease() { g_cRefDll--; }

// Registry key paths
#define TEXTSERVICE_CLSID L"{094A562B-D08B-4CAF-8E95-8F8031CFD24C}"
#define TEXTSERVICE_DESC L"KeyMagic 3"
#define TEXTSERVICE_MODEL L"Apartment"
#define TEXTSERVICE_ICON_INDEX 0

// Language IDs
// Use Myanmar language ID (0x0455)
#define TEXTSERVICE_LANGID 0x0455

// Define the extra info signature for our SendInput calls
#define KEYMAGIC_EXTRAINFO_SIGNATURE 0x4B4D4147 // "KMAG"

#endif // GLOBALS_H