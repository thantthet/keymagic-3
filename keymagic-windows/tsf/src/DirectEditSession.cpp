#include "DirectEditSession.h"
#include "KeyMagicTextService.h"
#include "Debug.h"
#include "Globals.h"
#include "../include/keymagic_ffi.h"

extern std::wstring ConvertUtf8ToUtf16(const std::string &utf8);
extern std::string ConvertUtf16ToUtf8(const std::wstring &utf16);

// CDirectEditSession implementation
CDirectEditSession::CDirectEditSession(CKeyMagicTextService *pTextService, ITfContext *pContext, 
                                       EditAction action, EngineHandle *pEngine)
{
    m_cRef = 1;
    m_pTextService = pTextService;
    m_pTextService->AddRef();
    m_pContext = pContext;
    m_pContext->AddRef();
    m_action = action;
    m_pEngine = pEngine;
    m_wParam = 0;
    m_lParam = 0;
    m_pfEaten = nullptr;
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
        *ppvObject = (ITfEditSession*)this;
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
    LONG cr = InterlockedDecrement(&m_cRef);

    if (cr == 0)
    {
        delete this;
    }

    return cr;
}

// ITfEditSession
STDAPI CDirectEditSession::DoEditSession(TfEditCookie ec)
{
    switch (m_action)
    {
        case EditAction::ProcessKey:
            return ProcessKey(ec);
            
        case EditAction::SyncEngine:
            return SyncEngineWithDocument(ec);
    }
    
    return S_OK;
}

void CDirectEditSession::SetKeyData(WPARAM wParam, LPARAM lParam, BOOL *pfEaten)
{
    m_wParam = wParam;
    m_lParam = lParam;
    m_pfEaten = pfEaten;
}

