# KeyMagic Script (KMS) File Format Documentation

## Overview

The KeyMagic Script (KMS) format is a text-based scripting language used to define keyboard layouts and input methods for the KeyMagic input method editor. KMS files are compiled into binary `.km2` files that can be loaded by the KeyMagic engine.

## File Structure

### Comments

```kms
// Single line comment
/* Multi-line 
   comment */
```

### Metadata Options

Options are declared within comments using the `@` symbol:

```kms
/*
@NAME = "Keyboard Layout Name"
@DESCRIPTION = "Description of the keyboard layout"
@FONTFAMILY = "Myanmar3"
@ICON = "icon.ico"
@HOTKEY = "CTRL+SHIFT+M"
@TRACK_CAPSLOCK = "FALSE"
@EAT_ALL_UNUSED_KEYS = "TRUE"
@US_LAYOUT_BASED = "TRUE"
@SMART_BACKSPACE = "TRUE"
@TREAT_CTRL_ALT_AS_RALT = "TRUE"
*/
```

#### Available Options

| Option | Description | Values |
|--------|-------------|---------|
| `@NAME` | Display name of the keyboard layout | String |
| `@DESCRIPTION` | Description of the keyboard | String |
| `@FONTFAMILY` | Preferred font family | String |
| `@ICON` | Icon file for the keyboard | Filename |
| `@HOTKEY` | Hotkey combination to switch to this keyboard | Key combination |
| `@TRACK_CAPSLOCK` | Whether to track Caps Lock state | "TRUE"/"FALSE" |
| `@EAT_ALL_UNUSED_KEYS` | Consume all unused key events | "TRUE"/"FALSE" |
| `@US_LAYOUT_BASED` | Use US keyboard layout as base | "TRUE"/"FALSE" |
| `@SMART_BACKSPACE` | Enable smart backspace behavior | "TRUE"/"FALSE" |
| `@TREAT_CTRL_ALT_AS_RALT` | Treat Ctrl+Alt as Right Alt | "TRUE"/"FALSE" |

### Metadata Syntax

Options must be declared within a comment block using the format `@OPTION_NAME = "VALUE"`:
- Option names are case-insensitive
- Values must be enclosed in double quotes
- Boolean values should be "TRUE" or "FALSE" (case-insensitive)
- One option per line

### Default Behavior

When no options are specified, KeyMagic uses these defaults:
- `@TRACK_CAPSLOCK = "FALSE"`
- `@EAT_ALL_UNUSED_KEYS = "FALSE"`
- `@US_LAYOUT_BASED = "FALSE"`
- `@SMART_BACKSPACE = "FALSE"`
- `@TREAT_CTRL_ALT_AS_RALT = "FALSE"`

## Variables

Variables are declared using the `$` prefix and can store strings or Unicode sequences:

```kms
$consonants = "ကခဂဃင"
$vowels = U1000 + U1001 + U1002
$combined = $consonants + $vowels
```

### Variable Concatenation

Use the `+` operator to concatenate strings and Unicode characters:

```kms
$consU = U1000 + u1001 + u1002 + u1003
$text = "hello" + " " + "world"
```

### Line Continuation

Use backslash `\` for multi-line variable definitions:

```kms
$consU = U1000 + u1001 + u1002 + u1003 + \
         U1005 + u1006 + u1007 + \
         U1008 + u1009
```

## Unicode Characters

Unicode characters can be specified in multiple ways:

```kms
// Hex notation (case insensitive)
U1000
u1000

// In strings
"\u1000"
"\x1000"

// Direct characters
"က"

// Predefined Unicode variables
$nbsp = U00A0        // Non-breaking space
$filler = u200B      // Zero-width space
$ZWS = U200B         // Zero-width space
```

### Escape Sequences in Strings

Within string literals, the following escape sequences are supported:

```kms
// Unicode escapes
"\u1000"   // 4-digit Unicode
"\x1000"   // Alternative hex notation

