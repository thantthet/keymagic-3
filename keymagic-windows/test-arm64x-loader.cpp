// test-arm64x-loader.cpp - Test application to verify ARM64X DLL loading
// This app attempts to load different DLL variants and reports the results

#include <windows.h>
#include <iostream>
#include <string>
#include <vector>
#include <iomanip>

// Function pointer types for the TSF exports
typedef HRESULT (STDAPICALLTYPE *DllGetClassObjectFunc)(REFCLSID, REFIID, LPVOID*);
typedef HRESULT (STDAPICALLTYPE *DllCanUnloadNowFunc)(void);
typedef HRESULT (STDAPICALLTYPE *DllRegisterServerFunc)(void);
typedef HRESULT (STDAPICALLTYPE *DllUnregisterServerFunc)(void);

struct DllInfo {
    std::wstring path;
    std::wstring description;
    HMODULE hModule;
    bool loaded;
    std::string error;
    
    // Function pointers
    DllGetClassObjectFunc pDllGetClassObject;
    DllCanUnloadNowFunc pDllCanUnloadNow;
    DllRegisterServerFunc pDllRegisterServer;
    DllUnregisterServerFunc pDllUnregisterServer;
};

// Get architecture string
std::string GetArchitecture() {
#if defined(_M_ARM64) || defined(__aarch64__)
    return "ARM64";
#elif defined(_M_AMD64) || defined(__x86_64__)
    return "x64";
#elif defined(_M_IX86) || defined(__i386__)
    return "x86";
#else
    return "Unknown";
#endif
}

// Get last error as string
std::string GetLastErrorString() {
    DWORD error = GetLastError();
    if (error == 0) return "No error";
    
    LPSTR messageBuffer = nullptr;
    DWORD size = FormatMessageA(
        FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_IGNORE_INSERTS,
        NULL, error, MAKELANGID(LANG_NEUTRAL, SUBLANG_DEFAULT),
        (LPSTR)&messageBuffer, 0, NULL);
    
    std::string message;
    if (size > 0 && messageBuffer) {
        message = std::string(messageBuffer, size);
        // Remove trailing newline
        if (!message.empty() && (message.back() == '\n' || message.back() == '\r')) {
            message.pop_back();
            if (!message.empty() && (message.back() == '\n' || message.back() == '\r')) {
                message.pop_back();
            }
        }
    } else {
        message = "Error " + std::to_string(error);
    }
    
    if (messageBuffer) {
        LocalFree(messageBuffer);
    }
    
    return message;
}

// Test DLL loading
void TestDllLoad(DllInfo& dll) {
    std::wcout << L"\nTesting: " << dll.path << std::endl;
    std::wcout << L"Description: " << dll.description << std::endl;
    
    // Check if file exists
    DWORD attributes = GetFileAttributesW(dll.path.c_str());
    if (attributes == INVALID_FILE_ATTRIBUTES) {
        dll.loaded = false;
        dll.error = "File not found";
        std::cout << "Result: FAILED - " << dll.error << std::endl;
        return;
    }
    
    // Try to load the DLL
    SetLastError(0);
    dll.hModule = LoadLibraryW(dll.path.c_str());
    
    if (dll.hModule) {
        dll.loaded = true;
        std::cout << "Result: SUCCESS - DLL loaded at 0x" << std::hex << dll.hModule << std::dec << std::endl;
        
        // Try to get function pointers
        dll.pDllGetClassObject = (DllGetClassObjectFunc)GetProcAddress(dll.hModule, "DllGetClassObject");
        dll.pDllCanUnloadNow = (DllCanUnloadNowFunc)GetProcAddress(dll.hModule, "DllCanUnloadNow");
        dll.pDllRegisterServer = (DllRegisterServerFunc)GetProcAddress(dll.hModule, "DllRegisterServer");
        dll.pDllUnregisterServer = (DllUnregisterServerFunc)GetProcAddress(dll.hModule, "DllUnregisterServer");
        
        std::cout << "Exports found:" << std::endl;
        std::cout << "  DllGetClassObject: " << (dll.pDllGetClassObject ? "YES" : "NO") << std::endl;
        std::cout << "  DllCanUnloadNow: " << (dll.pDllCanUnloadNow ? "YES" : "NO") << std::endl;
        std::cout << "  DllRegisterServer: " << (dll.pDllRegisterServer ? "YES" : "NO") << std::endl;
        std::cout << "  DllUnregisterServer: " << (dll.pDllUnregisterServer ? "YES" : "NO") << std::endl;
        
        // Test DllCanUnloadNow
        if (dll.pDllCanUnloadNow) {
            HRESULT hr = dll.pDllCanUnloadNow();
            std::cout << "  DllCanUnloadNow() returned: " << (hr == S_OK ? "S_OK (can unload)" : "S_FALSE (in use)") << std::endl;
        }
    } else {
        dll.loaded = false;
        dll.error = GetLastErrorString();
        std::cout << "Result: FAILED - " << dll.error << std::endl;
    }
}

