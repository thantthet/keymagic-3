# KeyMagic Key Processing Engine Logic

This document outlines the complete logic of the key processing engine in KeyMagic, including critical implementation details discovered through actual engine development.

## Core Components

The key processing engine is composed of the following main components:

- **Engine (`engine.rs`)**: The central component that orchestrates the entire key processing workflow.
- **State (`state.rs`)**: Manages the state of the input context and active states.
- **Input (`input.rs`)**: Represents a keyboard event with key codes and modifiers.
- **Matcher (`matcher.rs`)**: Matches input against predefined rules using segmented pattern matching.
- **Pattern (`pattern.rs`)**: Represents patterns with proper segmentation and reference handling.
- **Output (`output.rs`)**: Represents the result of key processing with precise action instructions.

## Architecture Overview

### Internal Encoding Strategy

**Implementation Requirement**: The engine MUST use UTF-16 encoding internally for all text processing to properly handle Unicode characters, especially complex scripts like Myanmar.

**Encoding Boundaries**:
- **Input**: Convert UTF-8 to UTF-16 at API entry points
- **Processing**: All pattern matching, captures, and text operations in UTF-16 (`std::u16string`)
- **Output**: Convert UTF-16 to UTF-8 at API exit points
- **Storage**: Composing buffer maintained in UTF-16

This ensures proper handling of multi-byte Unicode characters throughout the engine pipeline.

### Rule Segmentation Architecture

**Fundamental Requirement**: Rules cannot be processed as simple opcode sequences. They must be pre-processed into logical segments to handle references (`$1`, `$2`, `$3`) correctly.

**Key Insight**: Opcode indices ≠ Segment indices. Each rule's left-hand side (LHS) pattern gets broken into sequential segments, and right-hand side (RHS) references refer to these segment indices.

#### Segment Types

```cpp
enum class SegmentType {
    String,           // Literal text patterns: "ka", "abc"
    Variable,         // Variable references: $consonants, $vowels  
    AnyOfVariable,    // Wildcard patterns: $consonants[*]
    NotAnyOfVariable, // Exclusion patterns: $consonants[^]
    Any,              // ANY keyword (ASCII printable chars only)
    VirtualKey,       // Virtual key patterns: <VK_KEY_A>
    State,            // State conditions: ('my_state')
    Reference         // Back-references: $1, $2, $3
};
```

#### Segmentation Process

Rules are preprocessed using a `segmentateOpcodes()` function that:

1. **Parses opcodes sequentially** into logical segments
2. **Assigns 1-based indices** to each segment
3. **Handles modifier flags** (ANYOF, NANYOF) properly
4. **Groups related opcodes** (e.g., opVARIABLE + opMODIFIER = single segment)

**Example Segmentation**:
```kms
// KMS Rule: $consonants[*] + "a" + $vowels[^] => $1 + $vowels[$3]
// LHS Segments:
//   Segment 1: $consonants[*]     (AnyOfVariable)
//   Segment 2: "a"                (String) 
//   Segment 3: $vowels[^]         (NotAnyOfVariable)
// RHS References:
//   $1 refers to Segment 1 capture
//   $3 refers to Segment 3 capture position
```

## Processing Flow

### 1. Key Event Reception

When a key is pressed, the `process_key` method receives:
- **Virtual Key Code**: Platform-specific key identifier
- **Character**: Unicode character produced by the key
- **Modifiers**: Ctrl, Alt, Shift, etc. state flags

### 2. State Management

The engine maintains two types of state:

#### Persistent Composing Buffer
- **Never automatically cleared** during normal processing
- **Accumulates all input** across key events
- **Only clears when**:
  - Engine explicitly reset via `reset()`
  - Composing text explicitly set via `set_composing_text()`
  - A rule produces empty output (NULL)

#### Transient Active States
- **Active for next key event** when state appears in rule output
- **Cleared after each key event** unless explicitly re-activated
- **1-based integer IDs** assigned during compilation

### 3. Rule Matching with Suffix-Based Pattern Processing

**Important**: Rule matching operates exclusively on suffixes of the composing text, not arbitrary substrings.

#### Suffix Matching Logic

```cpp
// Rule can only match if pattern appears at END of composing text
bool canMatch = composingText.ends_with(patternText);

// Examples:
// Composing: "xyzabc", Pattern: "abc" → MATCH (suffix)
// Composing: "abcxyz", Pattern: "abc" → NO MATCH (not suffix)
```

#### Sequential Segment Processing

Pattern matching processes segments **right-to-left** (from end of composing text):

1. **Calculate total expected pattern length** from all segments
2. **Extract suffix** of that length from composing text
3. **Process each segment** from rightmost to leftmost:
   - Extract expected portion based on segment type and length
   - Validate match according to segment rules
   - Record capture with segment index and position data
