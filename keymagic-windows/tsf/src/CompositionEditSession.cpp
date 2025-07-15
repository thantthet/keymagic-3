#include "CompositionEditSession.h"
#include "KeyMagicTextService.h"
#include "Composition.h"
#include "Debug.h"
#include "Globals.h"
#include "../include/keymagic_ffi.h"

extern std::wstring ConvertUtf8ToUtf16(const std::string &utf8);
extern std::string ConvertUtf16ToUtf8(const std::wstring &utf16);

// CCompositionEditSession implementation
CCompositionEditSession::CCompositionEditSession(CKeyMagicTextService *pTextService, ITfContext *pContext, 
                                                 CCompositionManager *pCompositionManager,
                                                 EditAction action, EngineHandle *pEngine)
{
    m_cRef = 1;
    m_pTextService = pTextService;
    m_pTextService->AddRef();
    m_pContext = pContext;
    m_pContext->AddRef();
    m_pCompositionManager = pCompositionManager;
    m_pCompositionManager->AddRef();
    m_action = action;
    m_pEngine = pEngine;
    m_wParam = 0;
    m_lParam = 0;
    m_pfEaten = nullptr;
}

CCompositionEditSession::~CCompositionEditSession()
{
    m_pCompositionManager->Release();
    m_pContext->Release();
    m_pTextService->Release();
}

// IUnknown
STDAPI CCompositionEditSession::QueryInterface(REFIID riid, void **ppvObject)
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

STDAPI_(ULONG) CCompositionEditSession::AddRef()
{
    return InterlockedIncrement(&m_cRef);
}

STDAPI_(ULONG) CCompositionEditSession::Release()
{
    LONG cr = InterlockedDecrement(&m_cRef);

    if (cr == 0)
    {
        delete this;
    }

    return cr;
}

// ITfEditSession
STDAPI CCompositionEditSession::DoEditSession(TfEditCookie ec)
{
    switch (m_action)
    {
        case EditAction::ProcessKey:
            return ProcessKey(ec);
            
        case EditAction::SyncEngine:
            return SyncEngineWithDocument(ec);
            
        case EditAction::CommitAndRecompose:
            return CommitAndRecomposeAtCursor(ec);
            
        case EditAction::TerminateComposition:
            return TerminateComposition(ec);
    }
    
    return S_OK;
}

void CCompositionEditSession::SetKeyData(WPARAM wParam, LPARAM lParam, BOOL *pfEaten)
{
    m_wParam = wParam;
    m_lParam = lParam;
    m_pfEaten = pfEaten;
}

