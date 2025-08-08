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
#include "HUD.h"
#include "TrayClient.h"
#include "../../shared/include/RegistryUtils.h"
#include "../../shared/include/KeyboardInfo.h"
#include "../../shared/include/KeyMagicUtils.h"
#include <string>
#include <codecvt>
#include <locale>
#include <vector>
#include <algorithm>
#include <tlhelp32.h>
#include <functional>
#include <shlobj.h>

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
    
    // Initialize preserved key support
    m_pKeystrokeMgr = nullptr;
    
    // Initialize HUD
    KeyMagicHUD::GetInstance().Initialize();
    
    // Initialize TrayClient
    InitializeTrayClient();
    
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
    
    // Notify tray manager we're stopping
    if (m_pTrayClient) {
        m_pTrayClient->NotifyTipStopped();
    }
    
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

    if (IsEqualIID(riid, IID_IUnknown) || IsEqualIID(riid, IID_ITfTextInputProcessor) || IsEqualIID(riid, IID_ITfTextInputProcessorEx))
    {
        *ppvObject = static_cast<ITfTextInputProcessorEx*>(this);
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

// ITfTextInputProcessorEx
STDAPI CKeyMagicTextService::Activate(ITfThreadMgr *ptim, TfClientId tid)
{
    // Call ActivateEx with default flags (0)
    return ActivateEx(ptim, tid, 0);
}

STDAPI CKeyMagicTextService::ActivateEx(ITfThreadMgr *ptim, TfClientId tid, DWORD dwFlags)
{
    DEBUG_LOG_FUNC();
    
    // Log the activation flags
    DEBUG_LOG(L"ActivateEx called with dwFlags: 0x" + std::to_wstring(dwFlags));
    
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

    // Register key event sink and get keystroke manager interface
    if (SUCCEEDED(m_pThreadMgr->QueryInterface(IID_ITfKeystrokeMgr, (void**)&m_pKeystrokeMgr)))
    {
        m_pKeystrokeMgr->AdviseKeyEventSink(m_tfClientId, static_cast<ITfKeyEventSink*>(this), TRUE);
        // Keep the reference for preserved key registration
    }

    // Register display attribute GUID and create display attribute info
    RegisterDisplayAttributeGuid();
    CreateDisplayAttributeInfo();
    
    // Load initial keyboard and settings
    ReloadRegistrySettings();
    
    // Register preserved keys for keyboard switching
    RegisterPreservedKeys();
    
    LeaveCriticalSection(&m_cs);
    return S_OK;
}

STDAPI CKeyMagicTextService::Deactivate()
{
    EnterCriticalSection(&m_cs);
    
    // Unregister preserved keys
    UnregisterPreservedKeys();

    // Unregister key event sink
    if (m_pKeystrokeMgr)
    {
        m_pKeystrokeMgr->UnadviseKeyEventSink(m_tfClientId);
        m_pKeystrokeMgr->Release();
        m_pKeystrokeMgr = nullptr;
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
        DEBUG_LOG(L"Focus changed");
        // Notify tray manager that we have focus
        NotifyTrayManagerFocusChange(TRUE);

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
        DEBUG_LOG(L"Focus lost");
        
        // Keep engine state when losing focus - do not reset
        // This preserves the composing text and engine state when switching windows

        // Notify tray manager that we lost focus
        NotifyTrayManagerFocusChange(FALSE);
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
        // Notify tray manager that we gained focus
        NotifyTrayManagerFocusChange(TRUE);
    }
    else
    {
        DEBUG_LOG(L"Text service no longer active input processor");
        // We keep the monitoring thread running but it won't actively wait when document focus is lost
        // Notify tray manager that we lost focus
        NotifyTrayManagerFocusChange(FALSE);
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
    if (m_pEngine)
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
    else
    {
        // No engine loaded - eat all printable characters
        char character = MapVirtualKeyToChar(wParam, lParam);
        *pfEaten = IsPrintableAscii(character) ? TRUE : FALSE;
        if (*pfEaten)
        {
            DEBUG_LOG(L"No keyboard loaded - eating printable character");
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
    
    // TSF is now always enabled when selected - no need to check m_tsfEnabled
    // The service is only active when user selects it from language bar
    
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
    
    EnterCriticalSection(&m_cs);
    
    // Find which keyboard this preserved key maps to
    for (const auto& preservedKey : m_preservedKeys)
    {
        if (IsEqualGUID(rguid, preservedKey.guid))
        {
            DEBUG_LOG(L"Preserved key triggered for keyboard: " + preservedKey.keyboardId);
            
            // NOTE: We don't update registry here because TIP might run in containerized hosts
            // The tray manager will update the registry and signal the global event
            
            // Reload the keyboard
            LoadKeyboardByID(preservedKey.keyboardId);
            
            // Show HUD notification
            // Get keyboard display name from registry using shared utility
            std::wstring displayName = preservedKey.keyboardId;
            KeyboardInfo kbInfo;
            if (RegistryUtils::GetKeyboardInfoById(preservedKey.keyboardId, kbInfo))
            {
                if (!kbInfo.name.empty())
                {
                    displayName = kbInfo.name;
                }
            }
            
            KeyMagicHUD::GetInstance().ShowKeyboard(displayName);
            
            // Notify tray manager about the keyboard change
            // The tray manager will update the registry and signal the global event
            NotifyTrayManagerKeyboardChange();
            
            *pfEaten = TRUE;
            break;
        }
    }
    
    LeaveCriticalSection(&m_cs);
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

// Preserved key methods
HRESULT CKeyMagicTextService::RegisterPreservedKeys()
{
    if (!m_pKeystrokeMgr)
        return E_FAIL;
        
    // Clear any existing preserved keys
    UnregisterPreservedKeys();
    
    // Get all installed keyboards using shared utility
    std::vector<KeyboardInfo> keyboards = RegistryUtils::GetInstalledKeyboards();
    
    if (keyboards.empty())
    {
        DEBUG_LOG(L"No keyboards configured");
        return S_OK; // Not an error if no keyboards configured
    }
    
    // Process each keyboard
    for (const auto& keyboard : keyboards)
    {
        // Determine hotkey to use
        std::wstring hotkeyToUse;
        bool hotkeyFromKM2 = false;
        
        // Check if keyboard has a hotkey configured
        if (!keyboard.hotkey.empty())
        {
            // Hotkey explicitly set in registry
            hotkeyToUse = keyboard.hotkey;
            DEBUG_LOG(L"Using hotkey from registry for keyboard " + keyboard.id + L": " + hotkeyToUse);
        }
        else if (!keyboard.path.empty())
        {
            // No hotkey in registry - try to get from KM2 file
            hotkeyToUse = KeyMagicUtils::LoadHotkeyFromKm2(keyboard.path);
            if (!hotkeyToUse.empty())
            {
                hotkeyFromKM2 = true;
                DEBUG_LOG(L"Got hotkey from KM2 file for keyboard " + keyboard.id + L": " + hotkeyToUse);
            }
        }
        
        // Register the preserved key if we have a valid hotkey
        if (!hotkeyToUse.empty())
        {
            TF_PRESERVEDKEY tfKey;
            if (SUCCEEDED(ParseHotkeyString(hotkeyToUse, tfKey)))
            {
                // Generate unique GUID for this keyboard
                GUID guid = GenerateGuidForKeyboard(keyboard.id);
                
                // Register the preserved key
                HRESULT hr = m_pKeystrokeMgr->PreserveKey(m_tfClientId, guid, &tfKey, nullptr, 0);
                if (SUCCEEDED(hr))
                {
                    // Store the mapping
                    PreservedKeyInfo info;
                    info.keyboardId = keyboard.id;
                    info.tfKey = tfKey;
                    info.guid = guid;
                    m_preservedKeys.push_back(info);
                    
                    DEBUG_LOG(L"Registered preserved key for keyboard: " + keyboard.id + 
                             L" with hotkey: " + hotkeyToUse + (hotkeyFromKM2 ? L" [from KM2]" : L""));
                }
                else
                {
                    DEBUG_LOG(L"Failed to register preserved key for keyboard: " + keyboard.id);
                }
            }
        }
        else
        {
            DEBUG_LOG(L"No hotkey registered for keyboard: " + keyboard.id + L" (disabled or not configured)");
        }
    }
    return S_OK;
}

HRESULT CKeyMagicTextService::UnregisterPreservedKeys()
{
    if (!m_pKeystrokeMgr)
        return E_FAIL;
        
    for (const auto& preservedKey : m_preservedKeys)
    {
        m_pKeystrokeMgr->UnpreserveKey(preservedKey.guid, &preservedKey.tfKey);
    }
    
    m_preservedKeys.clear();
    return S_OK;
}

HRESULT CKeyMagicTextService::UpdatePreservedKeys()
{
    // Re-register all preserved keys (called when registry changes)
    return RegisterPreservedKeys();
}

HRESULT CKeyMagicTextService::ParseHotkeyString(const std::wstring& hotkeyStr, TF_PRESERVEDKEY& tfKey)
{
    // Convert wide string to narrow string for parsing
    std::string narrowStr;
    for (wchar_t wc : hotkeyStr)
    {
        narrowStr += static_cast<char>(wc);
    }
    
    // Use keymagic-core's hotkey parsing
    HotkeyInfo info;
    if (keymagic_parse_hotkey(narrowStr.c_str(), &info) != 1)
    {
        return E_FAIL;
    }
    
    // Convert VirtualKey enum to Windows VK code
    // The info.key_code is a VirtualKey enum value that needs to be converted
    // to the actual Windows Virtual Key code.
    // For now, we'll use a simple mapping for the most common keys
    UINT vkCode = 0;
    switch (info.key_code)
    {
        // Letter keys (VirtualKey enum values 26-51 map to VK codes 0x41-0x5A)
        case 26: vkCode = 0x41; break; // A
        case 27: vkCode = 0x42; break; // B
        case 28: vkCode = 0x43; break; // C
        case 29: vkCode = 0x44; break; // D
        case 30: vkCode = 0x45; break; // E
        case 31: vkCode = 0x46; break; // F
        case 32: vkCode = 0x47; break; // G
        case 33: vkCode = 0x48; break; // H
        case 34: vkCode = 0x49; break; // I
        case 35: vkCode = 0x4A; break; // J
        case 36: vkCode = 0x4B; break; // K
        case 37: vkCode = 0x4C; break; // L
        case 38: vkCode = 0x4D; break; // M
        case 39: vkCode = 0x4E; break; // N
        case 40: vkCode = 0x4F; break; // O
        case 41: vkCode = 0x50; break; // P
        case 42: vkCode = 0x51; break; // Q
        case 43: vkCode = 0x52; break; // R
        case 44: vkCode = 0x53; break; // S
        case 45: vkCode = 0x54; break; // T
        case 46: vkCode = 0x55; break; // U
        case 47: vkCode = 0x56; break; // V
        case 48: vkCode = 0x57; break; // W
        case 49: vkCode = 0x58; break; // X
        case 50: vkCode = 0x59; break; // Y
        case 51: vkCode = 0x5A; break; // Z
        
        // Number keys (VirtualKey enum values 52-61 map to VK codes 0x30-0x39)
        case 52: vkCode = 0x30; break; // 0
        case 53: vkCode = 0x31; break; // 1
        case 54: vkCode = 0x32; break; // 2
        case 55: vkCode = 0x33; break; // 3
        case 56: vkCode = 0x34; break; // 4
        case 57: vkCode = 0x35; break; // 5
        case 58: vkCode = 0x36; break; // 6
        case 59: vkCode = 0x37; break; // 7
        case 60: vkCode = 0x38; break; // 8
        case 61: vkCode = 0x39; break; // 9
        
        // Function keys
        case 71: vkCode = 0x70; break; // F1
        case 72: vkCode = 0x71; break; // F2
        case 73: vkCode = 0x72; break; // F3
        case 74: vkCode = 0x73; break; // F4
        case 75: vkCode = 0x74; break; // F5
        case 76: vkCode = 0x75; break; // F6
        case 77: vkCode = 0x76; break; // F7
        case 78: vkCode = 0x77; break; // F8
        case 79: vkCode = 0x78; break; // F9
        case 80: vkCode = 0x79; break; // F10
        case 81: vkCode = 0x7A; break; // F11
        case 82: vkCode = 0x7B; break; // F12
        
        // Special keys
        case 12: vkCode = 0x20; break; // Space
        case 9: vkCode = 0x09; break;  // Tab
        case 8: vkCode = 0x0D; break;  // Enter/Return
        
        default:
            DEBUG_LOG(L"Unknown virtual key code: " + std::to_wstring(info.key_code));
            return E_FAIL;
    }
    
    tfKey.uVKey = vkCode;
    
    // TSF preserved keys don't support Windows/Meta key
    if (info.meta)
    {
        DEBUG_LOG(L"Skipping hotkey with Windows/Meta key - not supported by TSF preserved keys");
        return E_FAIL;
    }
    
    tfKey.uModifiers = 0;
    if (info.ctrl)
        tfKey.uModifiers |= TF_MOD_CONTROL;
    if (info.alt)
        tfKey.uModifiers |= TF_MOD_ALT;
    if (info.shift)
        tfKey.uModifiers |= TF_MOD_SHIFT;
        
    return S_OK;
}

GUID CKeyMagicTextService::GenerateGuidForKeyboard(const std::wstring& keyboardId)
{
    // Create a deterministic GUID based on keyboard ID
    // This ensures the same keyboard always gets the same GUID
    GUID guid = GUID_KeyMagicPreservedKey; // Start with base GUID
    
    // Simple hash of keyboard ID to modify the GUID
    size_t hash = std::hash<std::wstring>{}(keyboardId);
    guid.Data1 = static_cast<unsigned long>(hash & 0xFFFFFFFF);
    guid.Data2 = static_cast<unsigned short>((hash >> 32) & 0xFFFF);
    guid.Data3 = static_cast<unsigned short>((hash >> 48) & 0xFFFF);
    
    return guid;
}

// Helper methods
// Always open the 64-bit view of the registry, regardless of host process
HKEY CKeyMagicTextService::OpenSettingsKey(REGSAM samDesired)
{
    HKEY hKey;
    const wchar_t* KEYMAGIC_REG_SETTINGS = L"Software\\KeyMagic\\Settings";
    if (RegOpenKeyExW(HKEY_CURRENT_USER, KEYMAGIC_REG_SETTINGS, 0, samDesired, &hKey) == ERROR_SUCCESS)
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

    std::string utf8Path = KeyMagicUtils::ConvertUtf16ToUtf8(km2Path);
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

    // Get keyboard info using shared utility
    KeyboardInfo kbInfo;
    if (!RegistryUtils::GetKeyboardInfoById(keyboardId, kbInfo))
    {
        DEBUG_LOG(L"Keyboard not found in registry: " + keyboardId);
        return FALSE;
    }
    
    // Check if keyboard has a valid path
    if (kbInfo.path.empty())
    {
        DEBUG_LOG(L"Keyboard has no path configured: " + keyboardId);
        return FALSE;
    }
    
    // Check if keyboard is enabled
    if (!kbInfo.enabled)
    {
        DEBUG_LOG(L"Keyboard is disabled: " + keyboardId);
        return FALSE;
    }
    
    // Load the keyboard
    BOOL result = LoadKeyboard(kbInfo.path.c_str());
    
    if (result)
    {
        // Store keyboard info
        m_currentKeyboardId = keyboardId;
        
        DEBUG_LOG(L"Loaded keyboard: " + kbInfo.name + L" (" + keyboardId + L")");
        
        // Notify tray manager of keyboard change
        NotifyTrayManagerKeyboardChange();
    }
    
    return result;
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
void CKeyMagicTextService::UpdateSettings(const std::wstring& keyboardId)
{
    DEBUG_LOG(L"UpdateSettings: keyboard=" + keyboardId);
    
    EnterCriticalSection(&m_cs);
    
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
    
    // Read DefaultKeyboard using RegistryUtils
    std::wstring defaultKeyboard;
    if (RegistryUtils::ReadKeyMagicSetting(L"DefaultKeyboard", defaultKeyboard))
    {
        DEBUG_LOG(L"Read DefaultKeyboard: " + defaultKeyboard);
    }
    
    // Determine UseCompositionEditSession based on current process
    m_useCompositionEditSession = ShouldUseCompositionEditSession();
    
    // Apply settings
    UpdateSettings(defaultKeyboard);
}

// Composition edit session determination
bool CKeyMagicTextService::ShouldUseCompositionEditSession()
{
    // Get the effective process name (handles parent process detection for WebView2)
    std::wstring processToCheck = ProcessDetector::GetEffectiveProcessName();
    
    DEBUG_LOG(L"Checking composition mode for process: " + processToCheck);
    
    // Read the list of executables that should use composition mode from registry
    std::vector<std::wstring> compositionModeHosts;
    if (RegistryUtils::ReadKeyMagicSetting(L"CompositionModeHosts", compositionModeHosts))
    {
        // Check if current process is in the list
        for (const auto& processName : compositionModeHosts)
        {
            // Convert to lowercase for comparison
            std::wstring lowerProcessName = processName;
            std::transform(lowerProcessName.begin(), lowerProcessName.end(), lowerProcessName.begin(), ::towlower);
            
            if (processToCheck == lowerProcessName)
            {
                DEBUG_LOG(L"Process found in composition mode list: " + processToCheck);
                return true;
            }
        }
        
        DEBUG_LOG(L"Process not found in composition mode list: " + processToCheck);
        return false;  // Not in the list, use direct mode
    }
    else
    {
        DEBUG_LOG(L"CompositionModeHosts value not found, using default list");
        
        // Use default list of processes that should use composition mode
        std::vector<std::wstring> defaultProcesses = {
            L"ms-teams.exe",
            L"excel.exe"
        };
        
        // Check if current process is in the default list
        for (const auto& process : defaultProcesses)
        {
            std::wstring lowerProcess = process;
            std::transform(lowerProcess.begin(), lowerProcess.end(), lowerProcess.begin(), ::towlower);
            
            if (processToCheck == lowerProcess)
            {
                DEBUG_LOG(L"Process found in default composition mode list: " + processToCheck);
                return true;
            }
        }
        
        DEBUG_LOG(L"Process not found in default composition mode list: " + processToCheck);
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
                
                // Update preserved keys in case hotkeys changed
                pThis->UpdatePreservedKeys();
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

// TrayClient integration methods
void CKeyMagicTextService::InitializeTrayClient()
{
    m_pTrayClient = std::make_unique<TrayClient>();
    if (m_pTrayClient) {
        m_pTrayClient->Connect();
        // Notify tray manager that we've started
        m_pTrayClient->NotifyTipStarted();
    }
}

void CKeyMagicTextService::NotifyTrayManagerFocusChange(BOOL hasFocus)
{
    if (!m_pTrayClient) {
        return;
    }
    
    if (hasFocus) {
        m_pTrayClient->NotifyFocusGained(m_currentKeyboardId);
    } else {
        m_pTrayClient->NotifyFocusLost();
    }
}

void CKeyMagicTextService::NotifyTrayManagerKeyboardChange()
{
    if (!m_pTrayClient) {
        return;
    }
    
    m_pTrayClient->NotifyKeyboardChanged(m_currentKeyboardId);
}