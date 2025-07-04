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
#define TEXTSERVICE_CLSID L"{12345678-1234-1234-1234-123456789ABC}"
#define TEXTSERVICE_DESC L"KeyMagic Text Service"
#define TEXTSERVICE_MODEL L"Apartment"
#define TEXTSERVICE_ICON_INDEX 0

// Language IDs
// Use Myanmar language ID (0x0455)
#define TEXTSERVICE_LANGID 0x0455

#endif // GLOBALS_H