#pragma once

#include <stddef.h>
#include <stdint.h>

#include "bus_device.h"

/// Must at least be 64 bytes and divisble by 64
constexpr uint64_t BLOCK_SIZE = 64;

typedef struct {
    uint64_t base;
    uint64_t range;
} block_range;

inline static bool intersects(block_range left, block_range right) {
    if (left.base <= right.base && left.base + left.range >= right.base ||
        right.base <= left.base && right.base + right.range >= left.base) {
        return true;
    }

    return false;
}

//
// typedef struct device_tree {
//     addr_range range;
//     struct device_tree* next;
// }device_tree;

constexpr size_t MAX_DEVICES = 30;

typedef struct address_bus {
    block_range ranges[MAX_DEVICES];
    bus_device* devices[MAX_DEVICES];
    size_t num_devices;
} address_bus;

void addr_bus_init(address_bus* bus);
// Cleans up all resources owned by this address bus
void addr_bus_destroy(address_bus* bus);

// Returns false if the device could not be added. No two addr_ranges may
// intersect. If the device you attempt to add intersects with any other address
// range this functin will fail and return false
//
// The address bus takes ownership over the device, it must be allocated on the
// heap.
//
// If the function returns true then the caller can safely free the device
bool addr_bus_add_device(address_bus* bus, bus_device* device);
// Prints the state of the address bus to stdout
void addr_bus_pretty_print(address_bus* bus);
// Evaluates if this range intersects any devices already in the bus
bool addr_bus_intersects(address_bus* bus, block_range range);

bool addr_bus_write_block(address_bus* bus, uint64_t addr, void* out);
bool addr_bus_read_block(address_bus* bus, uint64_t addr, void* in);

/// Locks the block at `addr` so no other threads can use it. Returns the
/// pointer to the block and the device that owns the block.
///
/// Returns NULL if `addr` isn't in a device range
uint8_t* addr_bus_lock_block(address_bus* bus, uint64_t addr, bus_device** out);

/// Unlocks the block at `addr` and lets other threads access it
///
/// This function must be preceded by addr_bus_lock_block, and you must pass in
/// the same addrses in both functions and pass the same device that was
/// returned from addr_bus_lock_block
void addr_bus_unlock_block(address_bus* bus, uint64_t addr, bus_device* device);
