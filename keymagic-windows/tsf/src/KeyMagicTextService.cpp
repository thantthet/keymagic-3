#include "KeyMagicTextService.h"
#include "Globals.h"
#include <string>
#include <codecvt>
#include <locale>

// Helper function to convert UTF-8 to UTF-16
std::wstring ConvertUtf8ToUtf16(const std::string& utf8)
{
    if (utf8.empty())
        return std::wstring();
        
    int size_needed = MultiByteToWideChar(CP_UTF8, 0, utf8.c_str(), static_cast<int>(utf8.length()), NULL, 0);
    std::wstring utf16(size_needed, 0);
    MultiByteToWideChar(CP_UTF8, 0, utf8.c_str(), static_cast<int>(utf8.length()), &utf16[0], size_needed);
    return utf16;
}

CKeyMagicTextService::CKeyMagicTextService()
{
    m_cRef = 1;
    m_pThreadMgr = nullptr;
    m_tfClientId = TF_CLIENTID_NULL;
    m_dwThreadMgrEventSinkCookie = TF_INVALID_COOKIE;
    m_pDocMgrFocus = nullptr;
    m_pEngine = nullptr;
    m_pComposition = nullptr;
    m_fComposing = FALSE;
    
    InitializeCriticalSection(&m_cs);
    DllAddRef();
}

CKeyMagicTextService::~CKeyMagicTextService()
{
    UninitializeEngine();
    DeleteCriticalSection(&m_cs);
    DllRelease();
}

// IUnknown
STDAPI CKeyMagicTextService::QueryInterface(REFIID riid, void **ppvObject)
{
    if (ppvObject == nullptr)
        return E_INVALIDARG;

    *ppvObject = nullptr;

    if (IsEqualIID(riid, IID_IUnknown))
    {
        *ppvObject = static_cast<IUnknown*>(static_cast<ITfTextInputProcessor*>(this));
    }
    else if (IsEqualIID(riid, IID_ITfTextInputProcessor))
    {
        *ppvObject = static_cast<ITfTextInputProcessor*>(this);
    }
    else if (IsEqualIID(riid, IID_ITfThreadMgrEventSink))
    {
        *ppvObject = static_cast<ITfThreadMgrEventSink*>(this);
    }
    else if (IsEqualIID(riid, IID_ITfKeyEventSink))
    {
        *ppvObject = static_cast<ITfKeyEventSink*>(this);
    }
    else if (IsEqualIID(riid, IID_ITfCompositionSink))
    {
        *ppvObject = static_cast<ITfCompositionSink*>(this);
    }
    else
    {
        return E_NOINTERFACE;
    }

    AddRef();
    return S_OK;
}

STDAPI_(ULONG) CKeyMagicTextService::AddRef()
{
    return InterlockedIncrement(&m_cRef);
}

STDAPI_(ULONG) CKeyMagicTextService::Release()
{
    LONG cRef = InterlockedDecrement(&m_cRef);
    if (cRef == 0)
    {
        delete this;
    }
    return cRef;
}

// ITfTextInputProcessor
STDAPI CKeyMagicTextService::Activate(ITfThreadMgr *ptim, TfClientId tid)
{
    m_pThreadMgr = ptim;
    m_pThreadMgr->AddRef();
    m_tfClientId = tid;

    // Initialize the engine
    if (!InitializeEngine())
        return E_FAIL;

    // Register thread manager event sink
    ITfSource *pSource = nullptr;
    if (SUCCEEDED(m_pThreadMgr->QueryInterface(IID_ITfSource, (void**)&pSource)))
    {
        pSource->AdviseSink(IID_ITfThreadMgrEventSink, 
                           static_cast<ITfThreadMgrEventSink*>(this), 
                           &m_dwThreadMgrEventSinkCookie);
        pSource->Release();
    }

    // Register key event sink
    ITfKeystrokeMgr *pKeystrokeMgr = nullptr;
    if (SUCCEEDED(m_pThreadMgr->QueryInterface(IID_ITfKeystrokeMgr, (void**)&pKeystrokeMgr)))
    {
        pKeystrokeMgr->AdviseKeyEventSink(m_tfClientId, 
                                          static_cast<ITfKeyEventSink*>(this), 
                                          TRUE);
        pKeystrokeMgr->Release();
    }

    return S_OK;
}

