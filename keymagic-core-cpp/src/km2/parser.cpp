#include "parser.h"
#include "../utils/utf8.h"
#include <keymagic/virtual_keys.h>

namespace keymagic {

class OpcodeParser {
public:
    static std::string parseOpcodeSequence(const std::vector<uint16_t>& opcodes, 
                                          const std::vector<StringEntry>& strings) {
        std::string result;
        size_t i = 0;
        
        while (i < opcodes.size()) {
            uint16_t opcode = opcodes[i];
            
            switch (opcode) {
                case OP_STRING:
                    i = parseString(opcodes, i, result);
                    break;
                    
                case OP_VARIABLE:
                    i = parseVariable(opcodes, i, strings, result);
                    break;
                    
                case OP_REFERENCE:
                    i = parseReference(opcodes, i, result);
                    break;
                    
                case OP_PREDEFINED:
                    i = parsePredefined(opcodes, i, result);
                    break;
                    
                case OP_MODIFIER:
                    i = parseModifier(opcodes, i, result);
                    break;
                    
                case OP_AND:
                    result += " & ";
                    i++;
                    break;
                    
                case OP_ANY:
                    result += "ANY";
                    i++;
                    break;
                    
                case OP_SWITCH:
                    i = parseSwitch(opcodes, i, result);
                    break;
                    
                default:
                    // Unknown opcode, skip it
                    i++;
                    break;
            }
        }
        
        return result;
    }
    
private:
    static size_t parseString(const std::vector<uint16_t>& opcodes, size_t i, std::string& result) {
        if (i + 1 >= opcodes.size()) return opcodes.size();
        
        uint16_t length = opcodes[++i];
        if (i + length >= opcodes.size()) return opcodes.size();
        
        std::u16string str;
        for (uint16_t j = 0; j < length; ++j) {
            str.push_back(static_cast<char16_t>(opcodes[++i]));
        }
        
        result += utils::utf16ToUtf8(str);
        return i + 1;
    }
    
    static size_t parseVariable(const std::vector<uint16_t>& opcodes, size_t i, 
                               const std::vector<StringEntry>& strings, std::string& result) {
        if (i + 1 >= opcodes.size()) return opcodes.size();
        
        uint16_t varIndex = opcodes[++i];
        if (varIndex > 0 && varIndex <= strings.size()) {
            // Variables are 1-indexed
            result += "$var" + std::to_string(varIndex);
        }
        
        return i + 1;
    }
    
    static size_t parseReference(const std::vector<uint16_t>& opcodes, size_t i, std::string& result) {
        if (i + 1 >= opcodes.size()) return opcodes.size();
        
        uint16_t refNum = opcodes[++i];
        result += "$" + std::to_string(refNum);
        
        return i + 1;
    }
    
    static size_t parsePredefined(const std::vector<uint16_t>& opcodes, size_t i, std::string& result) {
        if (i + 1 >= opcodes.size()) return opcodes.size();
        
        uint16_t vkValue = opcodes[++i];
        if (vkValue == 1) {
            result += "NULL";
        } else if (VirtualKeyHelper::isValid(vkValue)) {
            result += "VK_" + VirtualKeyHelper::toString(static_cast<VirtualKey>(vkValue));
        }
        
        return i + 1;
    }
    
    static size_t parseModifier(const std::vector<uint16_t>& opcodes, size_t i, std::string& result) {
        if (i + 1 >= opcodes.size()) return opcodes.size();
        
        uint16_t modValue = opcodes[++i];
        if (modValue == FLAG_ANYOF) {
            result += "[*]";
        } else if (modValue == FLAG_NANYOF) {
            result += "[^]";
        } else {
            result += "[$" + std::to_string(modValue) + "]";
        }
        
        return i + 1;
    }
    
    static size_t parseSwitch(const std::vector<uint16_t>& opcodes, size_t i, std::string& result) {
        if (i + 1 >= opcodes.size()) return opcodes.size();
        
        uint16_t stateId = opcodes[++i];
        result += "(state_" + std::to_string(stateId) + ")";
        
        return i + 1;
    }
};

} // namespace keymagic