4. **Pattern succeeds** only if ALL segments match

#### Pattern Length Calculation

**Implementation Detail**: Length calculation varies by segment type:

```cpp
size_t calculateSegmentLength(const Segment& segment) {
    switch (segment.type) {
        case SegmentType::String:
            return utf16_length(segment.stringValue);
        case SegmentType::AnyOfVariable:
        case SegmentType::NotAnyOfVariable:
            return 1;  // Wildcards match exactly 1 character
        case SegmentType::Variable:
            return utf16_length(getVariableContent(segment.variableIndex));
        case SegmentType::VirtualKey:
        case SegmentType::State:
            return 0;  // Don't consume composing text
        case SegmentType::Any:
            return 1;  // Matches 1 ASCII printable character
    }
}
```

### 4. Capture System with Segment Tracking

#### Capture Structure

```cpp
struct Capture {
    std::u16string value;     // The actual matched text
    size_t position;          // Position in variable (for wildcards)
    size_t segmentIndex;      // Which LHS segment produced this (1-based)
};
```

#### Wildcard Capture Behavior

**AnyOfVariable** (`[*]`): Captures character and its position in the variable
**NotAnyOfVariable** (`[^]`): Captures character that is NOT in the variable

```cpp
// Example: $consonants = "ကခဂဃ", text = "ခ"
// $consonants[*] matching "ခ" produces:
//   capture.value = "ခ"
//   capture.position = 1  (second character in variable, 0-based)
//   capture.segmentIndex = 1
```

### 5. Rule Priority and Sorting

Rules are pre-sorted to ensure deterministic matching:

1. **State-specific rules first**: Rules with state conditions `('state')`
2. **Virtual key combinations**: Rules with `<VK_...>` patterns  
3. **Longer text patterns first**: "abc" matches before "ab"
4. **First match wins**: No further rules tested after match

### 6. Output Generation and Text Replacement

#### Suffix-Only Replacement

**Important Behavior**: When a rule matches, ONLY the matched suffix gets replaced, preserving any unmatched prefix.

```cpp
// Example: Composing text "hello world", pattern "world" => "universe"
// Result: "hello universe" (NOT just "universe")

if (matchedLength > 0 && currentContext.size() >= matchedLength) {
    size_t unmatchedLength = currentContext.size() - matchedLength;
    std::u16string unmatchedPrefix = currentContext.substr(0, unmatchedLength);
    finalComposing = unmatchedPrefix + ruleOutput;
}
```

#### Variable Indexing in Output

**Advanced Feature**: RHS patterns can use `Variable[reference]` to extract specific characters from variables based on captured positions.

```cpp
// Pattern: $baseK[*] => $baseU[$1]
// If $1 captured position 2, output character at position 2 from $baseU
std::u16string processVariableIndex(uint16_t variableIndex, uint16_t referenceNum) {
    Capture* capture = findCaptureBySegment(referenceNum);
    if (capture && capture->position < getVariableLength(variableIndex)) {
        return getVariableCharAt(variableIndex, capture->position);
    }
    return u"";  // Handle invalid references gracefully
}
```

#### Reference Resolution

RHS references (`$1`, `$2`, `$3`) are resolved by finding captures with matching segment indices:

```cpp
std::u16string resolveReference(uint16_t segmentNum, const std::vector<Capture>& captures) {
    for (const auto& capture : captures) {
        if (capture.segmentIndex == segmentNum) {
            return capture.value;
        }
    }
    return u"";  // Invalid reference returns empty string
}
```

### 7. Action Generation

The engine generates specific action types based on text changes:

```cpp
enum class ActionType {
    Insert,                    // Insert new text at cursor
    BackspaceDelete,          // Delete N characters backward  
    BackspaceDeleteAndInsert  // Delete N chars, then insert text
};
```

**Action Logic**:
- **No rule match**: `Insert` the new character
- **Rule match**: `BackspaceDeleteAndInsert` with matched length + new output
- **Empty output**: `BackspaceDelete` only (removal)

### 8. Recursive Rule Processing

KeyMagic supports rule chaining where rule output can trigger additional rules:

#### Initial vs Recursive Matching

- **Initial matching**: Receives both VK code and character, can match virtual key rules
- **Recursive matching**: Receives only composing text, matches text-based rules only

#### Recursion Stop Conditions

Rule processing continues until:
1. **Empty output**: No characters remain
2. **Single ASCII character**: Exactly one printable ASCII character (! through ~, excluding space)
3. **No matching rule**: Current text doesn't trigger any rule

#### Infinite Loop Prevention

