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
#include <aclapi.h>
#include <sddl.h>
#include <accctrl.h>

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

// Helper function to set directory permissions for EVERYONE, ALL APPLICATION PACKAGES, and Low Integrity
// This preserves existing permissions while adding new ones
// Also explicitly ensures the owner has full control (fixes directories broken by buggy permission code)
static BOOL SetDirectoryPermissions(LPCWSTR pszDirectory)
{
    BOOL bResult = FALSE;
    PACL pOldDacl = NULL;
    PACL pNewDacl = NULL;
    PSECURITY_DESCRIPTOR pSD = NULL;
    EXPLICIT_ACCESS ea[5] = {0};  // Increased to 5 to include owner
    PSID pEveryoneSID = NULL;
    PSID pAllAppsSID = NULL;
    PSID pLowIntegritySID = NULL;
    PSID pUntrustedSID = NULL;
    PSID pCurrentUserSID = NULL;
    SID_IDENTIFIER_AUTHORITY SIDAuthWorld = SECURITY_WORLD_SID_AUTHORITY;
    SID_IDENTIFIER_AUTHORITY SIDAuthAppPackage = SECURITY_APP_PACKAGE_AUTHORITY;
    SID_IDENTIFIER_AUTHORITY SIDAuthMandatory = SECURITY_MANDATORY_LABEL_AUTHORITY;
    DWORD dwRes = 0;
    HANDLE hToken = NULL;
    PTOKEN_USER pTokenUser = NULL;
    DWORD dwTokenUserSize = 0;
    
    // Get the existing DACL
    dwRes = GetNamedSecurityInfo(pszDirectory, SE_FILE_OBJECT,
                                 DACL_SECURITY_INFORMATION,
                                 NULL, NULL, &pOldDacl, NULL, &pSD);
    
    if (dwRes != ERROR_SUCCESS)
    {
        // If we can't get existing permissions, still try to set new ones
        pOldDacl = NULL;
    }
    
    // Get the current user's SID
    if (OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &hToken))
    {
        // Get required buffer size
        GetTokenInformation(hToken, TokenUser, NULL, 0, &dwTokenUserSize);
        if (dwTokenUserSize > 0)
        {
            pTokenUser = (PTOKEN_USER)LocalAlloc(LPTR, dwTokenUserSize);
            if (pTokenUser && GetTokenInformation(hToken, TokenUser, pTokenUser, dwTokenUserSize, &dwTokenUserSize))
            {
                // We have the current user's SID in pTokenUser->User.Sid
                pCurrentUserSID = pTokenUser->User.Sid;
            }
        }
        CloseHandle(hToken);
    }
    
    // Create SID for EVERYONE
    if (!AllocateAndInitializeSid(&SIDAuthWorld, 1,
                                  SECURITY_WORLD_RID,
                                  0, 0, 0, 0, 0, 0, 0,
                                  &pEveryoneSID))
        goto Cleanup;
    
    // Create SID for ALL APPLICATION PACKAGES
    if (!AllocateAndInitializeSid(&SIDAuthAppPackage, 2,
                                  SECURITY_APP_PACKAGE_BASE_RID,
                                  SECURITY_BUILTIN_PACKAGE_ANY_PACKAGE,
                                  0, 0, 0, 0, 0, 0,
                                  &pAllAppsSID))
        goto Cleanup;
    
    // Create SID for Low Integrity Level (S-1-16-4096)
    if (!AllocateAndInitializeSid(&SIDAuthMandatory, 1,
                                  SECURITY_MANDATORY_LOW_RID,
                                  0, 0, 0, 0, 0, 0, 0,
                                  &pLowIntegritySID))
        goto Cleanup;
    
    // Create SID for Untrusted Integrity Level (S-1-16-0)
    if (!AllocateAndInitializeSid(&SIDAuthMandatory, 1,
                                  SECURITY_MANDATORY_UNTRUSTED_RID,
                                  0, 0, 0, 0, 0, 0, 0,
                                  &pUntrustedSID))
        goto Cleanup;
    
    // Set up permissions array
    int nEntries = 0;
    
    // If we have the current user's SID, explicitly grant full control
    // This fixes directories that lost owner permissions due to buggy code
    // Use SET_ACCESS to replace any existing permissions for this user
    if (pCurrentUserSID)
    {
        ea[nEntries].grfAccessPermissions = GENERIC_ALL;
        ea[nEntries].grfAccessMode = SET_ACCESS;  // Replace existing entries for this SID
        ea[nEntries].grfInheritance = CONTAINER_INHERIT_ACE | OBJECT_INHERIT_ACE;  // Apply to this folder, subfolders and files
        ea[nEntries].Trustee.TrusteeForm = TRUSTEE_IS_SID;
        ea[nEntries].Trustee.TrusteeType = TRUSTEE_IS_USER;
        ea[nEntries].Trustee.ptstrName = (LPWSTR)pCurrentUserSID;
        nEntries++;
    }
    
    // Set up EXPLICIT_ACCESS for EVERYONE
    // Use SET_ACCESS to avoid duplicate entries
    ea[nEntries].grfAccessPermissions = GENERIC_READ | GENERIC_EXECUTE;
    ea[nEntries].grfAccessMode = SET_ACCESS;  // Replace existing entries for this SID
    ea[nEntries].grfInheritance = CONTAINER_INHERIT_ACE | OBJECT_INHERIT_ACE;  // Apply to this folder, subfolders and files
    ea[nEntries].Trustee.TrusteeForm = TRUSTEE_IS_SID;
    ea[nEntries].Trustee.TrusteeType = TRUSTEE_IS_WELL_KNOWN_GROUP;
    ea[nEntries].Trustee.ptstrName = (LPWSTR)pEveryoneSID;
    nEntries++;
    
    // Set up EXPLICIT_ACCESS for ALL APPLICATION PACKAGES
    // Use SET_ACCESS to avoid duplicate entries
    ea[nEntries].grfAccessPermissions = GENERIC_READ | GENERIC_EXECUTE;
    ea[nEntries].grfAccessMode = SET_ACCESS;  // Replace existing entries for this SID
    ea[nEntries].grfInheritance = CONTAINER_INHERIT_ACE | OBJECT_INHERIT_ACE;  // Apply to this folder, subfolders and files
    ea[nEntries].Trustee.TrusteeForm = TRUSTEE_IS_SID;
    ea[nEntries].Trustee.TrusteeType = TRUSTEE_IS_WELL_KNOWN_GROUP;
    ea[nEntries].Trustee.ptstrName = (LPWSTR)pAllAppsSID;
    nEntries++;
    
    // Set up EXPLICIT_ACCESS for Low Integrity
    // Use SET_ACCESS to avoid duplicate entries
    ea[nEntries].grfAccessPermissions = GENERIC_READ | GENERIC_EXECUTE;
    ea[nEntries].grfAccessMode = SET_ACCESS;  // Replace existing entries for this SID
    ea[nEntries].grfInheritance = CONTAINER_INHERIT_ACE | OBJECT_INHERIT_ACE;  // Apply to this folder, subfolders and files
    ea[nEntries].Trustee.TrusteeForm = TRUSTEE_IS_SID;
    ea[nEntries].Trustee.TrusteeType = TRUSTEE_IS_GROUP;
    ea[nEntries].Trustee.ptstrName = (LPWSTR)pLowIntegritySID;
    nEntries++;
    
    // Set up EXPLICIT_ACCESS for Untrusted Integrity
    // Use SET_ACCESS to avoid duplicate entries
    ea[nEntries].grfAccessPermissions = GENERIC_READ | GENERIC_EXECUTE;
    ea[nEntries].grfAccessMode = SET_ACCESS;  // Replace existing entries for this SID
    ea[nEntries].grfInheritance = CONTAINER_INHERIT_ACE | OBJECT_INHERIT_ACE;  // Apply to this folder, subfolders and files
    ea[nEntries].Trustee.TrusteeForm = TRUSTEE_IS_SID;
    ea[nEntries].Trustee.TrusteeType = TRUSTEE_IS_GROUP;
    ea[nEntries].Trustee.ptstrName = (LPWSTR)pUntrustedSID;
    nEntries++;
    
    // Add the new entries to the existing DACL (preserving existing permissions)
    dwRes = SetEntriesInAcl(nEntries, ea, pOldDacl, &pNewDacl);
    if (dwRes != ERROR_SUCCESS)
        goto Cleanup;
    
    // Set the new DACL for the directory
    dwRes = SetNamedSecurityInfo((LPWSTR)pszDirectory, SE_FILE_OBJECT,
                                 DACL_SECURITY_INFORMATION,
                                 NULL, NULL, pNewDacl, NULL);
    
    if (dwRes == ERROR_SUCCESS)
    {
        bResult = TRUE;
    }
    
