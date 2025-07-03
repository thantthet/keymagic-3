#define UNICODE
#define _UNICODE

#include <windows.h>
#include <olectl.h>
#include <msctf.h>
#include "KeyMagicTextService.h"
#include "KeyMagicGuids.h"


// Called from Rust's DllMain
extern "C" BOOL InitializeDll(HINSTANCE hInstance) {
    g_hInst = hInstance;
    DisableThreadLibraryCalls(hInstance);
    return TRUE;
}

// DLL exports for COM registration
STDAPI DllGetClassObject(REFCLSID rclsid, REFIID riid, void** ppv) {
    if (IsEqualGUID(rclsid, CLSID_KeyMagicTextService)) {
        CClassFactory* pFactory = new CClassFactory();
        if (pFactory == NULL) {
            return E_OUTOFMEMORY;
        }
        
        HRESULT hr = pFactory->QueryInterface(riid, ppv);
        pFactory->Release();
        
        return hr;
    }
    
    return CLASS_E_CLASSNOTAVAILABLE;
}

STDAPI DllCanUnloadNow() {
    return g_cRefDll == 0 ? S_OK : S_FALSE;
}

// Helper function to create registry key and set value
LONG CreateRegKeyAndValue(HKEY hKeyParent, LPCWSTR lpszKeyName, LPCWSTR lpszValueName, LPCWSTR lpszValue) {
    HKEY hKey;
    LONG lResult = RegCreateKeyExW(hKeyParent, lpszKeyName, 0, NULL, REG_OPTION_NON_VOLATILE, KEY_WRITE, NULL, &hKey, NULL);
    
    if (lResult == ERROR_SUCCESS) {
        if (lpszValue != NULL) {
            lResult = RegSetValueExW(hKey, lpszValueName, 0, REG_SZ, (const BYTE*)lpszValue, (DWORD)(wcslen(lpszValue) + 1) * sizeof(WCHAR));
        }
        RegCloseKey(hKey);
    }
    
    return lResult;
}

// Register the COM server
STDAPI DllRegisterServer() {
    WCHAR szModule[MAX_PATH];
    GetModuleFileNameW(g_hInst, szModule, ARRAYSIZE(szModule));
    
    WCHAR szCLSID[256];
    StringFromGUID2(CLSID_KeyMagicTextService, szCLSID, ARRAYSIZE(szCLSID));
    
    WCHAR szSubKey[256];
    
    // Register CLSID
    wsprintfW(szSubKey, L"CLSID\\%s", szCLSID);
    CreateRegKeyAndValue(HKEY_CLASSES_ROOT, szSubKey, NULL, L"KeyMagic Text Service");
    
    wsprintfW(szSubKey, L"CLSID\\%s\\InProcServer32", szCLSID);
    CreateRegKeyAndValue(HKEY_CLASSES_ROOT, szSubKey, NULL, szModule);
    CreateRegKeyAndValue(HKEY_CLASSES_ROOT, szSubKey, L"ThreadingModel", L"Apartment");
    
    // Register text service
    ITfInputProcessorProfiles* pInputProcessProfiles;
    HRESULT hr = CoCreateInstance(CLSID_TF_InputProcessorProfiles, NULL, CLSCTX_INPROC_SERVER,
                                  IID_ITfInputProcessorProfiles, (void**)&pInputProcessProfiles);
    
    if (SUCCEEDED(hr)) {
        // Register the text service
        hr = pInputProcessProfiles->Register(CLSID_KeyMagicTextService);
        
        if (SUCCEEDED(hr)) {
            // Add language profile (0x0409 = English US, you can change this)
            hr = pInputProcessProfiles->AddLanguageProfile(
                CLSID_KeyMagicTextService,
                0x0409,  // Language ID
                GUID_KeyMagicProfile,
                L"KeyMagic",
                (ULONG)wcslen(L"KeyMagic"),  // Display name length
                szModule,
                (ULONG)-1,  // Module file index
                0  // Icon index
            );
        }
        
        pInputProcessProfiles->Release();
    }
    
    return hr;
}

// Unregister the COM server
STDAPI DllUnregisterServer() {
    ITfInputProcessorProfiles* pInputProcessProfiles;
    HRESULT hr = CoCreateInstance(CLSID_TF_InputProcessorProfiles, NULL, CLSCTX_INPROC_SERVER,
                                  IID_ITfInputProcessorProfiles, (void**)&pInputProcessProfiles);
    
    if (SUCCEEDED(hr)) {
        pInputProcessProfiles->Unregister(CLSID_KeyMagicTextService);
        pInputProcessProfiles->Release();
    }
    
    // Delete CLSID registry entries
    WCHAR szCLSID[256];
    StringFromGUID2(CLSID_KeyMagicTextService, szCLSID, ARRAYSIZE(szCLSID));
    
    WCHAR szSubKey[256];
    wsprintfW(szSubKey, L"CLSID\\%s", szCLSID);
    
    RegDeleteTreeW(HKEY_CLASSES_ROOT, szSubKey);
    
    return S_OK;
}