STDAPI CKeyMagicTextService::Deactivate()
{
    // Unregister key event sink
    ITfKeystrokeMgr *pKeystrokeMgr = nullptr;
    if (m_pThreadMgr && SUCCEEDED(m_pThreadMgr->QueryInterface(IID_ITfKeystrokeMgr, (void**)&pKeystrokeMgr)))
    {
        pKeystrokeMgr->UnadviseKeyEventSink(m_tfClientId);
        pKeystrokeMgr->Release();
    }

    // Unregister thread manager event sink
    ITfSource *pSource = nullptr;
    if (m_pThreadMgr && SUCCEEDED(m_pThreadMgr->QueryInterface(IID_ITfSource, (void**)&pSource)))
    {
        pSource->UnadviseSink(m_dwThreadMgrEventSinkCookie);
        pSource->Release();
    }

    // Clean up
    if (m_pThreadMgr)
    {
        m_pThreadMgr->Release();
        m_pThreadMgr = nullptr;
    }

    m_tfClientId = TF_CLIENTID_NULL;
    m_dwThreadMgrEventSinkCookie = TF_INVALID_COOKIE;

    // Uninitialize the engine
    UninitializeEngine();

    return S_OK;
}

// ITfThreadMgrEventSink
STDAPI CKeyMagicTextService::OnInitDocumentMgr(ITfDocumentMgr *pdim)
{
    return S_OK;
}

STDAPI CKeyMagicTextService::OnUninitDocumentMgr(ITfDocumentMgr *pdim)
{
    return S_OK;
}

STDAPI CKeyMagicTextService::OnSetFocus(ITfDocumentMgr *pdimFocus, ITfDocumentMgr *pdimPrevFocus)
{
    if (m_pDocMgrFocus)
        m_pDocMgrFocus->Release();

    m_pDocMgrFocus = pdimFocus;

    if (m_pDocMgrFocus)
        m_pDocMgrFocus->AddRef();

    // Reset engine when focus changes
    if (m_pEngine)
    {
        EnterCriticalSection(&m_cs);
        keymagic_engine_reset(m_pEngine);
        LeaveCriticalSection(&m_cs);
    }

    return S_OK;
}

STDAPI CKeyMagicTextService::OnPushContext(ITfContext *pic)
{
    return S_OK;
}

STDAPI CKeyMagicTextService::OnPopContext(ITfContext *pic)
{
    return S_OK;
}

// ITfKeyEventSink
STDAPI CKeyMagicTextService::OnSetFocus(BOOL fForeground)
{
    return S_OK;
}

STDAPI CKeyMagicTextService::OnTestKeyDown(ITfContext *pic, WPARAM wParam, LPARAM lParam, BOOL *pfEaten)
{
    *pfEaten = IsKeyEaten(pic, wParam, lParam);
    return S_OK;
}

STDAPI CKeyMagicTextService::OnKeyDown(ITfContext *pic, WPARAM wParam, LPARAM lParam, BOOL *pfEaten)
{
    ProcessKeyInput(pic, wParam, lParam, pfEaten);
    return S_OK;
}

STDAPI CKeyMagicTextService::OnTestKeyUp(ITfContext *pic, WPARAM wParam, LPARAM lParam, BOOL *pfEaten)
{
    *pfEaten = FALSE;
    return S_OK;
}

STDAPI CKeyMagicTextService::OnKeyUp(ITfContext *pic, WPARAM wParam, LPARAM lParam, BOOL *pfEaten)
{
    *pfEaten = FALSE;
    return S_OK;
}

STDAPI CKeyMagicTextService::OnPreservedKey(ITfContext *pic, REFGUID rguid, BOOL *pfEaten)
{
    *pfEaten = FALSE;
    return S_OK;
}

// ITfCompositionSink
STDAPI CKeyMagicTextService::OnCompositionTerminated(TfEditCookie ecWrite, ITfComposition *pComposition)
{
    // Composition has been terminated
    if (m_pComposition == pComposition)
    {
        m_pComposition = nullptr;
        m_fComposing = FALSE;
    }
    return S_OK;
}

