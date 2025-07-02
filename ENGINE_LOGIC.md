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

1.  **State Update**: The engine first updates its internal state based on the new input. This includes updating the context with the new character and resetting any flags.

2.  **Rule Matching**: The `Matcher` is invoked to find a matching rule in the keyboard layout. The matching process is as follows:
    - The `Matcher` iterates through the rules in the keyboard layout.
    - For each rule, it compares the current input context with the rule's pattern.
    - A rule is considered a match if the context matches the rule's pattern and any associated conditions (e.g., modifier keys) are met.

3.  **Recursive Matching and Output Generation**: Once a matching rule is found, the engine checks if new composed text appened with the output of the current rule can trigger another rule. This is done by recursively calling the `process_key` method with the composing text as the new input. This allows for complex, multi-level transformations.

    If no further matches are found, the `Output` component generates the final result. This can be one of the following:
    - **Text Insertion**: If the rule specifies text to be output, this text is returned.
    - **State Change**: The rule may specify a new state for the engine, which will be applied to subsequent key presses.
    - **Action**: The rule can also trigger other actions, such as deleting a character (backspace).

4.  **Return Value**: The `process_key` method returns an `Output` object containing the result of the key processing. The caller is then responsible for executing the actions specified in the `Output` object (e.g., inserting text into a text editor).

## Detailed Component Descriptions

### Engine

The `KeyMagicEngine` struct holds the current keyboard layout and the engine's state. Its primary responsibility is to manage the overall process and interact with the other components.

### State

The `EngineState` struct maintains the context of the input. This is crucial for multi-key sequences (e.g., typing "a" then "b" to get "c"). It also keeps track of which rule is currently matched.

### Matcher and Pattern

The `Matcher` and `Pattern` components work together to find the correct rule to apply. The `Pattern` defines what to look for, and the `Matcher` performs the search. This allows for complex rules, including those that depend on the preceding characters.

Key matching behaviors:
- **ANY keyword**: Matches only printable ASCII characters (0x20-0x7E), not Unicode or control characters
- **Wildcards**: Support for [*] (any character in set) and [^] (any character NOT in set)
- **State matching**: Rules can be state-specific, activated by state switches

### Input and Output

The `Input` and `Output` components serve as the data carriers for the engine. `Input` brings key events into the engine, and `Output` carries the results out. This design decouples the engine from the specifics of the operating system's input and output mechanisms.
