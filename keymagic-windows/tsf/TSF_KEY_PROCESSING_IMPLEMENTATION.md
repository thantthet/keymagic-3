# TSF Key Processing Implementation Details

## Overview
This document details the key processing implementation in the KeyMagic TSF component, explaining how keyboard input flows through the system and gets transformed into text.

## Key Processing Flow

### 1. Key Event Entry Points

The TSF framework calls our key event handlers:
```cpp
STDAPI CKeyMagicTextService::OnKeyDown(ITfContext *pic, WPARAM wParam, LPARAM lParam, BOOL *pfEaten)
STDAPI CKeyMagicTextService::OnKeyUp(ITfContext *pic, WPARAM wParam, LPARAM lParam, BOOL *pfEaten)
```

### 2. Key Processing Pipeline

#### 2.1 Initial Key Filtering
```cpp
// System keys pass through
if (wParam == VK_SHIFT || wParam == VK_CONTROL || wParam == VK_MENU) {
    *pfEaten = FALSE;
    return S_OK;
}
```

#### 2.2 Engine Processing
The key is sent to the keymagic-core engine:
```cpp
// Get character from virtual key
BYTE keyboardState[256];
GetKeyboardState(keyboardState);
WCHAR unicodeChar[4] = {0};
int charCount = ToUnicode(wParam, scanCode, keyboardState, unicodeChar, 4, 0);

// Process with engine
KeyMagicKeyEvent km_event = {0};
km_event.vk_code = static_cast<uint8_t>(wParam);
km_event.character = (charCount > 0) ? unicodeChar[0] : 0;
km_event.shift_pressed = (GetKeyState(VK_SHIFT) & 0x8000) != 0;
// ... other modifiers

KeyMagicOutput output = {0};
keymagic_engine_process_key(engine, &km_event, &output);
```

#### 2.3 Output Handling
Based on the engine output, different actions are taken:

**No Action**: Key passes through
```cpp
if (output.action_type == KeyMagicActionType_None) {
    *pfEaten = output.is_processed;
    return S_OK;
}
```

**Insert Text**: Add to composition
```cpp
case KeyMagicActionType_Insert:
    composingText = ConvertToWString(output.text_to_insert);
    shouldUpdate = true;
    break;
```

**Backspace + Insert**: Delete and replace
```cpp
case KeyMagicActionType_BackspaceDeleteAndInsert:
    // Backspace count times, then insert new text
    composingText = ConvertToWString(output.text_to_insert);
    shouldUpdate = true;
    break;
```

### 3. Commit Triggers

Special keys trigger composition commits:

#### Space Key
```cpp
case VK_SPACE:
    if (output.is_processed) {
        if (!composingText.empty() && composingText.back() == L' ') {
            shouldCommit = true;
            textToCommit = composingText;
        }
    } else {
        shouldCommit = true;
        textToCommit = composingText + L" ";
    }
    break;
```

#### Enter, Tab, Escape
- **Enter**: Commits and sends Enter key
- **Tab**: Commits and sends Tab key  
- **Escape**: Clears composition without committing

### 4. Edit Session Execution

All text modifications happen within edit sessions:

```cpp
if (shouldCommit || shouldUpdate) {
    CEditSession* pEditSession = new CEditSession(this, pic, 
        shouldCommit ? textToCommit : composingText, 
        shouldCommit);
    
    HRESULT hr;
    pic->RequestEditSession(tfClientId, pEditSession, 
        TF_ES_SYNC | TF_ES_READWRITE, &hr);
}
```

### 5. Composition Management

#### Creating Composition
```cpp
void UpdateComposition(ITfContext *pic, TfEditCookie ec, const std::wstring& text) {
    if (!composition) {
        // Start new composition
        ITfInsertAtSelection *pInsertAtSelection;
        pic->QueryInterface(IID_ITfInsertAtSelection, (void **)&pInsertAtSelection);
        pInsertAtSelection->InsertTextAtSelection(ec, TF_IAS_QUERYONLY, 
            NULL, 0, &range);
        
        ITfContextComposition *pContextComposition;
        pic->QueryInterface(IID_ITfContextComposition, (void **)&pContextComposition);
        pContextComposition->StartComposition(ec, range, 
            (ITfCompositionSink*)textService, &composition);
    }
}
```

#### Updating Composition Text
```cpp
// Set the display text
ITfRange *pRange;
composition->GetRange(&pRange);
pRange->SetText(ec, 0, text.c_str(), text.length());
```

#### Committing Text
```cpp
void CommitText(ITfContext *pic, TfEditCookie ec, const std::wstring& text) {
    if (composition) {
        ITfRange *pRange;
        composition->GetRange(&pRange);
        pRange->SetText(ec, 0, text.c_str(), text.length());
        composition->EndComposition(ec);
        composition->Release();
        composition = nullptr;
    }
}
```

## State Management

### Engine State
- Created once per text service instance
- Persists across key events
- Maintains composing buffer internally

### Composition State
- Track active composition object
- Clean up on commit or cancel
- Handle focus changes properly

### Thread Safety
- CRITICAL_SECTION protects engine access
- All modifications in edit sessions
- Proper COM reference counting

## Integration Points

### FFI Functions Used
- `keymagic_engine_new()` - Create engine
- `keymagic_engine_load_keyboard()` - Load keyboard
- `keymagic_engine_process_key()` - Process input
- `keymagic_engine_reset()` - Clear state
- `keymagic_engine_free()` - Cleanup

### Registry Integration
- Load default keyboard path from registry
- Path: `HKEY_CURRENT_USER\Software\KeyMagic\Settings\DefaultKeyboard`

## Error Handling

1. **Engine Creation Failure**: Disable text service
2. **Keyboard Load Failure**: Continue without keyboard
3. **Processing Errors**: Pass key through unmodified
4. **Edit Session Failure**: Log and continue

## Performance Considerations

1. **Synchronous Processing**: Using TF_ES_SYNC for immediate feedback
2. **Minimal Allocations**: Reuse buffers where possible
3. **Early Exit**: Filter system keys before processing
4. **State Caching**: Avoid redundant engine calls

## Testing Considerations

### Key Scenarios to Test
1. Basic typing with composition
2. Commit triggers (space, enter, tab)
3. Cancel with escape
4. Focus switching mid-composition
5. Rapid typing
6. Special characters and modifiers

### Edge Cases
1. Empty composition commits
2. Very long composition strings
3. Invalid keyboard files
4. Missing registry entries
5. Multiple simultaneous contexts

## Future Enhancements

1. **Display Attributes**: Underline styles for composition
2. **Candidate Window**: Show conversion options
3. **Reconversion**: Edit already committed text
4. **Context-Aware**: Different behavior based on application

This implementation provides a solid foundation for KeyMagic text input on Windows, with proper TSF integration and robust error handling.