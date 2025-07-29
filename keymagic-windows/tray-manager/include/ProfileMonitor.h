#pragma once

#include "Common.h"
#include <msctf.h>

class ProfileMonitor {
public:
    ProfileMonitor();
    ~ProfileMonitor();
    
    // Initialize COM and profile manager
    bool Initialize();
    
    // Check if KeyMagic is currently active
    bool IsKeyMagicActive();
    
    // Get active profile info
    bool GetActiveProfile(TF_INPUTPROCESSORPROFILE& profile);
    
    // Verify focus state reported by TIP
    bool VerifyFocusState(bool tipReportedFocus);
    
    // Get active keyboard ID from profile
    std::wstring GetActiveKeyboardId();

private:
    ITfInputProcessorProfileMgr* m_pProfileMgr;
    GUID m_keymagicClsid;
    bool m_initialized;
};