#include "ProfileMonitor.h"
#include <initguid.h>

// Define the GUID for KeyMagic TIP
DEFINE_GUID(CLSID_KeyMagicTIP, 0xB9F5A039, 0x9008, 0x4D0F, 0x97, 0xF5, 0x26, 0xAA, 0x6D, 0x3C, 0x5F, 0x06);

ProfileMonitor::ProfileMonitor()
    : m_pProfileMgr(nullptr)
    , m_initialized(false) {
    m_keymagicClsid = CLSID_KeyMagicTIP;
}

ProfileMonitor::~ProfileMonitor() {
    if (m_pProfileMgr) {
        m_pProfileMgr->Release();
        m_pProfileMgr = nullptr;
    }
}

bool ProfileMonitor::Initialize() {
    if (m_initialized) {
        return true;
    }
    
    HRESULT hr = CoCreateInstance(CLSID_TF_InputProcessorProfiles,
                                  nullptr,
                                  CLSCTX_INPROC_SERVER,
                                  IID_ITfInputProcessorProfileMgr,
                                  reinterpret_cast<void**>(&m_pProfileMgr));
    
    if (SUCCEEDED(hr)) {
        m_initialized = true;
        return true;
    }
    
    return false;
}

bool ProfileMonitor::IsKeyMagicActive() {
    if (!m_initialized || !m_pProfileMgr) {
        return false;
    }
    
    TF_INPUTPROCESSORPROFILE profile;
    HRESULT hr = m_pProfileMgr->GetActiveProfile(GUID_TFCAT_TIP_KEYBOARD, &profile);
    
    if (SUCCEEDED(hr)) {
        return IsEqualGUID(profile.clsid, m_keymagicClsid);
    }
    
    return false;
}

bool ProfileMonitor::GetActiveProfile(TF_INPUTPROCESSORPROFILE& profile) {
    if (!m_initialized || !m_pProfileMgr) {
        return false;
    }
    
    return SUCCEEDED(m_pProfileMgr->GetActiveProfile(GUID_TFCAT_TIP_KEYBOARD, &profile));
}

bool ProfileMonitor::VerifyFocusState(bool tipReportedFocus) {
    bool actuallyActive = IsKeyMagicActive();
    
    // If TSF says KeyMagic is active, trust that over TIP report
    // This handles cases where focus changes rapidly
    if (actuallyActive != tipReportedFocus) {
        return actuallyActive;
    }
    
    return tipReportedFocus;
}

std::wstring ProfileMonitor::GetActiveKeyboardId() {
    if (!m_initialized || !m_pProfileMgr) {
        return L"";
    }
    
    TF_INPUTPROCESSORPROFILE profile;
    if (GetActiveProfile(profile)) {
        if (IsEqualGUID(profile.clsid, m_keymagicClsid)) {
            // The profile GUID contains the keyboard ID in its data
            // For now, we'll return empty and rely on TIP reporting
            // In a full implementation, we would decode the profile data
            return L"";
        }
    }
    
    return L"";
}