#include "KeyMagicTextService.h"
#include "KeyMagicGuids.h"
#include "Globals.h"
#include "Debug.h"
#include <string>
#include <codecvt>
#include <locale>
#include <vector>

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

// Helper function to send Unicode text using SendInput
void SendUnicodeText(const std::wstring& text, ULONG_PTR dwExtraInfo = 0, DWORD* pLastSendTime = nullptr)
{
    if (text.empty())
        return;
        
    std::vector<INPUT> inputs;
    inputs.reserve(text.length() * 2); // Each char needs keydown + keyup
    
    for (wchar_t ch : text) {
        INPUT input = {0};
        input.type = INPUT_KEYBOARD;
        input.ki.wScan = ch;
        input.ki.dwFlags = KEYEVENTF_UNICODE;
        input.ki.dwExtraInfo = dwExtraInfo;
        inputs.push_back(input);
        
        // Key up
        input.ki.dwFlags = KEYEVENTF_UNICODE | KEYEVENTF_KEYUP;
        input.ki.dwExtraInfo = dwExtraInfo;
        inputs.push_back(input);
    }
    
    UINT sent = SendInput(static_cast<UINT>(inputs.size()), inputs.data(), sizeof(INPUT));
    if (sent != inputs.size()) {
        DEBUG_LOG(L"SendInput failed to send all inputs. Sent: " + std::to_wstring(sent) + 
                  L" of " + std::to_wstring(inputs.size()));
    }
    
    // Record the time we sent input
    if (pLastSendTime) {
        *pLastSendTime = GetTickCount();
    }
}

// Helper function to send backspace keys
void SendBackspaces(int count, ULONG_PTR dwExtraInfo = 0, DWORD* pLastSendTime = nullptr)
{
    if (count <= 0)
        return;
        
    std::vector<INPUT> inputs;
    inputs.reserve(count * 2);
    
    for (int i = 0; i < count; i++) {
        INPUT input = {0};
        input.type = INPUT_KEYBOARD;
        input.ki.wVk = VK_BACK;
        input.ki.dwExtraInfo = dwExtraInfo;
        inputs.push_back(input);
        
        // Key up
        input.ki.dwFlags = KEYEVENTF_KEYUP;
        input.ki.dwExtraInfo = dwExtraInfo;
        inputs.push_back(input);
    }
    
    UINT sent = SendInput(static_cast<UINT>(inputs.size()), inputs.data(), sizeof(INPUT));
    if (sent != inputs.size()) {
        DEBUG_LOG(L"SendInput failed to send all backspaces. Sent: " + std::to_wstring(sent) + 
                  L" of " + std::to_wstring(inputs.size()));
    }
    
    // Record the time we sent input
    if (pLastSendTime) {
        *pLastSendTime = GetTickCount();
    }
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
    m_pEngine = nullptr;
    m_tsfEnabled = true;  // Default to enabled
    m_pCompositionMgr = nullptr;
    m_supportsComposition = FALSE;  // Not using composition anymore
    
    // Initialize display attributes
    m_ppDisplayAttributeInfo = nullptr;
    m_displayAttributeInfoCount = 0;
    m_inputDisplayAttributeAtom = TF_INVALID_GUIDATOM;
    
    m_isProcessingKey = false;
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
    
    m_pThreadMgr = ptim;
    m_pThreadMgr->AddRef();
    m_tfClientId = tid;

    // Initialize engine
    if (!InitializeEngine())
    {
        DEBUG_LOG(L"Failed to initialize engine");
        LeaveCriticalSection(&m_cs);
        return E_FAIL;
    }
    DEBUG_LOG(L"Engine initialized successfully");

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

    // Register display attribute provider and create display attribute info
    RegisterDisplayAttributeProvider();
    CreateDisplayAttributeInfo();

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
    UnregisterDisplayAttributeProvider();

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
        }
    }

    // Reset engine when focus changes
    ResetEngine();

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
    return S_OK;
}

