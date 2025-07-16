#include "KeyMagicTextService.h"
#include "KeyMagicGuids.h"
#include "Globals.h"
#include "Debug.h"
#include "DirectEditSession.h"
#include "CompositionEditSession.h"
#include "Composition.h"
#include "ProcessDetector.h"
#include "KeyProcessingUtils.h"
#include "Registry.h"
#include <string>
#include <codecvt>
#include <locale>
#include <vector>
#include <algorithm>
#include <tlhelp32.h>

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

// Helper function to convert UTF-16 to UTF-8
std::string ConvertUtf16ToUtf8(const std::wstring& utf16)
{
    if (utf16.empty())
        return std::string();
        
    int size_needed = WideCharToMultiByte(CP_UTF8, 0, utf16.c_str(), static_cast<int>(utf16.length()), NULL, 0, NULL, NULL);
    std::string utf8(size_needed, 0);
    WideCharToMultiByte(CP_UTF8, 0, utf16.c_str(), static_cast<int>(utf16.length()), &utf8[0], size_needed, NULL, NULL);
    return utf8;
}

CKeyMagicTextService::CKeyMagicTextService()
{
    m_cRef = 1;
    m_pThreadMgr = nullptr;
    m_tfClientId = TF_CLIENTID_NULL;
    m_dwThreadMgrEventSinkCookie = TF_INVALID_COOKIE;
    m_dwTextEditSinkCookie = TF_INVALID_COOKIE;
    m_dwMouseSinkCookie = TF_INVALID_COOKIE;
    m_pDocMgrFocus = nullptr;
    m_pTextEditContext = nullptr;
    m_tsfEnabled = false;  // Default to disabled
    m_pCompositionMgr = nullptr;
    m_lastSendInputTime = 0;
    
    // Create engine
    m_pEngine = keymagic_engine_new();
    if (!m_pEngine)
    {
        DEBUG_LOG(L"Failed to create KeyMagic engine");
    }
    m_useCompositionEditSession = true;  // Default to using composition edit session
    
    // Create composition manager
    m_pCompositionMgr = new CCompositionManager(this);
    
    // Initialize display attributes
    m_ppDisplayAttributeInfo = nullptr;
    m_displayAttributeInfoCount = 0;
    m_inputDisplayAttributeAtom = TF_INVALID_GUIDATOM;
    
    m_isProcessingKey = false;
    m_lastTerminationSpaceTime = 0;
    
    // Initialize event monitoring
    m_hRegistryUpdateEvent = nullptr;
    m_hEventThread = nullptr;
    m_bEventThreadRunning = false;
    m_bIsActiveInputProcessor = false;
    m_lastSendInputTime = 0;
    
    
    InitializeCriticalSection(&m_cs);
    DllAddRef();
}

CKeyMagicTextService::~CKeyMagicTextService()
{
    
    if (m_pCompositionMgr)
    {
        m_pCompositionMgr->Release();
        m_pCompositionMgr = nullptr;
    }
    
    // Clean up display attributes
    if (m_ppDisplayAttributeInfo)
    {
        for (ULONG i = 0; i < m_displayAttributeInfoCount; i++)
        {
            if (m_ppDisplayAttributeInfo[i])
            {
                m_ppDisplayAttributeInfo[i]->Release();
            }
        }
        delete[] m_ppDisplayAttributeInfo;
        m_ppDisplayAttributeInfo = nullptr;
    }
    
    // Stop event monitoring
    StopEventMonitoring();
    
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

    if (IsEqualIID(riid, IID_IUnknown) || IsEqualIID(riid, IID_ITfTextInputProcessor))
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
    else if (IsEqualIID(riid, IID_ITfTextEditSink))
    {
        *ppvObject = static_cast<ITfTextEditSink*>(this);
    }
    else if (IsEqualIID(riid, IID_ITfMouseSink))
    {
        *ppvObject = static_cast<ITfMouseSink*>(this);
    }
    else if (IsEqualIID(riid, IID_ITfDisplayAttributeProvider))
    {
        *ppvObject = static_cast<ITfDisplayAttributeProvider*>(this);
    }

    if (*ppvObject)
    {
        AddRef();
        return S_OK;
    }

    return E_NOINTERFACE;
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
    DEBUG_LOG_FUNC();
    EnterCriticalSection(&m_cs);
    
    // Log host process path when service is activated
    wchar_t processPath[MAX_PATH];
    if (GetModuleFileNameW(NULL, processPath, MAX_PATH) > 0)
    {
        DEBUG_LOG(L"Service activated in host process: " + std::wstring(processPath));
    }
    else
    {
        DEBUG_LOG(L"Service activated in host process: <unknown>");
    }
    
    // Log the effective process name (what will be used for composition mode decisions)
    std::wstring effectiveProcessName = ProcessDetector::GetEffectiveProcessName();
    DEBUG_LOG(L"Effective process name for composition mode: " + effectiveProcessName);
    
    m_pThreadMgr = ptim;
    m_pThreadMgr->AddRef();
    m_tfClientId = tid;

    // Register thread manager event sink
    ITfSource *pSource;
    if (SUCCEEDED(m_pThreadMgr->QueryInterface(IID_ITfSource, (void**)&pSource)))
    {
        pSource->AdviseSink(IID_ITfThreadMgrEventSink, static_cast<ITfThreadMgrEventSink*>(this), &m_dwThreadMgrEventSinkCookie);
        pSource->Release();
    }

    // Register key event sink
    ITfKeystrokeMgr *pKeystrokeMgr;
    if (SUCCEEDED(m_pThreadMgr->QueryInterface(IID_ITfKeystrokeMgr, (void**)&pKeystrokeMgr)))
    {
        pKeystrokeMgr->AdviseKeyEventSink(m_tfClientId, static_cast<ITfKeyEventSink*>(this), TRUE);
        pKeystrokeMgr->Release();
    }

    // Register display attribute GUID and create display attribute info
    RegisterDisplayAttributeGuid();
    CreateDisplayAttributeInfo();
    
    // Load initial keyboard and settings
    ReloadRegistrySettings();
    

    LeaveCriticalSection(&m_cs);
    return S_OK;
}

