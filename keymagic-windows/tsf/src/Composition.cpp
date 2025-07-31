#include "Composition.h"
#include "KeyMagicTextService.h"
#include "Debug.h"
#include <string>

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
            
            // Move cursor to the end of the range (which should be at insertion point)
            ITfRange *pSelection;
            if (SUCCEEDED(pRange->Clone(&pSelection)))
            {
                // Collapse to end of range
                pSelection->Collapse(ec, TF_ANCHOR_END);
                
                // Set selection
                TF_SELECTION tfSelection;
                tfSelection.range = pSelection;
                tfSelection.style.ase = TF_AE_NONE;
                tfSelection.style.fInterimChar = FALSE;
                
                pContext->SetSelection(ec, 1, &tfSelection);
                pSelection->Release();
                
                DEBUG_LOG(L"Cursor positioned at composition start point");
            }
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
    DEBUG_LOG_TEXT(L"Updating composition with", text);
    
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
        
        // Move cursor to the end of composition
        ITfRange *pSelection;
        if (SUCCEEDED(pRange->Clone(&pSelection)))
        {
            // Collapse to end of range
            pSelection->Collapse(ec, TF_ANCHOR_END);
            
            // Set selection
            TF_SELECTION tfSelection;
            tfSelection.range = pSelection;
            tfSelection.style.ase = TF_AE_NONE;
            tfSelection.style.fInterimChar = FALSE;
            
            pContext->SetSelection(ec, 1, &tfSelection);
            pSelection->Release();
            
            DEBUG_LOG(L"Cursor moved to end of composition");
        }
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
    DEBUG_LOG_TEXT(L"Committing text", text);
    
    if (m_pComposition)
    {
        // Update the text one last time before committing
        if (!text.empty())
        {
            // Use UpdateComposition to set the final text
            // This also handles display attributes and cursor positioning
            UpdateComposition(pContext, ec, text);
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
            {
                // Move cursor to end of inserted text
                ITfRange *pSelection;
                if (SUCCEEDED(pRange->Clone(&pSelection)))
                {
                    pSelection->Collapse(ec, TF_ANCHOR_END);
                    
                    TF_SELECTION tfSelection;
                    tfSelection.range = pSelection;
                    tfSelection.style.ase = TF_AE_NONE;
                    tfSelection.style.fInterimChar = FALSE;
                    
                    pContext->SetSelection(ec, 1, &tfSelection);
                    pSelection->Release();
                }
                
                pRange->Release();
            }
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

HRESULT CCompositionManager::GetCompositionText(TfEditCookie ec, std::wstring &text)
{
    text.clear();
    
    if (!m_pComposition)
    {
        return E_FAIL;  // No active composition
    }
    
    // Get the composition range
    ITfRange *pRange;
    HRESULT hr = m_pComposition->GetRange(&pRange);
    if (FAILED(hr))
    {
        return hr;
    }
    
    // Read the text from the range
    WCHAR buffer[512];  // Should be enough for most compositions
    ULONG cch;
    hr = pRange->GetText(ec, 0, buffer, ARRAYSIZE(buffer) - 1, &cch);
    if (SUCCEEDED(hr))
    {
        buffer[cch] = L'\0';
        text = buffer;
    }
    
    pRange->Release();
    return hr;
}

HRESULT CCompositionManager::ApplyDisplayAttributes(ITfContext *pContext, TfEditCookie ec, ITfRange *pRange)
{
    // Get the display attribute property
    ITfProperty *pDisplayAttributeProperty;
    HRESULT hr = pContext->GetProperty(GUID_PROP_ATTRIBUTE, &pDisplayAttributeProperty);
    if (FAILED(hr))
    {
        DEBUG_LOG(L"Failed to get display attribute property");
        return hr;
    }
        
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
            DEBUG_LOG(L"Failed to apply display attribute, error: 0x" + std::to_wstring(hr));
        }
    }
    else
    {
        DEBUG_LOG(L"Warning: Display attribute GUID atom is invalid - composition will not have underline");
        // Don't fail - allow composition to continue without display attributes
        hr = S_OK;
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

