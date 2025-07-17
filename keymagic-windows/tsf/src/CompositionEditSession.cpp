#include "CompositionEditSession.h"
#include "KeyMagicTextService.h"
#include "Composition.h"
#include "Debug.h"
#include "Globals.h"
#include "KeyProcessingUtils.h"
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
        // Eat all printable characters when no keyboard is loaded
        char character = KeyProcessingUtils::MapVirtualKeyToChar(m_wParam, m_lParam);
        *m_pfEaten = KeyProcessingUtils::IsPrintableAscii(character) ? TRUE : FALSE;
        if (*m_pfEaten)
        {
            DEBUG_LOG(L"Eating printable character with no keyboard loaded");
        }
        return S_OK;
    }

    // Use consolidated key processing utility
    KeyProcessingUtils::KeyInputData keyInput = KeyProcessingUtils::PrepareKeyInput(m_wParam, m_lParam);
    
    // Log the key event using secure debug macro
    DEBUG_LOG_KEY(L"ProcessKey Input", m_wParam, m_lParam, keyInput.character);
    
    // Skip if needed (modifier/function keys)
    if (keyInput.shouldSkip)
    {
        *m_pfEaten = FALSE;
        return S_OK;
    }
    
    // Process with engine
    ProcessKeyOutput output = {0};
    
    KeyMagicResult result = keymagic_engine_process_key_win(
        m_pEngine, 
        static_cast<int>(m_wParam), 
        keyInput.character, 
        keyInput.shift, keyInput.ctrl, keyInput.alt, keyInput.capsLock, 
        &output
    );
    
    if (result != KeyMagicResult_Success)
    {
        DEBUG_LOG(L"Engine process key failed, terminating composition");
        m_pCompositionManager->EndComposition(ec);
        *m_pfEaten = FALSE;
        return S_OK;
    }
    
    // Process the output using secure debug macro
    DEBUG_LOG_ENGINE(output);
    
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
            if (output.is_processed) {
                // If the engine processed the key, we should update the composition
                m_pCompositionManager->UpdateComposition(m_pContext, ec, L"");
            }
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
                    // Mismatch detected - reset engine and end composition
                    DEBUG_LOG(L"Engine and document composition mismatch, resetting engine");
                    keymagic_engine_reset(m_pEngine);
                    m_pCompositionManager->EndComposition(ec);
                    return S_OK;
                }
            }
        }
    }

    // If not composing, just reset the engine
    DEBUG_LOG(L"Not composing, resetting engine");
    keymagic_engine_reset(m_pEngine);
    
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