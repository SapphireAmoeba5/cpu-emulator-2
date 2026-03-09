#include "address_bus.h"
#include "free_list.h"
#include "job_queue.h"
#include "thread_pool.h"
#include "util/barrier.h"
#include "util/common.h"
#include <assert.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <threads.h>

/// Returns the addrss mask of `size`.
/// Size must be a power of two
inline static uint64_t addrmask(uint64_t size) { return ~(size - 1); }

static inline bool address_intersects(uint64_t a, uint64_t a_size, uint64_t b,
                                      uint64_t b_size) {
    return (a <= b && a + a_size > b) || (b <= a && b + b_size > a);
}

static inline bool insert_mapping(address_bus* bus, bus_mapping mapping) {
    uint64_t base_address = mapping.base_address;
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

        // If `freelist_allocate` succeeds then the address range we are trying
        // to insert is garunteed to not overlap with any other address ranges
        if (base_address > left.base_address &&
            base_address < right.base_address) {
            memmove(&bus->mappings[i + 1], &bus->mappings[i],
                    bus->mappings_count * sizeof(bus->mappings[0]));

            bus->mappings[i] = mapping;
            return true;
        }
    }

    bus->mappings[bus->mappings_count++] = mapping;
    return true;
}

void address_bus_init(address_bus* bus) {
    memset(bus, 0, sizeof(*bus));
    thread_pool_init(&bus->pool);
    freelist_init(&bus->list);
}

void address_bus_deinit(address_bus* bus) {
    // TODO: freelist_deinit
    thread_pool_deinit(&bus->pool);
}

bool address_bus_write_n(address_bus* bus, uint64_t address, void* src,
                         uint64_t n) {
    for (uint64_t i = 0; i < bus->mem_count; i++) {
        memory* mem = &bus->mem[i];

        if ((address & addrmask(mem->size)) == mem->base_address) {
            uint64_t offset = address & (mem->size - 1);
            if (address + n > mem->base_address + mem->size) {
                return false;
            } else {
                // Use native CPU memory ordering.
                // Armv8 and x64 have equal or stricter memory ordering rules
                // than the architecture we are emulating which garuntees the
                // same as armv8.
                // TODO: At compile time check for if the platform supports
                // unaligned loads/stores to ensure that this runs properly
                BARRIER();
                memcpy(mem->memory + offset, src, n);
            }
            return true;
        }
    }

    for (uint64_t i = 0; i < bus->mmio_count; i++) {
        mmio* mmio = &bus->mmio[i];
        if ((address & addrmask(mmio->size)) == mmio->base_address) {
            if (address + n > mmio->base_address + mmio->size) {
                return false;
            }

            u64 offset = address & (mmio->size - 1);
            // If the address itself is greater than the mmio device's actual
            // size then just reeturn with `dst` set to all zero
            if (address < mmio->base_address + mmio->actual_size) {
                if (address + n >= mmio->base_address + mmio->actual_size) {
                    n = (mmio->base_address + mmio->actual_size) - address;
                }

                mmio->device->vtable->device_write_n(mmio->device, offset, src,
                                                     n);
            }

            return true;
        }
    }

    return false;
}

bool address_bus_read_n(address_bus* bus, uint64_t address, void* dst,
                        uint64_t n) {
    for (uint64_t i = 0; i < bus->mem_count; i++) {
        memory* mem = &bus->mem[i];

        if ((address & addrmask(mem->size)) == mem->base_address) {
            uint64_t offset = address & (mem->size - 1);
            if (address + n >= mem->base_address + mem->size) {
                return false;
            } else {
                // Use native CPU memory ordering.
                // Armv8 and x64 have equal or stricter memory ordering rules
                // than the architecture we are emulating which garuntees the
                // same as armv8.
                // TODO: At compile time check for if the platform supports
                // unaligned loads/stores to ensure that this runs properly
                memcpy(dst, mem->memory + offset, n);
                BARRIER();
            }

            return true;
        }
    }

    for (uint64_t i = 0; i < bus->mmio_count; i++) {
        mmio* mmio = &bus->mmio[i];
        if ((address & addrmask(mmio->size)) == mmio->base_address) {
            if (address + n > mmio->base_address + mmio->size) {
                return false;
            }

            u64 offset = address & (mmio->size - 1);
            memset(dst, 0, n);
            // If the address itself is greater than the mmio device's actual
            // size then just reeturn with `dst` set to all zero
            if (address < mmio->base_address + mmio->actual_size) {
                if (address + n >= mmio->base_address + mmio->actual_size) {
                    n = (mmio->base_address + mmio->actual_size) - address;
                }

                mmio->device->vtable->device_read_n(mmio->device, offset, dst,
                                                    n);
            }

            return true;
        }
    }
    return false;
}

bool address_bus_add_memory(address_bus* bus, uint64_t size) {
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

    bool success = false;
    uint64_t base_address = freelist_allocate(&bus->list, size, size, &success);

    if (!success) {
        return false;
    }

    char* block = malloc(size);

    bus_mapping mapping = {
        .device = block,
        .base_address = base_address,
        .size = size,
        .actual_size = size,
        .is_memory = true,
    };

    return insert_mapping(bus, mapping);
}

bool address_bus_add_device(address_bus* bus, bus_device* device) {
    if (bus->mappings_count >= MAX_DEVICES) {
        return false;
    }

    uint64_t actual_size;
    if (!device->vtable->device_init(device, &actual_size)) {
        return false;
    }

    // The size cannot be aligned to the next power of two since it will
    // overflow
    if (actual_size > 0x8000000000000000) {
        device->vtable->device_destroy(device);
    }

    if (actual_size == 0) {
        return true;
    }

    uint64_t size = next_p2(actual_size);

    bool success = false;
    uint64_t base_address = freelist_allocate(&bus->list, size, size, &success);

    if (!success) {
        return false;
    }

    bus_mapping mapping = {
        .device = device,
        .base_address = base_address,
        .size = size,
        .actual_size = actual_size,
        .is_memory = false,
    };

    return insert_mapping(bus, mapping);
}

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