// Common escapes
"\n"       // Newline
"\r"       // Carriage return
"\t"       // Tab
"\\"       // Backslash
"\'"       // Single quote
"\""       // Double quote
```

## Rules

Rules define input-to-output mappings using the `=>` operator:

### Basic Rules

```kms
// Simple character mapping
"ka" => "က"
'a' => U200B + $twh

// Unicode to Unicode
U0061 + U0062 => U1000

// Using variables
$input => $output
```

### Pattern Matching

#### Array Indexing and Wildcards

```kms
// [*] matches any character from the variable
$consonants[*] => "matched"

// [^] NOT operator - matches any character NOT in the variable
$notConsonants[^] + "a" => $1 + "vowel"

// Context preservation in output
$prefix[^] + $mc[*] + U102C => $1 + U102B

// Positional matching with back-references
$row1K[*] => $row1U[$1]    // $1 refers to matched position
$ldiaU[*] + $udia1K[*] => $udia1U[$2] + $1  // Multiple references
```

#### Back-References

Use `$1`, `$2`, `$3`, etc. to reference matched portions:

```kms
"k" + ANY => "က" + $2
$wDiaU[*] + $hDiaU[*] + u1031 + $yDiaK[*] => $yDiaU[$4] + $1 + $2 + $3
```

### Special Keywords

### Advanced Pattern Matching

#### String Indexing

Access specific characters in a variable by index (0-based):

```kms
$myvar = "abc"
$myvar[0] => "a"    // First character
$myvar[1] => "b"    // Second character
$myvar[2] => "c"    // Third character
```

#### Combining Operators

Operators can be combined for complex matching:

```kms
// Match consonant followed by any character not in vowels
$consonants[*] + $vowels[^] => $1 + "special"

// Multiple wildcards with back-references
$set1[*] + $set2[*] + $set3[*] => $3 + $2 + $1
```

#### ANY Keyword

Matches any single keystroke:

```kms
ANY + "a" => $1 + "အ"
('zg_gk') + ANY => $1 + ('zg_gk')
```

#### NULL Keyword

Used to delete/remove output:

```kms
U200B + U104D + <VK_BACK> => NULL
$ZWS + $row1U[*] + <VK_BACK> => NULL
```

## State Management

KMS supports state-based input using parentheses with quoted strings:

```kms
// Entering a state
< VK_CFLEX > => ('zg_key')

// Rules within a state
('zg_key') + '1' => U100D + U1039 + U100D
('zg_key') + '2' => U100E + U1039 + U100E

// State with ANY wildcard
('zg_gk') + ANY => $1 + ('zg_gk')

// Multiple states
< VK_CAPSLOCK > => ('zg_gk')
("RIGHT_SQUARE_BRACKET") + ANY => $1
```

### State Behavior

- States are boolean toggles - entering the same state again exits it
- State names are case-sensitive
- States persist across input until explicitly changed
- Multiple states can be active simultaneously
- State names should be unique within a keyboard layout

## Virtual Keys

### Key Combinations

Use angle brackets `< >` to define key combinations:

```kms
// Single keys
<VK_SHIFT & VK_KEY_A> => "အ"
<VK_CTRL & VK_ALT & VK_KEY_K> => "က"

// With Alt Gr
<VK_CFLEX & VK_ALT_GR> => "ရ်္"
<VK_KEY_1 & VK_ALT_GR> => "!"