```cpp
const int MAX_RECURSION_DEPTH = 100;  // Prevent infinite rule chains
int depth = 0;
while (hasMoreRules && ++depth < MAX_RECURSION_DEPTH) {
    // Process recursive rules...
}
```

### 9. External Control Interface

The engine provides methods for external state management:

#### Reset Operation
```cpp
void reset() {
    composingBuffer.clear();
    activeStates.clear();
    // Complete state cleanup
}
```

#### Composing Text Synchronization
```cpp
void setComposingText(const std::string& text) {
    composingBuffer = utf8_to_utf16(text);
    activeStates.clear();  // States don't persist across external changes
}
```

These capabilities handle:
- Focus changes between input fields
- User cancellation (ESC key)
- State leakage prevention
- Error recovery
- External editor synchronization

## Detailed Component Behaviors

### Matcher Component

#### ANY Keyword Specifics

The `ANY` keyword has very specific matching rules:
- **Matches**: Printable ASCII characters from `!` to `~` (U+0021 to U+007E)
- **Does NOT match**: Space character, Unicode characters, control characters
- **Use case**: Handling ASCII input in Unicode-heavy layouts

#### Wildcard Processing

```cpp
bool matchAnyOfVariable(char16_t ch, uint16_t variableIndex, size_t& position) {
    std::u16string variable = getVariable(variableIndex);
    auto pos = variable.find(ch);
    if (pos != std::u16string::npos) {
        position = pos;  // Store 0-based position for capture
        return true;
    }
    return false;
}
```

### Pattern Component

#### Virtual Key Combination Handling

Virtual key rules require special processing:
- Must use `opAND` opcode first
- Followed by sequence of `opPREDEFINED` opcodes
- Platform-specific VK codes converted to internal enum values
- Can combine multiple modifiers (Ctrl+Alt+Key)

### State Component

#### State Lifecycle Management

```cpp
class EngineState {
private:
    std::u16string composingText;      // Persistent across keys
    std::set<uint32_t> activeStates;   // Transient per key event
    
public:
    void processKeyEvent(const Input& input) {
        // 1. Use current active states for matching
        // 2. Clear all active states  
        // 3. Apply new states from rule output
        // 4. Update composing text
    }
};
```

## Error Handling and Validation

### Robustness Requirements

Production engines must handle:

#### Invalid References
```cpp
// Gracefully handle $N where N > segment count
if (referenceNum > segments.size()) {
    return u"";  // Return empty string, don't crash
}
```

#### Malformed Rules
- Variable references to non-existent variables
- Circular variable dependencies
- Invalid Unicode sequences
- Out-of-bounds array accesses

#### Resource Limits
```cpp
const size_t MAX_COMPOSING_LENGTH = 1000;     // Prevent memory exhaustion
const size_t MAX_VARIABLE_LENGTH = 500;      // Limit variable sizes
const int MAX_RULE_COUNT = 10000;            // Reasonable rule limits
```

### Debugging and Diagnostics

Essential debugging capabilities:
- Rule matching trace logs
- Segment breakdown visualization  
- Capture value inspection
- UTF-16/UTF-8 conversion verification
- Performance profiling for complex layouts

## Performance Considerations

### Optimization Strategies

1. **Rule Pre-sorting**: Sort once at load time, not per keystroke
2. **Suffix Trees**: Use trie structures for fast suffix matching
3. **Variable Caching**: Cache frequently accessed variable content
4. **UTF-16 Operations**: Minimize encoding conversions
5. **Early Termination**: Stop matching at first successful rule

### Memory Management

```cpp
// Efficient capture storage
std::vector<Capture> captures;
captures.reserve(estimatedCaptureCount);  // Avoid reallocations

// Composing buffer management
if (composingText.length() > MAX_COMPOSING_LENGTH) {
    // Trim or reset to prevent unbounded growth
}
```

## Integration Guidelines

### Platform Integration

1. **Input Conversion**: Platform key codes → internal VK enums
2. **Output Actions**: Internal actions → platform-specific text operations
3. **Unicode Handling**: Ensure proper UTF-8 ↔ UTF-16 conversions
4. **State Persistence**: Handle app focus changes and context switches

### Testing Requirements

Critical test categories:
- **Rule Processing**: Verify all segment types and references work
- **Unicode Support**: Test complex script handling (Myanmar, Arabic, etc.)
- **Edge Cases**: Empty inputs, malformed rules, resource limits
- **Performance**: Measure keystroke latency with real keyboard layouts
- **Integration**: Test with actual platform input systems

This comprehensive documentation provides implementers with the complete technical knowledge required to build a correct, robust KeyMagic engine that handles all the discovered complexities of rule processing, pattern matching, and text manipulation.