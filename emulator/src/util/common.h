#pragma once
#include <stdint.h>

#define UNUSED(x) (void)(x)
#define BIT(n) 1 << (n)

// Gets the next power of two greater than or equal to `n`
inline static uint64_t next_p2(uint64_t n) {
    n -= 1;
    n |= n >> 1;
    n |= n >> 2;
    n |= n >> 4;
    n |= n >> 8;
    n |= n >> 16;
    n |= n >> 32;
    n += 1;
    return n;
}
