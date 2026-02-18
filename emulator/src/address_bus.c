#include "address_bus.h"
#include "bus_device.h"
#include "devices/memory.h"
#include <assert.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

inline static uint64_t next_aligned(uint64_t n, uint64_t alignment) {
    return (n + alignment - 1) / alignment * alignment;
}

inline static bool dispatch_init(bus_device* device, uint64_t* requested) {
    switch (device->type) {
    case device_memory:
        return memory_init(device, requested);
    case device_custom:
        return device->vtable->device_init(device, requested);
    }
}

inline static bool dispatch_destroy(bus_device* device) {
    switch (device->type) {
    case device_memory:
        return memory_destroy(device);
    case device_custom:
        return device->vtable->device_destroy(device);
    }
}
inline static bool dispatch_read_block(bus_device* device, uint64_t off,
                                       void* out) {
    switch (device->type) {
    case device_memory:
        return memory_read_block(device, off, out);
    case device_custom:
        return device->vtable->device_read_block(device, off, out);
    }
}
inline static bool dispatch_write_block(bus_device* device, uint64_t off,
                                        void* in) {
    switch (device->type) {
    case device_memory:
        return memory_write_block(device, off, in);
    case device_custom:
        return device->vtable->device_write_block(device, off, in);
    }
}

void addr_bus_init(address_bus* bus) { memset(bus, 0, sizeof(address_bus)); }
void addr_bus_destroy(address_bus* bus) {
    for (int i = 0; i < bus->num_devices; i++) {
        bus_device* device = bus->devices[i];
        dispatch_destroy(device);
        free(device);
    }
}

bool addr_bus_add_device(address_bus* bus, bus_device* device) {
    if (bus->num_devices >= MAX_DEVICES) {
        return false;
    }

    uint64_t reqested;
    if (!dispatch_init(device, &reqested)) {
        return false;
    }

    block_range range;
    range.range = reqested;
    if (bus->num_devices == 0) {
        range.base = 0;
    } else {
        block_range last = bus->ranges[bus->num_devices - 1];
        range.base = last.base + last.range + 1;
    }
    bus->devices[bus->num_devices] = device;
    bus->ranges[bus->num_devices] = range;
    bus->num_devices += 1;

    return true;
}

void addr_bus_pretty_print(address_bus* bus) {
    printf("%zu devices:\n", bus->num_devices);
    for (int i = 0; i < bus->num_devices; i++) {
        block_range range = bus->ranges[i];

        printf("%016llx %016llx (%llu %llu)\n", range.base * BLOCK_SIZE,
               (range.base + range.range) * BLOCK_SIZE, range.base * BLOCK_SIZE,
               (range.base + range.range) * BLOCK_SIZE);
    }
}

bool addr_bus_intersects(address_bus* bus, block_range range) {
    for (int i = 0; i < bus->num_devices; i++) {
        if (intersects(range, bus->ranges[i])) {
            return true;
        }
    }

    return false;
}

inline static bool address_intersects(block_range range, uint64_t addr) {
    return addr >= range.base && addr <= range.base + range.range;
}

bool addr_bus_read_block(address_bus* bus, uint64_t addr, void* in) {
    uint64_t block = addr / BLOCK_SIZE;
    for (int i = 0; i < bus->num_devices; i++) {
        block_range range = bus->ranges[i];
        bus_device* device = bus->devices[i];

        if (address_intersects(range, addr)) {
            if (addr + BLOCK_SIZE - 1 > range.base + range.range) {
                return false;
            }
            return dispatch_read_block(device, block, in);
        }
    }
    return false;
}

bool addr_bus_write_block(address_bus* bus, uint64_t addr, void* out) {
    uint64_t block = addr / BLOCK_SIZE;
    for (int i = 0; i < bus->num_devices; i++) {
        block_range range = bus->ranges[i];
        bus_device* device = bus->devices[i];

        if (address_intersects(range, addr)) {
            if (addr + BLOCK_SIZE - 1 > range.base + range.range) {
                return false;
            }
            return dispatch_write_block(device, block, out);
        }
    }
    return false;
}
