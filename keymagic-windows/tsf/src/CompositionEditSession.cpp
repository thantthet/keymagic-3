#include "CompositionEditSession.h"
#include "KeyMagicTextService.h"
#include "Composition.h"
#include "Debug.h"
#include "../include/keymagic_ffi.h"

extern std::wstring ConvertUtf8ToUtf16(const std::string &utf8);
extern std::string ConvertUtf16ToUtf8(const std::wstring &utf16);

// CCompositionEditSession implementation
CCompositionEditSession::CCompositionEditSession(CKeyMagicTextService *pTextService, 
                                                 ITfContext *pContext,
                                                 CCompositionManager *pCompositionMgr,
                                                 CompositionAction action)
{
    m_cRef = 1;
    m_pTextService = pTextService;
    m_pTextService->AddRef();
    m_pContext = pContext;
    m_pContext->AddRef();
    m_pCompositionMgr = pCompositionMgr;
    m_action = action;
    m_wParam = 0;
    m_lParam = 0;
    m_pfEaten = nullptr;
}

CCompositionEditSession::~CCompositionEditSession()
{
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
        *ppvObject = static_cast<ITfEditSession*>(this);
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
        case CompositionAction::ProcessKey:
            return ProcessKeyInComposition(ec);
            
        case CompositionAction::UpdateText:
            if (m_pCompositionMgr)
                return m_pCompositionMgr->UpdateComposition(m_pContext, ec, m_text);
            break;
            
        case CompositionAction::Commit:
            if (m_pCompositionMgr)
                return m_pCompositionMgr->CommitComposition(m_pContext, ec, m_text);
            break;
            
        case CompositionAction::Cancel:
            if (m_pCompositionMgr)
                return m_pCompositionMgr->CancelComposition(ec);
            break;
    }
    
    return S_OK;
}

HRESULT CCompositionEditSession::ProcessKeyInComposition(TfEditCookie ec)
{
    if (!m_pTextService || !m_pfEaten)
        return E_FAIL;
        
    EngineHandle* pEngine = m_pTextService->GetEngineHandle();
    if (!pEngine)
    {
        *m_pfEaten = FALSE;
        return S_OK;
    }
    
    // Get modifiers
    int shift = (GetKeyState(VK_SHIFT) & 0x8000) ? 1 : 0;
    int ctrl = (GetKeyState(VK_CONTROL) & 0x8000) ? 1 : 0;
    int alt = (GetKeyState(VK_MENU) & 0x8000) ? 1 : 0;
    int capsLock = (GetKeyState(VK_CAPITAL) & 0x0001) ? 1 : 0;
    
    // Get character
    char character = m_pTextService->MapVirtualKeyToChar(m_wParam, m_lParam);
    if (!m_pTextService->IsPrintableAscii(character))
    {
        character = '\0';
    }
    
    ProcessKeyOutput output = {0};
    
    // Process key with engine
    KeyMagicResult result = keymagic_engine_process_key_win(
        pEngine, 
        static_cast<int>(m_wParam),
        character,
        shift, ctrl, alt, capsLock,
        &output
    );
    
    if (result == KeyMagicResult_Success)
    {
        *m_pfEaten = output.is_processed ? TRUE : FALSE;
        
        // Handle the output
        if (output.composing_text)
        {
            std::string composingUtf8(output.composing_text);
            std::wstring composingText = ConvertUtf8ToUtf16(composingUtf8);
            
            // Check for commit triggers
            bool shouldCommit = false;
            std::wstring textToCommit;
            
            switch (m_wParam)
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
                        *m_pfEaten = TRUE;  // Eat the space since we're handling it
                    }
                    break;
                    
                case VK_RETURN:
                case VK_TAB:
                    // Commit current composing text
                    shouldCommit = true;
                    textToCommit = composingText;
                    *m_pfEaten = FALSE;  // Let the key pass through after commit
                    break;
                    
                case VK_ESCAPE:
                    // Cancel composition
                    if (m_pCompositionMgr)
                        m_pCompositionMgr->CancelComposition(ec);
                    keymagic_engine_reset(pEngine);
                    *m_pfEaten = TRUE;
                    break;
            }
            
            if (shouldCommit)
            {
                // Commit the text
                if (m_pCompositionMgr)
                    m_pCompositionMgr->CommitComposition(m_pContext, ec, textToCommit);
                    
                // Reset engine after commit
                keymagic_engine_reset(pEngine);
            }
            else if (!composingText.empty())
            {
                // Update composition display
                if (m_pCompositionMgr)
                    m_pCompositionMgr->UpdateComposition(m_pContext, ec, composingText);
            }
            else
            {
                // Empty composing text, cancel composition
                if (m_pCompositionMgr)
                    m_pCompositionMgr->CancelComposition(ec);
            }
        }
        
        // Cleanup
        if (output.text) keymagic_free_string(output.text);
        if (output.composing_text) keymagic_free_string(output.composing_text);
    }
    
    return S_OK;
}

void CCompositionEditSession::SetKeyData(WPARAM wParam, LPARAM lParam, BOOL *pfEaten)
{
    m_wParam = wParam;
    m_lParam = lParam;
    m_pfEaten = pfEaten;
}

void CCompositionEditSession::SetText(const std::wstring &text)
{
    m_text = text;
}