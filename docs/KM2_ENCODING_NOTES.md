# KM2 Binary Format Encoding Notes

This document contains implementation notes and discoveries about the KM2 binary format based on reverse engineering the original KeyMagic parser.

## File Structure Overview

```
[Header - 18 bytes]
[Strings Section - variable length]
[Info Section - variable length]
[Rules Section - variable length]
```

## Header Format (18 bytes)

The header is written as a C struct with padding:

| Offset | Size | Field | Description |
|--------|------|-------|-------------|
| 0x00 | 4 | Magic Code | "KMKL" (0x4B4D4B4C) |
| 0x04 | 1 | Major Version | 1 |
| 0x05 | 1 | Minor Version | 5 |
| 0x06 | 2 | String Count | Number of strings (little-endian) |
| 0x08 | 2 | Info Count | Number of info entries (little-endian) |
| 0x0A | 2 | Rule Count | Number of rules (little-endian) |
| 0x0C | 1 | track_caps | Track CAPSLOCK state (0 or 1) |
| 0x0D | 1 | auto_bksp | Smart backspace (0 or 1) |
| 0x0E | 1 | eat | Eat all unused keys (0 or 1) |
| 0x0F | 1 | pos_based | US layout based (0 or 1) |
| 0x10 | 1 | right_alt | Treat Ctrl+Alt as Right Alt (0 or 1) |
| 0x11 | 1 | **Padding** | Always 0x00 (C++ struct alignment) |

### Layout Options Default Values

```c
trackCaps = true    // 1
smartBksp = false   // 0
eat = false         // 0
posBased = false    // 0
rightAlt = true     // 1
```

**Important**: The C++ parser writes the entire struct including padding, so the header is 18 bytes, not 17.

## Strings Section

Strings are stored sequentially in UTF-16LE format:

```
[Length (2 bytes)] [UTF-16LE data (Length * 2 bytes)]
```

### Special String Storage

1. **Variable Values**: Stored as regular strings
2. **State Names**: Stored with dummy value 'K' (single character)
   - States are registered when encountered in rules
   - Each state gets an entry in the strings section
   - The actual state name is NOT stored; only 'K' is stored
   - States are referenced by their string index in rules

### String Indexing

- Indices are 1-based when referenced in rules
- Variables and states share the same index space

## Info Section

Each info entry has this format:

```
[ID (4 bytes)] [Length (2 bytes)] [Data (Length bytes)]
```

### Info IDs

Info IDs are stored as **little-endian** multi-character constants:

| Constant | ASCII | Stored As | Hex (LE) |
|----------|-------|-----------|----------|
| 'name' | "name" | "eman" | 0x656D616E |
| 'desc' | "desc" | "csed" | 0x63736564 |
| 'font' | "font" | "tnof" | 0x746E6F66 |
| 'icon' | "icon" | "noci" | 0x6E6F6369 |
| 'htky' | "htky" | "ykth" | 0x796B7468 |

### Info Data Encoding

- **Text data**: Stored as UTF-8 (NOT UTF-16LE)
- **Icon data**: Raw BMP file data including headers
- **Hotkey data**: Binary format (modifier flags + virtual key)

## Rules Section

Each rule consists of:

```
[LHS Length (2 bytes)] [LHS Opcodes] [RHS Length (2 bytes)] [RHS Opcodes]
```

Lengths are in 16-bit words (opcodes), not bytes.

### Opcodes

| Opcode | Value | Parameters | Description |
|--------|-------|------------|-------------|
| opSTRING | 0x00F0 | Length + UTF-16LE | String literal |
| opVARIABLE | 0x00F1 | Index (1-based) | Variable reference |
| opREFERENCE | 0x00F2 | Index | Back-reference ($1, $2, etc.) |
| opPREDEFINED | 0x00F3 | VK code | Virtual key |
| opMODIFIER | 0x00F4 | Type/Index | Modifier (see below) |
| opANYOF | 0x00F5 | Index | [*] wildcard (deprecated) |
| opAND | 0x00F6 | None | Combines VK states |
| opNANYOF | 0x00F7 | Index | [^] negation (deprecated) |
| opANY | 0x00F8 | None | ANY keyword |
| opSWITCH | 0x00F9 | Index (1-based) | State switch |

