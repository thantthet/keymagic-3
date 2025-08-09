#ifndef KEYMAGIC_KM2_FORMAT_H
#define KEYMAGIC_KM2_FORMAT_H

#include <cstdint>
#include <vector>
#include <string>
#include <array>
#include <unordered_map>
#include <memory>

namespace keymagic {

// KM2 file magic code
constexpr char KM2_MAGIC_CODE[4] = {'K', 'M', 'K', 'L'};

// Binary opcodes for KM2 format
constexpr uint16_t OP_STRING = 0x00F0;
constexpr uint16_t OP_VARIABLE = 0x00F1;
constexpr uint16_t OP_REFERENCE = 0x00F2;
constexpr uint16_t OP_PREDEFINED = 0x00F3;
constexpr uint16_t OP_MODIFIER = 0x00F4;
constexpr uint16_t OP_AND = 0x00F6;
constexpr uint16_t OP_ANY = 0x00F8;
constexpr uint16_t OP_SWITCH = 0x00F9;

// Modifier flags (used with OP_MODIFIER)
constexpr uint16_t FLAG_ANYOF = 0x00F5;   // Match any character from variable
constexpr uint16_t FLAG_NANYOF = 0x00F7;  // Match any character NOT in variable

// Info section IDs (stored as little-endian in file)
// These are the byte representations of ASCII strings
constexpr uint8_t INFO_NAME[4] = {0x6E, 0x61, 0x6D, 0x65};  // "name" in LE
constexpr uint8_t INFO_DESC[4] = {0x64, 0x65, 0x73, 0x63};  // "desc" in LE
constexpr uint8_t INFO_FONT[4] = {0x66, 0x6F, 0x6E, 0x74};  // "font" in LE
constexpr uint8_t INFO_ICON[4] = {0x69, 0x63, 0x6F, 0x6E};  // "icon" in LE
constexpr uint8_t INFO_HTKY[4] = {0x68, 0x74, 0x6B, 0x79};  // "htky" in LE

// Pack structures to match binary format exactly
#pragma pack(push, 1)

// Version 1.3 layout options (without rightAlt)
struct LayoutOptions_1_3 {
    uint8_t trackCaps;    // 0 or 1
    uint8_t autoBksp;     // 0 or 1
    uint8_t eat;          // 0 or 1
    uint8_t posBased;     // 0 or 1
};

// Version 1.3 header (without info_count)
struct FileHeader_1_3 {
    uint8_t magicCode[4];           // "KMKL"
    uint8_t majorVersion;           // 1
    uint8_t minorVersion;           // 3
    uint16_t stringCount;           // Little-endian
    uint16_t ruleCount;             // Little-endian
    LayoutOptions_1_3 layoutOptions;
};

// Version 1.4 header (with info_count but without rightAlt)
struct FileHeader_1_4 {
    uint8_t magicCode[4];           // "KMKL"
    uint8_t majorVersion;           // 1
    uint8_t minorVersion;           // 4
    uint16_t stringCount;           // Little-endian
    uint16_t infoCount;             // Little-endian
    uint16_t ruleCount;             // Little-endian
    LayoutOptions_1_3 layoutOptions;
};

// Current version (1.5) layout options
struct KM2LayoutOptions {
    uint8_t trackCaps;    // 0 or 1
    uint8_t autoBksp;     // 0 or 1
    uint8_t eat;          // 0 or 1
    uint8_t posBased;     // 0 or 1
    uint8_t rightAlt;     // 0 or 1 (v1.5+)
    
    // Default constructor
    KM2LayoutOptions() 
        : trackCaps(1)    // true
        , autoBksp(0)     // false
        , eat(0)          // false
        , posBased(0)     // false
        , rightAlt(1)     // true
    {}
    
    // Convert to boolean options
    bool getTrackCaps() const { return trackCaps != 0; }
    bool getAutoBksp() const { return autoBksp != 0; }
    bool getEat() const { return eat != 0; }
    bool getPosBased() const { return posBased != 0; }
    bool getRightAlt() const { return rightAlt != 0; }
};

// Current version (1.5) file header
struct FileHeader {
    uint8_t magicCode[4];     // "KMKL"
    uint8_t majorVersion;     // 1
    uint8_t minorVersion;     // 5
    uint16_t stringCount;     // Little-endian
    uint16_t infoCount;       // Little-endian
    uint16_t ruleCount;       // Little-endian
    KM2LayoutOptions layoutOptions;
    
