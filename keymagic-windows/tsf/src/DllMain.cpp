#include <windows.h>
#include <ole2.h>
#include <msctf.h>
#include "Globals.h"
#include "ClassFactory.h"
#include "KeyMagicGuids.h"
#include "Registry.h"

// DLL entry point
BOOL WINAPI DllMain(HINSTANCE hInstance, DWORD dwReason, LPVOID pvReserved)
{
    switch (dwReason)
    {
    case DLL_PROCESS_ATTACH:
        g_hInst = hInstance;
        DisableThreadLibraryCalls(hInstance);
        break;

    case DLL_PROCESS_DETACH:
        break;
    }

    return TRUE;
}

// DllGetClassObject - COM entry point
STDAPI DllGetClassObject(REFCLSID rclsid, REFIID riid, void **ppvObj)
{
    *ppvObj = nullptr;

    if (!IsEqualGUID(rclsid, CLSID_KeyMagicTextService))
        return CLASS_E_CLASSNOTAVAILABLE;

    CClassFactory *pFactory = new (std::nothrow) CClassFactory();
    if (pFactory == nullptr)
        return E_OUTOFMEMORY;

    HRESULT hr = pFactory->QueryInterface(riid, ppvObj);
    pFactory->Release();

    return hr;
}

// DllCanUnloadNow - Check if DLL can be unloaded
STDAPI DllCanUnloadNow(void)
{
    return (g_cRefDll <= 0) ? S_OK : S_FALSE;
}

// DllRegisterServer - Register the text service
STDAPI DllRegisterServer(void)
{
    // Register the COM server
    if (!RegisterServer())
        return E_FAIL;

    // Register the text service
    if (!RegisterTextService())
        return E_FAIL;

    return S_OK;
}

// DllUnregisterServer - Unregister the text service
STDAPI DllUnregisterServer(void)
{
    // Unregister the text service
    UnregisterTextService();

    // Unregister the COM server
    UnregisterServer();

    return S_OK;
}