# KeyMagic Key Processing Engine Logic

This document outlines the logic of the key processing engine in KeyMagic.

## Core Components

The key processing engine is composed of the following main components:

- **Engine (`engine.rs`)**: The central component that orchestrates the entire key processing workflow.
- **State (`state.rs`)**: Manages the state of the input context.
- **Input (`input.rs`)**: Represents a keyboard event.
- **Matcher (`matcher.rs`)**: Matches the input against predefined rules.
- **Pattern (`pattern.rs`)**: Represents the patterns to be matched.
- **Output (`output.rs`)**: Represents the result of the key processing.

## Processing Flow

The key processing begins when a key is pressed, triggering the `process_key` method in the `KeyMagicEngine`. The engine then performs the following steps:

### External Control

The engine provides methods for external control:
- **Reset**: Clears both the composing buffer and active states, returning the engine to its initial state
- **Set Composing Text**: Sets the composing buffer to a specific value and resets all active states. This ensures clean state when synchronizing with external text

These capabilities are essential for:
- Handling focus changes between input fields
- Responding to user cancellation (e.g., ESC key)
- Preventing state leakage between different text contexts
- Recovering from error conditions
- Synchronizing engine state with editor content (e.g., when user moves cursor or selects text)
- Restoring composing state after external text modifications

1.  **State Update**: The engine first updates its internal state based on the new input. This includes updating the context with the new character and resetting any flags.

2.  **Rule Matching**: The `Matcher` is invoked to find a matching rule in the keyboard layout. The matching process is as follows:
    - The `Matcher` iterates through the rules in the keyboard layout.
    - For each rule, it compares the current input context with the rule's pattern.
    - A rule is considered a match if the context matches the rule's pattern and any associated conditions (e.g., modifier keys) are met.

3.  **Composing Text Management and Output Generation**: The engine maintains a composing text buffer that stores the accumulated output from matched rules. When a rule matches:
    - The output from the rule is appended to the composing text
    - The engine checks if the new composing text can trigger another rule through recursive matching
    - This allows for complex, multi-level transformations

    The engine tracks how the composing text changes and generates appropriate actions:
    - **Text Insertion**: Insert new text at the cursor
    - **Backspace + Insert**: Delete previous characters and insert new text (e.g., when "title" becomes "Title", action is "delete 4 characters and insert 'Title'")
    - **State Change**: Update the engine's state for subsequent key presses
    - **Delete Only**: Remove characters without inserting new ones

    Example flow:
    - Input keys: t, i, t, l → composing text: "titl"
    - Input key: e → matches rule 'title' => 'Title'
    - Composing text changes to: "Title"
    - Action generated: backspace 4, insert "Title"

4.  **Recursive Rule Matching**: KeyMagic supports recursive rule matching, allowing rule outputs to trigger additional rules. This enables complex transformations and rule chaining.

    **Initial vs Subsequent Matching**:
    - **Initial Rule Matching**: When a physical key is pressed, the matcher receives both the virtual key code (VK) and the character representation. This allows rules to match based on either the key itself (e.g., `<VK_KEY_A>`) or the character it produces (e.g., 'a').
    - **Subsequent Rule Matching**: During recursive matching (when processing rule output), the matcher receives only the composing text - no VK or character input. This ensures that only text-based rules can match during recursion, not key-based rules.

    **Rule Chaining Example**:
    ```
    <VK_KEY_A> => 'a'
    <VK_KEY_B> => 'b'
    <VK_KEY_C> => 'c'
    'abc' => 'x'
    ```

    With these rules, pressing keys A, B, C in sequence:
    1. Press A: Matches `<VK_KEY_A> => 'a'`, composing text becomes "a"
    2. Press B: Matches `<VK_KEY_B> => 'b'`, composing text becomes "ab"
    3. Press C: Matches `<VK_KEY_C> => 'c'`, composing text becomes "abc"
    4. Recursive match: The composing text "abc" matches `'abc' => 'x'`
    5. Final output: "x" (with action: backspace 2, insert "x")

    This design allows virtual key rules to produce characters that can then be transformed by text-based rules, enabling sophisticated input method behaviors while preventing infinite loops from key-based rules.

5.  **Return Value**: The `process_key` method returns an `Output` object containing:
    - **Composing Text**: The current accumulated text in the composing buffer
    - **Actions**: Specific instructions for modifying the text (insert, delete count, or combination)
    
    The caller is responsible for executing these actions in the text editor or application.

## Detailed Component Descriptions

### Engine

The `KeyMagicEngine` struct holds the current keyboard layout and the engine's state. Its primary responsibilities include:
- Managing the overall key processing workflow
- Interacting with other components
- Providing external control methods:
  - `reset()`: Clears both composing buffer and active states simultaneously
  - `set_composing_text(text)`: Sets the composing buffer and resets active states to ensure consistent state when synchronizing with external editor

### State

The `EngineState` struct maintains:
- **Composing Text Buffer**: Stores the accumulated output from matched rules
- **Input Context**: Tracks multi-key sequences for pattern matching
- **Active States**: Manages state switches for context-sensitive rules
- **Current Rule**: Keeps track of which rule is currently matched

### Matcher and Pattern

The `Matcher` and `Pattern` components work together to find the correct rule to apply. The `Pattern` defines what to look for, and the `Matcher` performs the search. This allows for complex rules, including those that depend on the preceding characters.

Key matching behaviors:
- **ANY keyword**: Matches only printable ASCII characters (0x20-0x7E), not Unicode or control characters
- **Wildcards**: Support for [*] (any character in set) and [^] (any character NOT in set)
- **State matching**: Rules can be state-specific, activated by state switches

### Input and Output

The `Input` and `Output` components serve as the data carriers for the engine:

**Input**: Brings key events into the engine, including:
- Key code
- Modifier states (Shift, Ctrl, Alt)
- Character representation

**Output**: Carries the processing results, containing:
- **Composing Text**: The current text in the composing buffer
- **Action Type**: The specific modification to perform:
  - `Insert(text)`: Add new text
  - `BackspaceDelete(count)`: Remove specified number of characters
  - `BackspaceDeleteAndInsert(count, text)`: Delete characters then insert new text
  - `None`: No action needed (e.g., for state changes only)

This design decouples the engine from the specifics of the operating system's input and output mechanisms.