Cleanup:
    if (pEveryoneSID) FreeSid(pEveryoneSID);
    if (pAllAppsSID) FreeSid(pAllAppsSID);
    if (pLowIntegritySID) FreeSid(pLowIntegritySID);
    if (pUntrustedSID) FreeSid(pUntrustedSID);
    if (pTokenUser) LocalFree(pTokenUser);  // This also frees pCurrentUserSID
    if (pNewDacl) LocalFree(pNewDacl);
    if (pSD) LocalFree(pSD);
    
    return bResult;
}

// Helper function to set registry key permissions for low integrity access
static BOOL SetRegistryPermissions(HKEY hKey)
{
    BOOL bResult = FALSE;
    PACL pOldDacl = NULL;
    PACL pNewDacl = NULL;
    PSECURITY_DESCRIPTOR pSD = NULL;
    EXPLICIT_ACCESS ea[5] = {0};
    PSID pEveryoneSID = NULL;
    PSID pAllAppsSID = NULL;
    PSID pLowIntegritySID = NULL;
    PSID pUntrustedSID = NULL;
    PSID pCurrentUserSID = NULL;
    SID_IDENTIFIER_AUTHORITY SIDAuthWorld = SECURITY_WORLD_SID_AUTHORITY;
    SID_IDENTIFIER_AUTHORITY SIDAuthAppPackage = SECURITY_APP_PACKAGE_AUTHORITY;
    SID_IDENTIFIER_AUTHORITY SIDAuthMandatory = SECURITY_MANDATORY_LABEL_AUTHORITY;
    DWORD dwRes = 0;
    HANDLE hToken = NULL;
    PTOKEN_USER pTokenUser = NULL;
    DWORD dwTokenUserSize = 0;
    
    // Get the existing DACL
    dwRes = GetSecurityInfo((HANDLE)hKey, SE_REGISTRY_KEY,
                            DACL_SECURITY_INFORMATION,
                            NULL, NULL, &pOldDacl, NULL, &pSD);
    
    if (dwRes != ERROR_SUCCESS)
    {
        // If we can't get existing permissions, still try to set new ones
        pOldDacl = NULL;
    }
    
    // Get the current user's SID
    if (OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &hToken))
    {
        // Get required buffer size
        GetTokenInformation(hToken, TokenUser, NULL, 0, &dwTokenUserSize);
        if (dwTokenUserSize > 0)
        {
            pTokenUser = (PTOKEN_USER)LocalAlloc(LPTR, dwTokenUserSize);
            if (pTokenUser && GetTokenInformation(hToken, TokenUser, pTokenUser, dwTokenUserSize, &dwTokenUserSize))
            {
                // We have the current user's SID in pTokenUser->User.Sid
                pCurrentUserSID = pTokenUser->User.Sid;
            }
        }
        CloseHandle(hToken);
    }
    
    // Create SID for EVERYONE
    if (!AllocateAndInitializeSid(&SIDAuthWorld, 1,
                                  SECURITY_WORLD_RID,
                                  0, 0, 0, 0, 0, 0, 0,
                                  &pEveryoneSID))
        goto Cleanup;
    
    // Create SID for ALL APPLICATION PACKAGES
    if (!AllocateAndInitializeSid(&SIDAuthAppPackage, 2,
                                  SECURITY_APP_PACKAGE_BASE_RID,
                                  SECURITY_BUILTIN_PACKAGE_ANY_PACKAGE,
                                  0, 0, 0, 0, 0, 0,
                                  &pAllAppsSID))
        goto Cleanup;
    
    // Create SID for Low Integrity Level
    if (!AllocateAndInitializeSid(&SIDAuthMandatory, 1,
                                  SECURITY_MANDATORY_LOW_RID,
                                  0, 0, 0, 0, 0, 0, 0,
                                  &pLowIntegritySID))
        goto Cleanup;
    
    // Create SID for Untrusted Integrity Level
    if (!AllocateAndInitializeSid(&SIDAuthMandatory, 1,
                                  SECURITY_MANDATORY_UNTRUSTED_RID,
                                  0, 0, 0, 0, 0, 0, 0,
                                  &pUntrustedSID))
        goto Cleanup;
    
    // Set up permissions array
    int nEntries = 0;
    
    // Grant current user full control
    if (pCurrentUserSID)
    {
        ea[nEntries].grfAccessPermissions = KEY_ALL_ACCESS;
        ea[nEntries].grfAccessMode = SET_ACCESS;
        ea[nEntries].grfInheritance = CONTAINER_INHERIT_ACE | OBJECT_INHERIT_ACE;
        ea[nEntries].Trustee.TrusteeForm = TRUSTEE_IS_SID;
        ea[nEntries].Trustee.TrusteeType = TRUSTEE_IS_USER;
        ea[nEntries].Trustee.ptstrName = (LPWSTR)pCurrentUserSID;
        nEntries++;
    }
    
    // Grant EVERYONE read access
    ea[nEntries].grfAccessPermissions = KEY_READ;
    ea[nEntries].grfAccessMode = SET_ACCESS;
    ea[nEntries].grfInheritance = CONTAINER_INHERIT_ACE | OBJECT_INHERIT_ACE;
    ea[nEntries].Trustee.TrusteeForm = TRUSTEE_IS_SID;
    ea[nEntries].Trustee.TrusteeType = TRUSTEE_IS_WELL_KNOWN_GROUP;
    ea[nEntries].Trustee.ptstrName = (LPWSTR)pEveryoneSID;
    nEntries++;
    
    // Grant ALL APPLICATION PACKAGES read access
    ea[nEntries].grfAccessPermissions = KEY_READ;
    ea[nEntries].grfAccessMode = SET_ACCESS;
    ea[nEntries].grfInheritance = CONTAINER_INHERIT_ACE | OBJECT_INHERIT_ACE;
    ea[nEntries].Trustee.TrusteeForm = TRUSTEE_IS_SID;
    ea[nEntries].Trustee.TrusteeType = TRUSTEE_IS_WELL_KNOWN_GROUP;
    ea[nEntries].Trustee.ptstrName = (LPWSTR)pAllAppsSID;
    nEntries++;
    
    // Grant Low Integrity read access
    ea[nEntries].grfAccessPermissions = KEY_READ;
    ea[nEntries].grfAccessMode = SET_ACCESS;
    ea[nEntries].grfInheritance = CONTAINER_INHERIT_ACE | OBJECT_INHERIT_ACE;
    ea[nEntries].Trustee.TrusteeForm = TRUSTEE_IS_SID;
    ea[nEntries].Trustee.TrusteeType = TRUSTEE_IS_GROUP;
    ea[nEntries].Trustee.ptstrName = (LPWSTR)pLowIntegritySID;
    nEntries++;
    
    // Grant Untrusted Integrity read access
    ea[nEntries].grfAccessPermissions = KEY_READ;
    ea[nEntries].grfAccessMode = SET_ACCESS;
    ea[nEntries].grfInheritance = CONTAINER_INHERIT_ACE | OBJECT_INHERIT_ACE;
    ea[nEntries].Trustee.TrusteeForm = TRUSTEE_IS_SID;
    ea[nEntries].Trustee.TrusteeType = TRUSTEE_IS_GROUP;
    ea[nEntries].Trustee.ptstrName = (LPWSTR)pUntrustedSID;
    nEntries++;
    
    // Add the new entries to the existing DACL
    dwRes = SetEntriesInAcl(nEntries, ea, pOldDacl, &pNewDacl);
    if (dwRes != ERROR_SUCCESS)
        goto Cleanup;
    
    // Set the new DACL for the registry key
    dwRes = SetSecurityInfo((HANDLE)hKey, SE_REGISTRY_KEY,
                           DACL_SECURITY_INFORMATION,
                           NULL, NULL, pNewDacl, NULL);
    
    if (dwRes == ERROR_SUCCESS)
    {
        bResult = TRUE;
    }
    
