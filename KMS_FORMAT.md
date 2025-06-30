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

// [^] matches context preservation/negation
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

## Include Directive

Scripts can include other KMS files:

```kms
include ( "common_rules.kms" )
include ( "Ayar-autocorrect.kms" )
include ( "Ayar-autocorrect2.kms" )
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