#include "Registry.h"
#include "Globals.h"
#include "KeyMagicGuids.h"
#include "resource.h"
#include <strsafe.h>
#include <msctf.h>
#include <windows.h>

// List of categories to register the text service under
static const GUID* g_SupportedCategories[] = {
    // &GUID_TFCAT_TIP_KEYBOARD,                    // Register as keyboard
    &GUID_TFCAT_TIPCAP_IMMERSIVESUPPORT,        // Support for Metro/UWP apps
    &GUID_TFCAT_TIPCAP_SYSTRAYSUPPORT,          // Support for system tray
    // Add more categories as needed:
    // &GUID_TFCAT_TIPCAP_INPUTMODECOMPARTMENT,  // For input mode switching
    // &GUID_TFCAT_TIPCAP_COMLESS,               // For COM-less activation
    // &GUID_TFCAT_TIPCAP_WOW16,                 // For 16-bit app support
    // &GUID_TFCAT_TIPCAP_UIELEMENTENABLED,      // For UI elements
    // &GUID_TFCAT_TIPCAP_SECUREMODE,            // For secure desktop
    // &GUID_TFCAT_DISPLAYATTRIBUTEPROVIDER,     // For display attributes
};

static const int g_SupportedCategoriesCount = ARRAYSIZE(g_SupportedCategories);

BOOL CreateRegKey(HKEY hKeyParent, LPCWSTR lpszKeyName, LPCWSTR lpszValue)
{
    HKEY hKey;
    LONG lResult = RegCreateKeyEx(hKeyParent, lpszKeyName, 0, nullptr,
                                  REG_OPTION_NON_VOLATILE, KEY_WRITE, nullptr, &hKey, nullptr);

    if (lResult != ERROR_SUCCESS)
        return FALSE;

    if (lpszValue != nullptr)
    {
        lResult = RegSetValueEx(hKey, nullptr, 0, REG_SZ,
                               (const BYTE*)lpszValue, (DWORD)(wcslen(lpszValue) + 1) * sizeof(WCHAR));
    }

    RegCloseKey(hKey);
    return (lResult == ERROR_SUCCESS);
}

BOOL DeleteRegKey(HKEY hKeyParent, LPCWSTR lpszKeyName)
{
    return (RegDeleteKey(hKeyParent, lpszKeyName) == ERROR_SUCCESS);
}

BOOL SetRegValue(HKEY hKeyParent, LPCWSTR lpszKeyName, LPCWSTR lpszValueName, LPCWSTR lpszValue)
{
    HKEY hKey;
    LONG lResult = RegOpenKeyEx(hKeyParent, lpszKeyName, 0, KEY_SET_VALUE, &hKey);
    
    if (lResult != ERROR_SUCCESS)
        return FALSE;

    lResult = RegSetValueEx(hKey, lpszValueName, 0, REG_SZ,
                           (const BYTE*)lpszValue, (DWORD)(wcslen(lpszValue) + 1) * sizeof(WCHAR));

    RegCloseKey(hKey);
    return (lResult == ERROR_SUCCESS);
}

BOOL RegisterServer()
{
    WCHAR szModule[MAX_PATH];
    if (GetModuleFileName(g_hInst, szModule, ARRAYSIZE(szModule)) == 0)
        return FALSE;

    // Create CLSID key
    WCHAR szCLSID[MAX_PATH];
    StringCchPrintf(szCLSID, ARRAYSIZE(szCLSID), L"CLSID\\%s", TEXTSERVICE_CLSID);

    if (!CreateRegKey(HKEY_CLASSES_ROOT, szCLSID, TEXTSERVICE_DESC))
        return FALSE;

    // Create InprocServer32 key
    WCHAR szInprocServer[MAX_PATH];
    StringCchPrintf(szInprocServer, ARRAYSIZE(szInprocServer), L"%s\\InprocServer32", szCLSID);

    if (!CreateRegKey(HKEY_CLASSES_ROOT, szInprocServer, szModule))
        return FALSE;

    if (!SetRegValue(HKEY_CLASSES_ROOT, szInprocServer, L"ThreadingModel", TEXTSERVICE_MODEL))
        return FALSE;

    return TRUE;
}

void UnregisterServer()
{
    WCHAR szCLSID[MAX_PATH];
    StringCchPrintf(szCLSID, ARRAYSIZE(szCLSID), L"CLSID\\%s", TEXTSERVICE_CLSID);

    DeleteRegKey(HKEY_CLASSES_ROOT, szCLSID);
}