// Process key implementation
HRESULT CDirectEditSession::ProcessKey(TfEditCookie ec)
{
    DEBUG_LOG_FUNC();
    
    if (!m_pEngine)
    {
        DEBUG_LOG(L"No engine available");
        *m_pfEaten = FALSE;
        return S_OK;
    }

    // Get character from virtual key
    char character = MapVirtualKeyToChar(m_wParam, m_lParam);
    
    // Skip modifier keys and function keys (let them pass through)
    if (m_wParam == VK_SHIFT || m_wParam == VK_CONTROL || m_wParam == VK_MENU ||
        m_wParam == VK_LWIN || m_wParam == VK_RWIN ||
        (m_wParam >= VK_F1 && m_wParam <= VK_F24))
    {
        DEBUG_LOG(L"Modifier or function key, passing through");
        *m_pfEaten = FALSE;
        return S_OK;
    }
    
    // Process with engine
    ProcessKeyOutput output = {0};
    
    // Get modifiers
    int shift = (GetKeyState(VK_SHIFT) & 0x8000) ? 1 : 0;
    int ctrl = (GetKeyState(VK_CONTROL) & 0x8000) ? 1 : 0;
    int alt = (GetKeyState(VK_MENU) & 0x8000) ? 1 : 0;
    int capsLock = (GetKeyState(VK_CAPITAL) & 0x0001) ? 1 : 0;

    // Only pass printable ASCII characters
    if (!IsPrintableAscii(character))
    {
        character = '\0';
    }
    
    // Log engine input parameters
    {
        std::wostringstream oss;
        oss << L"Engine Input - VK: 0x" << std::hex << m_wParam << std::dec;
        oss << L" (" << m_wParam << L")";
        
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
    
    KeyMagicResult result = keymagic_engine_process_key_win(
        m_pEngine, 
        static_cast<int>(m_wParam), 
        character, 
        shift, ctrl, alt, capsLock, 
        &output
    );
    
    if (result != KeyMagicResult_Success)
    {
        DEBUG_LOG(L"Engine process key failed");
        *m_pfEaten = FALSE;
        return S_OK;
    }
    
    // Process the output
    DEBUG_LOG(L"Engine output - action: " + std::to_wstring(output.action_type) + 
              L", is_processed: " + std::to_wstring(output.is_processed));
    
    if (output.action_type != 0) // Not None
    {
        // Handle backspace count
        if (output.delete_count > 0)
        {
            DEBUG_LOG(L"Sending " + std::to_wstring(output.delete_count) + L" backspaces");
            SendBackspaces(output.delete_count, KEYMAGIC_EXTRAINFO_SIGNATURE, nullptr);
        }
        
        // Handle text insertion
        if (output.text && strlen(output.text) > 0)
        {
            std::wstring textToInsert = ConvertUtf8ToUtf16(output.text);
            DEBUG_LOG(L"Sending text: \"" + textToInsert + L"\"");
            SendUnicodeText(textToInsert, KEYMAGIC_EXTRAINFO_SIGNATURE, nullptr);
        }
    }
    
    // Handle special keys that might trigger commit
    if (output.composing_text)
    {
        std::string composingUtf8(output.composing_text);
        std::wstring composingText = ConvertUtf8ToUtf16(composingUtf8);
        
        switch (m_wParam)
        {
            case VK_SPACE:
                if (output.is_processed)
                {
                    // Engine processed space - check if composing ends with space
                    if (!composingText.empty() && composingText.back() == L' ')
                    {
                        // Reset engine after space commit
                        keymagic_engine_reset(m_pEngine);
                    }
                }
                else
                {
                    // Engine didn't process space - append space and reset
                    if (!composingText.empty())
                    {
                        SendUnicodeText(L" ", KEYMAGIC_EXTRAINFO_SIGNATURE, nullptr);
                    }
                    keymagic_engine_reset(m_pEngine);
                }
                break;
                
            case VK_RETURN:
            case VK_TAB:
                // Reset engine after these keys
                keymagic_engine_reset(m_pEngine);
                break;
                
            case VK_ESCAPE:
                // Cancel composition
                keymagic_engine_reset(m_pEngine);
                break;
        }
    }
    
    *m_pfEaten = output.is_processed ? TRUE : FALSE;
    
    // Cleanup
    if (output.text) keymagic_free_string(output.text);
    if (output.composing_text) keymagic_free_string(output.composing_text);
    
    return S_OK;
}

// Sync engine with document implementation
HRESULT CDirectEditSession::SyncEngineWithDocument(TfEditCookie ec)
{
    if (!m_pEngine)
        return S_OK;

    // Try to read text from document to sync with engine
    std::wstring documentText;
    const int MAX_COMPOSE_LENGTH = 50; // Maximum reasonable compose length
    
    if (SUCCEEDED(ReadDocumentSuffix(ec, MAX_COMPOSE_LENGTH, documentText)))
    {
        if (!documentText.empty())
        {
            // Look for a reasonable break point (space, punctuation, etc.)
            size_t composeStart = documentText.length();
            
            // Find the last space or punctuation mark
            for (size_t i = documentText.length(); i > 0; --i)
            {
                wchar_t ch = documentText[i - 1];
                // Check for word boundaries
                if (ch == L' ' || ch == L'\t' || ch == L'\n' || ch == L'\r' ||
                    ch == L'.' || ch == L',' || ch == L'!' || ch == L'?' ||
                    ch == L';' || ch == L':' || ch == L'"' || ch == L'\'' ||
                    ch == L'(' || ch == L')' || ch == L'[' || ch == L']' ||
                    ch == L'{' || ch == L'}' || ch == L'<' || ch == L'>')
                {
                    composeStart = i;
                    break;
                }
            }
            
            // Extract potential compose text
            std::wstring composeText;
            if (composeStart < documentText.length())
            {
                composeText = documentText.substr(composeStart);
            }
            else
            {
                // No break found, take the last few characters
                const size_t REASONABLE_COMPOSE_LENGTH = 10;
                if (documentText.length() > REASONABLE_COMPOSE_LENGTH)
                {
                    composeText = documentText.substr(documentText.length() - REASONABLE_COMPOSE_LENGTH);
                }
                else
                {
                    composeText = documentText;
                }
            }
            
            // Convert to UTF-8 and set as engine composition
            std::string utf8Text = ConvertUtf16ToUtf8(composeText);
            DEBUG_LOG(L"Syncing engine with document text: \"" + composeText + L"\"");
            
            KeyMagicResult result = keymagic_engine_set_composition(m_pEngine, utf8Text.c_str());
            if (result == KeyMagicResult_Success)
            {
                DEBUG_LOG(L"Successfully set engine composition");
            }
            else
            {
                DEBUG_LOG(L"Failed to set engine composition, error: " + std::to_wstring(result));
                keymagic_engine_reset(m_pEngine);
            }
        }
        else
        {
            // Empty document, reset engine
            DEBUG_LOG(L"Document is empty, resetting engine");
            keymagic_engine_reset(m_pEngine);
        }
    }
    else
    {
        DEBUG_LOG(L"Failed to read document text for sync, resetting engine");
        keymagic_engine_reset(m_pEngine);
    }
    
    return S_OK;
}

// Text manipulation methods
void CDirectEditSession::SendBackspaces(int count, ULONG_PTR dwExtraInfo, DWORD* pLastSendTime)
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
        inputs.push_back(input);
    }
    
    if (!inputs.empty()) {
        SendInput(static_cast<UINT>(inputs.size()), inputs.data(), sizeof(INPUT));
        DWORD currentTime = GetTickCount();
        if (pLastSendTime) {
            *pLastSendTime = currentTime;
        }
        // Update the text service's last send time
        m_pTextService->m_lastSendInputTime = currentTime;
    }
}

void CDirectEditSession::SendUnicodeText(const std::wstring& text, ULONG_PTR dwExtraInfo, DWORD* pLastSendTime)
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
        inputs.push_back(input);
    }
    
    if (!inputs.empty()) {
        SendInput(static_cast<UINT>(inputs.size()), inputs.data(), sizeof(INPUT));
        DWORD currentTime = GetTickCount();
        if (pLastSendTime) {
            *pLastSendTime = currentTime;
        }
        // Update the text service's last send time
        m_pTextService->m_lastSendInputTime = currentTime;
    }
}

// Document reading method
HRESULT CDirectEditSession::ReadDocumentSuffix(TfEditCookie ec, int maxChars, std::wstring &text)
{
    text.clear();
    
    // Get current selection
    TF_SELECTION tfSelection;
    ULONG fetched;
    if (FAILED(m_pContext->GetSelection(ec, TF_DEFAULT_SELECTION, 1, &tfSelection, &fetched)) || fetched == 0)
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

// Helper methods
char CDirectEditSession::MapVirtualKeyToChar(WPARAM wParam, LPARAM lParam)
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

bool CDirectEditSession::IsPrintableAscii(char c)
{
    return c >= 0x20 && c <= 0x7E;
}