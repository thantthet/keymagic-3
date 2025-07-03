#include "KeyMagicTextService.h"
#include <string>
#include <vector>

// Global variables
HINSTANCE g_hInst = NULL;
LONG g_cRefDll = 0;

// Helper function to convert UTF-8 to UTF-16
std::wstring CKeyMagicTextService::_Utf8ToUtf16(const char* utf8) {
    if (!utf8) return L"";
    
    int len = MultiByteToWideChar(CP_UTF8, 0, utf8, -1, NULL, 0);
    if (len == 0) return L"";
    
    std::vector<wchar_t> buffer(len);
    MultiByteToWideChar(CP_UTF8, 0, utf8, -1, buffer.data(), len);
    
    return std::wstring(buffer.data());
}

// CClassFactory implementation
CClassFactory::CClassFactory() : m_cRef(1) {
    InterlockedIncrement(&g_cRefDll);
}

CClassFactory::~CClassFactory() {
    InterlockedDecrement(&g_cRefDll);
}

STDAPI CClassFactory::QueryInterface(REFIID riid, void** ppvObj) {
    if (IsEqualIID(riid, IID_IClassFactory) || IsEqualIID(riid, IID_IUnknown)) {
        *ppvObj = this;
        AddRef();
        return S_OK;
    }
    *ppvObj = NULL;
    return E_NOINTERFACE;
}

STDAPI_(ULONG) CClassFactory::AddRef() {
    return InterlockedIncrement(&m_cRef);
}

STDAPI_(ULONG) CClassFactory::Release() {
    LONG cRef = InterlockedDecrement(&m_cRef);
    if (cRef == 0) {
        delete this;
    }
    return cRef;
}

STDAPI CClassFactory::CreateInstance(IUnknown* pUnkOuter, REFIID riid, void** ppvObj) {
    if (pUnkOuter != NULL) {
        return CLASS_E_NOAGGREGATION;
    }
    
    CKeyMagicTextService* pTextService = new CKeyMagicTextService();
    if (!pTextService) {
        return E_OUTOFMEMORY;
    }
    
    HRESULT hr = pTextService->QueryInterface(riid, ppvObj);
    pTextService->Release();
    
    return hr;
}

STDAPI CClassFactory::LockServer(BOOL fLock) {
    if (fLock) {
        InterlockedIncrement(&g_cRefDll);
    } else {
        InterlockedDecrement(&g_cRefDll);
    }
    return S_OK;
}

// CKeyMagicTextService implementation
CKeyMagicTextService::CKeyMagicTextService() 
    : m_cRef(1)
    , m_pThreadMgr(NULL)
    , m_tfClientId(TF_CLIENTID_NULL)
    , m_pTextEditSink(NULL)
    , m_pComposition(NULL)
    , m_dwThreadMgrEventSinkCookie(TF_INVALID_COOKIE)
    , m_dwKeyEventSinkCookie(TF_INVALID_COOKIE)
    , m_pEngine(NULL)
    , m_fComposing(false) {
    
    InterlockedIncrement(&g_cRefDll);
    
    // Create KeyMagic engine instance
    m_pEngine = keymagic_engine_new();
}

CKeyMagicTextService::~CKeyMagicTextService() {
    if (m_pEngine) {
        keymagic_engine_free(m_pEngine);
    }
    InterlockedDecrement(&g_cRefDll);
}

STDAPI CKeyMagicTextService::QueryInterface(REFIID riid, void** ppvObj) {
    if (ppvObj == NULL) {
        return E_INVALIDARG;
    }
    
    *ppvObj = NULL;
    
    if (IsEqualIID(riid, IID_IUnknown) || IsEqualIID(riid, IID_ITfTextInputProcessor)) {
        *ppvObj = (ITfTextInputProcessor*)this;
    } else if (IsEqualIID(riid, IID_ITfThreadMgrEventSink)) {
        *ppvObj = (ITfThreadMgrEventSink*)this;
    } else if (IsEqualIID(riid, IID_ITfKeyEventSink)) {
        *ppvObj = (ITfKeyEventSink*)this;
    }
    
    if (*ppvObj) {
        AddRef();
        return S_OK;
    }
    
    return E_NOINTERFACE;
}