BOOL RegisterTextService()
{
    ITfInputProcessorProfiles *pInputProcessProfiles;
    HRESULT hr;

    hr = CoCreateInstance(CLSID_TF_InputProcessorProfiles, nullptr, CLSCTX_INPROC_SERVER,
                         IID_ITfInputProcessorProfiles, (void**)&pInputProcessProfiles);

    if (SUCCEEDED(hr))
    {
        // Register the text service
        hr = pInputProcessProfiles->Register(CLSID_KeyMagicTextService);

        if (SUCCEEDED(hr))
        {
            // Get module path for icon
            WCHAR szModule[MAX_PATH] = {0};
            GetModuleFileName(g_hInst, szModule, ARRAYSIZE(szModule));
            
            // Add language profile for Myanmar
            hr = pInputProcessProfiles->AddLanguageProfile(
                CLSID_KeyMagicTextService,
                TEXTSERVICE_LANGID,  // Myanmar (0x0455)
                GUID_KeyMagicProfile,
                TEXTSERVICE_DESC,
                (ULONG)wcslen(TEXTSERVICE_DESC),
                szModule,            // Icon file path
                (ULONG)(-IDI_KEYMAGIC), // Icon resource ID (negative for resource)
                0);
                
            // Also register for English to make it easier to find
            if (SUCCEEDED(hr))
            {
                hr = pInputProcessProfiles->AddLanguageProfile(
                    CLSID_KeyMagicTextService,
                    MAKELANGID(LANG_ENGLISH, SUBLANG_ENGLISH_US), // 0x0409
                    GUID_KeyMagicProfile,
                    TEXTSERVICE_DESC,
                    (ULONG)wcslen(TEXTSERVICE_DESC),
                    szModule,
                    (ULONG)(-IDI_KEYMAGIC),
                    0);
            }
            
            // Enable the profiles by default
            if (SUCCEEDED(hr))
            {
                pInputProcessProfiles->EnableLanguageProfile(
                    CLSID_KeyMagicTextService,
                    TEXTSERVICE_LANGID,
                    GUID_KeyMagicProfile,
                    TRUE);
                    
                pInputProcessProfiles->EnableLanguageProfile(
                    CLSID_KeyMagicTextService,
                    MAKELANGID(LANG_ENGLISH, SUBLANG_ENGLISH_US),
                    GUID_KeyMagicProfile,
                    TRUE);
            }
        }

        pInputProcessProfiles->Release();
    }

    // Register the text service under all supported categories
    if (SUCCEEDED(hr))
    {
        ITfCategoryMgr *pCategoryMgr;
        hr = CoCreateInstance(CLSID_TF_CategoryMgr, nullptr, CLSCTX_INPROC_SERVER,
                             IID_ITfCategoryMgr, (void**)&pCategoryMgr);
        
        if (SUCCEEDED(hr))
        {
            // Register all supported categories
            for (int i = 0; i < g_SupportedCategoriesCount && SUCCEEDED(hr); i++)
            {
                hr = pCategoryMgr->RegisterCategory(CLSID_KeyMagicTextService,
                                                   *g_SupportedCategories[i],
                                                   CLSID_KeyMagicTextService);
            }
            
            pCategoryMgr->Release();
        }
    }

    // Enable the profile by default
    if (SUCCEEDED(hr))
    {
        ITfInputProcessorProfileMgr *pProfileMgr;
        hr = CoCreateInstance(CLSID_TF_InputProcessorProfiles, nullptr, CLSCTX_INPROC_SERVER,
                             IID_ITfInputProcessorProfileMgr, (void**)&pProfileMgr);
        
        if (SUCCEEDED(hr))
        {
            // Enable for Myanmar language
            TF_INPUTPROCESSORPROFILE profile = {0};
            profile.dwProfileType = TF_PROFILETYPE_INPUTPROCESSOR;
            profile.langid = TEXTSERVICE_LANGID;
            profile.clsid = CLSID_KeyMagicTextService;
            profile.guidProfile = GUID_KeyMagicProfile;
            profile.dwFlags = 0;
            
            pProfileMgr->ActivateProfile(
                TF_PROFILETYPE_INPUTPROCESSOR,
                TEXTSERVICE_LANGID,
                CLSID_KeyMagicTextService,
                GUID_KeyMagicProfile,
                NULL,
                TF_IPPMF_DONTCARECURRENTINPUTLANGUAGE);
                
            pProfileMgr->Release();
        }
    }

    return SUCCEEDED(hr);
}

void UnregisterTextService()
{
    ITfInputProcessorProfiles *pInputProcessProfiles;
    HRESULT hr;

    hr = CoCreateInstance(CLSID_TF_InputProcessorProfiles, nullptr, CLSCTX_INPROC_SERVER,
                         IID_ITfInputProcessorProfiles, (void**)&pInputProcessProfiles);

    if (SUCCEEDED(hr))
    {
        // Unregister the text service
        pInputProcessProfiles->Unregister(CLSID_KeyMagicTextService);
        pInputProcessProfiles->Release();
    }

    // Unregister from all categories
    ITfCategoryMgr *pCategoryMgr;
    hr = CoCreateInstance(CLSID_TF_CategoryMgr, nullptr, CLSCTX_INPROC_SERVER,
                         IID_ITfCategoryMgr, (void**)&pCategoryMgr);
    
    if (SUCCEEDED(hr))
    {
        // Unregister all supported categories
        for (int i = 0; i < g_SupportedCategoriesCount; i++)
        {
            pCategoryMgr->UnregisterCategory(CLSID_KeyMagicTextService,
                                            *g_SupportedCategories[i],
                                            CLSID_KeyMagicTextService);
        }
        
        pCategoryMgr->Release();
    }
}