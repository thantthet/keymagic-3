#ifndef KEYMAGIC_TEXT_SERVICE_H
#define KEYMAGIC_TEXT_SERVICE_H

#include <windows.h>
#include <msctf.h>
#include <olectl.h>
#include <string>
#include "keymagic_ffi.h"

// Forward declarations
class CKeyMagicTextService;

// Class factory for creating text service instances
class CClassFactory : public IClassFactory {
public:
    // IUnknown
    STDMETHODIMP QueryInterface(REFIID riid, void** ppvObj);
    STDMETHODIMP_(ULONG) AddRef();
    STDMETHODIMP_(ULONG) Release();

    // IClassFactory
    STDMETHODIMP CreateInstance(IUnknown* pUnkOuter, REFIID riid, void** ppvObj);
    STDMETHODIMP LockServer(BOOL fLock);

    CClassFactory();
    ~CClassFactory();

private:
    LONG m_cRef;
};

// Main text service class implementing TSF interfaces
class CKeyMagicTextService : public ITfTextInputProcessor,
                             public ITfThreadMgrEventSink,
                             public ITfKeyEventSink {
public:
    // IUnknown
    STDMETHODIMP QueryInterface(REFIID riid, void** ppvObj);
    STDMETHODIMP_(ULONG) AddRef();
    STDMETHODIMP_(ULONG) Release();

    // ITfTextInputProcessor
    STDMETHODIMP Activate(ITfThreadMgr* ptim, TfClientId tid);
    STDMETHODIMP Deactivate();

    // ITfThreadMgrEventSink
    STDMETHODIMP OnInitDocumentMgr(ITfDocumentMgr* pdim);
    STDMETHODIMP OnUninitDocumentMgr(ITfDocumentMgr* pdim);
    STDMETHODIMP OnSetFocus(ITfDocumentMgr* pdimFocus, ITfDocumentMgr* pdimPrevFocus);
    STDMETHODIMP OnPushContext(ITfContext* pic);
    STDMETHODIMP OnPopContext(ITfContext* pic);

    // ITfKeyEventSink
    STDMETHODIMP OnSetFocus(BOOL fForeground);
    STDMETHODIMP OnTestKeyDown(ITfContext* pic, WPARAM wParam, LPARAM lParam, BOOL* pfEaten);
    STDMETHODIMP OnKeyDown(ITfContext* pic, WPARAM wParam, LPARAM lParam, BOOL* pfEaten);
    STDMETHODIMP OnTestKeyUp(ITfContext* pic, WPARAM wParam, LPARAM lParam, BOOL* pfEaten);
    STDMETHODIMP OnKeyUp(ITfContext* pic, WPARAM wParam, LPARAM lParam, BOOL* pfEaten);
    STDMETHODIMP OnPreservedKey(ITfContext* pic, REFGUID rguid, BOOL* pfEaten);

    CKeyMagicTextService();
    ~CKeyMagicTextService();

private:
    // Helper methods
    HRESULT _InitThreadMgrEventSink();
    HRESULT _UninitThreadMgrEventSink();
    HRESULT _InitKeyEventSink();
    HRESULT _UninitKeyEventSink();
    
    HRESULT _StartComposition(ITfContext* pContext);
    HRESULT _EndComposition(ITfContext* pContext);
    HRESULT _UpdateComposition(ITfContext* pContext, const std::wstring& text);
    HRESULT _CommitText(ITfContext* pContext, const std::wstring& text);

    // Convert UTF-8 to UTF-16
    std::wstring _Utf8ToUtf16(const char* utf8);
    
    // Member variables
    LONG m_cRef;
    ITfThreadMgr* m_pThreadMgr;
    TfClientId m_tfClientId;
    ITfContext* m_pTextEditSink;
    ITfComposition* m_pComposition;
    DWORD m_dwThreadMgrEventSinkCookie;
    DWORD m_dwKeyEventSinkCookie;
    
    // KeyMagic engine handle
    EngineHandle* m_pEngine;
    
    // Composition state
    bool m_fComposing;
    std::wstring m_compositionString;
};

// Global variables
extern HINSTANCE g_hInst;
extern LONG g_cRefDll;

// CLSID for the text service
// {12345678-1234-1234-1234-123456789ABC}
DEFINE_GUID(CLSID_KeyMagicTextService, 
    0x12345678, 0x1234, 0x1234, 0x12, 0x34, 0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC);

// Profile GUID
// {87654321-4321-4321-4321-CBA987654321}
DEFINE_GUID(GUID_KeyMagicProfile,
    0x87654321, 0x4321, 0x4321, 0x43, 0x21, 0xCB, 0xA9, 0x87, 0x65, 0x43, 0x21);

#endif // KEYMAGIC_TEXT_SERVICE_H