STDAPI CKeyMagicTextService::Deactivate()
{
    EnterCriticalSection(&m_cs);

    // Unregister key event sink
    ITfKeystrokeMgr *pKeystrokeMgr;
    if (m_pThreadMgr && SUCCEEDED(m_pThreadMgr->QueryInterface(IID_ITfKeystrokeMgr, (void**)&pKeystrokeMgr)))
    {
        pKeystrokeMgr->UnadviseKeyEventSink(m_tfClientId);
        pKeystrokeMgr->Release();
    }

    // Unregister thread manager event sink
    ITfSource *pSource;
    if (m_pThreadMgr && SUCCEEDED(m_pThreadMgr->QueryInterface(IID_ITfSource, (void**)&pSource)))
    {
        pSource->UnadviseSink(m_dwThreadMgrEventSinkCookie);
        pSource->Release();
    }

    // Clean up sinks
    UninitTextEditSink();
    UninitMouseSink();

    // Unregister display attribute provider

    // Release thread manager
    if (m_pThreadMgr)
    {
        m_pThreadMgr->Release();
        m_pThreadMgr = nullptr;
    }

    m_tfClientId = TF_CLIENTID_NULL;

    LeaveCriticalSection(&m_cs);
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
    DEBUG_LOG_FUNC();
    DEBUG_LOG(L"Focus changed");
    EnterCriticalSection(&m_cs);

    // Clean up previous sinks
    UninitTextEditSink();
    UninitMouseSink();

    // Release previous context
    if (m_pTextEditContext)
    {
        m_pTextEditContext->Release();
        m_pTextEditContext = nullptr;
    }

    // Update focus
    m_pDocMgrFocus = pdimFocus;

    // Get new context and set up sinks
    if (m_pDocMgrFocus)
    {
        ITfContext *pContext;
        if (SUCCEEDED(m_pDocMgrFocus->GetTop(&pContext)) && pContext)
        {
            m_pTextEditContext = pContext; // Takes ownership
            
            InitTextEditSink();
            InitMouseSink();
            
            // Sync engine with document content instead of resetting
            if (m_pEngine && pContext)
            {
                DEBUG_LOG(L"Syncing engine with document on focus change");
                
                // Create edit session to read document and sync engine
                if (m_useCompositionEditSession)
                {
                    CCompositionEditSession *pEditSession = new CCompositionEditSession(this, pContext,
                                                                                      m_pCompositionMgr,
                                                                                      CCompositionEditSession::EditAction::SyncEngine,
                                                                                      m_pEngine);
                    if (pEditSession)
                    {
                        HRESULT hr;
                        pContext->RequestEditSession(m_tfClientId, pEditSession, TF_ES_SYNC | TF_ES_READ, &hr);
                        pEditSession->Release();
                        
                        if (SUCCEEDED(hr))
                        {
                            DEBUG_LOG(L"Successfully synced engine with document");
                        }
                        else
                        {
                            DEBUG_LOG(L"Failed to sync engine with document, falling back to reset");
                            ResetEngine();
                        }
                    }
                    else
                    {
                        DEBUG_LOG(L"Failed to create edit session, falling back to reset");
                        ResetEngine();
                    }
                }
                else
                {
                    CDirectEditSession *pEditSession = new CDirectEditSession(this, pContext, 
                                                                              CDirectEditSession::EditAction::SyncEngine,
                                                                              m_pEngine);
                    if (pEditSession)
                    {
                        HRESULT hr;
                        pContext->RequestEditSession(m_tfClientId, pEditSession, TF_ES_SYNC | TF_ES_READ, &hr);
                        pEditSession->Release();
                        
                        if (SUCCEEDED(hr))
                        {
                            DEBUG_LOG(L"Successfully synced engine with document");
                        }
                        else
                        {
                            DEBUG_LOG(L"Failed to sync engine with document, falling back to reset");
                            ResetEngine();
                        }
                    }
                    else
                    {
                        DEBUG_LOG(L"Failed to create edit session, falling back to reset");
                        ResetEngine();
                    }
                }
            }
            else
            {
                // No context or engine, just reset
                ResetEngine();
            }
        }
        else
        {
            // No valid context, reset engine
            ResetEngine();
        }
        
        // Start event monitoring when gaining focus
        StartEventMonitoring();
        
        // Also reload registry settings immediately
        ReloadRegistrySettings();
    }
    else
    {
        // Lost focus entirely, reset engine
        ResetEngine();
    }

    LeaveCriticalSection(&m_cs);
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
    DEBUG_LOG_FUNC();
    
    m_bIsActiveInputProcessor = fForeground ? true : false;
    
    if (fForeground)
    {
        DEBUG_LOG(L"Text service became active input processor");
    }
    else
    {
        DEBUG_LOG(L"Text service no longer active input processor");
        // We keep the monitoring thread running but it won't actively wait when document focus is lost
    }
    
    return S_OK;
}

