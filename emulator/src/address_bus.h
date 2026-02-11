#pragma once

#include <stddef.h>
#include <stdint.h>

#include "bus_device.h"

/// Must at least be 64 bytes and divisble by 16
constexpr uint64_t BLOCK_SIZE = 64;

typedef struct addr_range {
    uint64_t address;
    uint64_t range;
} addr_range;

inline static bool intersects(addr_range left, addr_range right) {
    if (left.address <= right.address &&
            left.address + left.range >= right.address ||
        right.address <= left.address &&
            right.address + right.range >= left.address) {
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
    addr_range ranges[MAX_DEVICES];
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
bool addr_bus_add_device(address_bus* bus, addr_range range,
                         bus_device* device);
// Prints the state of the address bus to stdout
void addr_bus_pretty_print(address_bus* bus);
// Evaluates if this range intersects any devices already in the bus
bool addr_bus_intersects(address_bus* bus, addr_range range);

// Returns false if there was an error reading the value
bool addr_bus_read_8(address_bus* bus, uint64_t addr, uint64_t* out);
// Returns false if there was an error reading the value
bool addr_bus_read_4(address_bus* bus, uint64_t addr, uint32_t* out);
// Returns false if there was an error reading the value
bool addr_bus_read_2(address_bus* bus, uint64_t addr, uint16_t* out);
// Returns false if there was an error reading the value
bool addr_bus_read_1(address_bus* bus, uint64_t addr, uint8_t* out);
// Returns false if there was an error reading the value
bool addr_bus_read_n(address_bus* bus, uint64_t addr, void* out, uint64_t n);
// Returns false if there was an error reading the value.
// Reads `BLOCK_SIZE` bytes
bool addr_bus_read_block(address_bus* bus, uint64_t addr, void* out);

// Returns false if there was an error writing the value
bool addr_bus_write_8(address_bus* bus, uint64_t addr, uint64_t value);
// Returns false if there was an error writing the value
bool addr_bus_write_4(address_bus* bus, uint64_t addr, uint32_t value);
// Returns false if there was an error writing the value
bool addr_bus_write_2(address_bus* bus, uint64_t addr, uint16_t value);
// Returns false if there was an error writing the value
bool addr_bus_write_1(address_bus* bus, uint64_t addr, uint8_t value);
// Returns false if there was an error writing the value
bool addr_bus_write_n(address_bus* bus, uint64_t addr, void* in, uint64_t n);
// Returns false if there was an error reading the value.
// Writes `BLOCK_SIZE` bytes
bool addr_bus_read_block(address_bus* bus, uint64_t addr, void* in);
