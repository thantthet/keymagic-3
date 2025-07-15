#ifndef DIRECT_EDIT_SESSION_H
#define DIRECT_EDIT_SESSION_H

#include <windows.h>
#include <msctf.h>
#include <string>

// Forward declaration
class CKeyMagicTextService;

// Edit session for direct text manipulation (no composition)
class CDirectEditSession : public ITfEditSession
{
public:
    enum class EditAction
    {
        ProcessKey,
        SyncEngine
    };
    
    CDirectEditSession(CKeyMagicTextService *pTextService, ITfContext *pContext, 
                       EditAction action);
    ~CDirectEditSession();
    
    // IUnknown
    STDMETHODIMP QueryInterface(REFIID riid, void **ppvObject);
    STDMETHODIMP_(ULONG) AddRef();
    STDMETHODIMP_(ULONG) Release();
    
    // ITfEditSession
    STDMETHODIMP DoEditSession(TfEditCookie ec);
    
    // Set parameters for different actions
    void SetKeyData(WPARAM wParam, LPARAM lParam, BOOL *pfEaten);
    
private:
    LONG m_cRef;
    CKeyMagicTextService *m_pTextService;
    ITfContext *m_pContext;
    EditAction m_action;
    
    // Action-specific data
    WPARAM m_wParam;
    LPARAM m_lParam;
    BOOL *m_pfEaten;
};

#endif // DIRECT_EDIT_SESSION_H