Cleanup:
    if (pEveryoneSID) FreeSid(pEveryoneSID);
    if (pAllAppsSID) FreeSid(pAllAppsSID);
    if (pLowIntegritySID) FreeSid(pLowIntegritySID);
    if (pUntrustedSID) FreeSid(pUntrustedSID);
    if (pTokenUser) LocalFree(pTokenUser);  // This also frees pCurrentUserSID
    if (pNewDacl) LocalFree(pNewDacl);
    if (pSD) LocalFree(pSD);
    
    return bResult;
}

// Helper function to apply permissions to KeyMagic registry keys
static void ApplyKeyMagicRegistryPermissions()
{
    HKEY hKeyMagic = NULL;
    HKEY hSubKey = NULL;
    DWORD dwDisposition;
    
    // Create or open the main KeyMagic key
    if (RegCreateKeyExW(HKEY_CURRENT_USER, L"Software\\KeyMagic", 0, NULL, 
                        REG_OPTION_NON_VOLATILE, KEY_ALL_ACCESS, NULL, 
                        &hKeyMagic, &dwDisposition) == ERROR_SUCCESS)
    {
        // Set permissions on main key
        SetRegistryPermissions(hKeyMagic);
        
        // Create or open Settings subkey
        if (RegCreateKeyExW(hKeyMagic, L"Settings", 0, NULL,
                           REG_OPTION_NON_VOLATILE, KEY_ALL_ACCESS, NULL,
                           &hSubKey, &dwDisposition) == ERROR_SUCCESS)
        {
            SetRegistryPermissions(hSubKey);
            RegCloseKey(hSubKey);
        }
        
        // Create or open Keyboards subkey
        if (RegCreateKeyExW(hKeyMagic, L"Keyboards", 0, NULL,
                           REG_OPTION_NON_VOLATILE, KEY_ALL_ACCESS, NULL,
                           &hSubKey, &dwDisposition) == ERROR_SUCCESS)
        {
            SetRegistryPermissions(hSubKey);
            
            // Enumerate and set permissions on each keyboard entry
            DWORD dwIndex = 0;
            WCHAR szKeyName[256];
            DWORD cchKeyName;
            HKEY hKbKey = NULL;
            
            while (TRUE)
            {
                cchKeyName = ARRAYSIZE(szKeyName);
                if (RegEnumKeyExW(hSubKey, dwIndex, szKeyName, &cchKeyName, 
                                 NULL, NULL, NULL, NULL) != ERROR_SUCCESS)
                    break;
                
                if (RegOpenKeyExW(hSubKey, szKeyName, 0, KEY_ALL_ACCESS, &hKbKey) == ERROR_SUCCESS)
                {
                    SetRegistryPermissions(hKbKey);
                    RegCloseKey(hKbKey);
                }
                
                dwIndex++;
            }
            
            RegCloseKey(hSubKey);
        }
        
        RegCloseKey(hKeyMagic);
    }
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

    // Set directory permissions for KeyMagic folder
    // This ensures the owner has write access (fixes crash from buggy permission code)
    WCHAR szAppData[MAX_PATH];
    if (SHGetFolderPath(NULL, CSIDL_LOCAL_APPDATA, NULL, 0, szAppData) == S_OK)
    {
        WCHAR szKeyMagicDir[MAX_PATH];
        StringCchPrintf(szKeyMagicDir, ARRAYSIZE(szKeyMagicDir), L"%s\\KeyMagic", szAppData);
        
        // Create the directory if it doesn't exist
        if (GetFileAttributes(szKeyMagicDir) == INVALID_FILE_ATTRIBUTES)
        {
            if (CreateDirectory(szKeyMagicDir, NULL))
            {
                SetDirectoryPermissions(szKeyMagicDir);
            }
        }
        else
        {
            // Directory exists, ensure it has correct permissions
            SetDirectoryPermissions(szKeyMagicDir);
        }
    }
    
    // Apply registry permissions for low integrity access
    // This allows SearchHost.exe and other sandboxed processes to read our settings
    ApplyKeyMagicRegistryPermissions();

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