// Helper methods
BOOL CKeyMagicTextService::InitializeEngine()
{
    EnterCriticalSection(&m_cs);
    
    m_pEngine = keymagic_engine_new();
    
    if (m_pEngine)
    {
        // Try to load a default keyboard from registry
        HKEY hKey;
        if (RegOpenKeyEx(HKEY_CURRENT_USER, L"Software\\KeyMagic\\Settings", 
                        0, KEY_READ, &hKey) == ERROR_SUCCESS)
        {
            WCHAR defaultKeyboard[MAX_PATH] = {0};
            DWORD size = sizeof(defaultKeyboard);
            
            if (RegQueryValueEx(hKey, L"DefaultKeyboard", NULL, NULL, 
                              (LPBYTE)defaultKeyboard, &size) == ERROR_SUCCESS)
            {
                // Now get the path for this keyboard
                HKEY hKeyboardKey;
                std::wstring keyPath = L"Software\\KeyMagic\\Keyboards\\";
                keyPath += defaultKeyboard;
                
                if (RegOpenKeyEx(HKEY_CURRENT_USER, keyPath.c_str(), 
                               0, KEY_READ, &hKeyboardKey) == ERROR_SUCCESS)
                {
                    WCHAR km2Path[MAX_PATH] = {0};
                    size = sizeof(km2Path);
                    
                    if (RegQueryValueEx(hKeyboardKey, L"Path", NULL, NULL, 
                                      (LPBYTE)km2Path, &size) == ERROR_SUCCESS)
                    {
                        LoadKeyboard(km2Path);
                    }
                    
                    RegCloseKey(hKeyboardKey);
                }
            }
            
            RegCloseKey(hKey);
        }
    }
    
    LeaveCriticalSection(&m_cs);
    
    return (m_pEngine != nullptr);
}

void CKeyMagicTextService::UninitializeEngine()
{
    EnterCriticalSection(&m_cs);
    
    if (m_pEngine)
    {
        keymagic_engine_free(m_pEngine);
        m_pEngine = nullptr;
    }
    
    LeaveCriticalSection(&m_cs);
}

BOOL CKeyMagicTextService::LoadKeyboard(const std::wstring& km2Path)
{
    if (!m_pEngine)
        return FALSE;
        
    EnterCriticalSection(&m_cs);
    
    // Convert wide string to UTF-8
    std::wstring_convert<std::codecvt_utf8<wchar_t>> converter;
    std::string utf8Path = converter.to_bytes(km2Path);
    
    KeyMagicResult result = keymagic_engine_load_keyboard(m_pEngine, utf8Path.c_str());
    
    if (result == KeyMagicResult_Success)
    {
        m_currentKeyboardPath = km2Path;
    }
    
    LeaveCriticalSection(&m_cs);
    
    return (result == KeyMagicResult_Success);
}