STDAPI CKeyMagicTextService::OnTestKeyDown(ITfContext *pic, WPARAM wParam, LPARAM lParam, BOOL *pfEaten)
{
    DEBUG_LOG_FUNC();

    if (pfEaten == nullptr)
        return E_INVALIDARG;

    *pfEaten = FALSE;

    DEBUG_LOG_KEY(L"OnTestKeyDown", wParam, lParam, 0);

    // Check if this is our own SendInput by examining the extra info
    ULONG_PTR extraInfo = GetMessageExtraInfo();
    if (extraInfo == KEYMAGIC_EXTRAINFO_SIGNATURE)
    {
        return S_OK;
    }
    
    // Also check time-based filtering for VK_BACK as GetMessageExtraInfo is not reliable
    // Skip VK_BACK if we recently sent input (within 50ms)
    if (wParam == VK_BACK)
    {
        DWORD currentTime = GetTickCount();
        DWORD timeSinceLastInput = currentTime - m_lastSendInputTime;
        const DWORD IGNORE_KEY_TIMEOUT = 20; // milliseconds
        
        if (m_lastSendInputTime > 0 && timeSinceLastInput < IGNORE_KEY_TIMEOUT)
        {
            DEBUG_LOG(L"VK_BACK event within " + std::to_wstring(timeSinceLastInput) + L"ms of SendInput - ignoring");
            return S_OK;
        }
    }

    // Use engine test mode to determine if we should consume this key
    if (m_pEngine && m_tsfEnabled)
    {
        // Prepare key input using the utility
        KeyProcessingUtils::KeyInputData keyInput = KeyProcessingUtils::PrepareKeyInput(wParam, lParam);
        
        // Skip modifier and function keys
        if (keyInput.shouldSkip)
        {
            *pfEaten = FALSE;
            return S_OK;
        }
        
        // Test key processing without modifying engine state
        ProcessKeyOutput testOutput = {0};
        KeyMagicResult result = keymagic_engine_process_key_test_win(
            m_pEngine,
            static_cast<int>(wParam),
            keyInput.character,
            keyInput.shift,
            keyInput.ctrl,
            keyInput.alt,
            keyInput.capsLock,
            &testOutput
        );
        
        if (result == KeyMagicResult_Success)
        {
            // Use engine's decision on whether to consume the key
            *pfEaten = testOutput.is_processed ? TRUE : FALSE;
            
            // Clean up
            if (testOutput.text) keymagic_free_string(testOutput.text);
            if (testOutput.composing_text) keymagic_free_string(testOutput.composing_text);
        }
        else
        {
            // Fallback: don't consume keys if engine test fails
            *pfEaten = FALSE;
        }
    }

    DEBUG_LOG(L"OnTestKeyDown result: " + std::to_wstring(*pfEaten));

    return S_OK;
}

STDAPI CKeyMagicTextService::OnKeyDown(ITfContext *pic, WPARAM wParam, LPARAM lParam, BOOL *pfEaten)
{
    DEBUG_LOG_FUNC();
    
    if (pfEaten == nullptr)
        return E_INVALIDARG;

    *pfEaten = FALSE;
    
    // Mark that we're processing a key to help OnEndEdit
    m_isProcessingKey = true;

    // Check if this is our own SendInput by examining the extra info
    ULONG_PTR extraInfo = GetMessageExtraInfo();
    
    // Special handling for VK_SPACE sent for termination (used to terminate composition when disabling TSF)
    if (wParam == VK_SPACE && m_lastTerminationSpaceTime > 0)
    {
        DWORD currentTime = GetTickCount();
        DWORD timeSinceTerminationSpace = currentTime - m_lastTerminationSpaceTime;
        const DWORD TERMINATION_SPACE_TIMEOUT = 50; // milliseconds
        
        if (timeSinceTerminationSpace < TERMINATION_SPACE_TIMEOUT)
        {
            DEBUG_LOG(L"VK_SPACE within " + std::to_wstring(timeSinceTerminationSpace) + L"ms of termination space - terminating composition");
            
            // Clear the timestamp
            m_lastTerminationSpaceTime = 0;
            
            // Handle composition termination if using composition edit session
            if (m_useCompositionEditSession && m_pEngine)
            {
                // Create edit session to terminate composition
                CCompositionEditSession *pEditSession = new CCompositionEditSession(this, pic, 
                                                                                  m_pCompositionMgr,
                                                                                  CCompositionEditSession::EditAction::TerminateComposition,
                                                                                  m_pEngine);
                if (pEditSession)
                {
                    HRESULT hr;
                    pic->RequestEditSession(m_tfClientId, pEditSession, TF_ES_SYNC | TF_ES_READWRITE, &hr);
                    pEditSession->Release();
                }
            }
            else if (m_pEngine)
            {
                // If not using composition edit session, just reset engine
                keymagic_engine_reset(m_pEngine);
            }
            
            *pfEaten = TRUE;
            return S_OK;
        }
    }
    
    // Check if TSF is disabled (value is now updated by registry monitor thread)
    if (!m_tsfEnabled)
    {
        DEBUG_LOG(L"Key processing is disabled, not processing key");
        return S_OK;
    }
    
    // Skip other keys with our signature
    if (extraInfo == KEYMAGIC_EXTRAINFO_SIGNATURE)
    {
        DEBUG_LOG(L"Skipping key event from our SendInput");
        return S_OK;
    }
    
    
    // Also check time-based filtering for VK_BACK as GetMessageExtraInfo is not reliable
    // Skip VK_BACK if we recently sent input (within 50ms)
    if (wParam == VK_BACK)
    {
        DWORD currentTime = GetTickCount();
        DWORD timeSinceLastInput = currentTime - m_lastSendInputTime;
        const DWORD IGNORE_KEY_TIMEOUT = 20; // milliseconds
        
        if (m_lastSendInputTime > 0 && timeSinceLastInput < IGNORE_KEY_TIMEOUT)
        {
            DEBUG_LOG(L"VK_BACK event within " + std::to_wstring(timeSinceLastInput) + L"ms of SendInput - ignoring");
            return S_OK;
        }
    }
    
    char character = MapVirtualKeyToChar(wParam, lParam);
    DEBUG_LOG_KEY(L"OnKeyDown", wParam, lParam, character);

    // Create edit session for key processing
    if (m_useCompositionEditSession)
    {
        // Use composition-based edit session
        CCompositionEditSession *pEditSession = new CCompositionEditSession(this, pic, 
                                                                          m_pCompositionMgr,
                                                                          CCompositionEditSession::EditAction::ProcessKey,
                                                                          m_pEngine);
        if (pEditSession)
        {
            pEditSession->SetKeyData(wParam, lParam, pfEaten);
            HRESULT hr;
            pic->RequestEditSession(m_tfClientId, pEditSession, TF_ES_SYNC | TF_ES_READWRITE, &hr);
            pEditSession->Release();
        }
    }
    else
    {
        // Use direct key event edit session
        CDirectEditSession *pEditSession = new CDirectEditSession(this, pic, 
                                                                  CDirectEditSession::EditAction::ProcessKey,
                                                                  m_pEngine);
        if (pEditSession)
        {
            pEditSession->SetKeyData(wParam, lParam, pfEaten);
            HRESULT hr;
            pic->RequestEditSession(m_tfClientId, pEditSession, TF_ES_SYNC | TF_ES_READWRITE, &hr);
            pEditSession->Release();
        }
    }
    
    // Clear the processing flag after key is processed
    m_isProcessingKey = false;

    return S_OK;
}

