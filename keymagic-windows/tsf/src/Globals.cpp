#include "Globals.h"

// Global instance handle
HINSTANCE g_hInst = nullptr;

// Global DLL reference count
std::atomic<LONG> g_cRefDll(0);