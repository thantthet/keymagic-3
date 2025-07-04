#include "KeyMagicTextService.h"
#include "Globals.h"
#include <string>
#include <codecvt>
#include <locale>
#include <fstream>
#include <sstream>
#include <iomanip>
#include <chrono>
#include "Debug.h"

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
    DebugLog(L"=== KeyMagicTextService::Activate START ===");
    DebugLog(std::wstring(L"Thread ID: ") + std::to_wstring(GetCurrentThreadId()));
    DebugLog(std::wstring(L"Client ID: ") + std::to_wstring(tid));
    
    m_pThreadMgr = ptim;
    m_pThreadMgr->AddRef();
    m_tfClientId = tid;

    // Initialize the engine
    if (!InitializeEngine())
    {
        DebugLog(L"InitializeEngine failed");
        return E_FAIL;
    }

    // Register thread manager event sink
    ITfSource *pSource = nullptr;
    if (SUCCEEDED(m_pThreadMgr->QueryInterface(IID_ITfSource, (void**)&pSource)))
    {
        pSource->AdviseSink(IID_ITfThreadMgrEventSink, 
                           static_cast<ITfThreadMgrEventSink*>(this), 
                           &m_dwThreadMgrEventSinkCookie);
        pSource->Release();
        DebugLog(L"Registered thread manager event sink");
    }
    else
    {
        DebugLog(L"Failed to register thread manager event sink");
    }

    // Register key event sink
    ITfKeystrokeMgr *pKeystrokeMgr = nullptr;
    if (SUCCEEDED(m_pThreadMgr->QueryInterface(IID_ITfKeystrokeMgr, (void**)&pKeystrokeMgr)))
    {
        pKeystrokeMgr->AdviseKeyEventSink(m_tfClientId, 
                                          static_cast<ITfKeyEventSink*>(this), 
                                          TRUE);
        pKeystrokeMgr->Release();
        DebugLog(L"Registered key event sink");
    }
    else
    {
        DebugLog(L"Failed to register key event sink");
    }

    DebugLog(L"=== KeyMagicTextService::Activate END (SUCCESS) ===");
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
    DebugLog(L"=== InitializeEngine START ===");
    
    EnterCriticalSection(&m_cs);
    
    m_pEngine = keymagic_engine_new();
    
    if (m_pEngine)
    {
        DebugLog(L"Engine created successfully");
        
        // Try to load a default keyboard from registry
        HKEY hKey;
        if (RegOpenKeyEx(HKEY_CURRENT_USER, L"Software\\KeyMagic\\Settings", 
                        0, KEY_READ, &hKey) == ERROR_SUCCESS)
        {
            DebugLog(L"Found KeyMagic settings in registry");
            
            WCHAR defaultKeyboard[MAX_PATH] = {0};
            DWORD size = sizeof(defaultKeyboard);
            
            if (RegQueryValueEx(hKey, L"DefaultKeyboard", NULL, NULL, 
                              (LPBYTE)defaultKeyboard, &size) == ERROR_SUCCESS)
            {
                DebugLog(std::wstring(L"Default keyboard: ") + defaultKeyboard);
                
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
                        DebugLog(std::wstring(L"Loading keyboard from: ") + km2Path);
                        if (LoadKeyboard(km2Path))
                        {
                            DebugLog(L"Keyboard loaded successfully");
                        }
                        else
                        {
                            DebugLog(L"Failed to load keyboard");
                        }
                    }
                    else
                    {
                        DebugLog(L"Failed to read keyboard path from registry");
                    }
                    
                    RegCloseKey(hKeyboardKey);
                }
                else
                {
                    DebugLog(L"Failed to open keyboard registry key");
                }
            }
            else
            {
                DebugLog(L"No default keyboard configured");
            }
            
            RegCloseKey(hKey);
        }
        else
        {
            DebugLog(L"No KeyMagic settings found in registry");
        }
    }
    else
    {
        DebugLog(L"Failed to create engine");
    }
    
    LeaveCriticalSection(&m_cs);
    
    DebugLog(L"=== InitializeEngine END ===");
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
    DebugLog(L"=== LoadKeyboard START ===");
    DebugLog(std::wstring(L"Path: ") + km2Path);
    
    if (!m_pEngine)
    {
        DebugLog(L"ERROR: Engine is NULL");
        return FALSE;
    }
        
    EnterCriticalSection(&m_cs);
    
    // Convert wide string to UTF-8
    std::wstring_convert<std::codecvt_utf8<wchar_t>> converter;
    std::string utf8Path;
    
    try
    {
        utf8Path = converter.to_bytes(km2Path);
        DebugLog(std::wstring(L"UTF-8 path: ") + ConvertUtf8ToUtf16(utf8Path));
    }
    catch (const std::exception& e)
    {
        DebugLog(L"ERROR: Failed to convert path to UTF-8");
        LeaveCriticalSection(&m_cs);
        return FALSE;
    }
    
    KeyMagicResult result = keymagic_engine_load_keyboard(m_pEngine, utf8Path.c_str());
    DebugLog(std::wstring(L"Engine load result: ") + std::to_wstring(result));
    
    if (result == KeyMagicResult_Success)
    {
        m_currentKeyboardPath = km2Path;
        DebugLog(L"Keyboard loaded successfully");
    }
    else
    {
        DebugLog(L"Failed to load keyboard");
    }
    
    LeaveCriticalSection(&m_cs);
    
    DebugLog(L"=== LoadKeyboard END ===");
    return (result == KeyMagicResult_Success);
}

