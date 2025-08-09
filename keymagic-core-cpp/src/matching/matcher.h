#ifndef KEYMAGIC_MATCHER_H
#define KEYMAGIC_MATCHER_H

#include <keymagic/engine.h>
#include <keymagic/km2_format.h>

namespace keymagic {

class Matcher {
public:
    Matcher();
    ~Matcher();
    
    bool matchRule(const ProcessedRule& rule, const MatchContext& context, 
                  const Input& input, const std::vector<StringEntry>& strings);
    
    Output applyRule(const ProcessedRule& rule, const MatchContext& context,
                    const std::vector<StringEntry>& strings, EngineState* state);
    
private:
    bool matchPattern(const std::vector<uint16_t>& opcodes, const std::string& context,
                     const Input& input, const std::vector<StringEntry>& strings,
                     std::vector<Capture>& captures);
    
    std::string generateOutput(const std::vector<uint16_t>& opcodes, 
                              const std::vector<Capture>& captures,
                              const std::vector<StringEntry>& strings,
                              std::vector<int>& newStates);
    
    bool matchString(const std::string& pattern, const std::string& context, size_t& matchStart);
    bool matchVariable(uint16_t varIndex, uint16_t modifier, const std::string& context,
                      const std::vector<StringEntry>& strings, std::vector<Capture>& captures);
    bool matchVirtualKey(const std::vector<VirtualKey>& keys, const Input& input);
    bool matchAny(const std::string& context, std::vector<Capture>& captures);
    
    std::string processVariable(uint16_t varIndex, const std::vector<StringEntry>& strings);
    std::string processReference(uint16_t refNum, const std::vector<Capture>& captures);
};

} // namespace keymagic

#endif // KEYMAGIC_MATCHER_H