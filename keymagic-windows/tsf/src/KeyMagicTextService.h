#ifndef KEYMAGIC_TEXT_SERVICE_H
#define KEYMAGIC_TEXT_SERVICE_H

#include <windows.h>
#include <msctf.h>
#include <string>
#include <memory>
#include "../include/keymagic_ffi.h"
#include "Composition.h"
#include "DisplayAttribute.h"

// Forward declaration
class CKeyMagicTextService;
class CCompositionManager;

class CKeyMagicTextService : public ITfTextInputProcessor,
                            public ITfThreadMgrEventSink,
                            public ITfKeyEventSink,
                            public ITfTextEditSink,
                            public ITfMouseSink,
                            public ITfDisplayAttributeProvider
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
    
    // ITfTextEditSink
    STDMETHODIMP OnEndEdit(ITfContext *pic, TfEditCookie ecReadOnly, ITfEditRecord *pEditRecord);
    
    // ITfMouseSink
    STDMETHODIMP OnMouseEvent(ULONG uEdge, ULONG uQuadrant, DWORD dwBtnStatus, BOOL *pfEaten);

    // ITfDisplayAttributeProvider
    STDMETHODIMP EnumDisplayAttributeInfo(IEnumTfDisplayAttributeInfo **ppEnum);
    STDMETHODIMP GetDisplayAttributeInfo(REFGUID guidInfo, ITfDisplayAttributeInfo **ppInfo);

    // Public methods
    EngineHandle* GetEngineHandle() { return m_pEngine; }

private:
    // Helper methods
    BOOL InitializeEngine();
    void UninitializeEngine();
    HKEY OpenSettingsKey(REGSAM samDesired);
    BOOL LoadKeyboard(const std::wstring& km2Path);
    BOOL LoadKeyboardByID(const std::wstring& keyboardId);
    void CheckAndReloadKeyboard();
    void ProcessKeyInput(ITfContext *pic, WPARAM wParam, LPARAM lParam, BOOL *pfEaten);
    void ProcessKeyWithSendInput(ITfContext *pic, WPARAM wParam, LPARAM lParam, BOOL *pfEaten);
    void ResetEngine();
    void SyncEngineWithDocument(ITfContext *pic, TfEditCookie ec);
    
    // Application compatibility detection
    BOOL TestCompositionSupport(ITfContext *pContext);
    
    // Text manipulation
    HRESULT ReadDocumentSuffix(ITfContext *pic, TfEditCookie ec, int maxChars, std::wstring &text);
    HRESULT DeleteCharsBeforeCursor(ITfContext *pic, TfEditCookie ec, int count);
    HRESULT InsertTextAtCursor(ITfContext *pic, TfEditCookie ec, const std::wstring &text);
    HRESULT ExecuteTextAction(ITfContext *pic, const ProcessKeyOutput &output);
    
    // Key translation
    char MapVirtualKeyToChar(WPARAM wParam, LPARAM lParam);
    bool IsPrintableAscii(char c);
    
    // Sink management
    HRESULT InitTextEditSink();
    HRESULT UninitTextEditSink();
    HRESULT InitMouseSink(); 
    HRESULT UninitMouseSink();
    
    // Display attribute management
    HRESULT RegisterDisplayAttributeProvider();
    HRESULT UnregisterDisplayAttributeProvider();
    HRESULT CreateDisplayAttributeInfo();
    
    // Registry monitoring
    HRESULT StartRegistryMonitoring();
    HRESULT StopRegistryMonitoring();
    static DWORD WINAPI RegistryMonitorThread(LPVOID lpParam);
    void RegistryMonitorLoop();
    
    // Member variables
    LONG m_cRef;
    ITfThreadMgr *m_pThreadMgr;
    TfClientId m_tfClientId;
    DWORD m_dwThreadMgrEventSinkCookie;
    DWORD m_dwTextEditSinkCookie;
    DWORD m_dwMouseSinkCookie;
    ITfDocumentMgr *m_pDocMgrFocus;
    ITfContext *m_pTextEditContext;
    
    // KeyMagic engine
    EngineHandle *m_pEngine;
    std::wstring m_currentKeyboardPath;
    std::wstring m_currentKeyboardId;
    bool m_tsfEnabled;
    
    // Critical section for thread safety
    CRITICAL_SECTION m_cs;
    
    // Registry monitoring
    HANDLE m_hRegistryThread;
    HANDLE m_hRegistryStopEvent;
    HKEY m_hRegistryKey;
    HANDLE m_hRegistryChangeEvent;
    
    // Composition manager
    CCompositionManager *m_pCompositionMgr;
    
    // Application compatibility
    BOOL m_supportsComposition;
    
    // Display attributes
    ITfDisplayAttributeInfo **m_ppDisplayAttributeInfo;
    ULONG m_displayAttributeInfoCount;
    TfGuidAtom m_inputDisplayAttributeAtom;
    
    // SendInput signature
    static const ULONG_PTR KEYMAGIC_EXTRAINFO_SIGNATURE = 0x4B4D5453; // "KMTS" in hex
    
    // Processing state
    bool m_isProcessingKey;
    DWORD m_lastSendInputTime;
    
    // Friend classes
    friend class CDirectEditSession;
    friend class CCompositionEditSession;
    friend class CCompositionManager;
};

// Edit session for direct text manipulation (no composition)
class CDirectEditSession : public ITfEditSession
{
public:
    enum class EditAction
    {
        ProcessKey,
        SyncEngine,
        DeleteAndInsert
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
    void SetTextAction(int deleteCount, const std::wstring &insertText);
    
private:
    LONG m_cRef;
    CKeyMagicTextService *m_pTextService;
    ITfContext *m_pContext;
    EditAction m_action;
    
    // Action-specific data
    WPARAM m_wParam;
    LPARAM m_lParam;
    BOOL *m_pfEaten;
    int m_deleteCount;
    std::wstring m_insertText;
};

#endif // KEYMAGIC_TEXT_SERVICE_H