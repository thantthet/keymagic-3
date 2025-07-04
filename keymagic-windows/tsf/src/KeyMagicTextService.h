#ifndef KEYMAGIC_TEXT_SERVICE_H
#define KEYMAGIC_TEXT_SERVICE_H

#include <windows.h>
#include <msctf.h>
#include <string>
#include <memory>
#include "../include/keymagic_ffi.h"

// Forward declaration
class CKeyMagicTextService;

class CKeyMagicTextService : public ITfTextInputProcessor,
                            public ITfThreadMgrEventSink,
                            public ITfKeyEventSink,
                            public ITfCompositionSink
{
public:
    CKeyMagicTextService();
    ~CKeyMagicTextService();

    // IUnknown
    STDMETHODIMP QueryInterface(REFIID riid, void **ppvObject);
    STDMETHODIMP_(ULONG) AddRef();
    STDMETHODIMP_(ULONG) Release();

    // ITfTextInputProcessor
    STDMETHODIMP Activate(ITfThreadMgr *ptim, TfClientId tid);
    STDMETHODIMP Deactivate();

    // ITfThreadMgrEventSink
    STDMETHODIMP OnInitDocumentMgr(ITfDocumentMgr *pdim);
    STDMETHODIMP OnUninitDocumentMgr(ITfDocumentMgr *pdim);
    STDMETHODIMP OnSetFocus(ITfDocumentMgr *pdimFocus, ITfDocumentMgr *pdimPrevFocus);
    STDMETHODIMP OnPushContext(ITfContext *pic);
    STDMETHODIMP OnPopContext(ITfContext *pic);

    // ITfKeyEventSink
    STDMETHODIMP OnSetFocus(BOOL fForeground);
    STDMETHODIMP OnTestKeyDown(ITfContext *pic, WPARAM wParam, LPARAM lParam, BOOL *pfEaten);
    STDMETHODIMP OnKeyDown(ITfContext *pic, WPARAM wParam, LPARAM lParam, BOOL *pfEaten);
    STDMETHODIMP OnTestKeyUp(ITfContext *pic, WPARAM wParam, LPARAM lParam, BOOL *pfEaten);
    STDMETHODIMP OnKeyUp(ITfContext *pic, WPARAM wParam, LPARAM lParam, BOOL *pfEaten);
    STDMETHODIMP OnPreservedKey(ITfContext *pic, REFGUID rguid, BOOL *pfEaten);
    
    // ITfCompositionSink
    STDMETHODIMP OnCompositionTerminated(TfEditCookie ecWrite, ITfComposition *pComposition);

    // Public methods
    EngineHandle* GetEngineHandle() { return m_pEngine; }

private:
    // Helper methods
    BOOL InitializeEngine();
    void UninitializeEngine();
    BOOL LoadKeyboard(const std::wstring& km2Path);
    void ProcessKeyInput(ITfContext *pic, WPARAM wParam, LPARAM lParam, BOOL *pfEaten);
    void UpdateComposition(ITfContext *pic, bool shouldCommit, const std::wstring& textToCommit, const std::wstring& composingText);
    void CommitText(ITfContext *pic, const std::wstring& text);
    void TerminateComposition(ITfContext *pic);
    BOOL IsKeyEaten(ITfContext *pic, WPARAM wParam, LPARAM lParam);
    
    // Member variables
    LONG m_cRef;
    ITfThreadMgr *m_pThreadMgr;
    TfClientId m_tfClientId;
    DWORD m_dwThreadMgrEventSinkCookie;
    ITfDocumentMgr *m_pDocMgrFocus;
    
    // KeyMagic engine
    EngineHandle *m_pEngine;
    std::wstring m_currentKeyboardPath;
    
    // Composition state
    ITfComposition *m_pComposition;
    BOOL m_fComposing;
    
    // Critical section for thread safety
    CRITICAL_SECTION m_cs;
    
    // Friend class
    friend class CEditSession;
};

// Edit session for TSF operations
class CEditSession : public ITfEditSession
{
public:
    enum class EditAction
    {
        UpdateComposition,
        CommitText,
        TerminateComposition
    };
    
    CEditSession(CKeyMagicTextService *pTextService, ITfContext *pContext, 
                 EditAction action, const std::wstring& textToCommit, 
                 const std::wstring& composingText);
    ~CEditSession();
    
    // IUnknown
    STDMETHODIMP QueryInterface(REFIID riid, void **ppvObject);
    STDMETHODIMP_(ULONG) AddRef();
    STDMETHODIMP_(ULONG) Release();
    
    // ITfEditSession
    STDMETHODIMP DoEditSession(TfEditCookie ec);
    
private:
    LONG m_cRef;
    CKeyMagicTextService *m_pTextService;
    ITfContext *m_pContext;
    EditAction m_action;
    std::wstring m_textToCommit;
    std::wstring m_composingText;
    
    void UpdateCompositionString(TfEditCookie ec);
    void CommitText(TfEditCookie ec);
    void TerminateComposition(TfEditCookie ec);
    void StartComposition(TfEditCookie ec);
    void ApplyDisplayAttributes(TfEditCookie ec, ITfRange *pRange);
};

#endif // KEYMAGIC_TEXT_SERVICE_H