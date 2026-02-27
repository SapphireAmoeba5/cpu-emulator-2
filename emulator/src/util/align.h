#pragma once
#include <stdint.h>

/// Rounds `n` up to the next, or equal value that is aligned to `alignment`
inline static uint64_t align_next(uint64_t n, uint64_t alignment) {
    return (n + alignment - 1) / alignment * alignment;
}

/// Aligns `n` to the previous alignment that is aligned by `alignment`
/// If `n` is already aligned then this function returns `n`
inline static uint64_t align_prev(uint64_t n, uint64_t alignment) {
    return n - (n % alignment);
}
