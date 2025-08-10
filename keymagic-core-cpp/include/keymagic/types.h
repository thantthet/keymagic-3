#ifndef KEYMAGIC_TYPES_H
#define KEYMAGIC_TYPES_H

#include <cstdint>
#include <string>
#include <vector>
#include <memory>
#include <optional>
#include "virtual_keys.h"

namespace keymagic {

// Forward declarations
class KeyboardLayout;
class EngineState;
class Pattern;
class Rule;

// Error codes matching Rust KeyMagicResult
enum class Result {
    Success = 0,
    ErrorInvalidHandle = -1,
    ErrorInvalidParameter = -2,
    ErrorEngineFailure = -3,
    ErrorUtf8Conversion = -4,
    ErrorNoKeyboard = -5,
    ErrorFileNotFound = -6,
    ErrorInvalidFormat = -7,
    ErrorOutOfMemory = -8
};

// Action types for output
enum class ActionType {
    None = 0,
    Insert = 1,
    BackspaceDelete = 2,
    BackspaceDeleteAndInsert = 3
};

// Keyboard layout options
struct LayoutOptions {
    bool trackCapsLock = true;      // Track CAPSLOCK key state
    bool smartBackspace = false;     // Enable smart backspace behavior
    bool eatAllUnusedKeys = false;  // Consume keys that don't match any rule
    bool usLayoutBased = false;     // Layout based on US keyboard physical positions
    bool treatCtrlAltAsRightAlt = true; // Treat CTRL+ALT as Right Alt (AltGr)
    
    LayoutOptions() = default;
    LayoutOptions(bool trackCaps, bool autoBksp, bool eat, bool posBased, bool rightAlt)
        : trackCapsLock(trackCaps)
        , smartBackspace(autoBksp)
        , eatAllUnusedKeys(eat)
        , usLayoutBased(posBased)
        , treatCtrlAltAsRightAlt(rightAlt) {}
};

// Key modifiers
struct Modifiers {
    bool shift = false;
    bool ctrl = false;
    bool alt = false;
    bool capsLock = false;
    bool meta = false;  // Windows key / Command key
    
    Modifiers() = default;
    Modifiers(bool s, bool c, bool a, bool caps = false, bool m = false)
        : shift(s), ctrl(c), alt(a), capsLock(caps), meta(m) {}
    
    // Check if Right Alt (AltGr) is active
    bool isRightAlt(bool treatCtrlAltAsRightAlt) const {
        return alt && (!ctrl || treatCtrlAltAsRightAlt);
    }
    
    bool hasAnyModifier() const {
        return shift || ctrl || alt || meta;
    }
    
    bool operator==(const Modifiers& other) const {
        return shift == other.shift && 
               ctrl == other.ctrl && 
               alt == other.alt && 
               capsLock == other.capsLock &&
               meta == other.meta;
    }
};

// Input event
struct Input {
    VirtualKey keyCode;   // Virtual key code
    char32_t character;   // Unicode character (if applicable)
    Modifiers modifiers;
    
    Input() : keyCode(VirtualKey::Null), character(0) {}
    Input(VirtualKey kc, char32_t ch, const Modifiers& mods)
        : keyCode(kc), character(ch), modifiers(mods) {}
    // Constructor accepting int for compatibility
    Input(int kc, char32_t ch, const Modifiers& mods)
        : keyCode(static_cast<VirtualKey>(kc)), character(ch), modifiers(mods) {}
};

// Processing output
struct Output {
    ActionType action = ActionType::None;
    std::string text;           // UTF-8 encoded text
    int deleteCount = 0;        // Number of characters to delete
    std::string composingText;  // Current composing text (UTF-8)
    bool isProcessed = false;   // Whether the key was handled
    
    Output() = default;
    
    // Helper constructors
    static Output None() {
        return Output();
    }
    
    static Output Insert(const std::string& text, const std::string& composing) {
        Output out;
        out.action = ActionType::Insert;
        out.text = text;
        out.composingText = composing;
        out.isProcessed = true;
        return out;
    }
    
    static Output Delete(int count, const std::string& composing) {
        Output out;
        out.action = ActionType::BackspaceDelete;
        out.deleteCount = count;
        out.composingText = composing;
        out.isProcessed = true;
        return out;
    }
    
