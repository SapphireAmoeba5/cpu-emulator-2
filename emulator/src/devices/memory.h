#pragma once

#include "bus_device.h"


/// Implements address bus functinality for use in a CPU core
typedef struct memory memory;


// Creates a new memory to pass into the address bus
// Heap allocated, returns NULL if failed
memory* memory_create();

bool memory_init(bus_device* memory, size_t range);
bool memory_destroy(bus_device* memory);


// Returns false if there was an error laoding the value
bool memory_read_8(bus_device* bus, uint64_t addr, uint64_t* out);
// Returns false if there was an error laoding the value
bool memory_read_4(bus_device* bus, uint64_t addr, uint32_t* out);
// Returns false if there was an error laoding the value
bool memory_read_2(bus_device* bus, uint64_t addr, uint16_t* out);
// Returns false if there was an error laoding the value
bool memory_read_1(bus_device* bus, uint64_t addr, uint8_t* out);
// Read n bytes into `out`
bool memory_read_n(bus_device* bus, uint64_t addr, void* out, uint64_t n);
bool memory_read_block(bus_device* bus, uint64_t addr, void* out);

// Returns false if there was an error writing the value
bool memory_write_8(bus_device* bus, uint64_t addr, uint64_t value);
// Returns false if there was an error writing the value
bool memory_write_4(bus_device* bus, uint64_t addr, uint32_t value);
// Returns false if there was an error writing the value
bool memory_write_2(bus_device* bus, uint64_t addr, uint16_t value);
// Returns false if there was an error writing the value
bool memory_write_1(bus_device* bus, uint64_t addr, uint8_t value);
// Write n bytes from `in`
bool memory_write_n(bus_device* bus, uint64_t addr, void* in, uint64_t n);
bool memory_write_block(bus_device* bus, uint64_t addr, void* in);
