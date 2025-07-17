#include "Registry.h"
#include "Globals.h"
#include "KeyMagicGuids.h"
#include "LanguageUtils.h"
#include "resource.h"
#include <strsafe.h>
#include <msctf.h>
#include <windows.h>
#include <vector>
#include <string>

// List of categories to register the text service under
static const GUID* g_SupportedCategories[] = {
    &GUID_TFCAT_TIP_KEYBOARD,                    // Register as keyboard
    &GUID_TFCAT_TIPCAP_IMMERSIVESUPPORT,        // Support for Metro/UWP apps
    &GUID_TFCAT_TIPCAP_SYSTRAYSUPPORT,          // Support for system tray
    &GUID_TFCAT_DISPLAYATTRIBUTEPROVIDER,       // For display attributes
    // Add more categories as needed:
    // &GUID_TFCAT_TIPCAP_INPUTMODECOMPARTMENT,  // For input mode switching
    // &GUID_TFCAT_TIPCAP_COMLESS,               // For COM-less activation
    // &GUID_TFCAT_TIPCAP_WOW16,                 // For 16-bit app support
    // &GUID_TFCAT_TIPCAP_UIELEMENTENABLED,      // For UI elements
    // &GUID_TFCAT_TIPCAP_SECUREMODE,            // For secure desktop
};

static const int g_SupportedCategoriesCount = ARRAYSIZE(g_SupportedCategories);

// Read enabled languages from KeyMagic registry settings
std::vector<LANGID> GetEnabledLanguagesFromRegistry()
{
    std::vector<LANGID> languages;
    HKEY hKey;
    
    // Open KeyMagic settings key
    if (RegOpenKeyEx(HKEY_CURRENT_USER, L"Software\\KeyMagic\\Settings", 0, KEY_READ, &hKey) == ERROR_SUCCESS)
    {
        DWORD dwType = REG_MULTI_SZ;
        DWORD dwSize = 0;
        
        // Get size first
        if (RegQueryValueEx(hKey, L"EnabledLanguages", nullptr, &dwType, nullptr, &dwSize) == ERROR_SUCCESS && dwSize > 0)
        {
            // Allocate buffer and read data
            std::vector<WCHAR> buffer(dwSize / sizeof(WCHAR));
            if (RegQueryValueEx(hKey, L"EnabledLanguages", nullptr, &dwType, 
                               reinterpret_cast<LPBYTE>(buffer.data()), &dwSize) == ERROR_SUCCESS)
            {
                // Parse multi-string data
                LPCWSTR pszCurrent = buffer.data();
                while (*pszCurrent)
                {
                    std::wstring languageCode(pszCurrent);
                    LANGID langId = LanguageCodeToLangId(languageCode);
                    if (langId != 0)
                    {
                        languages.push_back(langId);
                    }
                    pszCurrent += languageCode.length() + 1;
                }
            }
        }
        
        RegCloseKey(hKey);
    }
    
    return languages;
}

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
    // Unregister the text service first
    UnregisterTextService();
    
    // Then delete the CLSID registry key
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
            // Use UpdateLanguageProfiles to handle language profile registration
            if (!UpdateLanguageProfiles())
            {
                hr = E_FAIL;
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
        // Remove all language profiles
        std::vector<LANGID> enabledLanguages = GetEnabledLanguagesFromRegistry();
        
        // If no languages specified, use defaults
        if (enabledLanguages.empty())
        {
            enabledLanguages.push_back(MAKELANGID(LANG_ENGLISH, SUBLANG_ENGLISH_US));
        }
        
        // Remove language profiles for all enabled languages
        for (size_t i = 0; i < enabledLanguages.size(); i++)
        {
            pInputProcessProfiles->RemoveLanguageProfile(
                CLSID_KeyMagicTextService,
                enabledLanguages[i],
                GUID_KeyMagicProfile);
        }
            
        // Then unregister the text service
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

BOOL UpdateLanguageProfiles()
{
    ITfInputProcessorProfiles *pInputProcessProfiles;
    HRESULT hr;

    hr = CoCreateInstance(CLSID_TF_InputProcessorProfiles, nullptr, CLSCTX_INPROC_SERVER,
                         IID_ITfInputProcessorProfiles, (void**)&pInputProcessProfiles);

    if (FAILED(hr))
        return FALSE;

    // Get enabled languages from registry
    std::vector<LANGID> enabledLanguages = GetEnabledLanguagesFromRegistry();
    if (enabledLanguages.empty())
    {
        enabledLanguages.push_back(MAKELANGID(LANG_ENGLISH, SUBLANG_ENGLISH_US));
    }

    // Get module path for icon
    WCHAR szModule[MAX_PATH] = {0};
    GetModuleFileName(g_hInst, szModule, ARRAYSIZE(szModule));

    // Register all enabled languages regardless of current registration status
    for (size_t i = 0; i < enabledLanguages.size(); i++)
    {
        // Add/update this language profile
        hr = pInputProcessProfiles->AddLanguageProfile(
            CLSID_KeyMagicTextService,
            enabledLanguages[i],
            GUID_KeyMagicProfile,
            TEXTSERVICE_DESC,
            (ULONG)wcslen(TEXTSERVICE_DESC),
            szModule,
            (ULONG)(-IDI_KEYMAGIC),
            0);
            
        if (SUCCEEDED(hr))
        {
            // Enable the language profile
            pInputProcessProfiles->EnableLanguageProfile(
                CLSID_KeyMagicTextService,
                enabledLanguages[i],
                GUID_KeyMagicProfile,
                TRUE);
        }
    }

    pInputProcessProfiles->Release();
    return TRUE;
}