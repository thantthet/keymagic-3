#include "debug.h"
#include <iostream>
#include <sstream>
#include <iomanip>

namespace keymagic {
namespace utils {

void debugLog(const std::string& message) {
#ifdef KEYMAGIC_DEBUG
    std::cerr << "[KeyMagic] " << message << std::endl;
#endif
}

std::string hexDump(const uint8_t* data, size_t length) {
    std::stringstream ss;
    
    for (size_t i = 0; i < length; i += 16) {
        // Address
        ss << std::setfill('0') << std::setw(8) << std::hex << i << "  ";
        
        // Hex bytes
        for (size_t j = 0; j < 16; ++j) {
            if (i + j < length) {
                ss << std::setfill('0') << std::setw(2) << std::hex 
                   << static_cast<int>(data[i + j]) << " ";
            } else {
                ss << "   ";
            }
            if (j == 7) ss << " ";
        }
        
        ss << " |";
        
        // ASCII representation
        for (size_t j = 0; j < 16 && i + j < length; ++j) {
            uint8_t byte = data[i + j];
            if (byte >= 32 && byte <= 126) {
                ss << static_cast<char>(byte);
            } else {
                ss << ".";
            }
        }
        
        ss << "|" << std::endl;
    }
    
    return ss.str();
}

} // namespace utils
} // namespace keymagic