void CKeyMagicTextService::ProcessKeyInput(ITfContext *pic, WPARAM wParam, LPARAM lParam, BOOL *pfEaten)
{
    *pfEaten = FALSE;
    
    // Log entry
    std::wstringstream logEntry;
    logEntry << L"=== ProcessKeyInput START ===" << std::endl;
    logEntry << L"VK Code: 0x" << std::hex << wParam << std::dec << L" (" << wParam << L")" << std::endl;
    
    if (!m_pEngine)
    {
        logEntry << L"ERROR: Engine is NULL" << std::endl;
        DebugLog(logEntry.str());
        return;
    }
        
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
        
        logEntry << L"ToUnicode result: " << result;
        if (result > 0)
        {
            logEntry << L", char: '" << (wchar_t)unicodeChar[0] << L"' (0x" << std::hex << unicodeChar[0] << std::dec << L")";
        }
        logEntry << std::endl;
    }
    
    // Get modifier states
    int shift = (GetKeyState(VK_SHIFT) & 0x8000) ? 1 : 0;
    int ctrl = (GetKeyState(VK_CONTROL) & 0x8000) ? 1 : 0;
    int alt = (GetKeyState(VK_MENU) & 0x8000) ? 1 : 0;
    int capsLock = (GetKeyState(VK_CAPITAL) & 0x0001) ? 1 : 0;
    
    logEntry << L"Modifiers: Shift=" << shift << L", Ctrl=" << ctrl 
             << L", Alt=" << alt << L", CapsLock=" << capsLock << std::endl;
    logEntry << L"Character to engine: '" << (character ? (wchar_t)character : L' ') 
             << L"' (0x" << std::hex << (int)character << std::dec << L")" << std::endl;
    
    // Process key through engine using Windows VK code variant
    logEntry << L"Calling keymagic_engine_process_key_win..." << std::endl;
    KeyMagicResult result = keymagic_engine_process_key_win(
        m_pEngine, keyCode, character, 
        shift, ctrl, alt, capsLock, &output
    );
    
    logEntry << L"Engine result: " << result << std::endl;
    logEntry << L"Output:" << std::endl;
    logEntry << L"  action_type: " << output.action_type << std::endl;
    logEntry << L"  delete_count: " << output.delete_count << std::endl;
    logEntry << L"  is_processed: " << output.is_processed << std::endl;
    
    if (output.text)
    {
        std::string textStr(output.text);
        logEntry << L"  text: \"" << ConvertUtf8ToUtf16(textStr) << L"\"" << std::endl;
    }
    else
    {
        logEntry << L"  text: NULL" << std::endl;
    }
    
    if (output.composing_text)
    {
        std::string compStr(output.composing_text);
        logEntry << L"  composing_text: \"" << ConvertUtf8ToUtf16(compStr) << L"\"" << std::endl;
    }
    else
    {
        logEntry << L"  composing_text: NULL" << std::endl;
    }
    
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
            
            logEntry << L"Composing text (UTF-16): \"" << composingText << L"\"" << std::endl;
            logEntry << L"Composing text length: " << composingText.length() << std::endl;
            logEntry << L"Current m_fComposing: " << m_fComposing << std::endl;
            
            // Check if we should commit based on key
            switch (wParam)
            {
                case VK_SPACE:
                    logEntry << L"Processing SPACE key" << std::endl;
                    if (output.is_processed)
                    {
                        // Engine processed space, check if composing ends with space
                        if (!composingText.empty() && composingText.back() == L' ')
                        {
                            logEntry << L"Composing ends with space, will commit" << std::endl;
                            shouldCommit = true;
                            textToCommit = composingText;
                        }
                        else
                        {
                            logEntry << L"Space processed but not at end, continue composing" << std::endl;
                        }
                    }
                    else
                    {
                        // Engine didn't process space, commit current text + space
                        logEntry << L"Engine didn't process space, commit current + space" << std::endl;
                        shouldCommit = true;
                        textToCommit = composingText + L" ";
                    }
                    break;
                    
                case VK_RETURN:  // Enter key - commit without adding newline
                case VK_TAB:     // Tab key - commit without adding tab
                    logEntry << L"Processing " << (wParam == VK_RETURN ? L"ENTER" : L"TAB") << L" key" << std::endl;
                    if (!composingText.empty())
                    {
                        logEntry << L"Has composing text, will commit" << std::endl;
                        shouldCommit = true;
                        textToCommit = composingText;
                        // Don't consume the key after committing
                        *pfEaten = FALSE;
                    }
                    else
                    {
                        logEntry << L"No composing text, nothing to commit" << std::endl;
                    }
                    break;
                    
                case VK_ESCAPE:  // Escape - cancel composition
                    logEntry << L"Processing ESCAPE key - cancel composition" << std::endl;
                    keymagic_engine_reset(m_pEngine);
                    if (m_pComposition)
                    {
                        // Request edit session to terminate composition
                        CEditSession *pEditSession = new CEditSession(this, pic, 
                            CEditSession::EditAction::TerminateComposition, L"", L"");
                        HRESULT hr;
                        pic->RequestEditSession(m_tfClientId, pEditSession, TF_ES_SYNC | TF_ES_READWRITE, &hr);
                        pEditSession->Release();
                        logEntry << L"Terminated composition" << std::endl;
                    }
                    *pfEaten = TRUE;
                    logEntry << L"=== ProcessKeyInput END (ESCAPE) ===" << std::endl;
                    DebugLog(logEntry.str());
                    goto cleanup;
                    
                case VK_BACK:  // Backspace - let engine handle it
                    logEntry << L"Processing BACKSPACE key" << std::endl;
                    // Engine will handle backspace internally
                    break;
                    
                default:
                    logEntry << L"Processing other key" << std::endl;
                    break;
            }
            
            // Log decision
            logEntry << L"Decision: shouldCommit=" << shouldCommit 
                     << L", textToCommit=\"" << textToCommit << L"\"" << std::endl;
            
            // Update composition or commit text
            if (shouldCommit || !composingText.empty())
            {
                CEditSession::EditAction action = shouldCommit ? 
                    CEditSession::EditAction::CommitText : 
                    CEditSession::EditAction::UpdateComposition;
                    
                logEntry << L"Creating edit session: " 
                         << (action == CEditSession::EditAction::CommitText ? L"CommitText" : L"UpdateComposition") 
                         << std::endl;
                
                CEditSession *pEditSession = new CEditSession(this, pic, 
                    action, textToCommit, composingText);
                HRESULT hr;
                pic->RequestEditSession(m_tfClientId, pEditSession, TF_ES_SYNC | TF_ES_READWRITE, &hr);
                pEditSession->Release();
                
                logEntry << L"Edit session result: 0x" << std::hex << hr << std::dec << std::endl;
            }
            else if (composingText.empty() && m_fComposing)
            {
                // If composing text is empty but we were composing, terminate composition
                logEntry << L"Empty composing text but was composing, terminating composition" << std::endl;
                CEditSession *pEditSession = new CEditSession(this, pic, 
                    CEditSession::EditAction::TerminateComposition, L"", L"");
                HRESULT hr;
                pic->RequestEditSession(m_tfClientId, pEditSession, TF_ES_SYNC | TF_ES_READWRITE, &hr);
                pEditSession->Release();
            }
        }
        else
        {
            logEntry << L"No composing_text in output" << std::endl;
        }
        
        // Set eaten flag based on whether we processed the key
        // Note: For Enter/Tab, we already set *pfEaten = FALSE above when committing
        if (wParam != VK_RETURN && wParam != VK_TAB)
        {
            *pfEaten = output.is_processed ? TRUE : FALSE;
        }
        logEntry << L"Setting pfEaten=" << (*pfEaten ? L"TRUE" : L"FALSE") << std::endl;
    }
    else
    {
        logEntry << L"Engine processing failed with error: " << result << std::endl;
    }
    