### Special Encoding Rules

#### 1. Variable Wildcards
- `$var[*]` → `opVARIABLE(index) + opMODIFIER + opANYOF`
- `$var[^]` → `opVARIABLE(index) + opMODIFIER + opNANYOF`
- `$var[$n]` → `opVARIABLE(index) + opMODIFIER + n`

#### 2. Virtual Key Combinations
- `<VK_SHIFT & VK_A>` → `opAND + opPREDEFINED(VK_SHIFT) + opPREDEFINED(VK_A)`
- `<VK_SPACE>` → `opAND + opPREDEFINED(VK_SPACE)`
- **Note**: opAND comes FIRST, before the keys
- **Important**: Even single virtual keys in angle brackets get opAND prepended

#### 3. NULL Output
- `NULL` → `opPREDEFINED(1)` where 1 = pdNULL

#### 4. State References
- States use opSWITCH with the string index (1-based)
- The string at that index contains only 'K'

### Virtual Key Values

Virtual keys use internal KeyMagic values starting from 1:

```
pdNULL = 1
pdVK_BACK = 2
pdVK_TAB = 3
pdVK_RETURN = 4
pdVK_SHIFT = 5
pdVK_CONTROL = 6
...
```

See `virtual_keys.rs` for the complete mapping.

## Implementation Quirks

1. **C++ Struct Alignment**: The header struct has 1 byte of padding at the end due to C++ compiler alignment rules.

2. **Multi-character Constants**: Info IDs like 'name' are C++ multi-character constants that get reversed on little-endian systems.

3. **State Storage**: States don't store their actual names in the strings section, only a dummy 'K' character.

4. **Modifier Opcode**: opMODIFIER is always followed by another value:
   - For wildcards: opANYOF or opNANYOF constant
   - For indexing: numeric index (e.g., 1 for $1)

5. **String Indices**: All string/variable/state references in opcodes use 1-based indexing.

6. **Virtual Key Encoding**: All virtual keys in angle brackets (even single keys like `<VK_SPACE>`) are encoded with opAND first, followed by the key code.

## Example Encodings

### Simple Rule: "ka" => "က"
```
LHS: 04 00 F0 00 02 00 6B 00 61 00
     ^len  ^opSTRING ^len 'k' 'a'

RHS: 03 00 F0 00 01 00 00 10
     ^len  ^opSTRING ^len 'က'
```

### Variable Wildcard: $consonants[*] + "a" => $1 + "ာ"
```
LHS: 07 00 F1 00 01 00 F4 00 F5 00 F0 00 01 00 61 00
     ^len  ^opVAR ^idx  ^opMOD ^ANYOF ^opSTR ^len 'a'

RHS: 05 00 F2 00 01 00 F0 00 01 00 2C 10
     ^len  ^opREF ^$1   ^opSTR ^len 'ာ'
```

### Virtual Key Combo: <VK_SHIFT & VK_A> => "အ"
```
LHS: 03 00 F6 00 F3 00 05 00 F3 00 1A 00
     ^len  ^opAND ^opPRE ^SHIFT ^opPRE ^KEY_A

RHS: 03 00 F0 00 01 00 21 10
     ^len  ^opSTR ^len  'အ'
```

## Validation Tips

1. Use hexdump to verify header structure and padding
2. Check that string count matches actual strings (including state 'K' entries)
3. Verify info IDs are little-endian reversed
4. Ensure all indices are 1-based
5. Confirm opAND precedes virtual keys in combinations
6. Check that NULL outputs use opPREDEFINED(1)