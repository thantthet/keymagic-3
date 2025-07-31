#include "Registry.h"
#include "Globals.h"
#include "KeyMagicGuids.h"
#include "LanguageUtils.h"
#include "resource.h"
#include "../../shared/include/RegistryUtils.h"
#include <strsafe.h>
#include <msctf.h>
#include <windows.h>
#include <vector>
#include <string>
#include <shlobj.h>
#include <fstream>

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
    
    // Get enabled language codes using shared utility
    std::vector<std::wstring> languageCodes = RegistryUtils::GetEnabledLanguages();
    
    // Convert language codes to LANGIDs
    for (const auto& languageCode : languageCodes)
    {
        LANGID langId = LanguageCodeToLangId(languageCode);
        if (langId != 0)
        {
            languages.push_back(langId);
        }
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

// Helper function to check if we're loaded through an ARM64X forwarder
static BOOL IsLoadedViaForwarder(LPCWSTR pszCurrentPath, LPWSTR pszForwarderPath, DWORD cchForwarderPath)
{
    // Check if our DLL name contains architecture suffix
    LPCWSTR pszFileName = wcsrchr(pszCurrentPath, L'\\');
    if (!pszFileName) pszFileName = pszCurrentPath;
    else pszFileName++; // Skip the backslash
    
    // Check for architecture-specific naming pattern (case-insensitive)
    size_t fileNameLen = wcslen(pszFileName);
    BOOL isArchSpecific = FALSE;
    
    if (fileNameLen >= 8 && _wcsicmp(pszFileName + fileNameLen - 8, L"_x64.dll") == 0)
    {
        isArchSpecific = TRUE;
    }
    else if (fileNameLen >= 10 && _wcsicmp(pszFileName + fileNameLen - 10, L"_arm64.dll") == 0)
    {
        isArchSpecific = TRUE;
    }
    
    if (isArchSpecific)
    {
        // We're likely loaded via forwarder
        // Construct the forwarder path by replacing the filename in the same directory
        StringCchCopy(pszForwarderPath, cchForwarderPath, pszCurrentPath);
        
        // Find the last backslash
        LPWSTR pszLastSlash = wcsrchr(pszForwarderPath, L'\\');
        if (pszLastSlash)
        {
            // Replace with forwarder name (KeyMagicTSF.dll) in the same directory
            StringCchCopy(pszLastSlash + 1, 
                         cchForwarderPath - (pszLastSlash - pszForwarderPath + 1), 
                         L"KeyMagicTSF.dll");
            
            // Verify the forwarder exists
            if (GetFileAttributes(pszForwarderPath) != INVALID_FILE_ATTRIBUTES)
            {
                return TRUE;
            }
        }
    }
    
    return FALSE;
}

// Helper function to get the keyboard icon path (same as used by GUI)
static BOOL GetKeyboardIconPath(LPWSTR pszIconPath, DWORD cchIconPath)
{
    // Get the local app data directory
    WCHAR szAppData[MAX_PATH];
    if (FAILED(SHGetFolderPath(NULL, CSIDL_LOCAL_APPDATA | CSIDL_FLAG_CREATE, NULL, 0, szAppData)))
    {
        // Fallback to APPDATA
        if (FAILED(SHGetFolderPath(NULL, CSIDL_APPDATA | CSIDL_FLAG_CREATE, NULL, 0, szAppData)))
        {
            return FALSE;
        }
    }
    
    // Construct the icon file path (same as GUI: %LOCALAPPDATA%\KeyMagic\keymagic-keyboard.ico)
    StringCchPrintf(pszIconPath, cchIconPath, L"%s\\KeyMagic\\keymagic-keyboard.ico", szAppData);
    
    // Check if the icon file exists (installed by installer or extracted by GUI)
    if (GetFileAttributes(pszIconPath) != INVALID_FILE_ATTRIBUTES)
    {
        return TRUE;
    }
    
    return FALSE;
}

BOOL RegisterServer()
{
    WCHAR szModule[MAX_PATH];
    if (GetModuleFileName(g_hInst, szModule, ARRAYSIZE(szModule)) == 0)
        return FALSE;

    // Check if we're loaded via ARM64X forwarder
    WCHAR szForwarderPath[MAX_PATH];
    if (IsLoadedViaForwarder(szModule, szForwarderPath, ARRAYSIZE(szForwarderPath)))
    {
        // Use the forwarder path for registration
        StringCchCopy(szModule, ARRAYSIZE(szModule), szForwarderPath);
    }

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

    // Get the keyboard icon path
    WCHAR szIconPath[MAX_PATH] = {0};
    if (!GetKeyboardIconPath(szIconPath, ARRAYSIZE(szIconPath)))
    {
        // Fallback to DLL path with embedded icon
        GetModuleFileName(g_hInst, szIconPath, ARRAYSIZE(szIconPath));
        
        // Check if we're loaded via ARM64X forwarder for icon path too
        WCHAR szForwarderPath[MAX_PATH];
        if (IsLoadedViaForwarder(szIconPath, szForwarderPath, ARRAYSIZE(szForwarderPath)))
        {
            // Use the forwarder path for icon
            StringCchCopy(szIconPath, ARRAYSIZE(szIconPath), szForwarderPath);
        }
    }

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
            szIconPath,
            (ULONG)(GetFileAttributes(szIconPath) != INVALID_FILE_ATTRIBUTES && 
                    wcsstr(szIconPath, L".ico") != NULL ? 0 : -IDI_KEYMAGIC),
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