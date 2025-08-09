#ifndef KEYMAGIC_KM2_VALIDATOR_H
#define KEYMAGIC_KM2_VALIDATOR_H

#include <keymagic/km2_format.h>

namespace keymagic {

class KM2Validator {
public:
    static bool validate(const KM2File& km2);
    static bool validateOpcodes(const std::vector<uint16_t>& opcodes, size_t stringCount);
};

} // namespace keymagic

#endif // KEYMAGIC_KM2_VALIDATOR_H