    static Output DeleteAndInsert(int count, const std::string& text, const std::string& composing) {
        Output out;
        out.action = ActionType::BackspaceDeleteAndInsert;
        out.deleteCount = count;
        out.text = text;
        out.composingText = composing;
        out.isProcessed = true;
        return out;
    }
    
    // Aliases for backspace-specific operations
    static Output BackspaceDelete(int count, const std::string& composing) {
        return Delete(count, composing);
    }
    
    static Output BackspaceDeleteAndInsert(int count, const std::string& text, const std::string& composing) {
        return DeleteAndInsert(count, text, composing);
    }
};

// Capture group for pattern matching
struct Capture {
    std::u16string value;
    size_t position;      // For Variable[*] wildcards, stores the position in the variable
    size_t segmentIndex;  // Which LHS segment this capture came from (1-based)
    
    Capture() : position(0), segmentIndex(0) {}
    Capture(const std::u16string& val, size_t pos, size_t seg = 0) : value(val), position(pos), segmentIndex(seg) {}
};

// Match context for rule matching
struct MatchContext {
    std::u16string context;           // Current context string
    std::vector<Capture> captures;    // Captured groups
    std::vector<int> activeStates;    // Active state IDs
    size_t matchedLength = 0;         // Length of matched text
    
    void clear() {
        context.clear();
        captures.clear();
        matchedLength = 0;
        // Note: activeStates are not cleared here
    }
    
    bool hasState(int stateId) const {
        for (int id : activeStates) {
            if (id == stateId) return true;
        }
        return false;
    }
};

// Hotkey information
struct HotkeyInfo {
    VirtualKey keyCode; // Virtual key code
    bool ctrl;
    bool alt;
    bool shift;
    bool meta;
    
    HotkeyInfo() : keyCode(VirtualKey::Null), ctrl(false), alt(false), shift(false), meta(false) {}
};

// Binary format version
struct KM2Version {
    uint8_t major;
    uint8_t minor;
    
    KM2Version(uint8_t maj = 1, uint8_t min = 5) : major(maj), minor(min) {}
    
    bool isCompatible() const {
        return major == 1 && minor >= 3 && minor <= 5;
    }
    
    bool hasInfoSection() const {
        return major == 1 && minor >= 4;
    }
    
    bool hasRightAltOption() const {
        return major == 1 && minor >= 5;
    }
};

// Rule priority for sorting
enum class RulePriority {
    StateSpecific = 0,    // Rules with state conditions
    VirtualKey = 1,       // Virtual key combinations
    LongPattern = 2,      // Longer text patterns
    ShortPattern = 3      // Shorter text patterns
};

// Rule segment type
enum class SegmentType {
    String,           // OP_STRING
    Variable,         // OP_VARIABLE (simple variable reference)
    AnyOfVariable,    // OP_VARIABLE with FLAG_ANYOF modifier ([*])
    NotAnyOfVariable, // OP_VARIABLE with FLAG_NANYOF modifier ([^])
    Any,             // OP_ANY
    VirtualKey,      // OP_PREDEFINED (with optional OP_AND)
    State,           // OP_SWITCH
    Reference        // OP_REFERENCE (used in RHS for $1, $2, $3, etc.)
};

// Represents a logical segment in a rule pattern
struct RuleSegment {
    SegmentType type;
    std::vector<uint16_t> opcodes;  // The opcodes that make up this segment
    
    RuleSegment(SegmentType t) : type(t) {}
    RuleSegment(SegmentType t, const std::vector<uint16_t>& ops) : type(t), opcodes(ops) {}
};

// Helper functions
inline std::string resultToString(Result result) {
    switch (result) {
        case Result::Success: return "Success";
        case Result::ErrorInvalidHandle: return "Invalid handle";
        case Result::ErrorInvalidParameter: return "Invalid parameter";
        case Result::ErrorEngineFailure: return "Engine failure";
        case Result::ErrorUtf8Conversion: return "UTF-8 conversion error";
        case Result::ErrorNoKeyboard: return "No keyboard loaded";
        case Result::ErrorFileNotFound: return "File not found";
        case Result::ErrorInvalidFormat: return "Invalid format";
        case Result::ErrorOutOfMemory: return "Out of memory";
        default: return "Unknown error";
    }
}

} // namespace keymagic

#endif // KEYMAGIC_TYPES_H