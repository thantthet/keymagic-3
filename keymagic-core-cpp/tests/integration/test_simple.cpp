#include <gtest/gtest.h>
#include <keymagic/km2_format.h>
#include <keymagic/keymagic.h>
#include "../../src/km2/loader.h"
#include "../../src/matching/matcher.h"
#include <iostream>

using namespace keymagic;

TEST(SimpleTest, DirectMatcherTest) {
    // Test the matcher directly
    Matcher matcher;
    
    // Create a simple rule
    ProcessedRule rule;
    rule.lhsOpcodes = {OP_STRING, 2, 'k', 'a'};
    rule.rhsOpcodes = {OP_STRING, 1, 0x1000};
    
    // Create context
    MatchContext context;
    context.context = "ka";
    
    // Create empty strings vector
    std::vector<StringEntry> strings;
    
    // Create a dummy state
    EngineState state;
    
    // Apply the rule
    Output output = matcher.applyRule(rule, context, strings, &state);
    
    std::cout << "Direct matcher test:" << std::endl;
    std::cout << "  Action: " << static_cast<int>(output.action) << std::endl;
    std::cout << "  Text: '" << output.text << "'" << std::endl;
    std::cout << "  Composing: '" << output.composingText << "'" << std::endl;
    std::cout << "  Delete count: " << output.deleteCount << std::endl;
    
    EXPECT_EQ(output.composingText, "က");
}