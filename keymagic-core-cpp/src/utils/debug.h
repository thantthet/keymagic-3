#ifndef KEYMAGIC_DEBUG_H
#define KEYMAGIC_DEBUG_H

#include <string>
#include <cstdint>

namespace keymagic {
namespace utils {

// Debug logging (only active in debug builds)
void debugLog(const std::string& message);

// Hex dump for binary data debugging
std::string hexDump(const uint8_t* data, size_t length);

} // namespace utils
} // namespace keymagic

#endif // KEYMAGIC_DEBUG_H