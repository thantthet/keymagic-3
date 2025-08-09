#include <keymagic/keymagic.h>
#include <keymagic/engine.h>
#include "../utils/utf8.h"
#include "../km2/loader.h"
#include <cstring>
#include <mutex>
#include <unordered_map>

// C API implementation stub
namespace {
std::mutex g_handleMutex;
}

extern "C" {

EngineHandle* keymagic_engine_new(void) {
    return nullptr;
}

void keymagic_engine_free(EngineHandle* handle) {
}

const char* keymagic_get_version(void) {
    return "1.0.0";
}

} // extern "C"