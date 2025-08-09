#include <keymagic/engine.h>
#include <keymagic/keymagic.h>
#include <keymagic/km2_format.h>
#include "../km2/loader.h"
#include "../utils/utf8.h"
#include "../utils/debug.h"
#include "../matching/matcher.h"
#include <algorithm>
#include <sstream>

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
    , maxRecursionDepth_(10)
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
    Input input(vkCode, static_cast<char32_t>(character), modifiers);
    return processKey(input);
}

Output Engine::testProcessKey(const Input& input) {
    return processKeyInternal(input, true);
}

Output Engine::processKeyInternal(const Input& input, bool testMode) {
    if (!keyboard_) {
        return Output::None();
    }
    
    // Save state for potential undo
    if (!testMode) {
        saveStateSnapshot();
    }
    
    // Get current context
    MatchContext context;
    context.context = state_->getComposingText();
    
    // Create potential new context for matching (current + new character)
    std::string potentialContext = context.context;
    if (input.character > 0 && input.character < 0x10000) {
        potentialContext += utils::utf32ToUtf8(input.character);
    }
    
    context.activeStates = std::vector<int>(state_->getActiveStates().begin(), 
                                           state_->getActiveStates().end());
    
    // Store previous composing text
    std::string oldComposing = state_->getComposingText();
    
    // Try to match rules against the potential new context
    bool matched = false;
    MatchContext matchContext = context;
    matchContext.context = potentialContext;
    
    for (const auto& rule : rules_) {
        if (matchRule(rule, matchContext, input)) {
            if (!testMode) {
                auto output = applyRule(rule, matchContext);
                
                // Adjust the delete count since we haven't actually added the character yet
                // The delete count should be based on the OLD composing text length
                if (output.action == ActionType::BackspaceDeleteAndInsert) {
                    // We matched with the potential context, but the actual context is shorter
                    // So we need to delete the old composing text length, not the potential length
                    output.deleteCount = utils::utf8CharCount(oldComposing);
                    output.action = output.deleteCount > 0 ? ActionType::BackspaceDeleteAndInsert : ActionType::Insert;
                }
                
                // Update composing text based on output
                if (output.action == ActionType::None && output.composingText.empty()) {
                    // Rule matched but produced no output - keep existing composing
                } else {
                    state_->setComposingText(output.composingText);
                }
                
                // Perform recursive matching if enabled
                if (recursionEnabled_ && currentRecursionDepth_ < maxRecursionDepth_) {
                    currentRecursionDepth_++;
                    Output recursiveOutput = performRecursiveMatching(state_->getComposingText());
                    currentRecursionDepth_ = 0;
                    
                    // Only use recursive output if it actually matched something
                    if (recursiveOutput.action != ActionType::None) {
                        output = recursiveOutput;
                    }
                }
                
                return output;
            } else {
                // Test mode - just return what would happen
                return applyRule(rule, matchContext);
            }
            matched = true;
            break;
        }
    }
    
    // No rule matched
    if (!matched) {
        // Check if we should eat the key
        if (keyboard_->eatsAllUnusedKeys()) {
            return Output::None();
        }
        
        // Append character to composing text if it's printable
        if (input.character > 0 && input.character < 0x10000) {
            if (!testMode) {
                std::string charStr = utils::utf32ToUtf8(input.character);
                state_->appendToComposingText(charStr);
            }
            
            std::string newComposing = oldComposing + utils::utf32ToUtf8(input.character);
            return Output::Insert(utils::utf32ToUtf8(input.character), newComposing);
        }
    }
    
    return Output::None();
}

void Engine::reset() {
    state_->reset();
    history_.clear();
    currentRecursionDepth_ = 0;
}

std::string Engine::getComposingText() const {
    return state_->getComposingText();
}

void Engine::setComposingText(const std::string& text) {
    saveStateSnapshot();
    state_->setComposingText(text);
    state_->clearActiveStates();
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
        rule.patternType = PatternType::String;
        return;
    }
    
    // Check for state pattern
    if (rule.lhsOpcodes[0] == OP_SWITCH) {
        rule.patternType = PatternType::State;
        if (rule.lhsOpcodes.size() > 1) {
            rule.stateId = rule.lhsOpcodes[1];
        }
        return;
    }
    
    // Check for virtual key pattern
    if (rule.lhsOpcodes[0] == OP_AND || rule.lhsOpcodes[0] == OP_PREDEFINED) {
        rule.patternType = PatternType::VirtualKey;
        // Extract virtual key information
        for (size_t i = 0; i < rule.lhsOpcodes.size(); ++i) {
            if (rule.lhsOpcodes[i] == OP_PREDEFINED && i + 1 < rule.lhsOpcodes.size()) {
                uint16_t vkValue = rule.lhsOpcodes[i + 1];
                if (VirtualKeyHelper::isValid(vkValue)) {
                    rule.keyCombo.push_back(static_cast<VirtualKey>(vkValue));
                }
            }
        }
        if (!rule.keyCombo.empty()) {
            rule.virtualKey = rule.keyCombo[0];
        }
        return;
    }
    
    // Check for ANY pattern
    if (rule.lhsOpcodes[0] == OP_ANY) {
        rule.patternType = PatternType::Any;
        return;
    }
    
    // Default to string pattern
    rule.patternType = PatternType::String;
    
    // Extract string pattern for matching
    rule.stringPattern = extractStringPattern(rule.lhsOpcodes);
    rule.patternLength = utils::utf8CharCount(rule.stringPattern);
}

