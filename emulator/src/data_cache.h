#pragma once

#include "address_bus.h"
#include <stdint.h>
#include <stdio.h>

// Must be a power of two
constexpr uint64_t CACHE_LINES = 1;

constexpr uint64_t UNOCCUPIED_LINE = (uint64_t)-1;

inline static uint64_t get_cache_line(uint64_t address) {
    // uint64_t cache_line =
    //     (address * 0x9e3779b97f4a7c15) & (CACHE_LINES - 1);
    uint64_t cache_line = address & (CACHE_LINES - 1);
    return cache_line;
}

static inline uint64_t align_to_block_boundary(uint64_t address) {
    return address - (address % BLOCK_SIZE);
}

typedef struct {
    uint8_t lines[CACHE_LINES][BLOCK_SIZE];
    // The address that is cached. To mark the cache as unoccupied it is set to
    // all 1's
    uint64_t addresses[CACHE_LINES];
    // Dirty flag, set to true if the cache line was written to
    bool dirty[CACHE_LINES];
} cache;

bool cache_write_8(cache* cache, address_bus* bus, uint64_t address,
                   uint64_t value);
bool cache_write_4(cache* cache, address_bus* bus, uint64_t address,
                   uint32_t value);
bool cache_write_2(cache* cache, address_bus* bus, uint64_t address,
                   uint16_t value);
bool cache_write_1(cache* cache, address_bus* bus, uint64_t address,
                   uint8_t value);

bool cache_read_8(cache* cache, address_bus* bus, uint64_t address,
                  uint64_t* dest);
bool cache_read_4(cache* cache, address_bus* bus, uint64_t address,
                  uint32_t* dest);
bool cache_read_2(cache* cache, address_bus* bus, uint64_t address,
                  uint16_t* dest);
bool cache_read_1(cache* cache, address_bus* bus, uint64_t address,
                  uint8_t* dest);