cleanup:
    // Free allocated strings
    if (output.text) keymagic_free_string(output.text);
    if (output.composing_text) keymagic_free_string(output.composing_text);
    
    logEntry << L"=== ProcessKeyInput END ===" << std::endl;
    DebugLog(logEntry.str());
    
    LeaveCriticalSection(&m_cs);
}

void CKeyMagicTextService::UpdateComposition(ITfContext *pic, bool shouldCommit, 
                                            const std::wstring& textToCommit, 
                                            const std::wstring& composingText)
{
    // These methods are no longer needed - functionality moved to CEditSession
    // Keep them for backward compatibility but they're not used
}

void CKeyMagicTextService::CommitText(ITfContext *pic, const std::wstring& text)
{
    // These methods are no longer needed - functionality moved to CEditSession
    // Keep them for backward compatibility but they're not used
}

void CKeyMagicTextService::TerminateComposition(ITfContext *pic)
{
    // These methods are no longer needed - functionality moved to CEditSession
    // Keep them for backward compatibility but they're not used
}

BOOL CKeyMagicTextService::IsKeyEaten(ITfContext *pic, WPARAM wParam, LPARAM lParam)
{
    // Always process these keys
    if (wParam == VK_BACK || wParam == VK_ESCAPE)
        return TRUE;
        
    // Process Enter and Tab only if we have composing text
    if ((wParam == VK_RETURN || wParam == VK_TAB) && m_fComposing)
        return TRUE;
        
    // Process Space key - let the engine decide if it should be consumed
    if (wParam == VK_SPACE)
        return TRUE;
        
    // Process all printable ASCII characters
    if (wParam >= 0x20 && wParam <= 0x7E)
        return TRUE;
        
    // Process function keys if needed by the keyboard
    if (wParam >= VK_F1 && wParam <= VK_F12)
        return TRUE;
        
    // Don't process navigation keys, system keys, etc.
    return FALSE;
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
            // No need to call TerminateComposition - CommitText handles it
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
            
            // Move the selection to the end of the composition
            // First, collapse the range to the end
            pRange->Collapse(ec, TF_ANCHOR_END);
            
            // Then set the selection to this position
            TF_SELECTION tfSelection;
            tfSelection.range = pRange;
            tfSelection.style.ase = TF_AE_NONE;
            tfSelection.style.fInterimChar = FALSE;
            
            m_pContext->SetSelection(ec, 1, &tfSelection);
            
            // Apply underline display attribute
            ITfRange *pCompRange = nullptr;
            if (SUCCEEDED(m_pTextService->m_pComposition->GetRange(&pCompRange)))
            {
                ApplyDisplayAttributes(ec, pCompRange);
                pCompRange->Release();
            }
            
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
    if (m_pTextService->m_pComposition)
    {
        // When we have a composition, we need to finalize it with the text we want
        // First, get the composition range
        ITfRange *pRange = nullptr;
        if (SUCCEEDED(m_pTextService->m_pComposition->GetRange(&pRange)))
        {
            // Set the final text in the composition range
            pRange->SetText(ec, 0, m_textToCommit.c_str(), static_cast<LONG>(m_textToCommit.length()));
            pRange->Release();
        }
        
        // Now end the composition - this will commit the text
        m_pTextService->m_pComposition->EndComposition(ec);
        m_pTextService->m_pComposition->Release();
        m_pTextService->m_pComposition = nullptr;
        m_pTextService->m_fComposing = FALSE;
    }
    else
    {
        // No composition active, insert text directly
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
    // The composition range automatically gets default display attributes (underline)
    // TSF handles this for us when we create a composition
    // If we need custom attributes in the future, we can implement ITfDisplayAttributeProvider
}