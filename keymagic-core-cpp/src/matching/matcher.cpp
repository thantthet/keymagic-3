#include "matcher.h"
#include "../utils/utf8.h"
#include <algorithm>
#include <sstream>

namespace keymagic {

Matcher::Matcher() {
}

Matcher::~Matcher() {
}

bool Matcher::matchRule(const ProcessedRule& rule, MatchContext& context, 
                       const Input& input, const std::vector<StringEntry>& strings) {
    // Check state conditions first (all states must be active)
    if (!rule.stateIds.empty()) {
        for (int stateId : rule.stateIds) {
            if (!context.hasState(stateId)) {
                return false;
            }
        }
        // Check if there's more pattern after the states
        // Count how many state segments we have
        size_t stateSegmentCount = 0;
        for (const auto& segment : rule.lhsSegments) {
            if (segment.type == SegmentType::State) {
                stateSegmentCount++;
            }
        }
        if (rule.lhsSegments.size() <= stateSegmentCount) {
            return true; // State-only rule
        }
    }
    
    // Check virtual key patterns
    if (rule.hasVirtualKey()) {
        bool vkMatched = matchVirtualKey(rule.keyCombo, input);
        
        if (!vkMatched) {
            return false; // VK didn't match, rule fails
        }
        
        // VK matched. Now check if there's also a string pattern to match
        if (!rule.stringPattern.empty() || rule.patternLength > 0) {
            // This rule has both VK and string pattern - need to match both
            std::vector<Capture> captures;
            size_t matchedLength = 0;
            bool stringMatched = matchPatternSegmented(rule.lhsSegments, context.context, input, strings, captures, matchedLength);
            
            if (!stringMatched) {
                return false; // String pattern didn't match
            }
            
            // Both VK and string pattern matched
            context.captures = captures;
            context.matchedLength = matchedLength;
            return true;
        } else {
            // VK-only rule (no string pattern)
            context.matchedLength = 0;
            context.captures.clear();
            return true;
        }
    }
    
    // Match text patterns using segments
    std::vector<Capture> captures;
    size_t matchedLength = 0;
    bool matched = matchPatternSegmented(rule.lhsSegments, context.context, input, strings, captures, matchedLength);
    
    if (matched) {
        // Store captures and matched length in context for rule application
        context.captures = captures;
        context.matchedLength = matchedLength;
    }
    return matched;
}

RuleApplicationResult Matcher::applyRule(const ProcessedRule& rule, const MatchContext& context,
                                        const std::vector<StringEntry>& strings) {
    // Process the RHS segments to generate output
    std::vector<Capture> captures = context.captures;
    std::vector<int> newStates;
    
    std::u16string ruleOutput = generateOutputSegmented(rule.rhsSegments, captures, strings, newStates);
    
    // Calculate the new context by replacing the matched portion
    std::u16string newContext;
    const std::u16string& currentContext = context.context;
    size_t matchedLength = context.matchedLength;
    
    // Keep the unmatched prefix and append/replace with the rule output
    if (matchedLength > 0 && currentContext.size() >= matchedLength) {
        // Text was matched - replace the matched suffix with rule output
        size_t unmatchedLength = currentContext.size() - matchedLength;
        newContext = currentContext.substr(0, unmatchedLength) + ruleOutput;
    } else {
        // No text was matched (e.g., VK rules) - append output to existing context
        newContext = currentContext + ruleOutput;
    }
    
    return RuleApplicationResult(newContext, newStates, matchedLength);
}

bool Matcher::matchPattern(const std::vector<uint16_t>& opcodes, const std::u16string& context,
                          const Input& input, const std::vector<StringEntry>& strings,
                          std::vector<Capture>& captures, size_t& matchedLength) {
    matchedLength = 0;
    
    // Step 1: Calculate the expected pattern length in UTF-16 characters
    size_t expectedPatternLength = calculatePatternLength(opcodes, strings);
    
    // Step 2: Extract suffix from context that matches the pattern length
    std::u16string matchContext;
    
    if (expectedPatternLength == 0) {
        // Empty pattern - should match empty context
        matchContext = u"";
    } else {
        // Get the suffix of the context with the expected length
        size_t contextCharCount = context.size();
        
        if (contextCharCount < expectedPatternLength) {
            return false; // Context too short for pattern
        }
        
        // Extract the last expectedPatternLength characters
        matchContext = utils::utf16Substring(context, contextCharCount - expectedPatternLength, expectedPatternLength);
    }
    
    // Step 3: Apply sequential matching to the suffix
    size_t opcodeIndex = 0;
    size_t contextPos = 0;
    size_t segmentIndex = 1;  // Track LHS segment number (1-based)
    
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
                
                // Check if pattern matches at current position in matchContext
                if (contextPos + str.size() > matchContext.size()) {
                    return false;
                }
                
                if (matchContext.substr(contextPos, str.size()) != str) {
                    return false;
                }
                
                // Capture the matched string with segment index
                captures.emplace_back(str, 0, segmentIndex);
                
                // Advance position
                contextPos += str.size();
                segmentIndex++;
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
                
                // For sequential matching, pass the remaining matchContext starting from contextPos
                std::u16string remainingContext = matchContext.substr(contextPos);
                
                if (!matchVariableSequential(varIndex, modifier, remainingContext, strings, captures, contextPos, matchContext.size(), segmentIndex)) {
                    return false;
                }
                segmentIndex++;
                opcodeIndex++;
                break;
            }
            