// Process key implementation using composition
HRESULT CCompositionEditSession::ProcessKey(TfEditCookie ec)
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
        DEBUG_LOG(L"Engine process key failed, terminating composition");
        m_pCompositionManager->EndComposition(ec);
        *m_pfEaten = FALSE;
        return S_OK;
    }
    
    // Process the output
    DEBUG_LOG(L"Engine output - action: " + std::to_wstring(output.action_type) + 
              L", is_processed: " + std::to_wstring(output.is_processed));
    
    // Handle composition based on engine's composing text
    if (output.composing_text && strlen(output.composing_text) > 0)
    {
        std::string composingUtf8(output.composing_text);
        std::wstring composingText = ConvertUtf8ToUtf16(composingUtf8);
        
        DEBUG_LOG_TEXT(L"Engine composing text", composingText);
        
        // Check if we should commit the composition
        if (ShouldCommitComposition(m_wParam, composingUtf8, output.is_processed))
        {
            DEBUG_LOG(L"Committing composition");
            
            // Determine text to commit
            std::wstring textToCommit = composingText;
            
            // For space key that wasn't processed, append space to commit
            if (m_wParam == VK_SPACE && !output.is_processed)
            {
                textToCommit += L' ';
            }
            
            // Commit the text
            m_pCompositionManager->CommitComposition(m_pContext, ec, textToCommit);
            
            // Reset engine after commit
            keymagic_engine_reset(m_pEngine);
        }
        else
        {
            // Update composition display with engine's composing text
            DEBUG_LOG_TEXT(L"Updating composition", composingText);
            
            if (!m_pCompositionManager->IsComposing())
            {
                m_pCompositionManager->StartComposition(m_pContext, ec);
            }
            
            m_pCompositionManager->UpdateComposition(m_pContext, ec, composingText);
        }
    }
    else
    {
        // Engine has no composing text - clear any existing composition
        DEBUG_LOG(L"Engine has no composing text, ending composition");
        if (m_pCompositionManager->IsComposing())
        {
            m_pCompositionManager->EndComposition(ec);
        }
        
        // For special keys that should reset the engine
        switch (m_wParam)
        {
            case VK_ESCAPE:
            case VK_RETURN:
            case VK_TAB:
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
HRESULT CCompositionEditSession::SyncEngineWithDocument(TfEditCookie ec)
{
    DEBUG_LOG_FUNC();
    
    if (!m_pEngine)
        return S_OK;
    
    // Check if we're actively composing
    if (m_pCompositionManager && m_pCompositionManager->IsComposing())
    {
        // Get current engine composition
        char* engineComposing = keymagic_engine_get_composition(m_pEngine);
        if (engineComposing)
        {
            std::string engineText(engineComposing);
            keymagic_free_string(engineComposing);
            
            // Get current document composition text
            std::wstring documentCompositionText;
            if (SUCCEEDED(m_pCompositionManager->GetCompositionText(ec, documentCompositionText)))
            {
                // Convert to UTF-8 for comparison
                std::string documentTextUtf8 = ConvertUtf16ToUtf8(documentCompositionText);
                
                // Compare engine and document composition text
                if (engineText == documentTextUtf8)
                {
                    DEBUG_LOG_TEXT(L"Engine and document composition are already in sync", documentCompositionText);
                    return S_OK;  // Already in sync, no need to sync again
                }
                else
                {
                    DEBUG_LOG_SYNC_MISMATCH(engineText, documentCompositionText);
                }
            }
        }
    }

    // Read text before cursor to sync with engine
    std::wstring documentText;
    const int MAX_COMPOSE_LENGTH = 50; // Maximum reasonable compose length
    
    if (SUCCEEDED(ReadTextBeforeCursor(ec, MAX_COMPOSE_LENGTH, documentText)))
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
            
            // First try to start composition on the existing text
            if (!composeText.empty())
            {
                DEBUG_LOG_TEXT(L"Starting composition on document text", composeText);
                
                // Use the new method to compose existing text at cursor position
                HRESULT hr = m_pCompositionManager->StartCompositionAtSelection(m_pContext, ec, composeText.length(), 0);
                
                if (SUCCEEDED(hr))
                {
                    DEBUG_LOG(L"Successfully started composition on document text");
                    
                    // Only set engine composition if we successfully started document composition
                    std::string utf8Text = ConvertUtf16ToUtf8(composeText);
                    KeyMagicResult result = keymagic_engine_set_composition(m_pEngine, utf8Text.c_str());
                    
                    if (result == KeyMagicResult_Success)
                    {
                        DEBUG_LOG(L"Successfully synced engine with document composition");
                    }
                    else
                    {
                        DEBUG_LOG(L"Failed to set engine composition, error: " + std::to_wstring(result));
                        // End the composition since we couldn't sync the engine
                        m_pCompositionManager->EndComposition(ec);
                        keymagic_engine_reset(m_pEngine);
                    }
                }
                else
                {
                    DEBUG_LOG(L"Failed to start composition on document text");
                    // Don't set engine composition if we couldn't start document composition
                    keymagic_engine_reset(m_pEngine);
                }
            }
            else
            {
                // No text to compose, just ensure engine is reset
                DEBUG_LOG(L"No text to compose, resetting engine");
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

// Commit and recompose at cursor position (for navigation keys)
HRESULT CCompositionEditSession::CommitAndRecomposeAtCursor(TfEditCookie ec)
{
    DEBUG_LOG_FUNC();
    
    // First, commit any existing composition
    if (m_pCompositionManager->IsComposing())
    {
        DEBUG_LOG(L"Committing existing composition");
        m_pCompositionManager->EndComposition(ec);
    }
    
    // Reset engine and sync with new cursor position
    keymagic_engine_reset(m_pEngine);
    
    // Now sync engine with text at new cursor position
    return SyncEngineWithDocument(ec);
}

// Document reading method - read text before cursor
HRESULT CCompositionEditSession::ReadTextBeforeCursor(TfEditCookie ec, int maxChars, std::wstring &text)
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
char CCompositionEditSession::MapVirtualKeyToChar(WPARAM wParam, LPARAM lParam)
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

bool CCompositionEditSession::IsPrintableAscii(char c)
{
    return c >= 0x20 && c <= 0x7E;
}

bool CCompositionEditSession::ShouldCommitComposition(WPARAM wParam, const std::string& composingText, bool isProcessed)
{
    // If the engine didn't process the key, we should commit the composition
    if (!isProcessed) {
        return true;
    }

    // Check special keys that should trigger commit
    switch (wParam)
    {
        case VK_SPACE:
            // Commit if engine processed space and composing text ends with space
            return !composingText.empty() && composingText.back() == ' ';
        case VK_RETURN:
        case VK_TAB:
        case VK_ESCAPE:
            // Always commit for these keys
            return true;
        default:
            // Don't commit for other keys
            return false;
    }
}

// Terminate composition and reset engine
HRESULT CCompositionEditSession::TerminateComposition(TfEditCookie ec)
{
    DEBUG_LOG_FUNC();
    
    // Terminate any active composition
    if (m_pCompositionManager && m_pCompositionManager->IsComposing())
    {
        DEBUG_LOG(L"Terminating active composition");
        m_pCompositionManager->EndComposition(ec);
    }
    
    // Reset the engine
    if (m_pEngine)
    {
        DEBUG_LOG(L"Resetting engine");
        keymagic_engine_reset(m_pEngine);
    }
    
    return S_OK;
}