// Check if a DLL depends on the forwarder
void CheckForwarderDependency(const DllInfo& forwarder, const DllInfo& dll) {
    if (!forwarder.loaded || !dll.loaded) return;
    
    // If the forwarder loaded and this is an architecture-specific DLL,
    // check if it was loaded as a dependency
    HMODULE hModule = GetModuleHandleW(dll.path.c_str());
    if (hModule) {
        std::wcout << L"Note: " << dll.path << L" is already loaded (possibly by the forwarder)" << std::endl;
    }
}

int wmain(int argc, wchar_t* argv[]) {
    std::cout << "=====================================\n";
    std::cout << "ARM64X DLL Loading Test\n";
    std::cout << "=====================================\n";
    std::cout << "Test app architecture: " << GetArchitecture() << std::endl;
    std::cout << "Process ID: " << GetCurrentProcessId() << std::endl;
    
    // Determine build configuration
    std::wstring config = L"Release";
    if (argc > 1 && (_wcsicmp(argv[1], L"debug") == 0 || _wcsicmp(argv[1], L"Debug") == 0)) {
        config = L"Debug";
    }
    std::wcout << L"Configuration: " << config << std::endl;
    
    // Define DLLs to test
    std::vector<DllInfo> dlls = {
        {L"tsf\\build-x64\\" + config + L"\\KeyMagicTSF_x64.dll", L"x64 Implementation DLL", nullptr, false, ""},
        {L"tsf\\build-arm64\\" + config + L"\\KeyMagicTSF_arm64.dll", L"ARM64 Implementation DLL", nullptr, false, ""},
        {L"tsf\\build-arm64x\\KeyMagicTSF.dll", L"ARM64X Forwarder DLL", nullptr, false, ""},
        {L"tsf\\build-arm64x\\KeyMagicTSF_x64.dll", L"x64 DLL (copied to ARM64X dir)", nullptr, false, ""},
        {L"tsf\\build-arm64x\\KeyMagicTSF_arm64.dll", L"ARM64 DLL (copied to ARM64X dir)", nullptr, false, ""}
    };
    
    // Test each DLL
    for (auto& dll : dlls) {
        TestDllLoad(dll);
    }
    
    // Summary
    std::cout << "\n=====================================\n";
    std::cout << "Summary:\n";
    std::cout << "=====================================\n";
    
    int successCount = 0;
    for (const auto& dll : dlls) {
        std::wcout << std::left << std::setw(40) << dll.path;
        if (dll.loaded) {
            std::cout << " [OK]" << std::endl;
            successCount++;
        } else {
            std::cout << " [FAILED] - " << dll.error << std::endl;
        }
    }
    
    std::cout << "\nLoaded: " << successCount << "/" << dlls.size() << " DLLs" << std::endl;
    
    // Architecture-specific expectations
    std::cout << "\nExpected behavior for " << GetArchitecture() << " process:\n";
    if (GetArchitecture() == "x64") {
        std::cout << "- ARM64X forwarder should load and forward to x64 DLL\n";
        std::cout << "- x64 DLLs should load directly\n";
        std::cout << "- ARM64 DLLs should fail to load (wrong architecture)\n";
    } else if (GetArchitecture() == "ARM64") {
        std::cout << "- ARM64X forwarder should load and forward to ARM64 DLL\n";
        std::cout << "- ARM64 DLLs should load directly\n";
        std::cout << "- x64 DLLs should fail to load (wrong architecture)\n";
    }
    
    // Check for forwarder behavior
    if (dlls[0].loaded) {
        std::cout << "\nChecking forwarder behavior:\n";
        CheckForwarderDependency(dlls[0], dlls[3]); // Check x64 in ARM64X dir
        CheckForwarderDependency(dlls[0], dlls[4]); // Check ARM64 in ARM64X dir
    }
    
    // Cleanup
    std::cout << "\nCleaning up...\n";
    for (auto& dll : dlls) {
        if (dll.hModule) {
            FreeLibrary(dll.hModule);
        }
    }
    
    std::cout << "\nTest complete. Press Enter to exit...";
    std::cin.get();
    
    return 0;
}