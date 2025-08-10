#ifndef KEYMAGIC_MATCHER_H
#define KEYMAGIC_MATCHER_H

#include <keymagic/engine.h>
#include <keymagic/km2_format.h>

namespace keymagic {

class Matcher {
public:
    Matcher();
    ~Matcher();
    
    bool matchRule(const ProcessedRule& rule, MatchContext& context, 
                  const Input& input, const std::vector<StringEntry>& strings);
    
    RuleApplicationResult applyRule(const ProcessedRule& rule, const MatchContext& context,
                                   const std::vector<StringEntry>& strings);
    
private:
    bool matchPattern(const std::vector<uint16_t>& opcodes, const std::u16string& context,
                     const Input& input, const std::vector<StringEntry>& strings,
                     std::vector<Capture>& captures, size_t& matchedLength);
    
    bool matchPatternSegmented(const std::vector<RuleSegment>& segments, const std::u16string& context,
                              const Input& input, const std::vector<StringEntry>& strings,
                              std::vector<Capture>& captures, size_t& matchedLength);
    
    std::u16string generateOutput(const std::vector<uint16_t>& opcodes, 
                                 const std::vector<Capture>& captures,
                                 const std::vector<StringEntry>& strings,
                                 std::vector<int>& newStates);
    
    std::u16string generateOutputSegmented(const std::vector<RuleSegment>& segments,
                                          const std::vector<Capture>& captures,
                                          const std::vector<StringEntry>& strings,
                                          std::vector<int>& newStates);
    
    bool matchString(const std::u16string& pattern, const std::u16string& context, size_t& matchStart);
    bool matchVariable(uint16_t varIndex, uint16_t modifier, const std::u16string& context,
                      const std::vector<StringEntry>& strings, std::vector<Capture>& captures);
    bool matchVariableSequential(uint16_t varIndex, uint16_t modifier, 
                                 const std::u16string& remainingContext,
                                 const std::vector<StringEntry>& strings, 
                                 std::vector<Capture>& captures,
                                 size_t& contextPos, size_t totalContextSize, size_t segmentIndex);
    bool matchVirtualKey(const std::vector<VirtualKey>& keys, const Input& input);
    bool matchAny(const std::u16string& context, std::vector<Capture>& captures);
    
    std::u16string processVariable(uint16_t varIndex, const std::vector<StringEntry>& strings);
    std::u16string processReference(uint16_t refNum, const std::vector<Capture>& captures);
    std::u16string processSegmentReference(uint16_t segmentNum, const std::vector<Capture>& captures);
    
    size_t calculatePatternLength(const std::vector<uint16_t>& opcodes, 
                                  const std::vector<StringEntry>& strings);
    
    // Segment-based matching functions
    bool matchSegment(const RuleSegment& segment, const std::u16string& matchContext,
                     size_t& contextPos, const std::vector<StringEntry>& strings,
                     std::vector<Capture>& captures, size_t segmentIndex);
    
    size_t calculateSegmentLength(const RuleSegment& segment, const std::vector<StringEntry>& strings);
};

} // namespace keymagic

#endif // KEYMAGIC_MATCHER_H