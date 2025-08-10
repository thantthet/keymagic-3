#include "loader.h"
#include "../utils/utf8.h"
#include "../utils/debug.h"
#include <fstream>
#include <cstring>
#include <stdexcept>

namespace keymagic {

class KM2LoaderImpl {
public:
    static std::unique_ptr<KM2File> loadFromFile(const std::string& path) {
        std::ifstream file(path, std::ios::binary | std::ios::ate);
        if (!file) {
            return nullptr;
        }
        
        size_t fileSize = file.tellg();
        file.seekg(0, std::ios::beg);
        
        std::vector<uint8_t> buffer(fileSize);
        if (!file.read(reinterpret_cast<char*>(buffer.data()), fileSize)) {
            return nullptr;
        }
        
        return loadFromMemory(buffer.data(), buffer.size());
    }
    
    static std::unique_ptr<KM2File> loadFromMemory(const uint8_t* data, size_t dataLen) {
        if (!data || dataLen < sizeof(FileHeader_1_3)) {
            return nullptr;
        }
        
        auto km2 = std::make_unique<KM2File>();
        size_t offset = 0;
        
        // Read and validate header
        if (!readHeader(data, dataLen, offset, km2->header)) {
            return nullptr;
        }
        
        // Read strings section
        if (!readStrings(data, dataLen, offset, km2->header.stringCount, km2->strings)) {
            return nullptr;
        }
        
        // Read info section (v1.4+)
        if (km2->hasInfoSection()) {
            std::vector<InfoEntry> infoEntries;
            if (!readInfoSection(data, dataLen, offset, km2->header.infoCount, infoEntries)) {
                return nullptr;
            }
            km2->metadata = Metadata(infoEntries);
        }
        
        // Read rules section
        if (!readRules(data, dataLen, offset, km2->header.ruleCount, km2->rules)) {
            return nullptr;
        }
        
        return km2;
    }
    
private:
    static bool readHeader(const uint8_t* data, size_t dataLen, size_t& offset, FileHeader& header) {
        // Try v1.5 header first
        if (dataLen >= sizeof(FileHeader) + 1) {  // +1 for padding byte
            std::memcpy(&header, data, sizeof(FileHeader));
            if (header.isValid() && header.minorVersion == 5) {
                // v1.5 files have an extra padding byte after the header
                offset = sizeof(FileHeader) + 1;  // Skip the padding byte
                return header.isCompatibleVersion();
            }
        }
        
        // Try v1.4 header
        if (dataLen >= sizeof(FileHeader_1_4)) {
            FileHeader_1_4 h14;
            std::memcpy(&h14, data, sizeof(FileHeader_1_4));
            if (h14.magicCode[0] == 'K' && h14.magicCode[1] == 'M' && 
                h14.magicCode[2] == 'K' && h14.magicCode[3] == 'L' && 
                h14.majorVersion == 1 && h14.minorVersion == 4) {
                
                // Convert to v1.5 header
                std::memcpy(header.magicCode, h14.magicCode, 4);
                header.majorVersion = h14.majorVersion;
                header.minorVersion = h14.minorVersion;
                header.stringCount = h14.stringCount;
                header.infoCount = h14.infoCount;
                header.ruleCount = h14.ruleCount;
                header.layoutOptions.trackCaps = h14.layoutOptions.trackCaps;
                header.layoutOptions.autoBksp = h14.layoutOptions.autoBksp;
                header.layoutOptions.eat = h14.layoutOptions.eat;
                header.layoutOptions.posBased = h14.layoutOptions.posBased;
                header.layoutOptions.rightAlt = 1; // Default for v1.4
                
                offset = sizeof(FileHeader_1_4);
                return true;
            }
        }
        
        // Try v1.3 header
        if (dataLen >= sizeof(FileHeader_1_3)) {
            FileHeader_1_3 h13;
            std::memcpy(&h13, data, sizeof(FileHeader_1_3));
            if (h13.magicCode[0] == 'K' && h13.magicCode[1] == 'M' && 
                h13.magicCode[2] == 'K' && h13.magicCode[3] == 'L' && 
                h13.majorVersion == 1 && h13.minorVersion == 3) {
                
                // Convert to v1.5 header
                std::memcpy(header.magicCode, h13.magicCode, 4);
                header.majorVersion = h13.majorVersion;
                header.minorVersion = h13.minorVersion;
                header.stringCount = h13.stringCount;
                header.infoCount = 0; // No info section in v1.3
                header.ruleCount = h13.ruleCount;
                header.layoutOptions.trackCaps = h13.layoutOptions.trackCaps;
                header.layoutOptions.autoBksp = h13.layoutOptions.autoBksp;
                header.layoutOptions.eat = h13.layoutOptions.eat;
                header.layoutOptions.posBased = h13.layoutOptions.posBased;
                header.layoutOptions.rightAlt = 1; // Default for v1.3
                
                offset = sizeof(FileHeader_1_3);
                return true;
            }
        }
        
        return false;
    }
    