STDAPI CKeyMagicTextService::OnTestKeyUp(ITfContext *pic, WPARAM wParam, LPARAM lParam, BOOL *pfEaten)
{
    if (pfEaten == nullptr)
        return E_INVALIDARG;

    *pfEaten = FALSE;
    return S_OK;
}

STDAPI CKeyMagicTextService::OnKeyUp(ITfContext *pic, WPARAM wParam, LPARAM lParam, BOOL *pfEaten)
{
    if (pfEaten == nullptr)
        return E_INVALIDARG;

    *pfEaten = FALSE;
    return S_OK;
}

STDAPI CKeyMagicTextService::OnPreservedKey(ITfContext *pic, REFGUID rguid, BOOL *pfEaten)
{
    if (pfEaten == nullptr)
        return E_INVALIDARG;

    *pfEaten = FALSE;
    return S_OK;
}

// ITfTextEditSink
STDAPI CKeyMagicTextService::OnEndEdit(ITfContext *pic, TfEditCookie ecReadOnly, ITfEditRecord *pEditRecord)
{
    // Check if selection changed (caret moved)
    BOOL fSelectionChanged;
    if (SUCCEEDED(pEditRecord->GetSelectionStatus(&fSelectionChanged)) && fSelectionChanged)
    {
        // Skip if we're actively processing a key
        if (m_isProcessingKey)
        {
            DEBUG_LOG(L"Selection changed during key processing - ignoring");
            return S_OK;
        }
        
        // Skip if we recently sent input (within 100ms)
        DWORD currentTime = GetTickCount();
        DWORD timeSinceLastInput = currentTime - m_lastSendInputTime;
        const DWORD IGNORE_SELECTION_TIMEOUT = 100; // milliseconds
        
        // Note: In practice, most OnEndEdit events from SendInput occur within 20ms,
        // but we use 100ms to provide a safety margin for slower systems or when
        // sending multiple characters that might take longer to process.
        if (m_lastSendInputTime > 0 && timeSinceLastInput < IGNORE_SELECTION_TIMEOUT)
        {
            DEBUG_LOG(L"Selection changed within " + std::to_wstring(timeSinceLastInput) + L"ms of SendInput - ignoring");
            return S_OK;
        }
        
        // If we get here, it's a genuine user-initiated selection change
        DEBUG_LOG(L"Selection changed by user - syncing engine with document");
        
        // Sync engine with document at new cursor position
        if (pic && m_pEngine)
        {
            // Create edit session to sync engine
            // Note: We cannot use the existing edit cookie from OnEndEdit, we need a new session
            if (m_useCompositionEditSession)
            {
                CCompositionEditSession *pEditSession = new CCompositionEditSession(this, pic,
                                                                                  m_pCompositionMgr,
                                                                                  CCompositionEditSession::EditAction::SyncEngine,
                                                                                  m_pEngine);
                if (pEditSession)
                {
                    HRESULT hr;
                    pic->RequestEditSession(m_tfClientId, pEditSession, TF_ES_SYNC | TF_ES_READ, &hr);
                    pEditSession->Release();
                }
            }
            else
            {
                CDirectEditSession *pEditSession = new CDirectEditSession(this, pic, 
                                                                          CDirectEditSession::EditAction::SyncEngine,
                                                                          m_pEngine);
                if (pEditSession)
                {
                    HRESULT hr;
                    pic->RequestEditSession(m_tfClientId, pEditSession, TF_ES_SYNC | TF_ES_READ, &hr);
                    pEditSession->Release();
                }
            }
        }
        else
        {
            // No context available, fall back to reset
            ResetEngine();
        }
    }

    return S_OK;
}

// ITfMouseSink
STDAPI CKeyMagicTextService::OnMouseEvent(ULONG uEdge, ULONG uQuadrant, DWORD dwBtnStatus, BOOL *pfEaten)
{
    if (pfEaten == nullptr)
        return E_INVALIDARG;

    *pfEaten = FALSE;

    // Sync engine on mouse click instead of resetting
    if (dwBtnStatus & MK_LBUTTON)
    {
        DEBUG_LOG(L"Mouse click detected - syncing engine with document");
        
        if (m_pTextEditContext && m_pEngine)
        {
            // Create edit session to sync engine
            if (m_useCompositionEditSession)
            {
                CCompositionEditSession *pEditSession = new CCompositionEditSession(this, m_pTextEditContext,
                                                                                  m_pCompositionMgr,
                                                                                  CCompositionEditSession::EditAction::SyncEngine,
                                                                                  m_pEngine);
                if (pEditSession)
                {
                    HRESULT hr;
                    m_pTextEditContext->RequestEditSession(m_tfClientId, pEditSession, TF_ES_SYNC | TF_ES_READ, &hr);
                    pEditSession->Release();
                    
                    if (FAILED(hr))
                    {
                        DEBUG_LOG(L"Failed to sync on mouse click, falling back to reset");
                        ResetEngine();
                    }
                }
                else
                {
                    ResetEngine();
                }
            }
            else
            {
                CDirectEditSession *pEditSession = new CDirectEditSession(this, m_pTextEditContext, 
                                                                          CDirectEditSession::EditAction::SyncEngine,
                                                                          m_pEngine);
                if (pEditSession)
                {
                    HRESULT hr;
                    m_pTextEditContext->RequestEditSession(m_tfClientId, pEditSession, TF_ES_SYNC | TF_ES_READ, &hr);
                    pEditSession->Release();
                    
                    if (FAILED(hr))
                    {
                        DEBUG_LOG(L"Failed to sync on mouse click, falling back to reset");
                        ResetEngine();
                    }
                }
                else
                {
                    ResetEngine();
                }
            }
        }
        else
        {
            ResetEngine();
        }
    }

    return S_OK;
}

