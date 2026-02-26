#include "address_bus.h"
#include <assert.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

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

/// Returns the addrss mask of `size`.
/// Size must be a power of two
inline static uint64_t addrmask(uint64_t size) { return ~(size - 1); }

bool address_intersects(uint64_t a, uint64_t a_size, uint64_t b,
                        uint64_t b_size) {
    return (a <= b && a + a_size >= b) || (b <= a && b + b_size >= a);
}

inline static uint64_t align_next(uint64_t n, uint64_t alignment) {
    return (n + alignment - 1) / alignment * alignment;
}

/// Aligns `n` to the previous alignment that is aligned by `alignment`
/// If `n` is already aligned then this function returns `n`
inline static uint64_t align_prev(uint64_t n, uint64_t alignment) {
    return n - (n % alignment);
}

void address_bus_init(address_bus* bus) { memset(bus, 0, sizeof(*bus)); }

bool address_bus_write_n(address_bus* bus, uint64_t address, void* src,
                         uint64_t n) {
    for (uint64_t i = 0; i < bus->mem_count; i++) {
        memory* mem = &bus->mem[i];

        if ((address & addrmask(mem->size)) == mem->base_address) {
            if (address + n > mem->base_address + mem->size) {
                return false;
            }

            uint64_t offset = address & (mem->size - 1);
            memcpy(mem->memory + offset, src, n);

            return true;
        }
    }

    // TODO: Iterate over devices
    return false;
}

bool address_bus_read_n(address_bus* bus, uint64_t address, void* dst,
                        uint64_t n) {
    for (uint64_t i = 0; i < bus->mem_count; i++) {
        memory* mem = &bus->mem[i];

        if ((address & addrmask(mem->size)) == mem->base_address) {
            if (address + n >= mem->base_address + mem->size) {
                return false;
            }

            uint64_t offset = address & (mem->size - 1);
            memcpy(dst, mem->memory + offset, n);

            return true;
        }
    }

    // TODO: Iterate over devices
    return false;
}

bool address_bus_add_memory(address_bus* bus, uint64_t address, uint64_t size) {
    // There is no power of two greater than this representable in an 8 byte
    // integer
    if (size > 0x8000000000000000) {
        return false;
    }

    if (bus->mappings_count >= MAX_DEVICES) {
        return false;
    }

    // Technically trying to add zero bytes is always successful and we don't
    // need to do any work
    if (size == 0) {
        return true;
    }

    size = next_p2(size);

    uint64_t base_address = align_prev(address, size);
    char* block = malloc(size);

    bus_mapping mapping = {
        .device = block,
        .base_address = base_address,
        .size = size,
        .actual_size = size,
        .is_memory = true,
    };

    if (bus->mappings_count == 0) {
        bus->mappings[bus->mappings_count++] = mapping;
        return true;

    } else if (mapping.base_address <= bus->mappings[0].base_address) {
        bus_mapping* first = &bus->mappings[0];
        if (address_intersects(mapping.base_address, mapping.size,
                               first->base_address, first->size)) {
            return false;
        }
        memmove(first + 1, first, bus->mappings_count * sizeof(*first));
        bus->mappings[0] = mapping;
        bus->mappings_count += 1;

        return true;
    }

    for (uint64_t i = 0; i < bus->mappings_count - 1; i++) {
        bus_mapping left = bus->mappings[i];
        bus_mapping right = bus->mappings[i + 1];

        if (address_intersects(base_address, size, left.base_address,
                               left.size)) {
            return false;
        } else if (address_intersects(base_address, size, right.base_address,
                                      right.size)) {
            return false;
        }

        if (base_address > left.base_address &&
            base_address < right.base_address) {
            memmove(&bus->mappings[i + 1], &bus->mappings[i],
                    bus->mappings_count * sizeof(bus->mappings[0]));

            bus->mappings[i] = mapping;
            return true;
        }
    }

    return false;
}

bool address_bus_add_device(address_bus* bus, bus_device* device) { abort(); }

void address_bus_finalize_mapping(address_bus* bus) {
    for (uint64_t i = 0; i < bus->mappings_count; i++) {
        bus_mapping* mapping = &bus->mappings[i];

        if (mapping->is_memory) {
            memory mem = {
                .memory = mapping->device,
                .base_address = mapping->base_address,
                .size = mapping->size,
            };
            bus->mem[bus->mem_count++] = mem;
        } else {
            mmio mmio = {
                .device = mapping->device,
                .base_address = mapping->base_address,
                .size = mapping->size,
                .actual_size = mapping->actual_size,
            };
            bus->mmio[bus->mmio_count++] = mmio;
        }
    }
}

void address_bus_debug_print_mapping(address_bus* bus) {
    printf("%llu devices:\n", bus->mappings_count);
    for (uint64_t i = 0; i < bus->mappings_count; i++) {
        uint64_t base = bus->mappings[i].base_address;
        uint64_t end =
            bus->mappings[i].base_address + bus->mappings[i].size - 1;

        printf("%08llx %08llx (%llu %llu) ", base, end, base, end);
        if (bus->mappings[i].is_memory) {
            printf("M\n");
        } else {
            printf("D\n");
        }
    }
}

void address_bus_debug_print_finalized(address_bus* bus) {
    if (bus->mem_count > 0) {
        printf("Memory regions:\n");
        for (uint64_t i = 0; i < bus->mem_count; i++) {
            uint64_t base = bus->mem[i].base_address;
            uint64_t end = bus->mem[i].base_address + bus->mem[i].size - 1;

            printf("%08llx %08llx (%llu %llu)\n", base, end, base, end);
        }
    }

    if (bus->mmio_count > 0) {
        printf("\nMMIO regions:\n");
        for (uint64_t i = 0; i < bus->mmio_count; i++) {
            uint64_t base = bus->mmio[i].base_address;
            uint64_t end = bus->mmio[i].base_address + bus->mmio[i].size - 1;

            printf("%08llx %08llx (%llu %llu)\n", base, end, base, end);
        }
    }
}
