#include "Composition.h"
#include "KeyMagicTextService.h"
#include "Debug.h"
#include <string>

// Helper function to convert UTF-8 to UTF-16
extern std::wstring ConvertUtf8ToUtf16(const std::string& utf8);

CCompositionManager::CCompositionManager(CKeyMagicTextService *pTextService)
{
    m_cRef = 1;
    m_pTextService = pTextService;
    m_pTextService->AddRef();
    m_pComposition = nullptr;
    m_pContext = nullptr;
}

CCompositionManager::~CCompositionManager()
{
    if (m_pComposition)
    {
        // Should already be terminated, but just in case
        m_pComposition->Release();
        m_pComposition = nullptr;
    }
    
    if (m_pContext)
    {
        m_pContext->Release();
        m_pContext = nullptr;
    }
    
    m_pTextService->Release();
}

// IUnknown
STDAPI CCompositionManager::QueryInterface(REFIID riid, void **ppvObject)
{
    if (ppvObject == nullptr)
        return E_INVALIDARG;

    *ppvObject = nullptr;

    if (IsEqualIID(riid, IID_IUnknown) || IsEqualIID(riid, IID_ITfCompositionSink))
    {
        *ppvObject = static_cast<ITfCompositionSink*>(this);
    }

    if (*ppvObject)
    {
        AddRef();
        return S_OK;
    }

    return E_NOINTERFACE;
}

STDAPI_(ULONG) CCompositionManager::AddRef()
{
    return InterlockedIncrement(&m_cRef);
}

STDAPI_(ULONG) CCompositionManager::Release()
{
    LONG cRef = InterlockedDecrement(&m_cRef);
    if (cRef == 0)
    {
        delete this;
    }
    return cRef;
}

HRESULT CCompositionManager::StartComposition(ITfContext *pContext, TfEditCookie ec)
{
    DEBUG_LOG_FUNC();
    
    if (m_pComposition)
    {
        DEBUG_LOG(L"Composition already active");
        return S_OK;  // Already composing
    }
    
    // Get the insertion point
    ITfInsertAtSelection *pInsertAtSelection;
    HRESULT hr = pContext->QueryInterface(IID_ITfInsertAtSelection, (void **)&pInsertAtSelection);
    if (FAILED(hr))
    {
        DEBUG_LOG(L"Failed to get ITfInsertAtSelection");
        return hr;
    }
    
    ITfRange *pRange;
    hr = pInsertAtSelection->InsertTextAtSelection(ec, TF_IAS_QUERYONLY, nullptr, 0, &pRange);
    pInsertAtSelection->Release();
    
    if (FAILED(hr) || !pRange)
    {
        DEBUG_LOG(L"Failed to get insertion range");
        return hr;
    }
    
    // Start composition
    ITfContextComposition *pContextComposition;
    hr = pContext->QueryInterface(IID_ITfContextComposition, (void **)&pContextComposition);
    if (SUCCEEDED(hr))
    {
        ITfCompositionSink *pCompositionSink = static_cast<ITfCompositionSink*>(this);
        hr = pContextComposition->StartComposition(ec, pRange, pCompositionSink, &m_pComposition);
        pContextComposition->Release();
        
        if (SUCCEEDED(hr) && m_pComposition)
        {
            DEBUG_LOG(L"Composition started successfully");
            
            // Store context for later use
            if (m_pContext)
                m_pContext->Release();
            m_pContext = pContext;
            m_pContext->AddRef();
        }
        else
        {
            DEBUG_LOG(L"StartComposition failed");
        }
    }
    
    pRange->Release();
    return hr;
}

HRESULT CCompositionManager::UpdateComposition(ITfContext *pContext, TfEditCookie ec, const std::wstring &text)
{
    DEBUG_LOG_FUNC();
    DEBUG_LOG(L"Updating composition with: \"" + text + L"\"");
    
    // Start composition if needed
    if (!m_pComposition)
    {
        HRESULT hr = StartComposition(pContext, ec);
        if (FAILED(hr))
        {
            DEBUG_LOG(L"Failed to start composition");
            return hr;
        }
    }
    
    if (!m_pComposition)
    {
        DEBUG_LOG(L"No composition object available");
        return E_FAIL;
    }
    
    // Get the composition range
    ITfRange *pRange;
    HRESULT hr = m_pComposition->GetRange(&pRange);
    if (FAILED(hr))
    {
        DEBUG_LOG(L"Failed to get composition range");
        return hr;
    }
    
    // Set the text
    hr = pRange->SetText(ec, 0, text.c_str(), static_cast<LONG>(text.length()));
    if (FAILED(hr))
    {
        DEBUG_LOG(L"Failed to set composition text");
    }
    else
    {
        DEBUG_LOG(L"Composition text updated successfully");
        
        // Apply display attributes (underline)
        ApplyDisplayAttributes(pContext, ec, pRange);
    }
    
    pRange->Release();
    return hr;
}