void CKeyMagicTextService::ProcessKeyInput(ITfContext *pic, WPARAM wParam, LPARAM lParam, BOOL *pfEaten)
{
    *pfEaten = FALSE;
    
    if (!m_pEngine)
        return;
        
    EnterCriticalSection(&m_cs);
    
    ProcessKeyOutput output = {0};
    
    // Convert Windows key to FFI format
    int keyCode = static_cast<int>(wParam);
    char character = 0;
    
    // Map virtual key to character
    BYTE keyState[256];
    if (GetKeyboardState(keyState))
    {
        WCHAR unicodeChar[2] = {0};
        int result = ToUnicode(static_cast<UINT>(wParam), 
                              static_cast<UINT>(lParam >> 16) & 0xFF, 
                              keyState, unicodeChar, 2, 0);
        if (result == 1 && unicodeChar[0] < 128)
        {
            character = static_cast<char>(unicodeChar[0]);
        }
    }
    
    // Get modifier states
    int shift = (GetKeyState(VK_SHIFT) & 0x8000) ? 1 : 0;
    int ctrl = (GetKeyState(VK_CONTROL) & 0x8000) ? 1 : 0;
    int alt = (GetKeyState(VK_MENU) & 0x8000) ? 1 : 0;
    int capsLock = (GetKeyState(VK_CAPITAL) & 0x0001) ? 1 : 0;
    
    // Process key through engine
    KeyMagicResult result = keymagic_engine_process_key(
        m_pEngine, keyCode, character, 
        shift, ctrl, alt, capsLock, &output
    );
    
    if (result == KeyMagicResult_Success)
    {
        // Check if we should commit text
        bool shouldCommit = false;
        std::wstring textToCommit;
        std::wstring composingText;
        
        if (output.composing_text)
        {
            // Convert UTF-8 to UTF-16
            std::string utf8Composing(output.composing_text);
            composingText = ConvertUtf8ToUtf16(utf8Composing);
            
            // Check if we should commit based on key
            switch (wParam)
            {
                case VK_SPACE:
                    if (output.is_processed)
                    {
                        // Engine processed space, check if composing ends with space
                        if (!composingText.empty() && composingText.back() == L' ')
                        {
                            shouldCommit = true;
                            textToCommit = composingText;
                        }
                    }
                    else
                    {
                        // Engine didn't process space, commit current text + space
                        shouldCommit = true;
                        textToCommit = composingText + L" ";
                    }
                    break;
                    
                case VK_RETURN:  // Enter key - commit without adding newline
                case VK_TAB:     // Tab key - commit without adding tab
                    if (!composingText.empty())
                    {
                        shouldCommit = true;
                        textToCommit = composingText;
                    }
                    break;
                    
                case VK_ESCAPE:  // Escape - cancel composition
                    keymagic_engine_reset(m_pEngine);
                    if (m_pComposition)
                    {
                        // Request edit session to terminate composition
                        CEditSession *pEditSession = new CEditSession(this, pic, 
                            CEditSession::EditAction::TerminateComposition, L"", L"");
                        HRESULT hr;
                        pic->RequestEditSession(m_tfClientId, pEditSession, TF_ES_SYNC | TF_ES_READWRITE, &hr);
                        pEditSession->Release();
                    }
                    *pfEaten = TRUE;
                    goto cleanup;
                    
                case VK_BACK:  // Backspace - let engine handle it
                    // Engine will handle backspace internally
                    break;
            }
            
            // Update composition or commit text
            if (shouldCommit || !composingText.empty())
            {
                CEditSession *pEditSession = new CEditSession(this, pic, 
                    shouldCommit ? CEditSession::EditAction::CommitText : CEditSession::EditAction::UpdateComposition,
                    textToCommit, composingText);
                HRESULT hr;
                pic->RequestEditSession(m_tfClientId, pEditSession, TF_ES_SYNC | TF_ES_READWRITE, &hr);
                pEditSession->Release();
            }
        }
        
        *pfEaten = output.is_processed ? TRUE : FALSE;
    }
    
cleanup:
    // Free allocated strings
    if (output.text) keymagic_free_string(output.text);
    if (output.composing_text) keymagic_free_string(output.composing_text);
    
    LeaveCriticalSection(&m_cs);
}

void CKeyMagicTextService::UpdateComposition(ITfContext *pic, bool shouldCommit, 
                                            const std::wstring& textToCommit, 
                                            const std::wstring& composingText)
{
    // TODO: Implement composition update logic
}

void CKeyMagicTextService::CommitText(ITfContext *pic, const std::wstring& text)
{
    // TODO: Implement text commit logic
}

void CKeyMagicTextService::TerminateComposition(ITfContext *pic)
{
    // TODO: Implement composition termination logic
}

BOOL CKeyMagicTextService::IsKeyEaten(ITfContext *pic, WPARAM wParam, LPARAM lParam)
{
    // TODO: Implement key filtering logic
    // For now, eat all printable keys
    return (wParam >= 0x20 && wParam <= 0x7E) || wParam == VK_BACK;
}

//////////////////////////////////////////////////////////////////////////////
// CEditSession implementation
//////////////////////////////////////////////////////////////////////////////

CEditSession::CEditSession(CKeyMagicTextService *pTextService, ITfContext *pContext, 
                         EditAction action, const std::wstring& textToCommit, 
                         const std::wstring& composingText)
{
    m_cRef = 1;
    m_pTextService = pTextService;
    m_pTextService->AddRef();
    m_pContext = pContext;
    m_pContext->AddRef();
    m_action = action;
    m_textToCommit = textToCommit;
    m_composingText = composingText;
}

CEditSession::~CEditSession()
{
    if (m_pTextService)
        m_pTextService->Release();
    if (m_pContext)
        m_pContext->Release();
}

// IUnknown
STDAPI CEditSession::QueryInterface(REFIID riid, void **ppvObject)
{
    if (ppvObject == nullptr)
        return E_INVALIDARG;

    *ppvObject = nullptr;

    if (IsEqualIID(riid, IID_IUnknown) || IsEqualIID(riid, IID_ITfEditSession))
    {
        *ppvObject = static_cast<ITfEditSession*>(this);
    }
    else
    {
        return E_NOINTERFACE;
    }

    AddRef();
    return S_OK;
}

