#ifndef COMPOSITION_H
#define COMPOSITION_H

#include <windows.h>
#include <msctf.h>
#include <string>

// Forward declarations
class CKeyMagicTextService;

// Manages TSF composition for displaying composing text with underline
class CCompositionManager : public ITfCompositionSink
{
public:
    CCompositionManager(CKeyMagicTextService *pTextService);
    ~CCompositionManager();
    
    // IUnknown
    STDMETHODIMP QueryInterface(REFIID riid, void **ppvObject);
    STDMETHODIMP_(ULONG) AddRef();
    STDMETHODIMP_(ULONG) Release();
    
    // Composition management
    HRESULT StartComposition(ITfContext *pContext, TfEditCookie ec);
    HRESULT UpdateComposition(ITfContext *pContext, TfEditCookie ec, const std::wstring &text);
    HRESULT EndComposition(TfEditCookie ec);
    HRESULT CommitComposition(ITfContext *pContext, TfEditCookie ec, const std::wstring &text);
    HRESULT CancelComposition(TfEditCookie ec);
    
    // Check if composition is active
    BOOL IsComposing() const { return m_pComposition != nullptr; }
    
    // ITfCompositionSink
    STDMETHODIMP OnCompositionTerminated(TfEditCookie ecWrite, ITfComposition *pComposition);
    
private:
    // Apply display attributes (underline) to composition
    HRESULT ApplyDisplayAttributes(ITfContext *pContext, TfEditCookie ec, ITfRange *pRange);
    
    // Clear text selection after composition
    void ClearSelection(ITfContext *pContext, TfEditCookie ec);
    
    LONG m_cRef;
    CKeyMagicTextService *m_pTextService;
    ITfComposition *m_pComposition;
    ITfContext *m_pContext;
};

// Edit session for composition operations
class CCompositionEditSession : public ITfEditSession
{
public:
    enum class CompositionAction
    {
        ProcessKey,
        UpdateText,
        Commit,
        Cancel
    };
    
    CCompositionEditSession(CKeyMagicTextService *pTextService, 
                           ITfContext *pContext,
                           CCompositionManager *pCompositionMgr,
                           CompositionAction action);
    ~CCompositionEditSession();
    
    // IUnknown
    STDMETHODIMP QueryInterface(REFIID riid, void **ppvObject);
    STDMETHODIMP_(ULONG) AddRef();
    STDMETHODIMP_(ULONG) Release();
    
    // ITfEditSession
    STDMETHODIMP DoEditSession(TfEditCookie ec);
    
    // Set parameters for different actions
    void SetKeyData(WPARAM wParam, LPARAM lParam, BOOL *pfEaten);
    void SetText(const std::wstring &text);
    
private:
    HRESULT ProcessKeyInComposition(TfEditCookie ec);
    
    LONG m_cRef;
    CKeyMagicTextService *m_pTextService;
    ITfContext *m_pContext;
    CCompositionManager *m_pCompositionMgr;
    CompositionAction m_action;
    
    // Action-specific data
    WPARAM m_wParam;
    LPARAM m_lParam;
    BOOL *m_pfEaten;
    std::wstring m_text;
};

#endif // COMPOSITION_H