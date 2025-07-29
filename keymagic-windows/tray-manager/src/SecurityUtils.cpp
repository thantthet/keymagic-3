#include "SecurityUtils.h"
#include <memory>

SECURITY_ATTRIBUTES* SecurityUtils::CreateLowIntegritySecurityAttributes() {
    const wchar_t* szSD = L"D:"                    // DACL
                         L"(A;;GA;;;WD)"           // Allow all users
                         L"(A;;GA;;;AC)"           // Allow all app containers
                         L"(A;;GA;;;S-1-15-2-1)"   // Allow all app packages
                         L"(A;;GA;;;S-1-16-4096)"; // Allow low integrity
    
    SECURITY_ATTRIBUTES* pSA = new SECURITY_ATTRIBUTES;
    pSA->nLength = sizeof(SECURITY_ATTRIBUTES);
    pSA->bInheritHandle = FALSE;
    
    if (!ConvertStringSecurityDescriptorToSecurityDescriptorW(
            szSD,
            SDDL_REVISION_1,
            &pSA->lpSecurityDescriptor,
            nullptr)) {
        delete pSA;
        return nullptr;
    }
    
    return pSA;
}

void SecurityUtils::FreeSecurityAttributes(SECURITY_ATTRIBUTES* pSA) {
    if (pSA) {
        if (pSA->lpSecurityDescriptor) {
            LocalFree(pSA->lpSecurityDescriptor);
        }
        delete pSA;
    }
}

HANDLE SecurityUtils::CreateGlobalEvent(const wchar_t* name, bool manualReset) {
    SECURITY_ATTRIBUTES* pSA = CreateLowIntegritySecurityAttributes();
    if (!pSA) {
        return nullptr;
    }
    
    HANDLE hEvent = CreateEventW(
        pSA,
        manualReset ? TRUE : FALSE,
        FALSE,
        name
    );
    
    FreeSecurityAttributes(pSA);
    return hEvent;
}

HANDLE SecurityUtils::OpenGlobalEvent(const wchar_t* name) {
    return OpenEventW(EVENT_ALL_ACCESS, FALSE, name);
}