// Helper methods
// Always open the 64-bit view of the registry, regardless of host process
HKEY CKeyMagicTextService::OpenSettingsKey(REGSAM samDesired)
{
    HKEY hKey;
    const wchar_t* KEYMAGIC_REG_SETTINGS = L"Software\\KeyMagic\\Settings";
    if (RegOpenKeyExW(HKEY_CURRENT_USER, KEYMAGIC_REG_SETTINGS, 0, samDesired | KEY_WOW64_64KEY, &hKey) == ERROR_SUCCESS)
    {
        return hKey;
    }
    return nullptr;
}


void CKeyMagicTextService::UninitializeEngine()
{
    if (m_pEngine)
    {
        keymagic_engine_free(m_pEngine);
        m_pEngine = nullptr;
    }
}

BOOL CKeyMagicTextService::LoadKeyboard(const std::wstring& km2Path)
{
    if (!m_pEngine)
        return FALSE;

    std::string utf8Path = ConvertUtf16ToUtf8(km2Path);
    KeyMagicResult result = keymagic_engine_load_keyboard(m_pEngine, utf8Path.c_str());
    
    if (result == KeyMagicResult_Success)
    {
        m_currentKeyboardPath = km2Path;
        DEBUG_LOG(L"Keyboard loaded successfully: " + km2Path);
        return TRUE;
    }

    DEBUG_LOG(L"Failed to load keyboard: " + km2Path);
    return FALSE;
}

BOOL CKeyMagicTextService::LoadKeyboardByID(const std::wstring& keyboardId)
{
    if (keyboardId.empty())
        return FALSE;
    
    // Ensure engine exists (should have been created in constructor)
    if (!m_pEngine)
    {
        m_pEngine = keymagic_engine_new();
        if (!m_pEngine)
        {
            DEBUG_LOG(L"Failed to create KeyMagic engine");
            return FALSE;
        }
    }

    // Build registry key path for this keyboard
    std::wstring keyPath = L"Software\\KeyMagic\\Keyboards\\" + keyboardId;
    HKEY hKey;
    
    if (RegOpenKeyExW(HKEY_CURRENT_USER, keyPath.c_str(), 0, KEY_READ | KEY_WOW64_64KEY, &hKey) == ERROR_SUCCESS)
    {
        // Read keyboard path
        wchar_t km2Path[MAX_PATH] = {0};
        DWORD dataSize = sizeof(km2Path);
        
        if (RegGetValueW(hKey, NULL, L"Path", RRF_RT_REG_SZ, NULL, km2Path, &dataSize) == ERROR_SUCCESS)
        {
            if (km2Path[0] != L'\0')
            {
                // Check if keyboard is enabled
                DWORD enabled = 0;
                dataSize = sizeof(enabled);
                
                if (RegGetValueW(hKey, NULL, L"Enabled", RRF_RT_REG_DWORD, NULL, &enabled, &dataSize) == ERROR_SUCCESS)
                {
                    if (!enabled)
                    {
                        DEBUG_LOG(L"Keyboard is disabled: " + keyboardId);
                        RegCloseKey(hKey);
                        return FALSE;
                    }
                }
                
                // Load the keyboard
                BOOL result = LoadKeyboard(km2Path);
                
                if (result)
                {
                    // Store keyboard info
                    m_currentKeyboardId = keyboardId;
                    
                    // Read keyboard name
                    wchar_t name[256] = {0};
                    dataSize = sizeof(name);
                    if (RegGetValueW(hKey, NULL, L"Name", RRF_RT_REG_SZ, NULL, name, &dataSize) == ERROR_SUCCESS)
                    {
                        DEBUG_LOG(L"Loaded keyboard: " + std::wstring(name) + L" (" + keyboardId + L")");
                    }
                }
                
                RegCloseKey(hKey);
                return result;
            }
        }
        
        RegCloseKey(hKey);
    }
    else
    {
        DEBUG_LOG(L"Keyboard not found in registry: " + keyboardId);
    }
    
    return FALSE;
}

void CKeyMagicTextService::ResetEngine()
{
    DEBUG_LOG_FUNC();
    EnterCriticalSection(&m_cs);
    
    if (m_pEngine)
    {
        keymagic_engine_reset(m_pEngine);
        DEBUG_LOG(L"Engine reset completed");
    }
    
    LeaveCriticalSection(&m_cs);
}


char CKeyMagicTextService::MapVirtualKeyToChar(WPARAM wParam, LPARAM lParam)
{
    BYTE keyState[256];
    GetKeyboardState(keyState);
    
    WCHAR buffer[2] = {0};
    int result = ToUnicode(static_cast<UINT>(wParam), (lParam >> 16) & 0xFF, keyState, buffer, 2, 0);
    
    if (result == 1 && buffer[0] < 128)
    {
        return static_cast<char>(buffer[0]);
    }
    
    return '\0';
}

bool CKeyMagicTextService::IsPrintableAscii(char c)
{
    return c >= 0x20 && c <= 0x7E;
}

HRESULT CKeyMagicTextService::InitTextEditSink()
{
    if (!m_pTextEditContext || m_dwTextEditSinkCookie != TF_INVALID_COOKIE)
        return S_OK;

    ITfSource *pSource;
    if (SUCCEEDED(m_pTextEditContext->QueryInterface(IID_ITfSource, (void**)&pSource)))
    {
        pSource->AdviseSink(IID_ITfTextEditSink, static_cast<ITfTextEditSink*>(this), &m_dwTextEditSinkCookie);
        pSource->Release();
    }

    return S_OK;
}

HRESULT CKeyMagicTextService::UninitTextEditSink()
{
    if (m_pTextEditContext && m_dwTextEditSinkCookie != TF_INVALID_COOKIE)
    {
        ITfSource *pSource;
        if (SUCCEEDED(m_pTextEditContext->QueryInterface(IID_ITfSource, (void**)&pSource)))
        {
            pSource->UnadviseSink(m_dwTextEditSinkCookie);
            pSource->Release();
        }
        m_dwTextEditSinkCookie = TF_INVALID_COOKIE;
    }

    return S_OK;
}

