# Fix for Duplicate Text Insertion Issue

## Problem
When committing text (e.g., pressing space), the text was being inserted twice:
1. Once when the composition was terminated (TSF automatically commits composition text)
2. Once when we explicitly inserted the text

## Root Cause
The original implementation was:
1. Terminating the composition (which commits the current composition text)
2. Then inserting the text again manually

This caused duplicate text insertion.

## Solution
Updated the `CommitText` method to properly handle composition finalization:

```cpp
void CEditSession::CommitText(TfEditCookie ec)
{
    if (m_pTextService->m_pComposition)
    {
        // Update the composition range with final text
        ITfRange *pRange = nullptr;
        if (SUCCEEDED(m_pTextService->m_pComposition->GetRange(&pRange)))
        {
            // Set the final text in the composition range
            pRange->SetText(ec, 0, m_textToCommit.c_str(), 
                          static_cast<LONG>(m_textToCommit.length()));
            pRange->Release();
        }
        
        // End composition - this commits the text
        m_pTextService->m_pComposition->EndComposition(ec);
        m_pTextService->m_pComposition->Release();
        m_pTextService->m_pComposition = nullptr;
        m_pTextService->m_fComposing = FALSE;
    }
    else
    {
        // No composition - insert directly
        // ... existing direct insertion code ...
    }
}
```

## Key Changes

1. **Set final text before ending composition**: We update the composition range with the text we want to commit
2. **Let TSF handle the commit**: When we call `EndComposition`, TSF commits whatever is in the composition range
3. **Remove redundant TerminateComposition call**: In `DoEditSession`, we no longer call `TerminateComposition` after `CommitText`

## Testing
After applying this fix:
- Type some text (e.g., "ka" → "က")
- Press space
- Should see only one instance of the text committed

## Files Modified
- `KeyMagicTextService.cpp`:
  - `CEditSession::CommitText()` - Rewritten to handle composition properly
  - `CEditSession::DoEditSession()` - Removed redundant TerminateComposition call