HRESULT CCompositionManager::EndComposition(TfEditCookie ec)
{
    DEBUG_LOG_FUNC();
    
    if (!m_pComposition)
    {
        DEBUG_LOG(L"No active composition to end");
        return S_OK;
    }
    
    HRESULT hr = m_pComposition->EndComposition(ec);
    if (SUCCEEDED(hr))
    {
        DEBUG_LOG(L"Composition ended successfully");
    }
    else
    {
        DEBUG_LOG(L"Failed to end composition");
    }
    
    m_pComposition->Release();
    m_pComposition = nullptr;
    
    return hr;
}

HRESULT CCompositionManager::CommitComposition(ITfContext *pContext, TfEditCookie ec, const std::wstring &text)
{
    DEBUG_LOG_FUNC();
    DEBUG_LOG(L"Committing text: \"" + text + L"\"");
    
    if (m_pComposition)
    {
        // Update the text one last time before committing
        if (!text.empty())
        {
            ITfRange *pRange;
            if (SUCCEEDED(m_pComposition->GetRange(&pRange)))
            {
                pRange->SetText(ec, 0, text.c_str(), static_cast<LONG>(text.length()));
                
                // Collapse the range to the end to avoid selection
                pRange->Collapse(ec, TF_ANCHOR_END);
                pRange->Release();
            }
        }
        
        // End the composition
        EndComposition(ec);
        
        // Clear selection after composition ends
        ClearSelection(pContext, ec);
    }
    else if (!text.empty())
    {
        // No composition active, just insert the text
        ITfInsertAtSelection *pInsertAtSelection;
        if (SUCCEEDED(pContext->QueryInterface(IID_ITfInsertAtSelection, (void **)&pInsertAtSelection)))
        {
            ITfRange *pRange;
            pInsertAtSelection->InsertTextAtSelection(ec, 0, text.c_str(), 
                                                    static_cast<LONG>(text.length()), &pRange);
            if (pRange)
                pRange->Release();
            pInsertAtSelection->Release();
        }
    }
    
    return S_OK;
}

HRESULT CCompositionManager::CancelComposition(TfEditCookie ec)
{
    DEBUG_LOG_FUNC();
    
    if (!m_pComposition)
        return S_OK;
        
    // Clear the composition text
    ITfRange *pRange;
    if (SUCCEEDED(m_pComposition->GetRange(&pRange)))
    {
        pRange->SetText(ec, 0, L"", 0);
        pRange->Release();
    }
    
    // End composition
    return EndComposition(ec);
}

HRESULT CCompositionManager::ApplyDisplayAttributes(ITfContext *pContext, TfEditCookie ec, ITfRange *pRange)
{
    // Get the display attribute property
    ITfProperty *pDisplayAttributeProperty;
    HRESULT hr = pContext->GetProperty(GUID_PROP_ATTRIBUTE, &pDisplayAttributeProperty);
    if (FAILED(hr))
        return hr;
        
    // Get the GUID atom for our input display attribute from the text service
    TfGuidAtom guidAtom = TF_INVALID_GUIDATOM;
    if (m_pTextService)
    {
        guidAtom = m_pTextService->m_inputDisplayAttributeAtom;
    }
    
    if (guidAtom != TF_INVALID_GUIDATOM)
    {
        // Create a variant with the proper GUID atom
        VARIANT var;
        var.vt = VT_I4;
        var.lVal = guidAtom;
        
        // Apply the attribute to the range
        hr = pDisplayAttributeProperty->SetValue(ec, pRange, &var);
        
        if (SUCCEEDED(hr))
        {
            DEBUG_LOG(L"Applied display attribute with GUID atom: " + std::to_wstring(guidAtom));
        }
        else
        {
            DEBUG_LOG(L"Failed to apply display attribute");
        }
    }
    else
    {
        DEBUG_LOG(L"Invalid GUID atom for display attribute");
        hr = E_FAIL;
    }
    
    pDisplayAttributeProperty->Release();
    return hr;
}

void CCompositionManager::ClearSelection(ITfContext *pContext, TfEditCookie ec)
{
    // Get current selection
    TF_SELECTION tfSelection;
    ULONG cFetched;
    if (SUCCEEDED(pContext->GetSelection(ec, TF_DEFAULT_SELECTION, 1, &tfSelection, &cFetched)) && cFetched > 0)
    {
        // Collapse the selection to the end
        tfSelection.range->Collapse(ec, TF_ANCHOR_END);
        
        // Update the selection
        pContext->SetSelection(ec, 1, &tfSelection);
        
        tfSelection.range->Release();
    }
}

// ITfCompositionSink implementation
STDAPI CCompositionManager::OnCompositionTerminated(TfEditCookie ecWrite, ITfComposition *pComposition)
{
    DEBUG_LOG_FUNC();
    
    // Clean up our reference
    if (m_pComposition == pComposition)
    {
        m_pComposition->Release();
        m_pComposition = nullptr;
    }
    
    return S_OK;
}

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
    LONG cRef = InterlockedDecrement(&m_cRef);
    if (cRef == 0)
    {
        delete this;
    }
    return cRef;
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