STDAPI CKeyMagicTextService::OnTestKeyDown(ITfContext *pic, WPARAM wParam, LPARAM lParam, BOOL *pfEaten)
{
    if (pfEaten == nullptr)
        return E_INVALIDARG;

    *pfEaten = FALSE;

    // Check if this is our own SendInput by examining the extra info
    ULONG_PTR extraInfo = GetMessageExtraInfo();
    if (extraInfo == KEYMAGIC_EXTRAINFO_SIGNATURE)
    {
        return S_OK;
    }
    
    // Mark that we're processing a key to help OnEndEdit
    m_isProcessingKey = true;

    // We want to process most keys
    if (m_pEngine && m_tsfEnabled)
    {
        // Let some keys pass through without processing
        switch (wParam)
        {
            case VK_SHIFT:
            case VK_CONTROL:
            case VK_MENU:
            case VK_LWIN:
            case VK_RWIN:
            case VK_APPS:
                *pfEaten = FALSE;
                break;
            default:
                *pfEaten = TRUE;
                break;
        }
    }

    return S_OK;
}

STDAPI CKeyMagicTextService::OnKeyDown(ITfContext *pic, WPARAM wParam, LPARAM lParam, BOOL *pfEaten)
{
    if (pfEaten == nullptr)
        return E_INVALIDARG;

    *pfEaten = FALSE;
    
    // Check if this is our own SendInput by examining the extra info
    ULONG_PTR extraInfo = GetMessageExtraInfo();
    if (extraInfo == KEYMAGIC_EXTRAINFO_SIGNATURE)
    {
        DEBUG_LOG(L"Skipping key event from our SendInput");
        return S_OK;
    }
    
    char character = MapVirtualKeyToChar(wParam, lParam);
    DEBUG_LOG_KEY(L"OnKeyDown", wParam, lParam, character);

    // Process key directly without using TSF text manipulation
    ProcessKeyWithSendInput(pic, wParam, lParam, pfEaten);
    
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
        DEBUG_LOG(L"Selection changed by user - resetting engine");
        ResetEngine();
    }

    return S_OK;
}

// ITfMouseSink
STDAPI CKeyMagicTextService::OnMouseEvent(ULONG uEdge, ULONG uQuadrant, DWORD dwBtnStatus, BOOL *pfEaten)
{
    if (pfEaten == nullptr)
        return E_INVALIDARG;

    *pfEaten = FALSE;

    // Reset engine on mouse click
    if (dwBtnStatus & MK_LBUTTON)
    {
        DEBUG_LOG(L"Mouse click detected - resetting engine");
        ResetEngine();
    }

    return S_OK;
}

