#include "data_cache.h"
#include "address_bus.h"
#include <stdio.h>
#include <string.h>

inline static bool validate_cache_line(cache* cache, address_bus* bus,
                                       uint64_t aligned_address,
                                       uint64_t cache_line) {
    if (cache->addresses[cache_line] != aligned_address) {
        if (cache->dirty[cache_line] &&
            !addr_bus_write_block(bus, cache->addresses[cache_line],
                                  &cache->lines[cache_line])) {
            cache->addresses[cache_line] = UNOCCUPIED_LINE;
            cache->dirty[cache_line] = false;
            return false;
        }

        cache->dirty[cache_line] = false;

        if (!addr_bus_read_block(bus, aligned_address,
                                 &cache->lines[cache_line])) {
            printf("Failure\n");
            cache->addresses[cache_line] = UNOCCUPIED_LINE;
            return false;
        }

        cache->addresses[cache_line] = aligned_address;
    }

    return true;
}

bool cache_read_8(cache* cache, address_bus* bus, uint64_t address,
                  uint64_t* ptr) {
    uint64_t cache_aligned = align_to_block_boundary(address);
    uint64_t cache_line = get_cache_line(cache_aligned);

    if (!validate_cache_line(cache, bus, cache_aligned, cache_line)) {
        return false;
    }

    uint64_t offset = address - cache_aligned;
    if (address + 8 > cache_aligned + BLOCK_SIZE) {
        uint64_t next_aligned = align_to_block_boundary(address + 8);

        int remaining = next_aligned - address;
        memcpy(ptr, &cache->lines[cache_line][offset], remaining);

        cache_line = get_cache_line(next_aligned);

        if (!validate_cache_line(cache, bus, next_aligned, cache_line)) {
            return false;
        }

        memcpy((char*)ptr + (8 - remaining), &cache->lines[cache_line],
               8 - remaining);
    } else {
        memcpy(ptr, &cache->lines[cache_line][offset], 8);
    }

    return true;
}

bool cache_read_4(cache* cache, address_bus* bus, uint64_t address,
                  uint32_t* ptr) {
    uint64_t cache_aligned = align_to_block_boundary(address);
    uint64_t cache_line = get_cache_line(cache_aligned);

    if (!validate_cache_line(cache, bus, cache_aligned, cache_line)) {
        return false;
    }

    uint64_t offset = address - cache_aligned;
    if (address + 4 > cache_aligned + BLOCK_SIZE) {
        uint64_t next_aligned = align_to_block_boundary(address + 8);

        int remaining = next_aligned - address;
        memcpy(ptr, &cache->lines[cache_line][offset], remaining);

        cache_line = get_cache_line(next_aligned);

        if (!validate_cache_line(cache, bus, next_aligned, cache_line)) {
            return false;
        }

        memcpy((char*)ptr + (4 - remaining), &cache->lines[cache_line],
               4 - remaining);
    } else {
        memcpy(ptr, &cache->lines[cache_line][offset], 4);
    }

    return true;
}

bool cache_read_2(cache* cache, address_bus* bus, uint64_t address,
                  uint16_t* ptr) {
    uint64_t cache_aligned = align_to_block_boundary(address);
    uint64_t cache_line = get_cache_line(cache_aligned);

    if (!validate_cache_line(cache, bus, cache_aligned, cache_line)) {
        return false;
    }

    uint64_t offset = address - cache_aligned;
    if (address + 2 > cache_aligned + BLOCK_SIZE) {
        uint64_t next_aligned = align_to_block_boundary(address + 8);

        int remaining = next_aligned - address;
        memcpy(ptr, &cache->lines[cache_line][offset], 1);

        cache_line = get_cache_line(next_aligned);

        if (!validate_cache_line(cache, bus, next_aligned, cache_line)) {
            return false;
        }

        memcpy((char*)ptr + 1, &cache->lines[cache_line], 1);
    } else {
        memcpy(ptr, &cache->lines[cache_line][offset], 2);
    }

    return true;
}

bool cache_read_1(cache* cache, address_bus* bus, uint64_t address,
                  uint8_t* ptr) {
    uint64_t cache_aligned = align_to_block_boundary(address);
    uint64_t cache_line = get_cache_line(cache_aligned);

    if (!validate_cache_line(cache, bus, cache_aligned, cache_line)) {
        return false;
    }

    uint64_t offset = address - cache_aligned;
    memcpy(ptr, &cache->lines[cache_line][offset], 1);

    return true;
}