STDAPI_(ULONG) CEditSession::AddRef()
{
    return InterlockedIncrement(&m_cRef);
}

STDAPI_(ULONG) CEditSession::Release()
{
    LONG cRef = InterlockedDecrement(&m_cRef);
    if (cRef == 0)
    {
        delete this;
    }
    return cRef;
}

// ITfEditSession
STDAPI CEditSession::DoEditSession(TfEditCookie ec)
{
    switch (m_action)
    {
        case EditAction::UpdateComposition:
            UpdateCompositionString(ec);
            break;
            
        case EditAction::CommitText:
            CommitText(ec);
            TerminateComposition(ec);
            // Reset engine after commit
            if (m_pTextService->GetEngineHandle())
            {
                keymagic_engine_reset(m_pTextService->GetEngineHandle());
            }
            break;
            
        case EditAction::TerminateComposition:
            TerminateComposition(ec);
            break;
    }
    
    return S_OK;
}

void CEditSession::UpdateCompositionString(TfEditCookie ec)
{
    if (!m_pTextService->m_pComposition)
    {
        StartComposition(ec);
    }
    
    if (m_pTextService->m_pComposition)
    {
        ITfRange *pRange = nullptr;
        if (SUCCEEDED(m_pTextService->m_pComposition->GetRange(&pRange)))
        {
            // Set the composition text
            pRange->SetText(ec, 0, m_composingText.c_str(), static_cast<LONG>(m_composingText.length()));
            
            // Apply underline display attribute
            ApplyDisplayAttributes(ec, pRange);
            
            pRange->Release();
        }
    }
}

void CEditSession::StartComposition(TfEditCookie ec)
{
    ITfInsertAtSelection *pInsertAtSelection = nullptr;
    ITfRange *pRange = nullptr;
    
    if (SUCCEEDED(m_pContext->QueryInterface(IID_ITfInsertAtSelection, (void**)&pInsertAtSelection)))
    {
        if (SUCCEEDED(pInsertAtSelection->InsertTextAtSelection(ec, TF_IAS_QUERYONLY, nullptr, 0, &pRange)))
        {
            ITfContextComposition *pContextComposition = nullptr;
            if (SUCCEEDED(m_pContext->QueryInterface(IID_ITfContextComposition, (void**)&pContextComposition)))
            {
                ITfComposition *pComposition = nullptr;
                if (SUCCEEDED(pContextComposition->StartComposition(ec, pRange, 
                    static_cast<ITfCompositionSink*>(m_pTextService), &pComposition)))
                {
                    m_pTextService->m_pComposition = pComposition;
                    m_pTextService->m_fComposing = TRUE;
                }
                pContextComposition->Release();
            }
            pRange->Release();
        }
        pInsertAtSelection->Release();
    }
}

void CEditSession::CommitText(TfEditCookie ec)
{
    // First terminate any existing composition
    if (m_pTextService->m_pComposition)
    {
        TerminateComposition(ec);
    }
    
    // Insert the committed text
    ITfInsertAtSelection *pInsertAtSelection = nullptr;
    ITfRange *pRange = nullptr;
    
    if (SUCCEEDED(m_pContext->QueryInterface(IID_ITfInsertAtSelection, (void**)&pInsertAtSelection)))
    {
        if (SUCCEEDED(pInsertAtSelection->InsertTextAtSelection(
            ec, 0, m_textToCommit.c_str(), static_cast<LONG>(m_textToCommit.length()), &pRange)))
        {
            pRange->Release();
        }
        pInsertAtSelection->Release();
    }
}

void CEditSession::TerminateComposition(TfEditCookie ec)
{
    if (m_pTextService->m_pComposition)
    {
        m_pTextService->m_pComposition->EndComposition(ec);
        m_pTextService->m_pComposition->Release();
        m_pTextService->m_pComposition = nullptr;
        m_pTextService->m_fComposing = FALSE;
    }
}

void CEditSession::ApplyDisplayAttributes(TfEditCookie ec, ITfRange *pRange)
{
    // For now, we'll rely on the default composition display attributes
    // which typically shows an underline automatically
    // TODO: Implement custom display attributes if needed
}