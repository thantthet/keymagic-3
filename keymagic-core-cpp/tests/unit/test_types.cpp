#include <gtest/gtest.h>
#include <keymagic/types.h>

using namespace keymagic;

TEST(TypesTest, ModifiersTest) {
    Modifiers mods;
    EXPECT_FALSE(mods.hasAnyModifier());
    
    mods.shift = true;
    EXPECT_TRUE(mods.hasAnyModifier());
    
    // Test Right Alt detection
    Modifiers altGr(false, false, true, false, false);
    EXPECT_TRUE(altGr.isRightAlt(true));
    
    Modifiers ctrlAlt(false, true, true, false, false);
    EXPECT_TRUE(ctrlAlt.isRightAlt(true));
    EXPECT_FALSE(ctrlAlt.isRightAlt(false));
}

TEST(TypesTest, OutputHelpers) {
    auto output = Output::Insert("test", "test");
    EXPECT_EQ(ActionType::Insert, output.action);
    EXPECT_EQ("test", output.text);
    EXPECT_EQ("test", output.composingText);
    EXPECT_TRUE(output.isProcessed);
    
    auto deleteOut = Output::Delete(3, "remaining");
    EXPECT_EQ(ActionType::BackspaceDelete, deleteOut.action);
    EXPECT_EQ(3, deleteOut.deleteCount);
    EXPECT_EQ("remaining", deleteOut.composingText);
    
    auto deleteInsert = Output::DeleteAndInsert(2, "new", "final");
    EXPECT_EQ(ActionType::BackspaceDeleteAndInsert, deleteInsert.action);
    EXPECT_EQ(2, deleteInsert.deleteCount);
    EXPECT_EQ("new", deleteInsert.text);
    EXPECT_EQ("final", deleteInsert.composingText);
}

TEST(TypesTest, KM2Version) {
    KM2Version v15(1, 5);
    EXPECT_TRUE(v15.isCompatible());
    EXPECT_TRUE(v15.hasInfoSection());
    EXPECT_TRUE(v15.hasRightAltOption());
    
    KM2Version v14(1, 4);
    EXPECT_TRUE(v14.isCompatible());
    EXPECT_TRUE(v14.hasInfoSection());
    EXPECT_FALSE(v14.hasRightAltOption());
    
    KM2Version v13(1, 3);
    EXPECT_TRUE(v13.isCompatible());
    EXPECT_FALSE(v13.hasInfoSection());
    EXPECT_FALSE(v13.hasRightAltOption());
    
    KM2Version invalid(2, 0);
    EXPECT_FALSE(invalid.isCompatible());
}