STDAPI_(ULONG) CKeyMagicTextService::AddRef() {
    return InterlockedIncrement(&m_cRef);
}

STDAPI_(ULONG) CKeyMagicTextService::Release() {
    LONG cRef = InterlockedDecrement(&m_cRef);
    if (cRef == 0) {
        delete this;
    }
    return cRef;
}

// ITfTextInputProcessor implementation
STDAPI CKeyMagicTextService::Activate(ITfThreadMgr* ptim, TfClientId tid) {
    m_pThreadMgr = ptim;
    m_pThreadMgr->AddRef();
    m_tfClientId = tid;
    
    // Initialize event sinks
    _InitThreadMgrEventSink();
    _InitKeyEventSink();
    
    return S_OK;
}

STDAPI CKeyMagicTextService::Deactivate() {
    // Cleanup event sinks
    _UninitKeyEventSink();
    _UninitThreadMgrEventSink();
    
    if (m_pThreadMgr) {
        m_pThreadMgr->Release();
        m_pThreadMgr = NULL;
    }
    
    m_tfClientId = TF_CLIENTID_NULL;
    
    return S_OK;
}

// ITfKeyEventSink implementation
STDAPI CKeyMagicTextService::OnSetFocus(BOOL fForeground) {
    return S_OK;
}

STDAPI CKeyMagicTextService::OnTestKeyDown(ITfContext* pic, WPARAM wParam, LPARAM lParam, BOOL* pfEaten) {
    *pfEaten = FALSE;
    
    // Quick test - check if this key might be handled
    // For now, we'll test all keys and decide in OnKeyDown
    *pfEaten = TRUE;
    
    return S_OK;
}

STDAPI CKeyMagicTextService::OnKeyDown(ITfContext* pic, WPARAM wParam, LPARAM lParam, BOOL* pfEaten) {
    *pfEaten = FALSE;
    
    if (!m_pEngine) {
        return S_OK;
    }
    
    // Get modifier states
    int shift = (GetKeyState(VK_SHIFT) & 0x8000) ? 1 : 0;
    int ctrl = (GetKeyState(VK_CONTROL) & 0x8000) ? 1 : 0;
    int alt = (GetKeyState(VK_MENU) & 0x8000) ? 1 : 0;
    int caps = (GetKeyState(VK_CAPITAL) & 0x0001) ? 1 : 0;
    
    // Process key through engine
    ProcessKeyOutput output = {0};
    KeyMagicResult result = keymagic_engine_process_key(
        m_pEngine,
        (int)wParam,
        shift,
        ctrl,
        alt,
        caps,
        &output
    );
    
    if (result != KEYMAGIC_SUCCESS) {
        return S_OK;
    }
    
    *pfEaten = output.consumed ? TRUE : FALSE;
    
    // Handle output based on action type
    if (output.action_type != 0 && output.text) {
        std::wstring text = _Utf8ToUtf16(output.text);
        
        switch (output.action_type) {
            case 1: // CommitText
                _CommitText(pic, text);
                break;
            case 2: // UpdateComposition
                _UpdateComposition(pic, text);
                break;
            case 3: // Reset
                _EndComposition(pic);
                break;
        }
    }
    
    // Free allocated string
    if (output.text) {
        keymagic_free_string(output.text);
    }
    
    return S_OK;
}

STDAPI CKeyMagicTextService::OnTestKeyUp(ITfContext* pic, WPARAM wParam, LPARAM lParam, BOOL* pfEaten) {
    *pfEaten = FALSE;
    return S_OK;
}

