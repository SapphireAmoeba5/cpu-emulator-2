#pragma once

#include "util/common.h"
#include "util/types.h"
#include <stdio.h>
#include <string.h>

#if defined(__GNUC__) || defined(__clang__)
#define CTZ(x) __builtin_ctzg((x))
#endif

// Interrupt flags
// The struct is organized like a 256 bit integer so it can be interpreted as
// one on platforms that support 256 bit integers. We assume the host
// architecture is little-endian
typedef struct iflag {
    alignas(256 / 8) u64 low_low;
    u64 low_high;
    u64 high_low;
    u64 high_high;
} iflag;

// Ensure no unexpected padding
static_assert(sizeof(iflag) == sizeof(u64) * 4, "iflag has unexpected padding");

/// Returns the number of trailing zeros (from the LSB to MSB). Undefined
/// behavior if there are no bits set
inline static u8 iflag_trailing_zeros(iflag* flag) {
    u64 values[4];
    constexpr int bits = sizeof(values[0]) * 8;
    memcpy(&values, flag, sizeof(values));

    for (int i = 0; i < 4; i++) {
        u64 value = values[i];
        if (value) {
#ifdef CTZ
            return i * bits + CTZ(value);
#else
            for (int j = 0; j < bits; j++) {
                if ((value >> j & 1) == 1) {
                    return i * bits + j;
                }
            }
#endif
        }
    }

    UNREACHABLE();
}

// Returns true if there are any bits set
inline static bool iflag_non_zero(iflag* flags) {
    return flags->high_high || flags->high_low || flags->low_high ||
           flags->low_low;
}

inline static void iflag_set_bit(iflag* flags, u8 bit) {
    if (bit < 64) {
        flags->low_low |= ((u64)1 << bit);
    } else if (bit < 128) {
        bit -= 64;
        flags->low_high |= ((u64)1 << bit);
    } else if (bit < 192) {
        bit -= 128;
        flags->high_low |= ((u64)1 << bit);
    } else {
        bit -= 192;
        flags->high_high |= ((u64)1 << bit);
    }
}

inline static void iflag_unset_bit(iflag* flags, u8 bit) {
    if (bit < 64) {
        flags->low_low &= ~((u64)1 << bit);
    } else if (bit < 128) {
        bit -= 64;
        flags->low_high &= ~((u64)1 << bit);
    } else if (bit < 192) {
        bit -= 128;
        flags->high_low &= ~((u64)1 << bit);
    } else {
        bit -= 192;
        flags->high_high &= ~((u64)1 << bit);
    }
}

/// Does an element wise bitwise-and between the two iflag's and returns true if
/// the result is non-zero, otherwise false.
inline static bool iflag_apply_mask(iflag* flags_a, iflag* flags_b) {
    u64 a[4];
    u64 b[4];
    memcpy(a, flags_a, sizeof(*flags_a));
    memcpy(b, flags_b, sizeof(*flags_b));

    for (int i = 0; i < 4; i++) {
        if (a[i] & b[i]) {
            return true;
        }
    }

    return false;
}
