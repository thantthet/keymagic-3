#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "../include/keymagic_ffi.h"

void test_engine() {
    printf("Testing KeyMagic FFI interface...\n");
    
    // Get version
    const char* version = keymagic_windows_version();
    printf("KeyMagic Windows version: %s\n", version);
    
    // Create engine
    EngineHandle* engine = keymagic_engine_new();
    if (!engine) {
        printf("Failed to create engine\n");
        return;
    }
    printf("Engine created successfully\n");
    
    // Test key processing (without loading a keyboard)
    ProcessKeyOutput output;
    memset(&output, 0, sizeof(output));
    
    // Test pressing 'A' key
    KeyMagicResult result = keymagic_engine_process_key(
        engine,
        65,  // VK_KEY_A
        0,   // no shift
        0,   // no ctrl
        0,   // no alt
        0,   // no caps
        &output
    );
    
    if (result == KEYMAGIC_SUCCESS) {
        printf("Key processed successfully\n");
        printf("  Action type: %d\n", output.action_type);
        printf("  Consumed: %d\n", output.consumed);
        if (output.text) {
            printf("  Text: %s\n", output.text);
            keymagic_free_string(output.text);
        }
        if (output.composing_text) {
            printf("  Composing: %s\n", output.composing_text);
            keymagic_free_string(output.composing_text);
        }
    } else {
        printf("Failed to process key: %d\n", result);
    }
    
    // Get composition
    char* composition = keymagic_engine_get_composition(engine);
    if (composition) {
        printf("Current composition: %s\n", composition);
        keymagic_free_string(composition);
    }
    
    // Reset engine
    result = keymagic_engine_reset(engine);
    printf("Engine reset: %s\n", result == KEYMAGIC_SUCCESS ? "success" : "failed");
    
    // Free engine
    keymagic_engine_free(engine);
    printf("Engine freed\n");
}

int main() {
    test_engine();
    return 0;
}