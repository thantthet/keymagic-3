#ifndef COMPOSITION_EDIT_SESSION_H
#define COMPOSITION_EDIT_SESSION_H

#include <windows.h>
#include <msctf.h>
#include <string>

// Forward declarations
class CKeyMagicTextService;
class CCompositionManager;

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

#endif // COMPOSITION_EDIT_SESSION_H