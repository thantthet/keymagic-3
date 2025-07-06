#ifndef DISPLAY_ATTRIBUTE_H
#define DISPLAY_ATTRIBUTE_H

#include <windows.h>
#include <msctf.h>
#include <string>

// Forward declarations
class CKeyMagicTextService;

// Display attribute information for KeyMagic composition
class CKeyMagicDisplayAttributeInfo : public ITfDisplayAttributeInfo
{
public:
    CKeyMagicDisplayAttributeInfo(const GUID &guid, const TF_DISPLAYATTRIBUTE &attribute, 
                                 const std::wstring &description, const std::wstring &owner);
    ~CKeyMagicDisplayAttributeInfo();
    
    // IUnknown
    STDMETHODIMP QueryInterface(REFIID riid, void **ppvObject);
    STDMETHODIMP_(ULONG) AddRef();
    STDMETHODIMP_(ULONG) Release();
    
    // ITfDisplayAttributeInfo
    STDMETHODIMP GetGUID(GUID *pguid);
    STDMETHODIMP GetDescription(BSTR *pbstrDesc);
    STDMETHODIMP GetAttributeInfo(TF_DISPLAYATTRIBUTE *ptfDisplayAttr);
    STDMETHODIMP SetAttributeInfo(const TF_DISPLAYATTRIBUTE *ptfDisplayAttr);
    STDMETHODIMP Reset();
    
private:
    LONG m_cRef;
    GUID m_guid;
    TF_DISPLAYATTRIBUTE m_attribute;
    std::wstring m_description;
    std::wstring m_owner;
};

// Enumerator for display attribute info objects
class CEnumDisplayAttributeInfo : public IEnumTfDisplayAttributeInfo
{
public:
    CEnumDisplayAttributeInfo();
    ~CEnumDisplayAttributeInfo();
    
    // Initialize with array of display attribute info objects
    HRESULT Initialize(ITfDisplayAttributeInfo **ppDisplayAttributeInfo, ULONG count);
    
    // IUnknown
    STDMETHODIMP QueryInterface(REFIID riid, void **ppvObject);
    STDMETHODIMP_(ULONG) AddRef();
    STDMETHODIMP_(ULONG) Release();
    
    // IEnumTfDisplayAttributeInfo
    STDMETHODIMP Clone(IEnumTfDisplayAttributeInfo **ppEnum);
    STDMETHODIMP Next(ULONG ulCount, ITfDisplayAttributeInfo **ppInfo, ULONG *pcFetched);
    STDMETHODIMP Reset();
    STDMETHODIMP Skip(ULONG ulCount);
    
private:
    LONG m_cRef;
    ITfDisplayAttributeInfo **m_ppDisplayAttributeInfo;
    ULONG m_count;
    ULONG m_index;
};

// Helper function to create standard display attribute
TF_DISPLAYATTRIBUTE CreateInputDisplayAttribute();

#endif // DISPLAY_ATTRIBUTE_H