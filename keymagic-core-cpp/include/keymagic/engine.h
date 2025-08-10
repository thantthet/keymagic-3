#ifndef KEYMAGIC_ENGINE_H
#define KEYMAGIC_ENGINE_H

#include "types.h"
#include "virtual_keys.h"
#include "km2_format.h"
#include <memory>
#include <vector>
#include <string>
#include <unordered_set>

namespace keymagic {

// Forward declarations
class EngineState;
class Matcher;
class Rule;
class Pattern;
class KM2File;

// Rule representation after preprocessing
struct ProcessedRule {
    size_t originalIndex;           // Index in original KM2 file
    std::vector<uint16_t> lhsOpcodes;  // Left-hand side opcodes
    std::vector<uint16_t> rhsOpcodes;  // Right-hand side opcodes
    
    // Segmented patterns for proper reference handling
    std::vector<RuleSegment> lhsSegments;  // LHS broken into logical segments
    std::vector<RuleSegment> rhsSegments;  // RHS broken into logical segments
    
    // Preprocessed pattern info for faster matching
    std::u16string stringPattern;     // For string patterns
    std::vector<int> stateIds;        // For state-based rules (can have multiple states)
    VirtualKey virtualKey;          // For VK-based rules
    std::vector<VirtualKey> keyCombo;  // For VK combinations
    size_t patternLength;           // Effective pattern length
    
    // Priority for sorting
    RulePriority priority;
    
    ProcessedRule() : originalIndex(0), virtualKey(VirtualKey::Null), 
                     patternLength(0), priority(RulePriority::ShortPattern) {}
    
    // Helper method to check if rule has VK components
    bool hasVirtualKey() const {
        for (const auto& op : lhsOpcodes) {
            if (op == OP_AND || op == OP_PREDEFINED) {
                return true;
            }
        }
        return false;
    }
};

// Internal engine class (implementation detail)
class Engine {
public:
    Engine();
    ~Engine();
    
    // Keyboard management
    Result loadKeyboard(std::unique_ptr<KM2File> km2File);
    Result loadKeyboardFromPath(const std::string& path);
    Result loadKeyboardFromMemory(const uint8_t* data, size_t dataLen);
    void unloadKeyboard();
    bool hasKeyboard() const;
    
    // Key processing
    Output processKey(const Input& input);
    Output processKeyWithVK(int vkCode, char character, const Modifiers& modifiers);
    Output testProcessKey(const Input& input);  // Non-modifying test mode
    
    // State management
    void reset();
    std::u16string getComposingText() const;
    std::string getComposingTextUtf8() const;  // UTF-8 convenience method
    void setComposingText(const std::u16string& text);
    void setComposingText(const std::string& text);  // UTF-8 convenience overload
    
    // Keyboard information
    std::string getKeyboardName() const;
    std::string getKeyboardDescription() const;
    std::string getKeyboardHotkey() const;
    const KM2LayoutOptions* getLayoutOptions() const;
    
    // State history (for undo functionality)
    bool canUndo() const;
    void undo();
    void clearHistory();
    
    // DEBUG: Temporary access methods
    const std::vector<ProcessedRule>& getRules() const { return rules_; }
    const KM2File* getKeyboard() const { return keyboard_.get(); }
    
private:
    // Internal types
    struct StateSnapshot;
    
    // Core processing methods
    Output processKeyInternal(const Input& input, bool testMode);
    bool matchRule(const ProcessedRule& rule, MatchContext& context, const Input& input);
    Output applyRule(const ProcessedRule& rule, const MatchContext& context);
    Output performRecursiveMatching(const std::u16string& text);
    
    // Rule preprocessing
    void preprocessRules();
    void sortRulesByPriority();
    RulePriority calculateRulePriority(const ProcessedRule& rule) const;
    size_t calculateCharLength(const ProcessedRule& rule) const;
    void analyzePattern(ProcessedRule& rule);
    std::u16string extractStringPattern(const std::vector<uint16_t>& opcodes);
    
    // Rule segmentation
    std::vector<RuleSegment> segmentateOpcodes(const std::vector<uint16_t>& opcodes);
    
    // State management
    void saveStateSnapshot();
    void restoreStateSnapshot(const StateSnapshot& snapshot);
    void updateActiveStates(const std::vector<int>& newStates);
    
    // Action generation
    Output generateAction(const std::u16string& oldText, const std::u16string& newText);
    int calculateDeleteCount(const std::u16string& oldText, const std::u16string& newText);
    
    // Helper methods
    bool shouldStopRecursion(const std::u16string& text) const;
    bool isSingleAsciiPrintable(const std::u16string& text) const;
    std::u16string applySmartBackspace(const std::u16string& text) const;
    
    // Members
    std::unique_ptr<EngineState> state_;
    std::unique_ptr<KM2File> keyboard_;
    std::vector<ProcessedRule> rules_;
    std::unique_ptr<Matcher> matcher_;
    std::vector<StateSnapshot> history_;
    size_t maxHistorySize_;
    
    // Configuration
    bool recursionEnabled_;
    int maxRecursionDepth_;
    int currentRecursionDepth_;
};

// Engine state management
class EngineState {
public:
    EngineState();
    ~EngineState();
    
    // Composing text buffer
    const std::u16string& getComposingText() const;
    void setComposingText(const std::u16string& text);
    void appendToComposingText(const std::u16string& text);
    void clearComposingText();
    
    // Active states (for state-based rules)
    const std::unordered_set<int>& getActiveStates() const;
    void setActiveStates(const std::unordered_set<int>& states);
    void addActiveState(int stateId);
    void removeActiveState(int stateId);
    void clearActiveStates();
    bool hasActiveState(int stateId) const;
    
    // Context for pattern matching
    std::u16string getContext(size_t maxLength) const;
    
    // Reset all state
    void reset();
    
    // Clone for snapshot
    std::unique_ptr<EngineState> clone() const;
    void copyFrom(const EngineState& other);
    
private:
    std::u16string composingText_;
    std::unordered_set<int> activeStates_;
};

} // namespace keymagic

#endif // KEYMAGIC_ENGINE_H