            case OP_ANY: {
                // For sequential matching, check if there's a character at current position
                if (contextPos >= matchContext.size()) {
                    return false;
                }
                
                // Get character at current position
                size_t charsConsumed = 0;
                char32_t ch = utils::utf16ToChar32(matchContext.substr(contextPos), charsConsumed);
                
                if (!utils::isAnyCharacter(ch)) {
                    return false;
                }
                
                captures.emplace_back(utils::utf32ToUtf16(ch), 0, segmentIndex);
                contextPos += charsConsumed;
                segmentIndex++;
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
    
    // For sequential patterns, we must have consumed the entire matchContext
    if (contextPos != matchContext.size()) {
        return false;
    }
    
    // Calculate the matched length - this is the length of the suffix that matched
    if (input.character > 0) {
        // Pattern matched some portion of context + typed character
        std::u16string typedChar = utils::utf32ToUtf16(input.character);
        matchedLength = matchContext.size() + typedChar.size();
    } else {
        // Pattern matched only context (recursive matching)
        matchedLength = matchContext.size();
    }
    
    return true;
}

std::u16string Matcher::generateOutput(const std::vector<uint16_t>& opcodes, 
                                      const std::vector<Capture>& captures,
                                      const std::vector<StringEntry>& strings,
                                      std::vector<int>& newStates) {
    std::u16string output;
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
                output += str;
                opcodeIndex++;
                break;
            }
            
