#include "DisplayAttribute.h"
#include "KeyMagicGuids.h"
#include "Debug.h"

// CKeyMagicDisplayAttributeInfo implementation
CKeyMagicDisplayAttributeInfo::CKeyMagicDisplayAttributeInfo(const GUID &guid, 
                                                           const TF_DISPLAYATTRIBUTE &attribute,
                                                           const std::wstring &description, 
                                                           const std::wstring &owner)
{
    m_cRef = 1;
    m_guid = guid;
    m_attribute = attribute;
    m_description = description;
    m_owner = owner;
}

CKeyMagicDisplayAttributeInfo::~CKeyMagicDisplayAttributeInfo()
{
}

// IUnknown
STDAPI CKeyMagicDisplayAttributeInfo::QueryInterface(REFIID riid, void **ppvObject)
{
    if (ppvObject == nullptr)
        return E_INVALIDARG;

    *ppvObject = nullptr;

    if (IsEqualIID(riid, IID_IUnknown) || IsEqualIID(riid, IID_ITfDisplayAttributeInfo))
    {
        *ppvObject = static_cast<ITfDisplayAttributeInfo*>(this);
    }

    if (*ppvObject)
    {
        AddRef();
        return S_OK;
    }

    return E_NOINTERFACE;
}

STDAPI_(ULONG) CKeyMagicDisplayAttributeInfo::AddRef()
{
    return InterlockedIncrement(&m_cRef);
}

STDAPI_(ULONG) CKeyMagicDisplayAttributeInfo::Release()
{
    LONG cRef = InterlockedDecrement(&m_cRef);
    if (cRef == 0)
    {
        delete this;
    }
    return cRef;
}

// ITfDisplayAttributeInfo
STDAPI CKeyMagicDisplayAttributeInfo::GetGUID(GUID *pguid)
{
    if (pguid == nullptr)
        return E_INVALIDARG;
        
    *pguid = m_guid;
    return S_OK;
}

STDAPI CKeyMagicDisplayAttributeInfo::GetDescription(BSTR *pbstrDesc)
{
    if (pbstrDesc == nullptr)
        return E_INVALIDARG;
        
    *pbstrDesc = SysAllocString(m_description.c_str());
    return (*pbstrDesc != nullptr) ? S_OK : E_OUTOFMEMORY;
}

STDAPI CKeyMagicDisplayAttributeInfo::GetAttributeInfo(TF_DISPLAYATTRIBUTE *ptfDisplayAttr)
{
    if (ptfDisplayAttr == nullptr)
        return E_INVALIDARG;
        
    *ptfDisplayAttr = m_attribute;
    return S_OK;
}

STDAPI CKeyMagicDisplayAttributeInfo::SetAttributeInfo(const TF_DISPLAYATTRIBUTE *ptfDisplayAttr)
{
    // KeyMagic display attributes are read-only
    return E_NOTIMPL;
}

STDAPI CKeyMagicDisplayAttributeInfo::Reset()
{
    // Reset to default - no action needed for our implementation
    return S_OK;
}

// CEnumDisplayAttributeInfo implementation
CEnumDisplayAttributeInfo::CEnumDisplayAttributeInfo()
{
    m_cRef = 1;
    m_ppDisplayAttributeInfo = nullptr;
    m_count = 0;
    m_index = 0;
}

CEnumDisplayAttributeInfo::~CEnumDisplayAttributeInfo()
{
    if (m_ppDisplayAttributeInfo)
    {
        for (ULONG i = 0; i < m_count; i++)
        {
            if (m_ppDisplayAttributeInfo[i])
            {
                m_ppDisplayAttributeInfo[i]->Release();
            }
        }
        delete[] m_ppDisplayAttributeInfo;
    }
}

HRESULT CEnumDisplayAttributeInfo::Initialize(ITfDisplayAttributeInfo **ppDisplayAttributeInfo, ULONG count)
{
    if (ppDisplayAttributeInfo == nullptr || count == 0)
        return E_INVALIDARG;
        
    // Clean up any existing data
    if (m_ppDisplayAttributeInfo)
    {
        for (ULONG i = 0; i < m_count; i++)
        {
            if (m_ppDisplayAttributeInfo[i])
            {
                m_ppDisplayAttributeInfo[i]->Release();
            }
        }
        delete[] m_ppDisplayAttributeInfo;
    }
    
    // Allocate new array
    m_ppDisplayAttributeInfo = new ITfDisplayAttributeInfo*[count];
    if (!m_ppDisplayAttributeInfo)
        return E_OUTOFMEMORY;
        
    // Copy and AddRef each object
    m_count = count;
    for (ULONG i = 0; i < count; i++)
    {
        m_ppDisplayAttributeInfo[i] = ppDisplayAttributeInfo[i];
        m_ppDisplayAttributeInfo[i]->AddRef();
    }
    
    m_index = 0;
    return S_OK;
}