    static bool readStrings(const uint8_t* data, size_t dataLen, size_t& offset, 
                          uint16_t count, std::vector<StringEntry>& strings) {
        strings.reserve(count);
        
        for (uint16_t i = 0; i < count; ++i) {
            if (offset + 2 > dataLen) {
                return false;
            }
            
            // Read string length (little-endian)
            uint16_t length = data[offset] | (data[offset + 1] << 8);
            offset += 2;
            
            if (offset + length * 2 > dataLen) {
                return false;
            }
            
            // Read UTF-16LE string
            std::u16string str = utils::utf16leToUtf16(&data[offset], length * 2);
            offset += length * 2;
            
            strings.emplace_back(str);
        }
        
        return true;
    }
    
    static bool readInfoSection(const uint8_t* data, size_t dataLen, size_t& offset,
                               uint16_t count, std::vector<InfoEntry>& entries) {
        entries.reserve(count);
        
        for (uint16_t i = 0; i < count; ++i) {
            if (offset + 6 > dataLen) return false;
            
            // Read info ID (4 bytes)
            uint8_t id[4];
            std::memcpy(id, &data[offset], 4);
            offset += 4;
            
            // Read data length (little-endian)
            uint16_t length = data[offset] | (data[offset + 1] << 8);
            offset += 2;
            
            if (offset + length > dataLen) return false;
            
            // Read data
            std::vector<uint8_t> infoData(data + offset, data + offset + length);
            offset += length;
            
            entries.emplace_back(id, infoData);
        }
        
        return true;
    }
    
    static bool readRules(const uint8_t* data, size_t dataLen, size_t& offset,
                        uint16_t count, std::vector<BinaryRule>& rules) {
        rules.reserve(count);
        
        for (uint16_t i = 0; i < count; ++i) {
            BinaryRule rule;
            
            // Read LHS
            if (!readRuleSide(data, dataLen, offset, rule.lhs)) {
                return false;
            }
            
            // Read RHS
            if (!readRuleSide(data, dataLen, offset, rule.rhs)) {
                return false;
            }
            
            rules.push_back(std::move(rule));
        }
        
        return true;
    }
    
    static bool readRuleSide(const uint8_t* data, size_t dataLen, size_t& offset,
                            std::vector<uint16_t>& opcodes) {
        if (offset + 2 > dataLen) {
            return false;
        }
        
        // Read length (in 16-bit units/words, NOT bytes!)
        // The Rust implementation multiplies this by 2 to get bytes
        uint16_t wordLength = data[offset] | (data[offset + 1] << 8);
        uint16_t byteLength = wordLength * 2;  // Convert to bytes
        
        offset += 2;
        
        if (offset + byteLength > dataLen) {
            return false;
        }
        
        // Read opcodes
        if (byteLength == 0) {
            // Empty rule side is valid (e.g., for NULL output)
            return true;
        }
        
        size_t opcodeCount = byteLength / 2;
        opcodes.reserve(opcodeCount);
        
        for (size_t i = 0; i < opcodeCount; ++i) {
            uint16_t opcode = data[offset] | (data[offset + 1] << 8);
            offset += 2;
            opcodes.push_back(opcode);
        }
        
        return true;
    }
};

// Metadata implementation
Metadata::Metadata(const std::vector<InfoEntry>& entries) {
    for (const auto& entry : entries) {
        std::array<uint8_t, 4> key;
        std::copy(entry.id, entry.id + 4, key.begin());
        entries_[key] = entry.data;
    }
}

const std::vector<uint8_t>* Metadata::get(const uint8_t id[4]) const {
    std::array<uint8_t, 4> key;
    std::copy(id, id + 4, key.begin());
    auto it = entries_.find(key);
    return it != entries_.end() ? &it->second : nullptr;
}

std::string Metadata::getString(const uint8_t id[4]) const {
    const auto* data = get(id);
    if (!data || data->empty()) {
        return std::string();
    }
    
    // The data is actually UTF-8, not UTF-16LE as originally thought
    // Based on the KM2 dump, info strings are stored as UTF-8
    return std::string(data->begin(), data->end());
}

bool Metadata::has(const uint8_t id[4]) const {
    std::array<uint8_t, 4> key;
    std::copy(id, id + 4, key.begin());
    return entries_.find(key) != entries_.end();
}

// KM2Loader public interface
std::unique_ptr<KM2File> KM2Loader::loadFromFile(const std::string& path) {
    return KM2LoaderImpl::loadFromFile(path);
}

std::unique_ptr<KM2File> KM2Loader::loadFromMemory(const uint8_t* data, size_t dataLen) {
    return KM2LoaderImpl::loadFromMemory(data, dataLen);
}

bool KM2Loader::validateFile(const std::string& path) {
    auto km2 = loadFromFile(path);
    return km2 && km2->isValid();
}

bool KM2Loader::validateMemory(const uint8_t* data, size_t dataLen) {
    auto km2 = loadFromMemory(data, dataLen);
    return km2 && km2->isValid();
}

} // namespace keymagic