// Special keys
<VK_BACK> => NULL
<VK_CAPSLOCK> => ('state')
```

### Available Virtual Keys

#### Basic Keys
- `VK_KEY_A` through `VK_KEY_Z`
- `VK_KEY_0` through `VK_KEY_9`

#### Modifiers
- `VK_SHIFT`
- `VK_CONTROL` / `VK_CTRL`
- `VK_ALT` / `VK_MENU`
- `VK_ALT_GR`

#### Special Keys
- `VK_SPACE`
- `VK_BACK`
- `VK_RETURN` / `VK_ENTER`
- `VK_TAB`
- `VK_DELETE`
- `VK_ESCAPE` / `VK_ESC`
- `VK_CAPSLOCK`
- `VK_CFLEX` (circumflex/caret key)

#### Function Keys
- `VK_F1` through `VK_F12`

#### Numpad
- `VK_NUMPAD0` through `VK_NUMPAD9`

#### OEM Keys
- `VK_OEM_1` (colon/semicolon)
- `VK_OEM_PLUS`
- `VK_OEM_MINUS`
- `VK_OEM_PERIOD`
- `VK_OEM_COMMA`
- `VK_OEM_2` (forward slash/question mark)
- `VK_OEM_3` (grave accent/tilde)
- `VK_OEM_4` (left square bracket)
- `VK_OEM_5` (backslash/pipe)
- `VK_OEM_6` (right square bracket)
- `VK_OEM_7` (single quote/double quote)
- `VK_OEM_8` (varies by keyboard)

#### Navigation Keys
- `VK_HOME`
- `VK_END`
- `VK_PRIOR` (Page Up)
- `VK_NEXT` (Page Down)
- `VK_LEFT`
- `VK_UP`
- `VK_RIGHT`
- `VK_DOWN`
- `VK_INSERT`

### Modifier Key Behavior

- Multiple modifiers can be combined with `&`
- Order of modifiers doesn't matter: `<VK_CTRL & VK_SHIFT>` equals `<VK_SHIFT & VK_CTRL>`
- Virtual key rules take precedence over string rules

## Rule Precedence and Matching Order

### Rule Priority

Rules are matched in the following order:
1. **Longer patterns first**: "abc" matches before "ab"
2. **Virtual key combinations before string patterns**
3. **State-specific rules before global rules**
4. **First match wins**: Once a rule matches, no further rules are tested

### Greedy Matching

KeyMagic uses greedy matching by default:
```kms
// Given input "kha", this rule:
"kh" => "ခ"
"a" => "ာ"
// Results in: "ခာ" (matches "kh" first, then "a")

// To override, use more specific rules:
"kha" => "ခါ"  // This matches before the individual rules
```

### Recursive Rule Matching

KeyMagic applies rules recursively to the output of each rule transformation. This allows for chain transformations where the output of one rule can trigger another rule.

#### Recursion Stop Conditions

Rule matching stops when one of these conditions is met:
1. **Empty output**: The result has no characters
2. **Single ASCII printable character**: The result is exactly one character AND that character is a printable ASCII character (excluding space)

This means:
- Rules continue to be applied until the output is empty
- OR until the output is a single printable ASCII character (! through ~, excluding space)
- Unicode characters and multi-character outputs will continue to be processed recursively

#### Example of Recursive Matching

```kms
// Recursion stops at single ASCII char:
"a" => "b"  // Stops here: "b" is a single ASCII character
"b" => "c"  // This rule won't be applied to the output

// Input "a" results in "b" (recursion stops)

// Recursion continues for Unicode:
"x" => "ျ"           // Unicode output, continues
"ျ" => "ြ"           // Still Unicode, continues  
"ြ" => "ွ"           // Still Unicode, continues
"ွ" => "a"           // Stops here: single ASCII character

// Recursion continues for multi-character output:
"q" => "ab"          // Multi-character, continues
"ab" => "xyz"        // Multi-character, continues
"xyz" => "test"      // Multi-character, continues
"test" => "က"        // Unicode, continues
"က" => "w"           // Stops here: single ASCII character
```

## Include Directive

Scripts can include other KMS files:

```kms
include ( "common_rules.kms" )
include ( "burmese_auto_corrections.kms" )
```

## Complex Rule Examples

### Context-Sensitive Replacements

```kms
// Vowel stacking
$consonants[*] + "a" => $1 + "ာ"
$consonants[*] + "i" => $1 + "ိ"