// IUnknown
STDAPI CEnumDisplayAttributeInfo::QueryInterface(REFIID riid, void **ppvObject)
{
    if (ppvObject == nullptr)
        return E_INVALIDARG;

    *ppvObject = nullptr;

    if (IsEqualIID(riid, IID_IUnknown) || IsEqualIID(riid, IID_IEnumTfDisplayAttributeInfo))
    {
        *ppvObject = static_cast<IEnumTfDisplayAttributeInfo*>(this);
    }

    if (*ppvObject)
    {
        AddRef();
        return S_OK;
    }

    return E_NOINTERFACE;
}

STDAPI_(ULONG) CEnumDisplayAttributeInfo::AddRef()
{
    return InterlockedIncrement(&m_cRef);
}

STDAPI_(ULONG) CEnumDisplayAttributeInfo::Release()
{
    LONG cRef = InterlockedDecrement(&m_cRef);
    if (cRef == 0)
    {
        delete this;
    }
    return cRef;
}

// IEnumTfDisplayAttributeInfo
STDAPI CEnumDisplayAttributeInfo::Clone(IEnumTfDisplayAttributeInfo **ppEnum)
{
    if (ppEnum == nullptr)
        return E_INVALIDARG;
        
    CEnumDisplayAttributeInfo *pClone = new CEnumDisplayAttributeInfo();
    if (!pClone)
        return E_OUTOFMEMORY;
        
    HRESULT hr = pClone->Initialize(m_ppDisplayAttributeInfo, m_count);
    if (FAILED(hr))
    {
        pClone->Release();
        return hr;
    }
    
    pClone->m_index = m_index;
    *ppEnum = pClone;
    return S_OK;
}

STDAPI CEnumDisplayAttributeInfo::Next(ULONG ulCount, ITfDisplayAttributeInfo **ppInfo, ULONG *pcFetched)
{
    if (ppInfo == nullptr)
        return E_INVALIDARG;
        
    ULONG fetched = 0;
    
    while (fetched < ulCount && m_index < m_count)
    {
        ppInfo[fetched] = m_ppDisplayAttributeInfo[m_index];
        ppInfo[fetched]->AddRef();
        fetched++;
        m_index++;
    }
    
    if (pcFetched)
        *pcFetched = fetched;
        
    return (fetched == ulCount) ? S_OK : S_FALSE;
}

STDAPI CEnumDisplayAttributeInfo::Reset()
{
    m_index = 0;
    return S_OK;
}

STDAPI CEnumDisplayAttributeInfo::Skip(ULONG ulCount)
{
    if (m_index + ulCount > m_count)
    {
        m_index = m_count;
        return S_FALSE;
    }
    
    m_index += ulCount;
    return S_OK;
}

// Helper functions to create standard display attributes
TF_DISPLAYATTRIBUTE CreateInputDisplayAttribute()
{
    TF_DISPLAYATTRIBUTE attr = {};
    
    // Text color - use default
    attr.crText.type = TF_CT_NONE;
    
    // Background color - use default
    attr.crBk.type = TF_CT_NONE;
    
    // Underline style - solid line
    attr.lsStyle = TF_LS_SOLID;
    attr.fBoldLine = FALSE;
    
    // Underline color - use system window text color
    attr.crLine.type = TF_CT_SYSCOLOR;
    attr.crLine.nIndex = COLOR_WINDOWTEXT;
    
    // Attribute type - input composition
    attr.bAttr = TF_ATTR_INPUT;
    
    return attr;
}

TF_DISPLAYATTRIBUTE CreateConvertedDisplayAttribute()
{
    TF_DISPLAYATTRIBUTE attr = {};
    
    // Text color - use default
    attr.crText.type = TF_CT_NONE;
    
    // Background color - use default
    attr.crBk.type = TF_CT_NONE;
    
    // Underline style - solid line, bold
    attr.lsStyle = TF_LS_SOLID;
    attr.fBoldLine = TRUE;
    
    // Underline color - use system window text color
    attr.crLine.type = TF_CT_SYSCOLOR;
    attr.crLine.nIndex = COLOR_WINDOWTEXT;
    
    // Attribute type - converted composition
    attr.bAttr = TF_ATTR_CONVERTED;
    
    return attr;
}