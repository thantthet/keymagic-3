#include "matcher.h"
#include "../utils/utf8.h"
#include <algorithm>
#include <sstream>

namespace keymagic {

Matcher::Matcher() {
}

Matcher::~Matcher() {
}

bool Matcher::matchRule(const ProcessedRule& rule, const MatchContext& context, 
                       const Input& input, const std::vector<StringEntry>& strings) {
    // Check state conditions first
    if (rule.patternType == PatternType::State) {
        if (!context.hasState(rule.stateId)) {
            return false;
        }
        // Check if there's more pattern after the state
        if (rule.lhsOpcodes.size() <= 2) {
            return true; // State-only rule
        }
    }
    
    // Check virtual key patterns
    if (rule.patternType == PatternType::VirtualKey) {
        return matchVirtualKey(rule.keyCombo, input);
    }
    
    // Match text patterns
    std::vector<Capture> captures;
    return matchPattern(rule.lhsOpcodes, context.context, input, strings, captures);
}

Output Matcher::applyRule(const ProcessedRule& rule, const MatchContext& context,
                         const std::vector<StringEntry>& strings, EngineState* state) {
    // Process the RHS to generate output
    std::vector<Capture> captures = context.captures;
    std::vector<int> newStates;
    
    std::string output = generateOutput(rule.rhsOpcodes, captures, strings, newStates);
    
    #ifdef DEBUG_MATCHER
    std::cerr << "Generated output: '" << output << "'" << std::endl;
    #endif
    
    // Update states
    if (!newStates.empty()) {
        state->clearActiveStates();
        for (int stateId : newStates) {
            state->addActiveState(stateId);
        }
    }
    
    // Generate appropriate action
    std::string oldText = context.context;
    std::string newText = output;
    
    // Calculate action type
    if (newText.empty() && !oldText.empty()) {
        return Output::Delete(utils::utf8CharCount(oldText), "");
    } else if (oldText == newText) {
        return Output::None();
    } else {
        // Find common prefix
        size_t commonLen = 0;
        while (commonLen < oldText.size() && commonLen < newText.size() &&
               oldText[commonLen] == newText[commonLen]) {
            commonLen++;
        }
        
        size_t deleteCount = utils::utf8CharCount(oldText.substr(commonLen));
        std::string insertText = newText.substr(commonLen);
        
        if (deleteCount > 0 && !insertText.empty()) {
            return Output::DeleteAndInsert(deleteCount, insertText, newText);
        } else if (deleteCount > 0) {
            return Output::Delete(deleteCount, newText);
        } else if (!insertText.empty()) {
            return Output::Insert(insertText, newText);
        }
    }
    
    return Output::None();
}

bool Matcher::matchPattern(const std::vector<uint16_t>& opcodes, const std::string& context,
                          const Input& input, const std::vector<StringEntry>& strings,
                          std::vector<Capture>& captures) {
    size_t opcodeIndex = 0;
    size_t contextPos = context.size();
    
    // Process opcodes from left to right
    while (opcodeIndex < opcodes.size()) {
        uint16_t opcode = opcodes[opcodeIndex];
        
        switch (opcode) {
            case OP_STRING: {
                if (opcodeIndex + 1 >= opcodes.size()) return false;
                uint16_t length = opcodes[++opcodeIndex];
                if (opcodeIndex + length >= opcodes.size()) return false;
                
                std::u16string str;
                for (uint16_t i = 0; i < length; ++i) {
                    str.push_back(static_cast<char16_t>(opcodes[++opcodeIndex]));
                }
                
                std::string pattern = utils::utf16ToUtf8(str);
                size_t matchStart = 0;
                if (!matchString(pattern, context, matchStart)) {
                    return false;
                }
                contextPos = matchStart;
                opcodeIndex++;
                break;
            }
            
            case OP_VARIABLE: {
                if (opcodeIndex + 1 >= opcodes.size()) return false;
                uint16_t varIndex = opcodes[++opcodeIndex];
                
                // Check for modifier
                uint16_t modifier = 0;
                if (opcodeIndex + 1 < opcodes.size() && opcodes[opcodeIndex + 1] == OP_MODIFIER) {
                    opcodeIndex++;
                    if (opcodeIndex + 1 < opcodes.size()) {
                        modifier = opcodes[++opcodeIndex];
                    }
                }
                
                if (!matchVariable(varIndex, modifier, context, strings, captures)) {
                    return false;
                }
                opcodeIndex++;
                break;
            }
            
            case OP_ANY: {
                if (!matchAny(context, captures)) {
                    return false;
                }
                opcodeIndex++;
                break;
            }
            
            case OP_SWITCH: {
                // State switches are handled at rule level
                opcodeIndex += 2;
                break;
            }
            
            case OP_PREDEFINED: {
                // Virtual key patterns are handled at rule level
                opcodeIndex += 2;
                break;
            }
            
            case OP_AND: {
                // AND is for virtual key combinations
                opcodeIndex++;
                break;
            }
            
            default:
                opcodeIndex++;
                break;
        }
    }
    
    return true;
}

std::string Matcher::generateOutput(const std::vector<uint16_t>& opcodes, 
                                   const std::vector<Capture>& captures,
                                   const std::vector<StringEntry>& strings,
                                   std::vector<int>& newStates) {
    std::string output;
    size_t opcodeIndex = 0;
    
    while (opcodeIndex < opcodes.size()) {
        uint16_t opcode = opcodes[opcodeIndex];
        
        switch (opcode) {
            case OP_STRING: {
                if (opcodeIndex + 1 >= opcodes.size()) break;
                uint16_t length = opcodes[++opcodeIndex];
                
                std::u16string str;
                for (uint16_t i = 0; i < length; ++i) {
                    if (++opcodeIndex >= opcodes.size()) break;
                    str.push_back(static_cast<char16_t>(opcodes[opcodeIndex]));
                }
                output += utils::utf16ToUtf8(str);
                opcodeIndex++;
                break;
            }
            
            case OP_VARIABLE: {
                if (opcodeIndex + 1 >= opcodes.size()) break;
                uint16_t varIndex = opcodes[++opcodeIndex];
                
                // Check for index modifier (for Variable[$1] patterns)
                if (opcodeIndex + 1 < opcodes.size() && opcodes[opcodeIndex + 1] == OP_MODIFIER) {
                    opcodeIndex++;
                    if (opcodeIndex + 1 < opcodes.size()) {
                        uint16_t indexRef = opcodes[++opcodeIndex];
                        // Get the capture value and use it as index
                        if (indexRef > 0 && indexRef <= captures.size()) {
                            std::string indexStr = captures[indexRef - 1].value;
                            try {
                                size_t index = std::stoull(indexStr);
                                if (varIndex > 0 && varIndex <= strings.size()) {
                                    std::u16string varContent = strings[varIndex - 1].value;
                                    if (index < varContent.size()) {
                                        output += utils::utf32ToUtf8(varContent[index]);
                                    }
                                }
                            } catch (...) {
                                // Invalid index
                            }
                        }
                    }
                } else {
                    // Simple variable output
                    output += processVariable(varIndex, strings);
                }
                opcodeIndex++;
                break;
            }
            
            case OP_REFERENCE: {
                if (opcodeIndex + 1 >= opcodes.size()) break;
                uint16_t refNum = opcodes[++opcodeIndex];
                output += processReference(refNum, captures);
                opcodeIndex++;
                break;
            }
            
            case OP_PREDEFINED: {
                if (opcodeIndex + 1 >= opcodes.size()) break;
                uint16_t vkValue = opcodes[++opcodeIndex];
                if (vkValue == 1) {
                    // NULL output - clear everything
                    output.clear();
                }
                opcodeIndex++;
                break;
            }
            
            case OP_SWITCH: {
                if (opcodeIndex + 1 >= opcodes.size()) break;
                uint16_t stateId = opcodes[++opcodeIndex];
                newStates.push_back(stateId);
                opcodeIndex++;
                break;
            }
            
            default:
                opcodeIndex++;
                break;
        }
    }
    
    return output;
}

bool Matcher::matchString(const std::string& pattern, const std::string& context, size_t& matchStart) {
    if (pattern.empty()) return true;
    if (context.size() < pattern.size()) return false;
    
    // Check if context ends with pattern
    size_t pos = context.rfind(pattern);
    if (pos != std::string::npos && pos + pattern.size() == context.size()) {
        matchStart = pos;
        return true;
    }
    
    return false;
}

bool Matcher::matchVariable(uint16_t varIndex, uint16_t modifier, const std::string& context,
                           const std::vector<StringEntry>& strings, std::vector<Capture>& captures) {
    if (varIndex == 0 || varIndex > strings.size()) return false;
    
    std::u16string varContent = strings[varIndex - 1].value;
    std::string varStr = utils::utf16ToUtf8(varContent);
    
    if (modifier == FLAG_ANYOF) {
        // Match any character from the variable
        if (!context.empty()) {
            size_t lastCharBytes = 0;
            char32_t lastChar = utils::utf8ToChar32(
                context.substr(context.size() - std::min(size_t(4), context.size())), 
                lastCharBytes);
            
            // Check if last character is in the variable
            for (char16_t ch : varContent) {
                if (static_cast<char32_t>(ch) == lastChar) {
                    // Capture the position
                    captures.emplace_back(std::to_string(std::distance(varContent.begin(), 
                                        std::find(varContent.begin(), varContent.end(), ch))), 0);
                    return true;
                }
            }
        }
        return false;
    } else if (modifier == FLAG_NANYOF) {
        // Match any character NOT in the variable
        if (!context.empty()) {
            size_t lastCharBytes = 0;
            char32_t lastChar = utils::utf8ToChar32(
                context.substr(context.size() - std::min(size_t(4), context.size())), 
                lastCharBytes);
            
            // Check if last character is NOT in the variable
            for (char16_t ch : varContent) {
                if (static_cast<char32_t>(ch) == lastChar) {
                    return false;
                }
            }
            
            // Capture the character
            captures.emplace_back(utils::utf32ToUtf8(lastChar), 0);
            return true;
        }
        return false;
    } else {
        // Simple variable match
        size_t matchStart = 0;
        return matchString(varStr, context, matchStart);
    }
}

bool Matcher::matchVirtualKey(const std::vector<VirtualKey>& keys, const Input& input) {
    if (keys.empty()) return false;
    
    // Check if the input matches the virtual key combination
    for (const auto& key : keys) {
        int windowsVK = VirtualKeyHelper::toWindowsVK(key);
        
        // Check if this is a modifier key
        if (isModifierKey(key)) {
            switch (key) {
                case VirtualKey::Shift:
                case VirtualKey::LShift:
                case VirtualKey::RShift:
                    if (!input.modifiers.shift) return false;
                    break;
                    
                case VirtualKey::Control:
                case VirtualKey::LControl:
                case VirtualKey::RControl:
                case VirtualKey::Ctrl:
                    if (!input.modifiers.ctrl) return false;
                    break;
                    
                case VirtualKey::Menu:
                case VirtualKey::LMenu:
                case VirtualKey::RMenu:
                case VirtualKey::Alt:
                case VirtualKey::AltGr:
                    if (!input.modifiers.alt) return false;
                    break;
                    
                default:
                    break;
            }
        } else {
            // Check if the key code matches
            if (input.keyCode != windowsVK) {
                return false;
            }
        }
    }
    
    return true;
}

bool Matcher::matchAny(const std::string& context, std::vector<Capture>& captures) {
    if (context.empty()) return false;
    
    // Get last character
    size_t lastCharBytes = 0;
    char32_t lastChar = utils::utf8ToChar32(
        context.substr(context.size() - std::min(size_t(4), context.size())), 
        lastCharBytes);
    
    // Check if it's in the ANY range (ASCII printable excluding space)
    if (utils::isAnyCharacter(lastChar)) {
        captures.emplace_back(utils::utf32ToUtf8(lastChar), 0);
        return true;
    }
    
    return false;
}

std::string Matcher::processVariable(uint16_t varIndex, const std::vector<StringEntry>& strings) {
    if (varIndex > 0 && varIndex <= strings.size()) {
        return utils::utf16ToUtf8(strings[varIndex - 1].value);
    }
    return "";
}

std::string Matcher::processReference(uint16_t refNum, const std::vector<Capture>& captures) {
    if (refNum > 0 && refNum <= captures.size()) {
        return captures[refNum - 1].value;
    }
    return "";
}

} // namespace keymagic