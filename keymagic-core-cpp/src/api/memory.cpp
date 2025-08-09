#include <cstdlib>

namespace keymagic {

void* allocateMemory(size_t size) {
    return std::malloc(size);
}

void freeMemory(void* ptr) {
    std::free(ptr);
}

} // namespace keymagic