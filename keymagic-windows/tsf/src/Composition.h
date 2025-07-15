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
    
    // Get current composition text
    HRESULT GetCompositionText(TfEditCookie ec, std::wstring &text);
    
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


#endif // COMPOSITION_H