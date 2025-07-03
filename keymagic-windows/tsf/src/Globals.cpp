#define UNICODE
#define _UNICODE

#include "KeyMagicGuids.h"

// Global variables definition
HINSTANCE g_hInst = NULL;
LONG g_cRefDll = 0;

// GUID definitions
// {12345678-1234-1234-1234-123456789ABC}
extern "C" const GUID CLSID_KeyMagicTextService = 
    { 0x12345678, 0x1234, 0x1234, { 0x12, 0x34, 0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC } };

// {87654321-4321-4321-4321-BA9876543210}
extern "C" const GUID GUID_KeyMagicProfile = 
    { 0x87654321, 0x4321, 0x4321, { 0x43, 0x21, 0xBA, 0x98, 0x76, 0x54, 0x32, 0x10 } };