            case OP_VARIABLE: {
                if (opcodeIndex + 1 >= opcodes.size()) break;
                uint16_t varIndex = opcodes[++opcodeIndex];
                
                // Check for index modifier (for Variable[$1] patterns)
                if (opcodeIndex + 2 < opcodes.size() && opcodes[opcodeIndex + 1] == OP_MODIFIER) {
                    opcodeIndex++; // Skip OP_MODIFIER
                    uint16_t indexRef = opcodes[++opcodeIndex]; // Read the reference index value
                    
                    // Get the capture position and use it as index into the variable
                    if (indexRef > 0 && indexRef <= captures.size()) {
                        size_t index = captures[indexRef - 1].position;
                        if (varIndex > 0 && varIndex <= strings.size()) {
                            std::u16string varContent = strings[varIndex - 1].value;
                            if (index < varContent.size()) {
                                output += varContent[index];
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
                output += processSegmentReference(refNum, captures);
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

std::u16string Matcher::generateOutputSegmented(const std::vector<RuleSegment>& segments,
                                              const std::vector<Capture>& captures,
                                              const std::vector<StringEntry>& strings,
                                              std::vector<int>& newStates) {
    std::u16string output;
    
    for (const auto& segment : segments) {
        switch (segment.type) {
            case SegmentType::String: {
                // Extract string from segment opcodes
                if (segment.opcodes.size() < 2) break;
                uint16_t length = segment.opcodes[1];
                if (segment.opcodes.size() < 2 + length) break;
                
                std::u16string str;
                for (size_t i = 2; i < 2 + length; ++i) {
                    str.push_back(static_cast<char16_t>(segment.opcodes[i]));
                }
                output += str;
                break;
            }
            
            case SegmentType::Variable: {
                // Simple variable output
                if (segment.opcodes.size() < 2) break;
                uint16_t varIndex = segment.opcodes[1];
                
                // Check for index modifier (Variable[$X] patterns)
                if (segment.opcodes.size() >= 4 && segment.opcodes[2] == OP_MODIFIER) {
                    uint16_t indexRef = segment.opcodes[3];
                    
                    // Find the capture from the referenced segment
                    for (const auto& capture : captures) {
                        if (capture.segmentIndex == indexRef) {
                            // Use the capture's position as index into the variable
                            if (varIndex > 0 && varIndex <= strings.size()) {
                                std::u16string varContent = strings[varIndex - 1].value;
                                if (capture.position < varContent.size()) {
                                    output += varContent[capture.position];
                                }
                            }
                            break;
                        }
                    }
                } else {
                    // Simple variable reference
                    output += processVariable(varIndex, strings);
                }
                break;
            }
            
            case SegmentType::Reference: {
                // Direct reference to a segment ($1, $2, $3, etc.)
                if (segment.opcodes.size() < 2) break;
                uint16_t segmentNum = segment.opcodes[1];
                output += processSegmentReference(segmentNum, captures);
                break;
            }
            
            case SegmentType::State: {
                // State activation
                if (segment.opcodes.size() < 2) break;
                uint16_t stateId = segment.opcodes[1];
                newStates.push_back(stateId);
                break;
            }
            
            case SegmentType::VirtualKey: {
                // VirtualKey segments in RHS shouldn't happen in valid KM2 files
                // (except for error recovery when OP_PREDEFINED appears without OP_AND)
                break;
            }
            
            case SegmentType::Null: {
                // NULL output (OP_PREDEFINED with value 1 in RHS)
                // NULL clears all output
                output.clear();
                break;
            }
            
            default:
                // Other segment types don't produce output
                break;
        }
    }
    
    return output;
}

bool Matcher::matchString(const std::u16string& pattern, const std::u16string& context, size_t& matchStart) {
    if (pattern.empty()) return true;
    if (context.size() < pattern.size()) return false;
    
    // Check if context ends with pattern
    size_t pos = context.rfind(pattern);
    if (pos != std::u16string::npos && pos + pattern.size() == context.size()) {
        matchStart = pos;
        return true;
    }
    
    return false;
}

bool Matcher::matchVariableSequential(uint16_t varIndex, uint16_t modifier, 
                                     const std::u16string& remainingContext,
                                     const std::vector<StringEntry>& strings, 
                                     std::vector<Capture>& captures,
                                     size_t& contextPos, size_t totalContextSize, size_t segmentIndex) {
    if (varIndex == 0 || varIndex > strings.size()) return false;
    
    std::u16string varContent = strings[varIndex - 1].value;
    
    if (modifier == FLAG_ANYOF) {
        // Match any single character from the variable at current position
        if (remainingContext.empty()) {
            return false;
        }
        
        // Get the first character from remaining context
        size_t charsConsumed = 0;
        char32_t ch = utils::utf16ToChar32(remainingContext, charsConsumed);
        
        // Check if character is in the variable
        for (size_t i = 0; i < varContent.size(); ++i) {
            char16_t varCh = varContent[i];
            if (static_cast<char32_t>(varCh) == ch) {
                // Capture the matched character and its position in the variable
                std::u16string matchedChar = utils::utf32ToUtf16(ch);
                captures.emplace_back(matchedChar, i, segmentIndex);
                contextPos += charsConsumed;  // Advance position
                return true;
            }
        }
        return false;
        
    } else if (modifier == FLAG_NANYOF) {
        // Match any character NOT in the variable at current position
        if (remainingContext.empty()) {
            return false;
        }
        
        size_t charsConsumed = 0;
        char32_t ch = utils::utf16ToChar32(remainingContext, charsConsumed);
        
        // Check if character is NOT in the variable
        for (char16_t varCh : varContent) {
            if (static_cast<char32_t>(varCh) == ch) {
                return false;
            }
        }
        
        // Capture the character
        captures.emplace_back(utils::utf32ToUtf16(ch), 0, segmentIndex);
        contextPos += charsConsumed;  // Advance position
        return true;
        
    } else {
        // Simple variable match - check if remaining context starts with variable content
        if (remainingContext.size() < varContent.size()) {
            return false;
        }
        
        if (remainingContext.substr(0, varContent.size()) != varContent) {
            return false;
        }
        
        contextPos += varContent.size();  // Advance position
        return true;
    }
}

bool Matcher::matchVariable(uint16_t varIndex, uint16_t modifier, const std::u16string& context,
                           const std::vector<StringEntry>& strings, std::vector<Capture>& captures) {
    if (varIndex == 0 || varIndex > strings.size()) return false;
    
    std::u16string varContent = strings[varIndex - 1].value;
    
    if (modifier == FLAG_ANYOF) {
        // Match any character from the variable
        if (!context.empty()) {
            // Get the last character from the context
            size_t charsConsumed = 0;
            char32_t lastChar = utils::utf16ToChar32(context.substr(context.size() - 1), charsConsumed);
            
            // Check if last character is in the variable
            for (size_t i = 0; i < varContent.size(); ++i) {
                char16_t ch = varContent[i];
                if (static_cast<char32_t>(ch) == lastChar) {
                    // Capture the matched character and its position
                    std::u16string matchedChar = utils::utf32ToUtf16(lastChar);
                    captures.emplace_back(matchedChar, i);
                    return true;
                }
            }
        }
        return false;
    } else if (modifier == FLAG_NANYOF) {
        // Match any character NOT in the variable
        if (!context.empty()) {
            size_t charsConsumed = 0;
            char32_t lastChar = utils::utf16ToChar32(context.substr(context.size() - 1), charsConsumed);
            
            // Check if last character is NOT in the variable
            for (char16_t ch : varContent) {
                if (static_cast<char32_t>(ch) == lastChar) {
                    return false;
                }
            }
            
            // Capture the character
            captures.emplace_back(utils::utf32ToUtf16(lastChar), 0);
            return true;
        }
        return false;
    } else {
        // Simple variable match
        size_t matchStart = 0;
        return matchString(varContent, context, matchStart);
    }
}

bool Matcher::matchVirtualKey(const std::vector<VirtualKey>& keys, const Input& input) {
    if (keys.empty()) return false;
    
    // Check if the input matches the virtual key combination
    for (const auto& key : keys) {
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
            // Check if the key code matches (compare internal VK codes)
            if (input.keyCode != key) {
                return false;
            }
        }
    }
    
    return true;
}

bool Matcher::matchAny(const std::u16string& context, std::vector<Capture>& captures) {
    if (context.empty()) return false;
    
    // Get last character
    size_t charsConsumed = 0;
    char32_t lastChar = utils::utf16ToChar32(context.substr(context.size() - 1), charsConsumed);
    
    // Check if it's in the ANY range (ASCII printable excluding space)
    if (utils::isAnyCharacter(lastChar)) {
        captures.emplace_back(utils::utf32ToUtf16(lastChar), 0);
        return true;
    }
    
    return false;
}

std::u16string Matcher::processVariable(uint16_t varIndex, const std::vector<StringEntry>& strings) {
    if (varIndex > 0 && varIndex <= strings.size()) {
        return strings[varIndex - 1].value;
    }
    return u"";
}

std::u16string Matcher::processReference(uint16_t refNum, const std::vector<Capture>& captures) {
    if (refNum > 0 && refNum <= captures.size()) {
        return captures[refNum - 1].value;
    }
    return u"";
}

std::u16string Matcher::processSegmentReference(uint16_t segmentNum, const std::vector<Capture>& captures) {
    // Find the capture that came from the specified segment
    for (const auto& capture : captures) {
        if (capture.segmentIndex == segmentNum) {
            return capture.value;
        }
    }
    return u"";
}

size_t Matcher::calculatePatternLength(const std::vector<uint16_t>& opcodes, 
                                      const std::vector<StringEntry>& strings) {
    size_t length = 0;  // Count in UTF-16 characters
    size_t opcodeIndex = 0;
    
    while (opcodeIndex < opcodes.size()) {
        uint16_t opcode = opcodes[opcodeIndex];
        
        switch (opcode) {
            case OP_STRING: {
                if (opcodeIndex + 1 >= opcodes.size()) break;
                uint16_t strLength = opcodes[++opcodeIndex];
                if (opcodeIndex + strLength >= opcodes.size()) break;
                
                // Count characters in the string (strLength is already the character count)
                length += strLength;
                
                // Skip the string data
                opcodeIndex += strLength + 1;
                break;
            }
            
            case OP_VARIABLE: {
                if (opcodeIndex + 1 >= opcodes.size()) break;
                uint16_t varIndex = opcodes[++opcodeIndex];
                
                // Check for modifier (ANYOF/NANYOF)
                if (opcodeIndex + 1 < opcodes.size() && opcodes[opcodeIndex + 1] == OP_MODIFIER) {
                    opcodeIndex += 2;  // Skip OP_MODIFIER and modifier value
                    // ANYOF/NANYOF matches exactly 1 character
                    length += 1;
                } else {
                    // Regular variable - get its content character length
                    if (varIndex > 0 && varIndex <= strings.size()) {
                        std::u16string varContent = strings[varIndex - 1].value;
                        length += varContent.size();  // UTF-16 string size = character count
                    }
                }
                opcodeIndex++;
                break;
            }
            
            case OP_ANY: {
                // ANY matches exactly 1 character
                length += 1;
                opcodeIndex++;
                break;
            }
            
            case OP_SWITCH: {
                // State switches don't contribute to pattern length
                opcodeIndex += 2;
                break;
            }
            
            case OP_PREDEFINED: {
                // Virtual keys don't contribute to text pattern length
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
    
    return length;
}

bool Matcher::matchPatternSegmented(const std::vector<RuleSegment>& segments, const std::u16string& context,
                                   const Input& input, const std::vector<StringEntry>& strings,
                                   std::vector<Capture>& captures, size_t& matchedLength) {
    matchedLength = 0;
    
    // Step 1: Calculate the expected pattern length from segments
    size_t expectedPatternLength = 0;
    for (const auto& segment : segments) {
        expectedPatternLength += calculateSegmentLength(segment, strings);
    }
    
    // Step 2: Extract suffix from context that matches the pattern length
    std::u16string matchContext;
    
    if (expectedPatternLength == 0) {
        matchContext = u"";
    } else {
        size_t contextCharCount = context.size();
        if (contextCharCount < expectedPatternLength) {
            return false; // Context too short
        }
        matchContext = utils::utf16Substring(context, contextCharCount - expectedPatternLength, expectedPatternLength);
    }
    
    // Step 3: Match each segment sequentially
    size_t contextPos = 0;
    size_t segmentIndex = 1; // 1-based segment indexing
    
    for (const auto& segment : segments) {
        if (!matchSegment(segment, matchContext, contextPos, strings, captures, segmentIndex)) {
            return false;
        }
        segmentIndex++;
    }
    
    // Verify we consumed the entire match context
    if (contextPos != matchContext.size()) {
        return false;
    }
    
    // Set matched length to the expected pattern length (what was actually consumed from the suffix)
    // This is critical for proper text replacement in the engine
    matchedLength = expectedPatternLength;
    
    return true;
}

bool Matcher::matchSegment(const RuleSegment& segment, const std::u16string& matchContext,
                          size_t& contextPos, const std::vector<StringEntry>& strings,
                          std::vector<Capture>& captures, size_t segmentIndex) {
    
    switch (segment.type) {
        case SegmentType::String: {
            // Extract string from opcodes
            if (segment.opcodes.size() < 2) return false;
            uint16_t length = segment.opcodes[1];
            if (segment.opcodes.size() < 2 + length) return false;
            
            std::u16string str;
            for (size_t i = 2; i < 2 + length; ++i) {
                str.push_back(static_cast<char16_t>(segment.opcodes[i]));
            }
            
            // Check match at current position
            if (contextPos + str.size() > matchContext.size()) return false;
            if (matchContext.substr(contextPos, str.size()) != str) return false;
            
            // Capture the matched string
            captures.emplace_back(str, 0, segmentIndex);
            contextPos += str.size();
            return true;
        }
        
        case SegmentType::AnyOfVariable: {
            // Extract variable index from opcodes: OP_VARIABLE varIndex OP_MODIFIER FLAG_ANYOF
            if (segment.opcodes.size() < 4) return false;
            uint16_t varIndex = segment.opcodes[1];
            
            if (contextPos >= matchContext.size()) return false;
            if (varIndex == 0 || varIndex > strings.size()) return false;
            
            std::u16string varContent = strings[varIndex - 1].value;
            
            // Get character at current position
            size_t charsConsumed = 0;
            char32_t ch = utils::utf16ToChar32(matchContext.substr(contextPos), charsConsumed);
            
            // Check if character is in variable
            for (size_t i = 0; i < varContent.size(); ++i) {
                char16_t varCh = varContent[i];
                if (static_cast<char32_t>(varCh) == ch) {
                    // Capture with position in variable and segment index
                    std::u16string matchedChar = utils::utf32ToUtf16(ch);
                    captures.emplace_back(matchedChar, i, segmentIndex);
                    contextPos += charsConsumed;
                    return true;
                }
            }
            return false;
        }
        
        case SegmentType::NotAnyOfVariable: {
            // Similar to AnyOfVariable but inverse logic
            if (segment.opcodes.size() < 4) return false;
            uint16_t varIndex = segment.opcodes[1];
            
            if (contextPos >= matchContext.size()) return false;
            if (varIndex == 0 || varIndex > strings.size()) return false;
            
            std::u16string varContent = strings[varIndex - 1].value;
            
            size_t charsConsumed = 0;
            char32_t ch = utils::utf16ToChar32(matchContext.substr(contextPos), charsConsumed);
            
            // Check character is NOT in variable
            for (char16_t varCh : varContent) {
                if (static_cast<char32_t>(varCh) == ch) {
                    return false;
                }
            }
            
            // Capture the character
            captures.emplace_back(utils::utf32ToUtf16(ch), 0, segmentIndex);
            contextPos += charsConsumed;
            return true;
        }
        
        case SegmentType::Variable: {
            // Simple variable reference
            if (segment.opcodes.size() < 2) return false;
            uint16_t varIndex = segment.opcodes[1];
            
            if (varIndex == 0 || varIndex > strings.size()) return false;
            std::u16string varContent = strings[varIndex - 1].value;
            
            if (contextPos + varContent.size() > matchContext.size()) return false;
            if (matchContext.substr(contextPos, varContent.size()) != varContent) return false;
            
            captures.emplace_back(varContent, 0, segmentIndex);
            contextPos += varContent.size();
            return true;
        }
        
        case SegmentType::Any: {
            if (contextPos >= matchContext.size()) return false;
            
            size_t charsConsumed = 0;
            char32_t ch = utils::utf16ToChar32(matchContext.substr(contextPos), charsConsumed);
            
            if (!utils::isAnyCharacter(ch)) return false;
            
            captures.emplace_back(utils::utf32ToUtf16(ch), 0, segmentIndex);
            contextPos += charsConsumed;
            return true;
        }
        
        case SegmentType::Reference:
            // References only appear in RHS, not in LHS matching
            return true;
            
        default:
            // Other segment types (State, VirtualKey) are handled at rule level
            return true;
    }
}

size_t Matcher::calculateSegmentLength(const RuleSegment& segment, const std::vector<StringEntry>& strings) {
    switch (segment.type) {
        case SegmentType::String: {
            if (segment.opcodes.size() < 2) return 0;
            return segment.opcodes[1]; // String length
        }
        case SegmentType::AnyOfVariable:
        case SegmentType::NotAnyOfVariable:
        case SegmentType::Any:
            return 1; // These match exactly 1 character
        case SegmentType::Variable: {
            if (segment.opcodes.size() < 2) return 0;
            uint16_t varIndex = segment.opcodes[1];
            if (varIndex > 0 && varIndex <= strings.size()) {
                return strings[varIndex - 1].value.size();
            }
            return 0;
        }
        case SegmentType::Reference:
            return 0; // References are for output generation, not input matching
            
        default:
            return 0; // State, VirtualKey don't contribute to text length
    }
}

} // namespace keymagic