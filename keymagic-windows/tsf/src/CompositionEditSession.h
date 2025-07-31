#ifndef COMPOSITION_EDIT_SESSION_H
#define COMPOSITION_EDIT_SESSION_H

#include <windows.h>
#include <msctf.h>
#include <string>
#include "../../shared/include/keymagic_ffi.h"

// Forward declaration
class CKeyMagicTextService;
class CCompositionManager;

// Edit session for composition-based text handling
class CCompositionEditSession : public ITfEditSession
{
public:
    enum class EditAction
    {
        ProcessKey,
        SyncEngine,
        CommitAndRecompose,
        TerminateComposition
    };
    
    CCompositionEditSession(CKeyMagicTextService *pTextService, ITfContext *pContext, 
                           CCompositionManager *pCompositionManager,
                           EditAction action, EngineHandle *pEngine);
    ~CCompositionEditSession();
    
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
    CCompositionManager *m_pCompositionManager;
    EditAction m_action;
    EngineHandle *m_pEngine;
    
    // Action-specific data
    WPARAM m_wParam;
    LPARAM m_lParam;
    BOOL *m_pfEaten;
    
    // Action implementations
    HRESULT ProcessKey(TfEditCookie ec);
    HRESULT SyncEngineWithDocument(TfEditCookie ec);
    HRESULT CommitAndRecomposeAtCursor(TfEditCookie ec);
    HRESULT TerminateComposition(TfEditCookie ec);
    
    // Document reading methods
    HRESULT ReadTextBeforeCursor(TfEditCookie ec, int maxChars, std::wstring &text);
    
    // Helper methods
    bool ShouldCommitComposition(WPARAM wParam, const std::string& composingText, bool isProcessed);
};

#endif // COMPOSITION_EDIT_SESSION_H