HRESULT CKeyMagicTextService::InitMouseSink()
{
    // Mouse sink initialization is complex and not strictly necessary for basic functionality
    // Skip for now to simplify implementation
    return S_OK;
}

HRESULT CKeyMagicTextService::UninitMouseSink()
{
    if (m_pTextEditContext && m_dwMouseSinkCookie != TF_INVALID_COOKIE)
    {
        ITfMouseTracker *pMouseTracker;
        if (SUCCEEDED(m_pTextEditContext->QueryInterface(IID_ITfMouseTracker, (void**)&pMouseTracker)))
        {
            pMouseTracker->UnadviseMouseSink(m_dwMouseSinkCookie);
            pMouseTracker->Release();
        }
        m_dwMouseSinkCookie = TF_INVALID_COOKIE;
    }

    return S_OK;
}

// ITfDisplayAttributeProvider implementation
STDAPI CKeyMagicTextService::EnumDisplayAttributeInfo(IEnumTfDisplayAttributeInfo **ppEnum)
{
    DEBUG_LOG_FUNC();
    
    if (ppEnum == nullptr)
        return E_INVALIDARG;
        
    *ppEnum = nullptr;
    
    if (!m_ppDisplayAttributeInfo || m_displayAttributeInfoCount == 0)
    {
        DEBUG_LOG(L"No display attribute info available");
        return E_FAIL;
    }
    
    CEnumDisplayAttributeInfo *pEnum = new CEnumDisplayAttributeInfo();
    if (!pEnum)
        return E_OUTOFMEMORY;
        
    HRESULT hr = pEnum->Initialize(m_ppDisplayAttributeInfo, m_displayAttributeInfoCount);
    if (FAILED(hr))
    {
        pEnum->Release();
        return hr;
    }
    
    *ppEnum = pEnum;
    DEBUG_LOG(L"Enumerated " + std::to_wstring(m_displayAttributeInfoCount) + L" display attributes");
    return S_OK;
}

STDAPI CKeyMagicTextService::GetDisplayAttributeInfo(REFGUID guidInfo, ITfDisplayAttributeInfo **ppInfo)
{
    DEBUG_LOG_FUNC();
    
    if (ppInfo == nullptr)
        return E_INVALIDARG;
        
    *ppInfo = nullptr;
    
    // Search for the requested GUID
    for (ULONG i = 0; i < m_displayAttributeInfoCount; i++)
    {
        if (m_ppDisplayAttributeInfo[i])
        {
            GUID guid;
            if (SUCCEEDED(m_ppDisplayAttributeInfo[i]->GetGUID(&guid)) && 
                IsEqualGUID(guid, guidInfo))
            {
                *ppInfo = m_ppDisplayAttributeInfo[i];
                (*ppInfo)->AddRef();
                DEBUG_LOG(L"Found display attribute info for requested GUID");
                return S_OK;
            }
        }
    }
    
    DEBUG_LOG(L"Display attribute info not found for requested GUID");
    return E_INVALIDARG;
}

// Display attribute management
HRESULT CKeyMagicTextService::RegisterDisplayAttributeGuid()
{
    DEBUG_LOG_FUNC();
    
    ITfCategoryMgr *pCategoryMgr;
    HRESULT hr = CoCreateInstance(CLSID_TF_CategoryMgr, NULL, CLSCTX_INPROC_SERVER,
                                 IID_ITfCategoryMgr, (void**)&pCategoryMgr);
    if (FAILED(hr))
    {
        DEBUG_LOG(L"Failed to create category manager");
        return hr;
    }
    
    // Register our display attribute GUID and get atom
    hr = pCategoryMgr->RegisterGUID(GUID_KeyMagicDisplayAttributeInput, &m_inputDisplayAttributeAtom);
    if (SUCCEEDED(hr))
    {
        DEBUG_LOG(L"Registered input display attribute GUID, atom: " + std::to_wstring(m_inputDisplayAttributeAtom));
    }
    else
    {
        DEBUG_LOG(L"Failed to register input display attribute GUID");
    }
    
    pCategoryMgr->Release();
    return hr;
}


HRESULT CKeyMagicTextService::CreateDisplayAttributeInfo()
{
    DEBUG_LOG_FUNC();
    
    // Clean up any existing display attribute info
    if (m_ppDisplayAttributeInfo)
    {
        for (ULONG i = 0; i < m_displayAttributeInfoCount; i++)
        {
            if (m_ppDisplayAttributeInfo[i])
            {
                m_ppDisplayAttributeInfo[i]->Release();
            }
        }
        delete[] m_ppDisplayAttributeInfo;
    }
    
    // Create display attribute info objects
    m_displayAttributeInfoCount = 1;  // Only input composition attribute
    m_ppDisplayAttributeInfo = new ITfDisplayAttributeInfo*[m_displayAttributeInfoCount];
    if (!m_ppDisplayAttributeInfo)
    {
        m_displayAttributeInfoCount = 0;
        return E_OUTOFMEMORY;
    }
    
    // Create input display attribute
    TF_DISPLAYATTRIBUTE inputAttr = CreateInputDisplayAttribute();
    m_ppDisplayAttributeInfo[0] = new CKeyMagicDisplayAttributeInfo(
        GUID_KeyMagicDisplayAttributeInput,
        inputAttr,
        L"KeyMagic Composing Text",
        L"KeyMagic"
    );
    
    DEBUG_LOG(L"Created " + std::to_wstring(m_displayAttributeInfoCount) + L" display attribute info objects");
    return S_OK;
}