STDAPI CKeyMagicTextService::OnKeyUp(ITfContext* pic, WPARAM wParam, LPARAM lParam, BOOL* pfEaten) {
    *pfEaten = FALSE;
    return S_OK;
}

STDAPI CKeyMagicTextService::OnPreservedKey(ITfContext* pic, REFGUID rguid, BOOL* pfEaten) {
    *pfEaten = FALSE;
    return S_OK;
}

// Helper methods
HRESULT CKeyMagicTextService::_InitThreadMgrEventSink() {
    ITfSource* pSource;
    if (SUCCEEDED(m_pThreadMgr->QueryInterface(IID_ITfSource, (void**)&pSource))) {
        pSource->AdviseSink(IID_ITfThreadMgrEventSink, (ITfThreadMgrEventSink*)this, &m_dwThreadMgrEventSinkCookie);
        pSource->Release();
    }
    return S_OK;
}

HRESULT CKeyMagicTextService::_UninitThreadMgrEventSink() {
    ITfSource* pSource;
    if (m_dwThreadMgrEventSinkCookie != TF_INVALID_COOKIE) {
        if (SUCCEEDED(m_pThreadMgr->QueryInterface(IID_ITfSource, (void**)&pSource))) {
            pSource->UnadviseSink(m_dwThreadMgrEventSinkCookie);
            pSource->Release();
        }
    }
    return S_OK;
}

HRESULT CKeyMagicTextService::_InitKeyEventSink() {
    ITfKeystrokeMgr* pKeystrokeMgr;
    if (SUCCEEDED(m_pThreadMgr->QueryInterface(IID_ITfKeystrokeMgr, (void**)&pKeystrokeMgr))) {
        pKeystrokeMgr->AdviseKeyEventSink(m_tfClientId, (ITfKeyEventSink*)this, TRUE);
        pKeystrokeMgr->Release();
    }
    return S_OK;
}

HRESULT CKeyMagicTextService::_UninitKeyEventSink() {
    ITfKeystrokeMgr* pKeystrokeMgr;
    if (SUCCEEDED(m_pThreadMgr->QueryInterface(IID_ITfKeystrokeMgr, (void**)&pKeystrokeMgr))) {
        pKeystrokeMgr->UnadviseKeyEventSink(m_tfClientId);
        pKeystrokeMgr->Release();
    }
    return S_OK;
}

// ITfThreadMgrEventSink methods (simplified implementation)
STDAPI CKeyMagicTextService::OnInitDocumentMgr(ITfDocumentMgr* pdim) {
    return S_OK;
}

STDAPI CKeyMagicTextService::OnUninitDocumentMgr(ITfDocumentMgr* pdim) {
    return S_OK;
}

STDAPI CKeyMagicTextService::OnSetFocus(ITfDocumentMgr* pdimFocus, ITfDocumentMgr* pdimPrevFocus) {
    return S_OK;
}

STDAPI CKeyMagicTextService::OnPushContext(ITfContext* pic) {
    return S_OK;
}

STDAPI CKeyMagicTextService::OnPopContext(ITfContext* pic) {
    return S_OK;
}

// Composition handling (simplified - full implementation would be more complex)
HRESULT CKeyMagicTextService::_StartComposition(ITfContext* pContext) {
    // TODO: Implement composition start
    m_fComposing = true;
    return S_OK;
}

HRESULT CKeyMagicTextService::_EndComposition(ITfContext* pContext) {
    // TODO: Implement composition end
    m_fComposing = false;
    m_compositionString.clear();
    return S_OK;
}

HRESULT CKeyMagicTextService::_UpdateComposition(ITfContext* pContext, const std::wstring& text) {
    // TODO: Implement composition update
    m_compositionString = text;
    return S_OK;
}

HRESULT CKeyMagicTextService::_CommitText(ITfContext* pContext, const std::wstring& text) {
    // TODO: Implement text commit
    _EndComposition(pContext);
    return S_OK;
}