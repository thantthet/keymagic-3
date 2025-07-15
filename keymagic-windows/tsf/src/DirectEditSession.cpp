#include "DirectEditSession.h"
#include "KeyMagicTextService.h"
#include "Debug.h"
#include "../include/keymagic_ffi.h"

extern std::wstring ConvertUtf8ToUtf16(const std::string &utf8);
extern std::string ConvertUtf16ToUtf8(const std::wstring &utf16);

// CDirectEditSession implementation
CDirectEditSession::CDirectEditSession(CKeyMagicTextService *pTextService, ITfContext *pContext, EditAction action)
{
    m_cRef = 1;
    m_pTextService = pTextService;
    m_pTextService->AddRef();
    m_pContext = pContext;
    m_pContext->AddRef();
    m_action = action;
    m_wParam = 0;
    m_lParam = 0;
    m_pfEaten = nullptr;
    m_deleteCount = 0;
}

CDirectEditSession::~CDirectEditSession()
{
    m_pContext->Release();
    m_pTextService->Release();
}

// IUnknown
STDAPI CDirectEditSession::QueryInterface(REFIID riid, void **ppvObject)
{
    if (ppvObject == nullptr)
        return E_INVALIDARG;

    *ppvObject = nullptr;

    if (IsEqualIID(riid, IID_IUnknown) || IsEqualIID(riid, IID_ITfEditSession))
    {
        *ppvObject = (ITfEditSession*)this;
    }

    if (*ppvObject)
    {
        AddRef();
        return S_OK;
    }

    return E_NOINTERFACE;
}

STDAPI_(ULONG) CDirectEditSession::AddRef()
{
    return InterlockedIncrement(&m_cRef);
}

STDAPI_(ULONG) CDirectEditSession::Release()
{
    LONG cr = InterlockedDecrement(&m_cRef);

    if (cr == 0)
    {
        delete this;
    }

    return cr;
}

// ITfEditSession
STDAPI CDirectEditSession::DoEditSession(TfEditCookie ec)
{
    switch (m_action)
    {
        case EditAction::ProcessKey:
        {
            // First sync engine with document
            char* engineComposing = keymagic_engine_get_composition(m_pTextService->GetEngineHandle());
            if (engineComposing)
            {
                std::string engineText(engineComposing);
                keymagic_free_string(engineComposing);
                
                // Read document suffix
                std::wstring documentText;
                int compareLength = static_cast<int>(engineText.length());
                if (compareLength > 0)
                {
                    m_pTextService->ReadDocumentSuffix(m_pContext, ec, compareLength, documentText);
                    
                    // Compare texts
                    std::string docUtf8 = ConvertUtf16ToUtf8(documentText);
                    
                    DEBUG_LOG(L"Comparing engine text with document");
                    DEBUG_LOG(L"Engine: \"" + ConvertUtf8ToUtf16(engineText) + L"\"");
                    DEBUG_LOG(L"Document: \"" + documentText + L"\"");
                    
                    if (docUtf8 != engineText)
                    {
                        DEBUG_LOG(L"Text mismatch - resetting engine");
                        // Texts don't match, reset engine
                        m_pTextService->ResetEngine();
                    }
                }
            }
            
            // Process the key
            m_pTextService->ProcessKeyInput(m_pContext, m_wParam, m_lParam, m_pfEaten);
            break;
        }
        
        case EditAction::SyncEngine:
        {
            m_pTextService->SyncEngineWithDocument(m_pContext, ec);
            break;
        }
        
        case EditAction::DeleteAndInsert:
        {
            // Delete characters if needed
            if (m_deleteCount > 0)
            {
                DEBUG_LOG(L"Deleting " + std::to_wstring(m_deleteCount) + L" characters");
                m_pTextService->DeleteCharsBeforeCursor(m_pContext, ec, m_deleteCount);
            }
            
            // Insert new text
            if (!m_insertText.empty())
            {
                DEBUG_LOG(L"Inserting text: \"" + m_insertText + L"\"");
                m_pTextService->InsertTextAtCursor(m_pContext, ec, m_insertText);
            }
            break;
        }
    }
    
    return S_OK;
}

void CDirectEditSession::SetKeyData(WPARAM wParam, LPARAM lParam, BOOL *pfEaten)
{
    m_wParam = wParam;
    m_lParam = lParam;
    m_pfEaten = pfEaten;
}

void CDirectEditSession::SetTextAction(int deleteCount, const std::wstring &insertText)
{
    m_deleteCount = deleteCount;
    m_insertText = insertText;
}