#pragma once

#include <windows.h>
#include <sddl.h>
#include <memory>

class SecurityUtils {
public:
    // Create security descriptor allowing access from low integrity and app containers
    static SECURITY_ATTRIBUTES* CreateLowIntegritySecurityAttributes();
    
    // Free security attributes
    static void FreeSecurityAttributes(SECURITY_ATTRIBUTES* pSA);
    
    // Create global event with appropriate permissions
    static HANDLE CreateGlobalEvent(const wchar_t* name, bool manualReset = false);
    
    // Open existing global event
    static HANDLE OpenGlobalEvent(const wchar_t* name);
};