// Settings update method
void CKeyMagicTextService::UpdateSettings(bool enabled, const std::wstring& keyboardId)
{
    DEBUG_LOG(L"UpdateSettings: enabled=" + std::to_wstring(enabled) + L", keyboard=" + keyboardId);
    
    EnterCriticalSection(&m_cs);
    
    // Update TSF enabled state
    if (enabled != m_tsfEnabled)
    {
        DEBUG_LOG(L"KeyProcessingEnabled changed from " + std::to_wstring(m_tsfEnabled) + L" to " + std::to_wstring(enabled));
        
        // If disabling TSF and using composition edit session, record timestamp and simulate SPACE to terminate composition
        if (!enabled && m_useCompositionEditSession)
        {
            DEBUG_LOG(L"TSF being disabled - recording timestamp and simulating SPACE key to terminate composition");
            
            // Record the timestamp before sending the key
            m_lastTerminationSpaceTime = GetTickCount();
            
            // Prepare INPUT structures for SPACE key down and up
            INPUT inputs[2] = {0};
            
            // SPACE key down
            inputs[0].type = INPUT_KEYBOARD;
            inputs[0].ki.wVk = VK_SPACE;
            inputs[0].ki.dwFlags = 0;
            inputs[0].ki.dwExtraInfo = KEYMAGIC_EXTRAINFO_SIGNATURE;
            
            // SPACE key up
            inputs[1].type = INPUT_KEYBOARD;
            inputs[1].ki.wVk = VK_SPACE;
            inputs[1].ki.dwFlags = KEYEVENTF_KEYUP;
            inputs[1].ki.dwExtraInfo = KEYMAGIC_EXTRAINFO_SIGNATURE;
            
            // Send the input
            UINT sent = SendInput(2, inputs, sizeof(INPUT));
            if (sent == 2)
            {
                DEBUG_LOG(L"Successfully sent SPACE key to terminate composition");
            }
            else
            {
                DEBUG_LOG(L"Failed to send SPACE key, sent=" + std::to_wstring(sent));
                // Clear the timestamp if we failed to send the key
                m_lastTerminationSpaceTime = 0;
            }
        }
        
        m_tsfEnabled = enabled;
    }
    
    // Update keyboard if changed
    if (!keyboardId.empty() && keyboardId != m_currentKeyboardId)
    {
        DEBUG_LOG(L"Default keyboard changed from \"" + m_currentKeyboardId + L"\" to \"" + keyboardId + L"\"");
        LoadKeyboardByID(keyboardId);
    }
    
    LeaveCriticalSection(&m_cs);
}

// Registry reload implementation
void CKeyMagicTextService::ReloadRegistrySettings()
{
    DEBUG_LOG(L"Reloading registry settings");
    
    // Read from registry
    HKEY hKey = OpenSettingsKey(KEY_READ);
    if (hKey)
    {
        // Read KeyProcessingEnabled
        DWORD keyProcessingEnabled = 0;  // Default to disabled
        DWORD dataSize = sizeof(keyProcessingEnabled);
        if (RegGetValueW(hKey, NULL, L"KeyProcessingEnabled", RRF_RT_REG_DWORD, 
                         NULL, &keyProcessingEnabled, &dataSize) == ERROR_SUCCESS)
        {
            DEBUG_LOG(L"Read KeyProcessingEnabled: " + std::to_wstring(keyProcessingEnabled));
        }
        else
        {
            DEBUG_LOG(L"KeyProcessingEnabled not found in registry, using default: 0 (disabled)");
        }
        
        // Read DefaultKeyboard
        wchar_t defaultKeyboard[256] = {0};
        dataSize = sizeof(defaultKeyboard);
        if (RegGetValueW(hKey, NULL, L"DefaultKeyboard", RRF_RT_REG_SZ, 
                         NULL, defaultKeyboard, &dataSize) == ERROR_SUCCESS)
        {
            DEBUG_LOG(L"Read DefaultKeyboard: " + std::wstring(defaultKeyboard));
        }
        
        // Determine UseCompositionEditSession based on current process
        m_useCompositionEditSession = ShouldUseCompositionEditSession();
        
        RegCloseKey(hKey);
        
        // Apply settings
        UpdateSettings(keyProcessingEnabled != 0, defaultKeyboard);
    }
    else
    {
        DEBUG_LOG(L"Failed to open registry key for reading");
    }
}

// Composition edit session determination
bool CKeyMagicTextService::ShouldUseCompositionEditSession()
{
    // Get the effective process name (handles parent process detection for WebView2)
    std::wstring processToCheck = ProcessDetector::GetEffectiveProcessName();
    
    DEBUG_LOG(L"Checking composition mode for process: " + processToCheck);
    
    // Read the list of executables that should use composition mode from registry
    HKEY hKey = OpenSettingsKey(KEY_READ);
    if (hKey)
    {
        // Try to read the CompositionModeProcesses value
        DWORD dataSize = 0;
        LONG result = RegGetValueW(hKey, NULL, L"CompositionModeProcesses", RRF_RT_REG_MULTI_SZ, NULL, NULL, &dataSize);
        
        if (result == ERROR_SUCCESS && dataSize > 0)
        {
            // Allocate buffer for the multi-string data
            std::vector<wchar_t> buffer(dataSize / sizeof(wchar_t));
            result = RegGetValueW(hKey, NULL, L"CompositionModeProcesses", RRF_RT_REG_MULTI_SZ, NULL, buffer.data(), &dataSize);
            
            if (result == ERROR_SUCCESS)
            {
                // Parse the multi-string data
                wchar_t* current = buffer.data();
                while (*current != L'\0')
                {
                    std::wstring processName(current);
                    
                    // Convert to lowercase for comparison
                    std::transform(processName.begin(), processName.end(), processName.begin(), ::towlower);
                    
                    if (processToCheck == processName)
                    {
                        DEBUG_LOG(L"Process found in composition mode list: " + processToCheck);
                        RegCloseKey(hKey);
                        return true;
                    }
                    
                    // Move to next string
                    current += wcslen(current) + 1;
                }
                
                DEBUG_LOG(L"Process not found in composition mode list: " + processToCheck);
                RegCloseKey(hKey);
                return false;  // Not in the list, use direct mode
            }
            else
            {
                DEBUG_LOG(L"Failed to read CompositionModeProcesses value, using default");
            }
        }
        else
        {
            DEBUG_LOG(L"CompositionModeProcesses value not found, creating default list");
            
            // Create default list of processes that should use composition mode
            std::vector<std::wstring> defaultProcesses = {
                L"ms-teams.exe"
            };
            
            // Write the default list to registry
            std::wstring multiString;
            for (const auto& process : defaultProcesses)
            {
                multiString += process + L'\0';
            }
            multiString += L'\0';  // Double null terminator
            
            RegSetValueExW(hKey, L"CompositionModeProcesses", 0, REG_MULTI_SZ, 
                          reinterpret_cast<const BYTE*>(multiString.c_str()), 
                          static_cast<DWORD>(multiString.length() * sizeof(wchar_t)));
            
            // Check if current process is in the default list
            for (const auto& process : defaultProcesses)
            {
                std::wstring lowerProcess = process;
                std::transform(lowerProcess.begin(), lowerProcess.end(), lowerProcess.begin(), ::towlower);
                
                if (processToCheck == lowerProcess)
                {
                    DEBUG_LOG(L"Process found in default composition mode list: " + processToCheck);
                    RegCloseKey(hKey);
                    return true;
                }
            }
            
            DEBUG_LOG(L"Process not found in default composition mode list: " + processToCheck);
        }
        
        RegCloseKey(hKey);
    }
    else
    {
        DEBUG_LOG(L"Failed to open registry key, defaulting to composition mode");
        return true;  // Default to composition mode if registry access fails
    }
    
    return false;  // Default to direct mode
}

