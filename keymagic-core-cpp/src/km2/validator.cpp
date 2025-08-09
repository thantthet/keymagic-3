#include "validator.h"
#include <keymagic/km2_format.h>

namespace keymagic {

bool KM2Validator::validate(const KM2File& km2) {
    // Validate header
    if (!km2.isValid()) {
        return false;
    }
    
    // Validate rule opcodes
    for (const auto& rule : km2.rules) {
        if (!validateOpcodes(rule.lhs, km2.strings.size()) ||
            !validateOpcodes(rule.rhs, km2.strings.size())) {
            return false;
        }
    }
    
    return true;
}

bool KM2Validator::validateOpcodes(const std::vector<uint16_t>& opcodes, size_t stringCount) {
    size_t i = 0;
    
    while (i < opcodes.size()) {
        uint16_t opcode = opcodes[i];
        
        switch (opcode) {
            case OP_STRING:
                if (i + 1 >= opcodes.size()) return false;
                {
                    uint16_t length = opcodes[i + 1];
                    if (i + 1 + length >= opcodes.size()) return false;
                    i += 2 + length;
                }
                break;
                
            case OP_VARIABLE:
            case OP_REFERENCE:
            case OP_PREDEFINED:
            case OP_MODIFIER:
            case OP_SWITCH:
                if (i + 1 >= opcodes.size()) return false;
                i += 2;
                break;
                
            case OP_AND:
            case OP_ANY:
                i++;
                break;
                
            default:
                // Unknown opcode
                return false;
        }
    }
    
    return true;
}

} // namespace keymagic