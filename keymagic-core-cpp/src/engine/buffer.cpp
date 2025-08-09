#include <keymagic/engine.h>
#include "../utils/utf8.h"

namespace keymagic {

// Buffer utility functions for composing text management

std::string removeLastNCharacters(const std::string& text, size_t n) {
    if (n == 0) return text;
    
    size_t charCount = utils::utf8CharCount(text);
    if (n >= charCount) return "";
    
    return utils::utf8Substring(text, 0, charCount - n);
}

std::string getLastNCharacters(const std::string& text, size_t n) {
    size_t charCount = utils::utf8CharCount(text);
    if (n >= charCount) return text;
    
    return utils::utf8Substring(text, charCount - n, n);
}

bool endsWithString(const std::string& text, const std::string& suffix) {
    if (suffix.size() > text.size()) return false;
    return text.substr(text.size() - suffix.size()) == suffix;
}

std::string replaceLastOccurrence(const std::string& text, const std::string& search, const std::string& replace) {
    size_t pos = text.rfind(search);
    if (pos == std::string::npos) return text;
    
    std::string result = text;
    result.replace(pos, search.size(), replace);
    return result;
}

} // namespace keymagic