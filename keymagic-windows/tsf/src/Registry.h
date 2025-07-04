#ifndef REGISTRY_H
#define REGISTRY_H

#include <windows.h>

// Registry helper functions
BOOL RegisterServer();
void UnregisterServer();
BOOL RegisterTextService();
void UnregisterTextService();

// Helper functions
BOOL CreateRegKey(HKEY hKeyParent, LPCWSTR lpszKeyName, LPCWSTR lpszValue = nullptr);
BOOL DeleteRegKey(HKEY hKeyParent, LPCWSTR lpszKeyName);
BOOL SetRegValue(HKEY hKeyParent, LPCWSTR lpszKeyName, LPCWSTR lpszValueName, LPCWSTR lpszValue);

#endif // REGISTRY_H