// Helper methods
BOOL CKeyMagicTextService::InitializeEngine()
{
    if (m_pEngine)
        return TRUE;

    m_pEngine = keymagic_engine_new();
    if (!m_pEngine)
        return FALSE;

    // Load default keyboard from registry
    HKEY hKey;
    const wchar_t* KEYMAGIC_REG_SETTINGS = L"Software\\KeyMagic\\Settings";
    
    if (RegOpenKeyExW(HKEY_CURRENT_USER, KEYMAGIC_REG_SETTINGS, 0, KEY_READ, &hKey) == ERROR_SUCCESS)
    {
        wchar_t defaultKeyboard[256] = {0};
        DWORD dataSize = sizeof(defaultKeyboard);
        DWORD dataType;
        
        if (RegQueryValueExW(hKey, L"DefaultKeyboard", NULL, &dataType, (LPBYTE)defaultKeyboard, &dataSize) == ERROR_SUCCESS)
        {
            if (dataType == REG_SZ && defaultKeyboard[0] != L'\0')
            {
                DEBUG_LOG(L"Default keyboard from registry: " + std::wstring(defaultKeyboard));
                
                // Load the keyboard
                LoadKeyboardByID(defaultKeyboard);
            }
        }
        else
        {
            DEBUG_LOG(L"No default keyboard set in registry");
        }
        
        RegCloseKey(hKey);
    }
    else
    {
        DEBUG_LOG(L"Failed to open KeyMagic settings registry key");
    }
    
    return TRUE;
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
    if (!m_pEngine || keyboardId.empty())
        return FALSE;

    // Build registry key path for this keyboard
    std::wstring keyPath = L"Software\\KeyMagic\\Keyboards\\" + keyboardId;
    HKEY hKey;
    
    if (RegOpenKeyExW(HKEY_CURRENT_USER, keyPath.c_str(), 0, KEY_READ, &hKey) == ERROR_SUCCESS)
    {
        // Read keyboard path
        wchar_t km2Path[MAX_PATH] = {0};
        DWORD dataSize = sizeof(km2Path);
        DWORD dataType;
        
        if (RegQueryValueExW(hKey, L"Path", NULL, &dataType, (LPBYTE)km2Path, &dataSize) == ERROR_SUCCESS)
        {
            if (dataType == REG_SZ && km2Path[0] != L'\0')
            {
                // Check if keyboard is enabled
                DWORD enabled = 0;
                dataSize = sizeof(enabled);
                
                if (RegQueryValueExW(hKey, L"Enabled", NULL, &dataType, (LPBYTE)&enabled, &dataSize) == ERROR_SUCCESS)
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
                    if (RegQueryValueExW(hKey, L"Name", NULL, &dataType, (LPBYTE)name, &dataSize) == ERROR_SUCCESS)
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

void CKeyMagicTextService::CheckAndReloadKeyboard()
{
    // Check if default keyboard has changed
    HKEY hKey;
    const wchar_t* KEYMAGIC_REG_SETTINGS = L"Software\\KeyMagic\\Settings";
    
    if (RegOpenKeyExW(HKEY_CURRENT_USER, KEYMAGIC_REG_SETTINGS, 0, KEY_READ, &hKey) == ERROR_SUCCESS)
    {
        // First check if TSF is enabled
        DWORD tsfEnabled = 1; // Default to enabled
        DWORD dataSize = sizeof(tsfEnabled);
        DWORD dataType;
        
        RegQueryValueExW(hKey, L"TSFEnabled", NULL, &dataType, (LPBYTE)&tsfEnabled, &dataSize);
        m_tsfEnabled = (tsfEnabled != 0);
        
        if (!m_tsfEnabled)
        {
            DEBUG_LOG(L"TSF is disabled");
            RegCloseKey(hKey);
            return;
        }
        
        // Check default keyboard
        wchar_t defaultKeyboard[256] = {0};
        dataSize = sizeof(defaultKeyboard);
        
        if (RegQueryValueExW(hKey, L"DefaultKeyboard", NULL, &dataType, (LPBYTE)defaultKeyboard, &dataSize) == ERROR_SUCCESS)
        {
            if (dataType == REG_SZ && defaultKeyboard[0] != L'\0')
            {
                std::wstring newKeyboardId(defaultKeyboard);
                if (newKeyboardId != m_currentKeyboardId)
                {
                    DEBUG_LOG(L"Default keyboard changed from \"" + m_currentKeyboardId + L"\" to \"" + newKeyboardId + L"\"");
                    LoadKeyboardByID(newKeyboardId);
                }
            }
        }
        
        RegCloseKey(hKey);
    }
}

void CKeyMagicTextService::ProcessKeyWithSendInput(ITfContext *pic, WPARAM wParam, LPARAM lParam, BOOL *pfEaten)
{
    DEBUG_LOG_FUNC();
    EnterCriticalSection(&m_cs);

    if (!m_pEngine)
    {
        DEBUG_LOG(L"No engine available");
        LeaveCriticalSection(&m_cs);
        return;
    }
    
    // Check if keyboard needs to be reloaded
    CheckAndReloadKeyboard();
    
    // Check if TSF is disabled
    if (!m_tsfEnabled)
    {
        DEBUG_LOG(L"TSF is disabled, not processing key");
        *pfEaten = FALSE;
        LeaveCriticalSection(&m_cs);
        return;
    }

    // Get modifiers
    int shift = (GetKeyState(VK_SHIFT) & 0x8000) ? 1 : 0;
    int ctrl = (GetKeyState(VK_CONTROL) & 0x8000) ? 1 : 0;
    int alt = (GetKeyState(VK_MENU) & 0x8000) ? 1 : 0;
    int capsLock = (GetKeyState(VK_CAPITAL) & 0x0001) ? 1 : 0;

    // Translate VK to character
    char character = MapVirtualKeyToChar(wParam, lParam);
    
    // Only pass printable ASCII characters
    if (!IsPrintableAscii(character))
    {
        character = '\0';
    }

    ProcessKeyOutput output = {0};
    
    // Log engine input parameters
    {
        std::wostringstream oss;
        oss << L"Engine Input - VK: 0x" << std::hex << wParam << std::dec;
        oss << L" (" << wParam << L")";
        
        if (character != '\0') {
            if (character >= 0x20 && character <= 0x7E) {
                oss << L", Char: '" << (wchar_t)character << L"' (0x" << std::hex << (int)(unsigned char)character << std::dec << L")";
            } else {
                oss << L", Char: 0x" << std::hex << (int)(unsigned char)character << std::dec;
            }
        } else {
            oss << L", Char: NULL";
        }
        
        oss << L", Modifiers: ";
        oss << L"Shift=" << shift;
        oss << L" Ctrl=" << ctrl;
        oss << L" Alt=" << alt;
        oss << L" Caps=" << capsLock;
        
        DEBUG_LOG(oss.str());
    }
    
    // Process key with engine
    KeyMagicResult result = keymagic_engine_process_key_win(
        m_pEngine, 
        static_cast<int>(wParam),
        character,
        shift, ctrl, alt, capsLock,
        &output
    );

    if (result == KeyMagicResult_Success)
    {
        DEBUG_LOG_ENGINE(output);
        *pfEaten = output.is_processed ? TRUE : FALSE;
        
        // Execute text action using SendInput if processed
        if (output.is_processed && output.action_type != 0) // Not None
        {
            // Handle backspaces
            if (output.delete_count > 0)
            {
                DEBUG_LOG(L"Sending " + std::to_wstring(output.delete_count) + L" backspaces");
                SendBackspaces(output.delete_count, KEYMAGIC_EXTRAINFO_SIGNATURE, &m_lastSendInputTime);
            }
            
            // Handle text insertion
            if (output.text && strlen(output.text) > 0)
            {
                std::wstring textToInsert = ConvertUtf8ToUtf16(output.text);
                DEBUG_LOG(L"Sending text: \"" + textToInsert + L"\"");
                SendUnicodeText(textToInsert, KEYMAGIC_EXTRAINFO_SIGNATURE, &m_lastSendInputTime);
            }
        }
        
        // Handle special keys that might trigger commit
        if (output.composing_text)
        {
            std::string composingUtf8(output.composing_text);
            std::wstring composingText = ConvertUtf8ToUtf16(composingUtf8);
            
            switch (wParam)
            {
                case VK_SPACE:
                    if (!output.is_processed || (composingText.length() > 0 && composingText.back() == L' '))
                    {
                        // Reset engine after space
                        DEBUG_LOG(L"Space key - resetting engine");
                        keymagic_engine_reset(m_pEngine);
                    }
                    break;
                    
                case VK_RETURN:
                case VK_TAB:
                    // Reset engine after these keys
                    DEBUG_LOG(L"Enter/Tab key - resetting engine");
                    keymagic_engine_reset(m_pEngine);
                    break;
                    
                case VK_ESCAPE:
                    // Cancel and reset
                    DEBUG_LOG(L"Escape key - resetting engine");
                    keymagic_engine_reset(m_pEngine);
                    *pfEaten = TRUE;
                    break;
            }
        }
        
        // Cleanup
        if (output.text) keymagic_free_string(output.text);
        if (output.composing_text) keymagic_free_string(output.composing_text);
    }
    else
    {
        DEBUG_LOG(L"Engine process_key failed");
    }

    LeaveCriticalSection(&m_cs);
}

void CKeyMagicTextService::ProcessKeyInput(ITfContext *pic, WPARAM wParam, LPARAM lParam, BOOL *pfEaten)
{
    DEBUG_LOG_FUNC();
    EnterCriticalSection(&m_cs);

    if (!m_pEngine)
    {
        DEBUG_LOG(L"No engine available");
        LeaveCriticalSection(&m_cs);
        return;
    }
    
    // Check if keyboard needs to be reloaded
    CheckAndReloadKeyboard();
    
    // Check if TSF is disabled
    if (!m_tsfEnabled)
    {
        DEBUG_LOG(L"TSF is disabled, not processing key");
        *pfEaten = FALSE;
        LeaveCriticalSection(&m_cs);
        return;
    }

    // Get modifiers
    int shift = (GetKeyState(VK_SHIFT) & 0x8000) ? 1 : 0;
    int ctrl = (GetKeyState(VK_CONTROL) & 0x8000) ? 1 : 0;
    int alt = (GetKeyState(VK_MENU) & 0x8000) ? 1 : 0;
    int capsLock = (GetKeyState(VK_CAPITAL) & 0x0001) ? 1 : 0;

    // Translate VK to character
    char character = MapVirtualKeyToChar(wParam, lParam);
    
    // Only pass printable ASCII characters
    if (!IsPrintableAscii(character))
    {
        character = '\0';
    }

    ProcessKeyOutput output = {0};
    
    // Log engine input parameters
    {
        std::wostringstream oss;
        oss << L"Engine Input - VK: 0x" << std::hex << wParam << std::dec;
        oss << L" (" << wParam << L")";
        
        if (character != '\0') {
            if (character >= 0x20 && character <= 0x7E) {
                oss << L", Char: '" << (wchar_t)character << L"' (0x" << std::hex << (int)(unsigned char)character << std::dec << L")";
            } else {
                oss << L", Char: 0x" << std::hex << (int)(unsigned char)character << std::dec;
            }
        } else {
            oss << L", Char: NULL";
        }
        
        oss << L", Modifiers: ";
        oss << L"Shift=" << shift;
        oss << L" Ctrl=" << ctrl;
        oss << L" Alt=" << alt;
        oss << L" Caps=" << capsLock;
        
        DEBUG_LOG(oss.str());
    }
    
    // Process key with engine
    KeyMagicResult result = keymagic_engine_process_key_win(
        m_pEngine, 
        static_cast<int>(wParam),
        character,
        shift, ctrl, alt, capsLock,
        &output
    );

    if (result == KeyMagicResult_Success)
    {
        DEBUG_LOG_ENGINE(output);
        *pfEaten = output.is_processed ? TRUE : FALSE;
        
        // Execute text action if processed
        if (output.is_processed)
        {
            ExecuteTextAction(pic, output);
        }
        
        // Cleanup
        if (output.text) keymagic_free_string(output.text);
        if (output.composing_text) keymagic_free_string(output.composing_text);
    }
    else
    {
        DEBUG_LOG(L"Engine process_key failed");
    }

    LeaveCriticalSection(&m_cs);
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

BOOL CKeyMagicTextService::TestCompositionSupport(ITfContext *pContext)
{
    DEBUG_LOG_FUNC();
    
    if (!pContext)
    {
        DEBUG_LOG(L"No context for composition test");
        return FALSE;
    }
    
    // Test if the context supports composition by checking for ITfContextComposition
    ITfContextComposition *pContextComposition;
    HRESULT hr = pContext->QueryInterface(IID_ITfContextComposition, (void **)&pContextComposition);
    
    if (FAILED(hr))
    {
        DEBUG_LOG(L"Context does not support ITfContextComposition interface - using direct editing");
        return FALSE;
    }
    
    DEBUG_LOG(L"Context supports ITfContextComposition interface");
    
    // For now, assume that if the context supports ITfContextComposition interface,
    // it supports composition. Most modern applications including Edge, Explorer, Word should support this.
    // We can add more sophisticated testing later if needed.
    
    // Check if it's likely a simple application by testing for basic TSF capabilities
    ITfInsertAtSelection *pInsertAtSelection;
    hr = pContext->QueryInterface(IID_ITfInsertAtSelection, (void **)&pInsertAtSelection);
    if (FAILED(hr))
    {
        DEBUG_LOG(L"Context does not support ITfInsertAtSelection - likely a very basic app, using direct editing");
        pContextComposition->Release();
        return FALSE;
    }
    
    pInsertAtSelection->Release();
    
    DEBUG_LOG(L"Context supports both composition and insertion interfaces - using composition mode");
    pContextComposition->Release();
    return TRUE;
}

void CKeyMagicTextService::SyncEngineWithDocument(ITfContext *pic, TfEditCookie ec)
{
    if (!m_pEngine)
        return;

    // Read up to 30 characters from document
    std::wstring documentText;
    if (SUCCEEDED(ReadDocumentSuffix(pic, ec, 30, documentText)))
    {
        // Convert to UTF-8 and set as engine composition
        std::string utf8Text = ConvertUtf16ToUtf8(documentText);
        DEBUG_LOG(L"Syncing engine with document text: \"" + documentText + L"\"");
        keymagic_engine_set_composition(m_pEngine, utf8Text.c_str());
    }
    else
    {
        DEBUG_LOG(L"Failed to read document text for sync");
    }
}

HRESULT CKeyMagicTextService::ReadDocumentSuffix(ITfContext *pic, TfEditCookie ec, int maxChars, std::wstring &text)
{
    text.clear();
    
    // Get current selection
    TF_SELECTION tfSelection;
    ULONG fetched;
    if (FAILED(pic->GetSelection(ec, TF_DEFAULT_SELECTION, 1, &tfSelection, &fetched)) || fetched == 0)
        return E_FAIL;

    ITfRange *pRange = tfSelection.range;
    
    // Clone range for manipulation
    ITfRange *pRangeStart;
    if (FAILED(pRange->Clone(&pRangeStart)))
    {
        pRange->Release();
        return E_FAIL;
    }

    // Move start back by maxChars
    LONG shifted;
    pRangeStart->ShiftStart(ec, -maxChars, &shifted, nullptr);

    // Read text
    WCHAR buffer[256];
    ULONG cch;
    HRESULT hr = pRangeStart->GetText(ec, 0, buffer, ARRAYSIZE(buffer) - 1, &cch);
    if (SUCCEEDED(hr))
    {
        buffer[cch] = L'\0';
        text = buffer;
    }

    pRangeStart->Release();
    pRange->Release();
    
    return hr;
}

HRESULT CKeyMagicTextService::DeleteCharsBeforeCursor(ITfContext *pic, TfEditCookie ec, int count)
{
    if (count <= 0)
        return S_OK;

    // Get current selection
    TF_SELECTION tfSelection;
    ULONG fetched;
    if (FAILED(pic->GetSelection(ec, TF_DEFAULT_SELECTION, 1, &tfSelection, &fetched)) || fetched == 0)
        return E_FAIL;

    ITfRange *pRange = tfSelection.range;
    
    // Shift start back by count
    LONG shifted;
    pRange->ShiftStart(ec, -count, &shifted, nullptr);
    
    // Delete the text
    HRESULT hr = pRange->SetText(ec, 0, L"", 0);
    
    pRange->Release();
    return hr;
}

HRESULT CKeyMagicTextService::InsertTextAtCursor(ITfContext *pic, TfEditCookie ec, const std::wstring &text)
{
    if (text.empty())
        return S_OK;

    // Get ITfInsertAtSelection interface
    ITfInsertAtSelection *pInsertAtSelection;
    if (FAILED(pic->QueryInterface(IID_ITfInsertAtSelection, (void**)&pInsertAtSelection)))
        return E_FAIL;

    // Insert text
    ITfRange *pRange;
    HRESULT hr = pInsertAtSelection->InsertTextAtSelection(ec, 0, text.c_str(), text.length(), &pRange);
    
    if (SUCCEEDED(hr) && pRange)
    {
        pRange->Release();
    }
    
    pInsertAtSelection->Release();
    return hr;
}

HRESULT CKeyMagicTextService::ExecuteTextAction(ITfContext *pic, const ProcessKeyOutput &output)
{
    // Create edit session for text manipulation
    CDirectEditSession *pEditSession = new CDirectEditSession(this, pic, 
                                                              CDirectEditSession::EditAction::DeleteAndInsert);
    if (!pEditSession)
        return E_OUTOFMEMORY;

    // Set action parameters based on output
    std::wstring insertText;
    if (output.text)
    {
        insertText = ConvertUtf8ToUtf16(output.text);
    }
    
    pEditSession->SetTextAction(output.delete_count, insertText);
    
    HRESULT hr;
    pic->RequestEditSession(m_tfClientId, pEditSession, TF_ES_SYNC | TF_ES_READWRITE, &hr);
    pEditSession->Release();
    
    return hr;
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
HRESULT CKeyMagicTextService::RegisterDisplayAttributeProvider()
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
    
    // Register as display attribute provider
    hr = pCategoryMgr->RegisterCategory(CLSID_KeyMagicTextService,
                                       GUID_TFCAT_DISPLAYATTRIBUTEPROVIDER,
                                       CLSID_KeyMagicTextService);
    if (FAILED(hr))
    {
        DEBUG_LOG(L"Failed to register display attribute provider");
    }
    else
    {
        DEBUG_LOG(L"Successfully registered display attribute provider");
        
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
    }
    
    pCategoryMgr->Release();
    return hr;
}

HRESULT CKeyMagicTextService::UnregisterDisplayAttributeProvider()
{
    DEBUG_LOG_FUNC();
    
    ITfCategoryMgr *pCategoryMgr;
    HRESULT hr = CoCreateInstance(CLSID_TF_CategoryMgr, NULL, CLSCTX_INPROC_SERVER,
                                 IID_ITfCategoryMgr, (void**)&pCategoryMgr);
    if (FAILED(hr))
        return hr;
    
    // Unregister display attribute provider
    hr = pCategoryMgr->UnregisterCategory(CLSID_KeyMagicTextService,
                                         GUID_TFCAT_DISPLAYATTRIBUTEPROVIDER,
                                         CLSID_KeyMagicTextService);
    if (SUCCEEDED(hr))
    {
        DEBUG_LOG(L"Successfully unregistered display attribute provider");
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

// CDirectEditSession implementation
CDirectEditSession::CDirectEditSession(CKeyMagicTextService *pTextService, ITfContext *pContext, EditAction action)
{
    m_cRef = 1;
    m_pTextService = pTextService;
    m_pTextService->AddRef();
    m_pContext = pContext;
    m_pContext->AddRef();
    m_action = action;
    m_wParam = 0;
    m_lParam = 0;
    m_pfEaten = nullptr;
    m_deleteCount = 0;
}

CDirectEditSession::~CDirectEditSession()
{
    m_pContext->Release();
    m_pTextService->Release();
}

// IUnknown
STDAPI CDirectEditSession::QueryInterface(REFIID riid, void **ppvObject)
{
    if (ppvObject == nullptr)
        return E_INVALIDARG;

    *ppvObject = nullptr;

    if (IsEqualIID(riid, IID_IUnknown) || IsEqualIID(riid, IID_ITfEditSession))
    {
        *ppvObject = static_cast<ITfEditSession*>(this);
    }

    if (*ppvObject)
    {
        AddRef();
        return S_OK;
    }

    return E_NOINTERFACE;
}

STDAPI_(ULONG) CDirectEditSession::AddRef()
{
    return InterlockedIncrement(&m_cRef);
}

STDAPI_(ULONG) CDirectEditSession::Release()
{
    LONG cRef = InterlockedDecrement(&m_cRef);
    if (cRef == 0)
    {
        delete this;
    }
    return cRef;
}

// ITfEditSession
STDAPI CDirectEditSession::DoEditSession(TfEditCookie ec)
{
    switch (m_action)
    {
        case EditAction::ProcessKey:
        {
            // First sync engine with document
            char* engineComposing = keymagic_engine_get_composition(m_pTextService->GetEngineHandle());
            if (engineComposing)
            {
                std::string engineText(engineComposing);
                keymagic_free_string(engineComposing);
                
                // Read document suffix
                std::wstring documentText;
                int compareLength = static_cast<int>(engineText.length());
                if (compareLength > 0)
                {
                    m_pTextService->ReadDocumentSuffix(m_pContext, ec, compareLength, documentText);
                    
                    // Compare texts
                    std::string docUtf8 = ConvertUtf16ToUtf8(documentText);
                    
                    DEBUG_LOG(L"Comparing engine text with document");
                    DEBUG_LOG(L"Engine: \"" + ConvertUtf8ToUtf16(engineText) + L"\"");
                    DEBUG_LOG(L"Document: \"" + documentText + L"\"");
                    
                    if (docUtf8 != engineText)
                    {
                        DEBUG_LOG(L"Text mismatch - resetting engine");
                        // Texts don't match, reset engine
                        m_pTextService->ResetEngine();
                    }
                }
            }
            
            // Process the key
            m_pTextService->ProcessKeyInput(m_pContext, m_wParam, m_lParam, m_pfEaten);
            break;
        }
        
        case EditAction::SyncEngine:
        {
            m_pTextService->SyncEngineWithDocument(m_pContext, ec);
            break;
        }
        
        case EditAction::DeleteAndInsert:
        {
            // Delete characters if needed
            if (m_deleteCount > 0)
            {
                DEBUG_LOG(L"Deleting " + std::to_wstring(m_deleteCount) + L" characters");
                m_pTextService->DeleteCharsBeforeCursor(m_pContext, ec, m_deleteCount);
            }
            
            // Insert new text
            if (!m_insertText.empty())
            {
                DEBUG_LOG(L"Inserting text: \"" + m_insertText + L"\"");
                m_pTextService->InsertTextAtCursor(m_pContext, ec, m_insertText);
            }
            break;
        }
    }
    
    return S_OK;
}

void CDirectEditSession::SetKeyData(WPARAM wParam, LPARAM lParam, BOOL *pfEaten)
{
    m_wParam = wParam;
    m_lParam = lParam;
    m_pfEaten = pfEaten;
}

void CDirectEditSession::SetTextAction(int deleteCount, const std::wstring &insertText)
{
    m_deleteCount = deleteCount;
    m_insertText = insertText;
}