// Event monitoring implementation
HRESULT CKeyMagicTextService::StartEventMonitoring()
{
    DEBUG_LOG(L"StartEventMonitoring called");
    
    // Don't start if already running
    if (m_bEventThreadRunning)
    {
        DEBUG_LOG(L"Event monitoring already running");
        return S_OK;
    }
    
    // Try to open the global event first
    m_hRegistryUpdateEvent = OpenEventW(
        SYNCHRONIZE | EVENT_MODIFY_STATE,
        FALSE,
        L"Global\\KeyMagicRegistryUpdate"
    );
    
    if (!m_hRegistryUpdateEvent)
    {
        DWORD dwError = GetLastError();
        DEBUG_LOG(L"Failed to open registry update event. Error: " + std::to_wstring(dwError));
        
        // If event doesn't exist (ERROR_FILE_NOT_FOUND), try to create it
        if (dwError == ERROR_FILE_NOT_FOUND)
        {
            DEBUG_LOG(L"Event doesn't exist, trying to create it");
            
            // Create security descriptor with NULL DACL for universal access
            SECURITY_DESCRIPTOR sd;
            InitializeSecurityDescriptor(&sd, SECURITY_DESCRIPTOR_REVISION);
            SetSecurityDescriptorDacl(&sd, TRUE, NULL, FALSE);
            
            SECURITY_ATTRIBUTES sa;
            sa.nLength = sizeof(SECURITY_ATTRIBUTES);
            sa.lpSecurityDescriptor = &sd;
            sa.bInheritHandle = FALSE;
            
            m_hRegistryUpdateEvent = CreateEventW(
                &sa,
                TRUE,  // Manual reset
                FALSE, // Initial state
                L"Global\\KeyMagicRegistryUpdate"
            );
            
            if (!m_hRegistryUpdateEvent)
            {
                dwError = GetLastError();
                DEBUG_LOG(L"Failed to create registry update event. Error: " + std::to_wstring(dwError));
                // Don't fail completely - TSF can still work without event monitoring
                return S_OK;
            }
            else
            {
                DEBUG_LOG(L"Successfully created registry update event");
            }
        }
        else
        {
            // Other error (e.g., access denied)
            // Don't fail completely - TSF can still work without event monitoring
            return S_OK;
        }
    }
    
    // Create monitoring thread
    m_bEventThreadRunning = true;
    m_hEventThread = CreateThread(
        nullptr,
        0,
        EventMonitorThreadProc,
        this,
        0,
        nullptr
    );
    
    if (!m_hEventThread)
    {
        DEBUG_LOG(L"Failed to create event monitor thread");
        m_bEventThreadRunning = false;
        CloseHandle(m_hRegistryUpdateEvent);
        m_hRegistryUpdateEvent = nullptr;
        return E_FAIL;
    }
    
    DEBUG_LOG(L"Event monitoring started successfully");
    return S_OK;
}

HRESULT CKeyMagicTextService::StopEventMonitoring()
{
    DEBUG_LOG(L"StopEventMonitoring called");
    
    // Signal thread to stop
    m_bEventThreadRunning = false;
    
    // Signal the event to wake up the thread if it's waiting
    if (m_hRegistryUpdateEvent)
    {
        SetEvent(m_hRegistryUpdateEvent);
    }
    
    // Wait for thread to finish
    if (m_hEventThread)
    {
        WaitForSingleObject(m_hEventThread, 1000);  // Wait max 1 second
        CloseHandle(m_hEventThread);
        m_hEventThread = nullptr;
    }
    
    // Close event handle
    if (m_hRegistryUpdateEvent)
    {
        CloseHandle(m_hRegistryUpdateEvent);
        m_hRegistryUpdateEvent = nullptr;
    }
    
    DEBUG_LOG(L"Event monitoring stopped");
    return S_OK;
}

DWORD WINAPI CKeyMagicTextService::EventMonitorThreadProc(LPVOID lpParam)
{
    CKeyMagicTextService* pThis = static_cast<CKeyMagicTextService*>(lpParam);
    
    DEBUG_LOG(L"Event monitor thread started");
    
    while (pThis->m_bEventThreadRunning)
    {
        // Only wait for events if we have document focus (window is foreground and has focus)
        if (pThis->m_pDocMgrFocus && pThis->m_hRegistryUpdateEvent)
        {
            DWORD dwResult = WaitForSingleObject(pThis->m_hRegistryUpdateEvent, 500);  // Check every 500ms
            
            if (dwResult == WAIT_OBJECT_0)
            {
                DEBUG_LOG(L"Registry update event signaled");
                
                // Reset the event (manual reset event)
                ResetEvent(pThis->m_hRegistryUpdateEvent);
                
                // Reload registry settings
                pThis->ReloadRegistrySettings();
            }
        }
        else
        {
            // No document focus, just sleep
            Sleep(500);
        }
    }
    
    DEBUG_LOG(L"Event monitor thread exiting");
    return 0;
}