    bool isValid() const {
        return magicCode[0] == 'K' && 
               magicCode[1] == 'M' && 
               magicCode[2] == 'K' && 
               magicCode[3] == 'L';
    }
    
    bool isCompatibleVersion() const {
        return majorVersion == 1 && minorVersion >= 3 && minorVersion <= 5;
    }
};

#pragma pack(pop)

// String entry in the strings section
struct StringEntry {
    std::u16string value;  // UTF-16LE in file, converted to u16string
    
    StringEntry() = default;
    explicit StringEntry(const std::u16string& val) : value(val) {}
};

// Info entry in the info section
struct InfoEntry {
    uint8_t id[4];
    std::vector<uint8_t> data;
    
    InfoEntry() { std::fill(std::begin(id), std::end(id), 0); }
    InfoEntry(const uint8_t* idBytes, const std::vector<uint8_t>& d) : data(d) {
        std::copy(idBytes, idBytes + 4, id);
    }
    
    bool isName() const { return std::equal(std::begin(id), std::end(id), INFO_NAME); }
    bool isDescription() const { return std::equal(std::begin(id), std::end(id), INFO_DESC); }
    bool isFont() const { return std::equal(std::begin(id), std::end(id), INFO_FONT); }
    bool isIcon() const { return std::equal(std::begin(id), std::end(id), INFO_ICON); }
    bool isHotkey() const { return std::equal(std::begin(id), std::end(id), INFO_HTKY); }
};

// Metadata container for info entries
class Metadata {
public:
    Metadata() = default;
    explicit Metadata(const std::vector<InfoEntry>& entries);
    
    // Get raw data by ID
    const std::vector<uint8_t>* get(const uint8_t id[4]) const;
    
    // Get data as UTF-16LE string (converted to UTF-8)
    std::string getString(const uint8_t id[4]) const;
    
    // Convenience accessors
    std::string getName() const { return getString(INFO_NAME); }
    std::string getDescription() const { return getString(INFO_DESC); }
    std::string getFontFamily() const { return getString(INFO_FONT); }
    std::string getHotkey() const { return getString(INFO_HTKY); }
    const std::vector<uint8_t>* getIcon() const { return get(INFO_ICON); }
    
    // Check if entry exists
    bool has(const uint8_t id[4]) const;
    
    // Get number of entries
    size_t size() const { return entries_.size(); }
    bool empty() const { return entries_.empty(); }
    
private:
    // Helper to convert 4-byte array to comparable key
    struct IdHash {
        size_t operator()(const std::array<uint8_t, 4>& id) const {
            return *reinterpret_cast<const uint32_t*>(id.data());
        }
    };
    
    std::unordered_map<std::array<uint8_t, 4>, std::vector<uint8_t>, IdHash> entries_;
};

// Binary rule representation
struct BinaryRule {
    std::vector<uint16_t> lhs;  // Left-hand side (pattern)
    std::vector<uint16_t> rhs;  // Right-hand side (output)
    
    BinaryRule() = default;
    BinaryRule(const std::vector<uint16_t>& l, const std::vector<uint16_t>& r)
        : lhs(l), rhs(r) {}
};

// Complete KM2 file representation
class KM2File {
public:
    KM2File() = default;
    
    // File header
    FileHeader header;
    
    // Sections
    std::vector<StringEntry> strings;
    Metadata metadata;
    std::vector<BinaryRule> rules;
    
    // Validation
    bool isValid() const {
        return header.isValid() && header.isCompatibleVersion();
    }
    
    // Get version info
    uint8_t getMajorVersion() const { return header.majorVersion; }
    uint8_t getMinorVersion() const { return header.minorVersion; }
    
    // Check feature support
    bool hasInfoSection() const {
        return header.majorVersion == 1 && header.minorVersion >= 4;
    }
    
    bool hasRightAltOption() const {
        return header.majorVersion == 1 && header.minorVersion >= 5;
    }
    
    // Get layout options as boolean flags
    bool tracksCapsLock() const { return header.layoutOptions.getTrackCaps(); }
    bool hasSmartBackspace() const { return header.layoutOptions.getAutoBksp(); }
    bool eatsAllUnusedKeys() const { return header.layoutOptions.getEat(); }
    bool isUSLayoutBased() const { return header.layoutOptions.getPosBased(); }
    bool treatsCtrlAltAsRightAlt() const { 
        return hasRightAltOption() ? header.layoutOptions.getRightAlt() : true; 
    }
};

} // namespace keymagic

#endif // KEYMAGIC_KM2_FORMAT_H