std::string Engine::extractStringPattern(const std::vector<uint16_t>& opcodes) {
    std::string pattern;
    
    for (size_t i = 0; i < opcodes.size(); ++i) {
        if (opcodes[i] == OP_STRING && i + 1 < opcodes.size()) {
            uint16_t length = opcodes[i + 1];
            i += 2;
            
            std::u16string str;
            for (uint16_t j = 0; j < length && i < opcodes.size(); ++j, ++i) {
                str.push_back(static_cast<char16_t>(opcodes[i]));
            }
            pattern += utils::utf16ToUtf8(str);
            i--; // Adjust for loop increment
        }
    }
    
    return pattern;
}

void Engine::sortRulesByPriority() {
    std::stable_sort(rules_.begin(), rules_.end(), 
        [](const ProcessedRule& a, const ProcessedRule& b) {
            if (a.priority != b.priority) {
                return a.priority < b.priority;
            }
            // For same priority, maintain original order
            return a.originalIndex < b.originalIndex;
        });
}

RulePriority Engine::calculateRulePriority(const ProcessedRule& rule) const {
    // State-specific rules have highest priority
    if (rule.patternType == PatternType::State && rule.stateId >= 0) {
        return RulePriority::StateSpecific;
    }
    
    // Virtual key rules
    if (rule.patternType == PatternType::VirtualKey) {
        return RulePriority::VirtualKey;
    }
    
    // String patterns - longer patterns have higher priority
    if (rule.patternType == PatternType::String) {
        if (rule.patternLength > 3) {
            return RulePriority::LongPattern;
        }
    }
    
    return RulePriority::ShortPattern;
}

bool Engine::matchRule(const ProcessedRule& rule, const MatchContext& context, const Input& input) {
    // Delegate to matcher
    return matcher_->matchRule(rule, context, input, keyboard_->strings);
}

Output Engine::applyRule(const ProcessedRule& rule, const MatchContext& context) {
    // Apply the rule and generate output
    return matcher_->applyRule(rule, context, keyboard_->strings, state_.get());
}

Output Engine::performRecursiveMatching(const std::string& text) {
    // Check recursion stop conditions
    if (shouldStopRecursion(text)) {
        return Output::None();
    }
    
    // Create a dummy input for recursive matching
    Input dummyInput;
    MatchContext recursiveContext;
    recursiveContext.context = text;
    recursiveContext.activeStates = std::vector<int>(state_->getActiveStates().begin(),
                                                    state_->getActiveStates().end());
    
    // Try to match rules with text only (no VK input)
    for (const auto& rule : rules_) {
        // Skip VK-only rules in recursive matching
        if (rule.patternType == PatternType::VirtualKey) {
            continue;
        }
        
        if (matchRule(rule, recursiveContext, dummyInput)) {
            return applyRule(rule, recursiveContext);
        }
    }
    
    return Output::None();
}

bool Engine::shouldStopRecursion(const std::string& text) const {
    // Stop if text is empty
    if (text.empty()) return true;
    
    // Stop if text is a single ASCII printable character (excluding space)
    if (isSingleAsciiPrintable(text)) return true;
    
    return false;
}

bool Engine::isSingleAsciiPrintable(const std::string& text) const {
    return utils::isSingleAsciiPrintable(text);
}

std::string Engine::applySmartBackspace(const std::string& text) const {
    if (!keyboard_ || !keyboard_->hasSmartBackspace()) {
        return text;
    }
    
    // Smart backspace logic - remove complete clusters
    // This is a simplified version - real implementation would need
    // language-specific cluster detection
    return utils::utf8Substring(text, 0, utils::utf8CharCount(text) - 1);
}

Output Engine::generateAction(const std::string& oldText, const std::string& newText) {
    if (oldText == newText) {
        return Output::None();
    }
    
    // Calculate how many characters to delete
    int deleteCount = calculateDeleteCount(oldText, newText);
    
    if (deleteCount > 0) {
        std::string insertText = newText.substr(oldText.size() - deleteCount);
        if (!insertText.empty()) {
            return Output::DeleteAndInsert(deleteCount, insertText, newText);
        } else {
            return Output::Delete(deleteCount, newText);
        }
    } else if (newText.size() > oldText.size()) {
        std::string insertText = newText.substr(oldText.size());
        return Output::Insert(insertText, newText);
    }
    
    return Output::None();
}

int Engine::calculateDeleteCount(const std::string& oldText, const std::string& newText) {
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

Output KeyMagicEngine::processKey(int keyCode, char32_t character, const Modifiers& modifiers) {
    Input input(keyCode, character, modifiers);
    return engine_->processKey(input);
}

Output KeyMagicEngine::processWindowsKey(int vkCode, char character, const Modifiers& modifiers) {
    return engine_->processKeyWithVK(vkCode, character, modifiers);
}

Output KeyMagicEngine::testProcessWindowsKey(int vkCode, char character, const Modifiers& modifiers) {
    Input input(vkCode, static_cast<char32_t>(character), modifiers);
    return engine_->testProcessKey(input);
}

void KeyMagicEngine::reset() {
    engine_->reset();
}

std::string KeyMagicEngine::getComposition() const {
    return engine_->getComposingText();
}

void KeyMagicEngine::setComposition(const std::string& text) {
    engine_->setComposingText(text);
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