// Complex context matching
u101E + u103C + u1031 + u102C + $asatK[*] => u102A

// Smart vowel handling
"ာ" + "ာ" => "ါ"
$ldiaU[*] + $udia1K[*] => $udia1U[$2] + $1
```

### Backspace Handling

```kms
// Remove specific sequences on backspace
$vowels[*] + VK_BACK => NULL
U200B + U104D + <VK_BACK> => NULL
$ZWS + $row1U[*] + <VK_BACK> => NULL
```

### State-Based Input

```kms
// Enter Zawgyi mode
< VK_CFLEX > => ('zg_key')

// Zawgyi digit handling
('zg_key') + '1' => U100D + U1039 + U100D
('zg_key') + '2' => U100E + U1039 + U100E

// Maintain state
('zg_gk') + ANY => $1 + ('zg_gk')
```

## Complete Example

```kms
/*
@NAME = "Myanmar Unicode"
@FONTFAMILY = "Myanmar3"
@DESCRIPTION = "Myanmar Unicode Keyboard Layout"
@TRACK_CAPSLOCK = "FALSE"
@SMART_BACKSPACE = "TRUE"
@US_LAYOUT_BASED = "TRUE"
*/

// Include common rules
include ( "myanmar-common.kms" )

// Define character groups
$consonants = "ကခဂဃငစဆဇဈညဋဌဍဎဏတထဒဓနပဖဗဘမယရလဝသဟဠအ"
$vowels = "ါာိီုူေဲ"
$ZWS = U200B

// Basic consonant mappings
"ka" => "က"
"kha" => "ခ"
"ga" => "ဂ"

// Vowel combinations with back-reference
$consonants[*] + "a" => $1 + "ာ"
$consonants[*] + "i" => $1 + "ိ"
$consonants[*] + "u" => $1 + "ု"

// Key combinations
<VK_SHIFT & VK_KEY_K> => "ခ"
<VK_ALT_GR & VK_KEY_1> => "၁"

// State management
< VK_CFLEX > => ('special_mode')
('special_mode') + "a" => "special_a_output"

// Smart replacements
"ာ" + "ာ" => "ါ"
$ZWS + "က" + "်" => "က်"

// Backspace handling
$vowels[*] + <VK_BACK> => NULL
$ZWS + $consonants[*] + <VK_BACK> => $2
```

## Compilation

KMS files are compiled using the KeyMagic parser into binary `.km2` files:

1. **Lexical Analysis**: Flex-based scanner tokenizes the input
2. **Parsing**: Bison-based parser builds syntax tree
3. **Code Generation**: Driver converts parsed rules to binary format

The resulting `.km2` files are loaded by the KeyMagic engine for runtime input processing.

## Best Practices

### Performance Optimization

1. **Order rules efficiently**: Place frequently-used rules near the top
2. **Use variables**: Group related characters to reduce rule count
3. **Minimize wildcards**: Specific patterns match faster than wildcards
4. **Avoid overlapping rules**: Reduces ambiguity and improves speed

### Debugging Tips

1. **Test incrementally**: Add rules gradually and test each addition
2. **Use comments liberally**: Document complex rules and their purpose
3. **Check rule order**: Remember that first match wins
4. **Verify state transitions**: Ensure states enter/exit as expected
5. **Test edge cases**: Special keys, modifier combinations, rapid typing

### Common Pitfalls

1. **Forgetting quotes**: String literals must be quoted
2. **Case sensitivity**: Variable names and states are case-sensitive
3. **Missing operators**: Don't forget `+` between pattern elements
4. **Circular references**: Avoid variables that reference themselves
5. **State persistence**: Remember states remain active until changed

## Limitations

- Maximum rule count: Implementation-dependent (typically thousands)
- Maximum variable length: Implementation-dependent
- State name length: Should be kept reasonable for performance
- Pattern complexity: Very complex patterns may impact typing speed
- Unicode support: Full Unicode range supported, but font display varies