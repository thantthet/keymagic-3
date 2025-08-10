#include <keymagic/engine.h>
#include <keymagic/keymagic.h>
#include <keymagic/km2_format.h>
#include "../km2/loader.h"
#include "../utils/utf8.h"
#include "../matching/matcher.h"
#include <algorithm>
#include <sstream>
#include <set>

namespace keymagic {


// StateSnapshot for undo functionality
struct Engine::StateSnapshot {
    std::unique_ptr<EngineState> state;
};

// Engine implementation
Engine::Engine() 
    : state_(std::make_unique<EngineState>())
    , matcher_(std::make_unique<Matcher>())
    , maxHistorySize_(50)
    , recursionEnabled_(true)
    , maxRecursionDepth_(100)
    , currentRecursionDepth_(0) {
}

Engine::~Engine() = default;

Result Engine::loadKeyboard(std::unique_ptr<KM2File> km2File) {
    if (!km2File || !km2File->isValid()) {
        return Result::ErrorInvalidFormat;
    }
    
    keyboard_ = std::move(km2File);
    preprocessRules();
    reset();
    
    return Result::Success;
}

Result Engine::loadKeyboardFromPath(const std::string& path) {
    auto km2 = KM2Loader::loadFromFile(path);
    if (!km2) {
        return Result::ErrorFileNotFound;
    }
    
    return loadKeyboard(std::move(km2));
}

Result Engine::loadKeyboardFromMemory(const uint8_t* data, size_t dataLen) {
    auto km2 = KM2Loader::loadFromMemory(data, dataLen);
    if (!km2) {
        return Result::ErrorInvalidFormat;
    }
    
    return loadKeyboard(std::move(km2));
}

void Engine::unloadKeyboard() {
    keyboard_.reset();
    rules_.clear();
    reset();
}

bool Engine::hasKeyboard() const {
    return keyboard_ != nullptr;
}

Output Engine::processKey(const Input& input) {
    return processKeyInternal(input, false);
}

Output Engine::processKeyWithVK(int vkCode, char character, const Modifiers& modifiers) {
    // Convert Windows VK code to internal VirtualKey
    VirtualKey internalVK = VirtualKeyHelper::fromWindowsVK(vkCode);
    Input input(internalVK, static_cast<char32_t>(character), modifiers);
    return processKey(input);
}

Output Engine::testProcessKey(const Input& input) {
    return processKeyInternal(input, true);
}

Output Engine::processKeyInternal(const Input& input, bool testMode) {
    if (!keyboard_) {
        return Output::None();
    }
    
    // For test mode, save the current state and restore it at the end
    std::unique_ptr<EngineState> savedState;
    if (testMode) {
        savedState = state_->clone();
    }
    
    // Track whether a key is processed
    bool isProcessed = false;
    
    // Check if this is a backspace key (we'll handle it later but need to know now for history)
    bool isBackspace = (input.keyCode == VirtualKey::Back);
    
    // Save state for potential undo BEFORE processing (but NOT for backspace operations)
    // This matches Rust behavior where backspace operations don't record history
    bool shouldRecordHistory = !testMode && !isBackspace;
    
    // Get current context
    MatchContext context;
    context.context = state_->getComposingText();
    context.activeStates = std::vector<int>(state_->getActiveStates().begin(), 
                                           state_->getActiveStates().end());
    
    
    // Store previous composing text
    std::u16string oldComposing = state_->getComposingText();
    
    // Try to match rules
    bool matched = false;
    
    // Save current states for the next key event
    auto statesForNextKey = std::vector<int>(state_->getActiveStates().begin(), 
                                             state_->getActiveStates().end());
    
    for (const auto& rule : rules_) {
        // Determine what text to match against based on rule type
        MatchContext matchContext = context;
        
        // For non-VK patterns, we need to include the typed character in the match context
        // For VK patterns, we only use the composing text (VK is checked separately)
        if (!rule.hasVirtualKey() && 
            input.character > 0) {
            // Non-VK pattern - append character to context for matching
            matchContext.context = context.context + utils::utf32ToUtf16(input.character);
        }
        
        if (matchRule(rule, matchContext, input)) {
            matched = true;
            isProcessed = true;
            
            
            // Record history before applying the rule (if not backspace and not test mode)
            if (shouldRecordHistory) {
                saveStateSnapshot();
            }
            
            // Apply the rule to get the new context and states
            auto result = applyRule(rule, matchContext);
            
            
            // Update the composing text with the new context
            state_->setComposingText(result.newContext);
            
            // Clear active states and set new states from the rule
            // This matches Rust behavior where states are cleared and new states set
            state_->clearActiveStates();
            for (int stateId : result.newStates) {
                state_->addActiveState(stateId);
            }
            
            
            std::u16string finalComposing = result.newContext;
            
            // Calculate proper action based on before/after text
            Output output = generateAction(oldComposing, finalComposing);
            output.composingText = utils::utf16ToUtf8(finalComposing);
            output.isProcessed = true;
            
            // Perform recursive matching if enabled
            if (recursionEnabled_ && currentRecursionDepth_ < maxRecursionDepth_) {
                currentRecursionDepth_++;
                auto recursiveResult = performRecursiveMatching(state_->getComposingText());
                currentRecursionDepth_ = 0;
                
                // Check if recursive matching changed anything
                if (recursiveResult.newContext != state_->getComposingText() || 
                    recursiveResult.newStates != std::vector<int>(state_->getActiveStates().begin(), 
                                                                   state_->getActiveStates().end())) {
                    // Update text if changed
                    if (recursiveResult.newContext != state_->getComposingText()) {
                        state_->setComposingText(recursiveResult.newContext);
                    }
                    
                    // Update states - clear and set new ones
                    state_->clearActiveStates();
                    for (int stateId : recursiveResult.newStates) {
                        state_->addActiveState(stateId);
                    }
                    
                    output = generateAction(oldComposing, state_->getComposingText());
                    output.composingText = utils::utf16ToUtf8(state_->getComposingText());
                    output.isProcessed = true;
                }
            }
            
            // For test mode, get the final composing text before restoring
            if (testMode) {
                output.composingText = utils::utf16ToUtf8(state_->getComposingText());
                // Restore the saved state
                state_->copyFrom(*savedState);
            }
            
            return output;
            break;
        }
    }
    
    // No rule matched
    if (!matched) {
        // Check if this is a backspace key
        if (isBackspace && !state_->getComposingText().empty()) {
            isProcessed = true;
            
            // Handle backspace based on auto_bksp option
            if (keyboard_->getLayoutOptions().getAutoBksp()) {
                // Smart backspace with history restoration
                if (!history_.empty() && !testMode) {
                    // Restore from history (undo-like behavior)
                    auto& previousSnapshot = history_.back();
                    state_->copyFrom(*previousSnapshot.state);
                    history_.pop_back();
                } else if (!history_.empty() && testMode) {
                    // Test mode: simulate history restoration without modifying actual state
                    auto& previousSnapshot = history_.back();
                    std::u16string restoredComposing = previousSnapshot.state->getComposingText();
                    
                    // Restore the saved state before returning
                    if (savedState) {
                        state_->copyFrom(*savedState);
                    }
                    
                    return Output::BackspaceDeleteAndInsert(
                        static_cast<int>(oldComposing.size()),
                        utils::utf16ToUtf8(restoredComposing),
                        utils::utf16ToUtf8(restoredComposing)
                    );
                } else {
                    // No history available, delete one character backward
                    std::u16string newComposing = utils::utf16Substring(oldComposing, 0, 
                                                                       oldComposing.size() - 1);
                    if (!testMode) {
                        state_->setComposingText(newComposing);
                    } else {
                        // Restore the saved state before returning
                        if (savedState) {
                            state_->copyFrom(*savedState);
                        }
                    }
                    return Output::BackspaceDelete(1, utils::utf16ToUtf8(newComposing));
                }
            } else {
                // Simple backspace: delete one character
                std::u16string newComposing = utils::utf16Substring(oldComposing, 0, 
                                                                   oldComposing.size() - 1);
                if (!testMode) {
                    state_->setComposingText(newComposing);
                } else {
                    // Restore the saved state before returning
                    if (savedState) {
                        state_->copyFrom(*savedState);
                    }
                }
                return Output::BackspaceDelete(1, utils::utf16ToUtf8(newComposing));
            }
            
            // If we restored from history successfully in non-test mode, generate proper output
            if (!testMode && keyboard_->getLayoutOptions().getAutoBksp()) {
                std::u16string newComposing = state_->getComposingText();
                
                // Check if this is just a simple single character deletion
                if (oldComposing.size() == newComposing.size() + 1 &&
                    oldComposing.substr(0, newComposing.size()) == newComposing) {
                    // Simple backspace - just deleted one character from the end
                    return Output::BackspaceDelete(1, utils::utf16ToUtf8(newComposing));
                } else {
                    // Complex restoration from history - calculate the actual difference
                    // Find the common prefix length
                    size_t commonPrefix = 0;
                    size_t minLen = std::min(oldComposing.size(), newComposing.size());
                    while (commonPrefix < minLen && oldComposing[commonPrefix] == newComposing[commonPrefix]) {
                        commonPrefix++;
                    }
                    
                    // Calculate how many characters to delete from the end
                    int deleteCount = static_cast<int>(oldComposing.size() - commonPrefix);
                    
                    // Get the text to insert (the part after the common prefix)
                    std::u16string insertText = newComposing.substr(commonPrefix);
                    
                    if (deleteCount > 0 && !insertText.empty()) {
                        // Need to both delete and insert
                        return Output::BackspaceDeleteAndInsert(
                            deleteCount,
                            utils::utf16ToUtf8(insertText),
                            utils::utf16ToUtf8(newComposing)
                        );
                    } else if (deleteCount > 0) {
                        // Only delete
                        return Output::BackspaceDelete(deleteCount, utils::utf16ToUtf8(newComposing));
                    } else if (!insertText.empty()) {
                        // Only insert (shouldn't happen for backspace but handle it)
                        return Output::Insert(utils::utf16ToUtf8(insertText), utils::utf16ToUtf8(newComposing));
                    } else {
                        // No change (shouldn't happen but handle it)
                        return Output::None();
                    }
                }
            }
            
            // Clear active states after backspace
            if (!testMode) {
                state_->clearActiveStates();
            } else {
                // Test mode: get the result before restoring
                std::u16string resultComposing = state_->getComposingText();
                // Restore the saved state
                if (savedState) {
                    state_->copyFrom(*savedState);
                }
                // Return with the simulated result
                return Output::BackspaceDelete(1, utils::utf16ToUtf8(resultComposing));
            }
            
            // Backspace was processed (normal mode)
            return Output::BackspaceDelete(1, utils::utf16ToUtf8(state_->getComposingText()));
        }
        
        // Check if we should eat the key
        if (keyboard_->eatsAllUnusedKeys()) {
            // Restore test mode state before returning
            if (testMode && savedState) {
                state_->copyFrom(*savedState);
            }
            return Output::None();
        }
        
        // Append character to composing text if it's printable
        if (input.character > 0) {
            isProcessed = true;
            
            // Record history before adding character (if not test mode)
            if (shouldRecordHistory) {
                saveStateSnapshot();
            }
            
            if (!testMode) {
                std::u16string charStr = utils::utf32ToUtf16(input.character);
                state_->appendToComposingText(charStr);
                
                // Clear active states after adding character
                state_->clearActiveStates();
            }
            
            std::u16string newComposing = oldComposing + utils::utf32ToUtf16(input.character);
            
            // Restore test mode state before returning
            if (testMode && savedState) {
                state_->copyFrom(*savedState);
            }
            
            return Output::Insert(utils::utf32ToUtf8(input.character), utils::utf16ToUtf8(newComposing));
        }
        
        // Clear active states for unused keys
        if (!testMode) {
            state_->clearActiveStates();
        }
    }
    
    // For test mode, restore the saved state before returning
    if (testMode && savedState) {
        state_->copyFrom(*savedState);
    }
    
    return Output::None();
}

void Engine::reset() {
    state_->reset();
    history_.clear();
    currentRecursionDepth_ = 0;
}

std::u16string Engine::getComposingText() const {
    return state_->getComposingText();
}

std::string Engine::getComposingTextUtf8() const {
    return utils::utf16ToUtf8(state_->getComposingText());
}

void Engine::setComposingText(const std::u16string& text) {
    // Clear history when composing text is set externally
    history_.clear();
    state_->setComposingText(text);
    state_->clearActiveStates();
}

void Engine::setComposingText(const std::string& text) {
    setComposingText(utils::utf8ToUtf16(text));
}

std::string Engine::getKeyboardName() const {
    if (!keyboard_) return "";
    return keyboard_->metadata.getName();
}

std::string Engine::getKeyboardDescription() const {
    if (!keyboard_) return "";
    return keyboard_->metadata.getDescription();
}

std::string Engine::getKeyboardHotkey() const {
    if (!keyboard_) return "";
    return keyboard_->metadata.getHotkey();
}

const KM2LayoutOptions* Engine::getLayoutOptions() const {
    if (!keyboard_) return nullptr;
    return &keyboard_->header.layoutOptions;
}

bool Engine::canUndo() const {
    return !history_.empty();
}

void Engine::undo() {
    if (!history_.empty()) {
        auto& snapshot = history_.back();
        state_->copyFrom(*snapshot.state);
        history_.pop_back();
    }
}

void Engine::clearHistory() {
    history_.clear();
}

void Engine::preprocessRules() {
    if (!keyboard_) return;
    
    rules_.clear();
    rules_.reserve(keyboard_->rules.size());
    
    for (size_t i = 0; i < keyboard_->rules.size(); ++i) {
        ProcessedRule processedRule;
        processedRule.originalIndex = i;
        processedRule.lhsOpcodes = keyboard_->rules[i].lhs;
        processedRule.rhsOpcodes = keyboard_->rules[i].rhs;
        
        // Segment the opcodes into logical components
        processedRule.lhsSegments = segmentateOpcodes(keyboard_->rules[i].lhs);
        processedRule.rhsSegments = segmentateOpcodes(keyboard_->rules[i].rhs);
        
        // Analyze pattern type and extract key information
        analyzePattern(processedRule);
        
        // Calculate priority
        processedRule.priority = calculateRulePriority(processedRule);
        
        rules_.push_back(processedRule);
    }
    
    // Sort rules by priority
    sortRulesByPriority();
}

void Engine::analyzePattern(ProcessedRule& rule) {
    if (rule.lhsOpcodes.empty()) {
        return;
    }
    
    // Extract all state IDs from the pattern (can have multiple states)
    for (size_t i = 0; i < rule.lhsOpcodes.size(); i++) {
        if (rule.lhsOpcodes[i] == OP_SWITCH && i + 1 < rule.lhsOpcodes.size()) {
            rule.stateIds.push_back(rule.lhsOpcodes[i + 1]);
            i++; // Skip the state ID
        }
    }
    
    // Extract virtual key information
    // In LHS, OP_PREDEFINED must ALWAYS be preceded by OP_AND
    // Standalone OP_PREDEFINED without OP_AND is illegal in LHS
    for (size_t i = 0; i < rule.lhsOpcodes.size(); ++i) {
        if (rule.lhsOpcodes[i] == OP_AND) {
            // OP_AND indicates a VK combination follows
            // All OP_PREDEFINED following OP_AND are part of the same combination
            i++; // Move past OP_AND
            while (i < rule.lhsOpcodes.size() && rule.lhsOpcodes[i] == OP_PREDEFINED) {
                if (i + 1 < rule.lhsOpcodes.size()) {
                    uint16_t vkValue = rule.lhsOpcodes[i + 1];
                    if (VirtualKeyHelper::isValid(vkValue)) {
                        rule.keyCombo.push_back(static_cast<VirtualKey>(vkValue));
                    }
                    i++; // Skip to the VK value
                }
                i++; // Move past the VK value
            }
            i--; // Adjust for loop increment
            break; // Only one VK combination per rule
        }
        // Note: OP_PREDEFINED without OP_AND is illegal in LHS, so we don't handle it
    }
    
    // For all patterns, extract string content if present
    rule.stringPattern = extractStringPattern(rule.lhsOpcodes);
    rule.patternLength = rule.stringPattern.size();  // UTF-16 character count
}

std::u16string Engine::extractStringPattern(const std::vector<uint16_t>& opcodes) {
    std::u16string pattern;
    
    for (size_t i = 0; i < opcodes.size(); ++i) {
        if (opcodes[i] == OP_STRING && i + 1 < opcodes.size()) {
            uint16_t length = opcodes[i + 1];
            i += 2;
            
            for (uint16_t j = 0; j < length && i < opcodes.size(); ++j, ++i) {
                pattern.push_back(static_cast<char16_t>(opcodes[i]));
            }
            i--; // Adjust for loop increment
        }
    }
    
    return pattern;
}

void Engine::sortRulesByPriority() {
    std::stable_sort(rules_.begin(), rules_.end(), 
        [](const ProcessedRule& a, const ProcessedRule& b) {
            // State-specific rules ALWAYS come before non-state rules
            bool aHasState = !a.stateIds.empty();
            bool bHasState = !b.stateIds.empty();
            
            if (aHasState && !bHasState) {
                return true;  // a (with state) comes before b (no state)
            }
            if (!aHasState && bHasState) {
                return false; // b (with state) comes before a (no state)
            }
            
            // Both have states or both don't have states - use priority
            if (a.priority != b.priority) {
                return static_cast<int>(a.priority) > static_cast<int>(b.priority);
            }
            // For same priority, maintain original order
            return a.originalIndex < b.originalIndex;
        });
    
}

RulePriority Engine::calculateRulePriority(const ProcessedRule& rule) const {
    // Calculate exact char_length like Rust implementation
    size_t charLength = calculateCharLength(rule);
    size_t stateCount = rule.stateIds.size();  // Use the extracted state IDs
    size_t vkCount = 0;
    
    // Count virtual keys
    for (size_t i = 0; i < rule.lhsOpcodes.size(); ++i) {
        uint16_t op = rule.lhsOpcodes[i];
        
        if (op == OP_PREDEFINED || op == OP_AND) {
            if (op == OP_AND) {
                // Count subsequent OP_PREDEFINED elements
                size_t j = i + 1;
                while (j < rule.lhsOpcodes.size() && rule.lhsOpcodes[j] == OP_PREDEFINED) {
                    vkCount++;
                    j += 2; // Skip OP_PREDEFINED and its value
                }
                i = j - 1; // Continue from last processed position
            } else {
                vkCount++;
                i++; // Skip predefined value
            }
        }
    }
    
    // Use Rust-style priority calculation based on counts
    // Higher numbers get higher priority (checked first)
    RulePriority priority;
    if (stateCount > 0) {
        // State-specific rules: priority = 1000 + stateCount + vkCount + charLength
        priority = static_cast<RulePriority>(1000 + stateCount * 100 + vkCount * 10 + charLength);
    } else if (vkCount > 0) {
        // Virtual key rules: priority = 500 + vkCount + charLength  
        priority = static_cast<RulePriority>(500 + vkCount * 10 + charLength);
    } else {
        // Text rules: priority = charLength (longer patterns get higher priority)
        priority = static_cast<RulePriority>(charLength);
    }
    
    
    return priority;
}

size_t Engine::calculateCharLength(const ProcessedRule& rule) const {
    size_t charLength = 0;
    
    for (size_t i = 0; i < rule.lhsOpcodes.size(); ++i) {
        uint16_t op = rule.lhsOpcodes[i];
        
        if (op == OP_STRING) {
            // Count actual characters in the string
            if (i + 1 < rule.lhsOpcodes.size()) {
                uint16_t len = rule.lhsOpcodes[i + 1];
                
                // Count UTF-16 characters (each counts as 1)
                charLength += len;
                
                // Skip OP_STRING, length, and string data
                i += 1 + len; // Skip OP_STRING + length + len characters
            }
        } else if (op == OP_VARIABLE) {
            i++; // Skip variable index
            
            // Check if followed by modifier (wildcard)
            if (i + 1 < rule.lhsOpcodes.size() && rule.lhsOpcodes[i + 1] == OP_MODIFIER) {
                uint16_t modifier = rule.lhsOpcodes[i + 2];
                if (modifier == FLAG_ANYOF || modifier == FLAG_NANYOF) {
                    // Variable with wildcard [*] or [^] counts as 1 character
                    charLength += 1;
                }
                i += 2; // Skip OP_MODIFIER and modifier value
            }
            // Variable without wildcard doesn't contribute to charLength during pattern creation
        } else if (op == OP_ANY) {
            // ANY matches exactly 1 character
            charLength += 1;
        } else if (op == OP_SWITCH) {
            // Skip state switch and its ID
            i++; 
        } else if (op == OP_AND || op == OP_PREDEFINED) {
            // Virtual keys don't contribute to character length
            if (op == OP_PREDEFINED) {
                i++; // Skip predefined value
            }
        } else if (op == OP_MODIFIER) {
            // Skip modifier value
            i++;
        }
    }
    
    return charLength;
}

bool Engine::matchRule(const ProcessedRule& rule, MatchContext& context, const Input& input) {
    // Delegate to matcher
    return matcher_->matchRule(rule, context, input, keyboard_->strings);
}

RuleApplicationResult Engine::applyRule(const ProcessedRule& rule, const MatchContext& context) {
    // Apply the rule and generate the new context and states
    return matcher_->applyRule(rule, context, keyboard_->strings);
}

RuleApplicationResult Engine::performRecursiveMatching(const std::u16string& text) {
    // Check recursion stop conditions
    if (shouldStopRecursion(text)) {
        // Return unchanged text with current states
        std::vector<int> currentStates(state_->getActiveStates().begin(),
                                      state_->getActiveStates().end());
        return RuleApplicationResult(text, currentStates, 0);
    }
    
    std::u16string currentText = text;
    std::u16string lastText;
    
    // Start with current active states
    std::vector<int> currentStates(state_->getActiveStates().begin(),
                                  state_->getActiveStates().end());
    
    // Keep applying rules until no more rules match or text doesn't change
    while (currentText != lastText && !shouldStopRecursion(currentText)) {
        lastText = currentText;
        
        // Create a dummy input for recursive matching
        Input dummyInput;
        MatchContext recursiveContext;
        recursiveContext.context = currentText;
        recursiveContext.activeStates = currentStates;  // Use our tracked states
        
        // Try to match rules with text only (no VK input)
        bool matched = false;
        for (const auto& rule : rules_) {
            // Skip rules with ANY VK components in recursive matching
            // This includes pure VK rules and mixed VK+text rules
            bool hasVK = false;
            for (const auto& op : rule.lhsOpcodes) {
                if (op == OP_AND || op == OP_PREDEFINED) {
                    hasVK = true;
                    break;
                }
            }
            if (hasVK) {
                continue;
            }
            
            if (matchRule(rule, recursiveContext, dummyInput)) {
                // Apply the rule to get new context
                auto result = applyRule(rule, recursiveContext);
                
                // Update current text with the result
                currentText = result.newContext;
                
                // Update our tracked states if the rule provided new ones
                if (!result.newStates.empty()) {
                    currentStates = result.newStates;
                }
                
                matched = true;
                break; // Start over with the new text
            }
        }
        
        if (!matched) {
            break; // No more rules match, stop
        }
    }
    
    // Return the final text and states after all recursive matching
    return RuleApplicationResult(currentText, currentStates, 0);
}

bool Engine::shouldStopRecursion(const std::u16string& text) const {
    // Stop if text is empty
    if (text.empty()) return true;
    
    // Stop if text is a single ASCII printable character (excluding space)
    if (isSingleAsciiPrintable(text)) return true;
    
    return false;
}

bool Engine::isSingleAsciiPrintable(const std::u16string& text) const {
    return utils::isSingleAsciiPrintable(text);
}

std::u16string Engine::applySmartBackspace(const std::u16string& text) const {
    if (!keyboard_ || !keyboard_->hasSmartBackspace()) {
        return text;
    }
    
    // Smart backspace logic - remove complete clusters
    // This is a simplified version - real implementation would need
    // language-specific cluster detection
    return utils::utf16Substring(text, 0, text.size() - 1);
}

Output Engine::generateAction(const std::u16string& oldText, const std::u16string& newText) {
    if (oldText == newText) {
        return Output::None();
    }
    
    // Calculate how many characters to delete
    int deleteCount = calculateDeleteCount(oldText, newText);
    
    if (deleteCount > 0) {
        std::u16string insertText = newText.substr(oldText.size() - deleteCount);
        if (!insertText.empty()) {
            return Output::DeleteAndInsert(deleteCount, utils::utf16ToUtf8(insertText), utils::utf16ToUtf8(newText));
        } else {
            return Output::Delete(deleteCount, utils::utf16ToUtf8(newText));
        }
    } else if (newText.size() > oldText.size()) {
        std::u16string insertText = newText.substr(oldText.size());
        return Output::Insert(utils::utf16ToUtf8(insertText), utils::utf16ToUtf8(newText));
    }
    
    return Output::None();
}

int Engine::calculateDeleteCount(const std::u16string& oldText, const std::u16string& newText) {
    // Find common prefix
    size_t commonPrefix = 0;
    size_t oldLen = oldText.size();
    size_t newLen = newText.size();
    
    while (commonPrefix < oldLen && commonPrefix < newLen &&
           oldText[commonPrefix] == newText[commonPrefix]) {
        commonPrefix++;
    }
    
    return oldLen - commonPrefix;
}

void Engine::saveStateSnapshot() {
    if (history_.size() >= maxHistorySize_) {
        history_.erase(history_.begin());
    }
    
    StateSnapshot snapshot;
    snapshot.state = state_->clone();
    history_.push_back(std::move(snapshot));
}

void Engine::restoreStateSnapshot(const StateSnapshot& snapshot) {
    state_->copyFrom(*snapshot.state);
}

void Engine::updateActiveStates(const std::vector<int>& newStates) {
    state_->clearActiveStates();
    for (int stateId : newStates) {
        state_->addActiveState(stateId);
    }
}

std::vector<RuleSegment> Engine::segmentateOpcodes(const std::vector<uint16_t>& opcodes) {
    std::vector<RuleSegment> segments;
    size_t i = 0;
    
    while (i < opcodes.size()) {
        uint16_t op = opcodes[i];
        
        if (op == OP_STRING) {
            // OP_STRING length char1 char2 ... charN
            if (i + 1 >= opcodes.size()) break;
            
            uint16_t length = opcodes[i + 1];
            std::vector<uint16_t> segmentOpcodes;
            
            // Include OP_STRING, length, and all characters
            for (size_t j = 0; j < 2 + length && i + j < opcodes.size(); ++j) {
                segmentOpcodes.push_back(opcodes[i + j]);
            }
            
            segments.emplace_back(SegmentType::String, segmentOpcodes);
            i += 2 + length;
            
        } else if (op == OP_VARIABLE) {
            // OP_VARIABLE varIndex [OP_MODIFIER modifierValue]
            if (i + 1 >= opcodes.size()) break;
            
            std::vector<uint16_t> segmentOpcodes = { op, opcodes[i + 1] };
            SegmentType segmentType = SegmentType::Variable;
            i += 2;
            
            // Check for modifier
            if (i < opcodes.size() && opcodes[i] == OP_MODIFIER) {
                if (i + 1 < opcodes.size()) {
                    uint16_t modifier = opcodes[i + 1];
                    segmentOpcodes.push_back(opcodes[i]);     // OP_MODIFIER
                    segmentOpcodes.push_back(opcodes[i + 1]); // modifier value
                    
                    // Determine segment type based on modifier
                    if (modifier == FLAG_ANYOF) {
                        segmentType = SegmentType::AnyOfVariable;
                    } else if (modifier == FLAG_NANYOF) {
                        segmentType = SegmentType::NotAnyOfVariable;
                    }
                    i += 2;
                }
            }
            
            segments.emplace_back(segmentType, segmentOpcodes);
            
        } else if (op == OP_ANY) {
            // OP_ANY
            segments.emplace_back(SegmentType::Any, std::vector<uint16_t>{op});
            i++;
            
        } else if (op == OP_SWITCH) {
            // OP_SWITCH stateId
            if (i + 1 >= opcodes.size()) break;
            std::vector<uint16_t> segmentOpcodes = { op, opcodes[i + 1] };
            segments.emplace_back(SegmentType::State, segmentOpcodes);
            i += 2;
            
        } else if (op == OP_AND) {
            // OP_AND OP_PREDEFINED vk1 OP_PREDEFINED vk2 ...
            // In LHS: OP_PREDEFINED must always be preceded by OP_AND
            // Collect all the virtual keys in this combination
            std::vector<uint16_t> segmentOpcodes = { op };
            i++; // Skip OP_AND
            
            while (i < opcodes.size() && opcodes[i] == OP_PREDEFINED) {
                if (i + 1 >= opcodes.size()) break;
                segmentOpcodes.push_back(opcodes[i]);     // OP_PREDEFINED
                segmentOpcodes.push_back(opcodes[i + 1]); // VK value
                i += 2;
            }
            
            segments.emplace_back(SegmentType::VirtualKey, segmentOpcodes);
            
        } else if (op == OP_PREDEFINED) {
            // Standalone OP_PREDEFINED without OP_AND
            // This is only legal in RHS for NULL (value 1)
            if (i + 1 >= opcodes.size()) break;
            uint16_t value = opcodes[i + 1];
            std::vector<uint16_t> segmentOpcodes = { op, value };
            
            if (value == 1) {
                // NULL value in RHS
                segments.emplace_back(SegmentType::Null, segmentOpcodes);
            } else {
                // This shouldn't happen in valid KM2 files
                // In LHS, OP_PREDEFINED must have OP_AND
                // In RHS, only NULL (value 1) is allowed
                // Treat it as a VirtualKey segment for error recovery
                segments.emplace_back(SegmentType::VirtualKey, segmentOpcodes);
            }
            i += 2;
            
        } else if (op == OP_REFERENCE) {
            // OP_REFERENCE refNum (used in RHS)
            if (i + 1 >= opcodes.size()) break;
            std::vector<uint16_t> segmentOpcodes = { op, opcodes[i + 1] };
            segments.emplace_back(SegmentType::Reference, segmentOpcodes);
            i += 2;
            
        } else {
            // Unknown opcode, skip it
            i++;
        }
    }
    
    return segments;
}

// KeyMagicEngine public API implementation
KeyMagicEngine::KeyMagicEngine()
    : engine_(std::make_unique<Engine>()) {
}

KeyMagicEngine::~KeyMagicEngine() = default;

KeyMagicEngine::KeyMagicEngine(KeyMagicEngine&&) noexcept = default;
KeyMagicEngine& KeyMagicEngine::operator=(KeyMagicEngine&&) noexcept = default;

Result KeyMagicEngine::loadKeyboard(const std::string& km2Path) {
    return engine_->loadKeyboardFromPath(km2Path);
}

Result KeyMagicEngine::loadKeyboardFromMemory(const uint8_t* data, size_t dataLen) {
    return engine_->loadKeyboardFromMemory(data, dataLen);
}

Output KeyMagicEngine::processKey(const Input& input) {
    return engine_->processKey(input);
}

Output KeyMagicEngine::processKey(VirtualKey keyCode, char32_t character, const Modifiers& modifiers) {
    Input input(keyCode, character, modifiers);
    return engine_->processKey(input);
}

Output KeyMagicEngine::processWindowsKey(int vkCode, char character, const Modifiers& modifiers) {
    return engine_->processKeyWithVK(vkCode, character, modifiers);
}

Output KeyMagicEngine::testProcessWindowsKey(int vkCode, char character, const Modifiers& modifiers) {
    // Convert Windows VK code to internal VirtualKey
    VirtualKey internalVK = VirtualKeyHelper::fromWindowsVK(vkCode);
    Input input(internalVK, static_cast<char32_t>(character), modifiers);
    return engine_->testProcessKey(input);
}

void KeyMagicEngine::reset() {
    engine_->reset();
}

std::string KeyMagicEngine::getComposition() const {
    std::u16string composing = engine_->getComposingText();
    return utils::utf16ToUtf8(composing);
}

void KeyMagicEngine::setComposition(const std::string& text) {
    engine_->setComposingText(utils::utf8ToUtf16(text));
}

bool KeyMagicEngine::hasKeyboard() const {
    return engine_->hasKeyboard();
}

std::string KeyMagicEngine::getKeyboardName() const {
    return engine_->getKeyboardName();
}

std::string KeyMagicEngine::getKeyboardDescription() const {
    return engine_->getKeyboardDescription();
}

std::string KeyMagicEngine::getVersion() {
    return "1.0.0";
}

} // namespace keymagic