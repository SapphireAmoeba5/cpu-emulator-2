#pragma once

#include <stddef.h>
#include <stdint.h>
#include <string.h>
#include <threads.h>

#include "bus_device.h"
#include "free_list.h"
#include "job_queue.h"
#include "thread_pool.h"
#include "util/types.h"

struct Cpu;

constexpr uint64_t MAX_DEVICES = 32;

constexpr uint64_t BLOCK_SIZE = 64;
static_assert(BLOCK_SIZE >= 64);

typedef struct {
    uint64_t base;
    uint64_t range;
} memory_region;

typedef struct {
    void* device;

    // Base address this mmio mapping is stored at. It must be aligned to `size`
    uint64_t base_address;
    // The size of the address range the device takes up on the bus
    uint64_t size;
    // The actual size that the device requested
    uint64_t actual_size;

    // If this is true then `device` is a pointer to a block of allocated
    // memory directly read/writable with normal memory operation, otherwise
    // `device` is a pointer to a bus_device
    bool is_memory;
} bus_mapping;

typedef struct {
    char* memory;
    uint64_t base_address;
    uint64_t size;
} memory;

typedef struct {
    bus_device* device;
    uint64_t base_address;
    uint64_t size;
    uint64_t actual_size;
} mmio;

typedef struct {
    memory mem[MAX_DEVICES];
    uint64_t mem_count;

    mmio mmio[MAX_DEVICES];
    uint64_t mmio_count;

    bus_mapping mappings[MAX_DEVICES];
    uint64_t mappings_count;

    free_list list;
} address_bus;

/// Initializes the address bus to a proper default state
void address_bus_init(address_bus* bus);
/// This function is not thread safe. It may only be called when no other
/// threads are using this
void address_bus_deinit(address_bus* bus);

/// Thread-safe
bool address_bus_write_n(address_bus* bus, uint64_t address, void* src,
                         uint64_t n);
/// Thread-safe
bool address_bus_read_n(address_bus* bus, uint64_t address, void* dst,
                        uint64_t n);

/// Insert at least `size` bytes of memory into the bus
///
/// `size` will be rounded to the next power of two that is equal or greater
/// than `size`.
///
/// Not thread-safe
///
/// Returns:
/// If the memory was successfully added into the bus, returns true.
/// If the memory was not able to be added it will return false
/// This function will fail to add the memory if there are `MAX_DEVICES`
/// devices, if `size` will overflow when rounded to the next power of two,
/// and if the address range of the memory overlaps any other range.
bool address_bus_add_memory(address_bus* bus, uint64_t size);

/// Not thread-safe
/// Returns:
/// If the device was successfully added, returns true.
/// If the device was unable to be added, returns false.
/// The function will fail for reasons such as but not exclusive to; if there
/// are `MAX_DEVICES` devices
bool address_bus_add_device(address_bus* bus, bus_device* device);

// Finalizes the mappings into an internal structure for fast access
// Everytime you add any devices this function must be called before they can be
// accessed
//
// WARNING:
// This function is not thread safe because it will modify the same data
// structure that bus accesses use without synchronization
void address_bus_finalize_mapping(address_bus* bus);

void address_bus_debug_print_mapping(address_bus* bus);
void address_bus_debug_print_finalized(address_bus* bus);
