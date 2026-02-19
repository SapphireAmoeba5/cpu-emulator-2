#pragma once

#include "bus_device.h"


typedef struct memory {
    BUS_DEVICE_IMPL
    uint8_t* data;
    size_t length;
} memory;

memory* memory_create(uint64_t length);

bool memory_init(bus_device* memory, uint64_t* requested_range);
bool memory_destroy(bus_device* memory);


bool memory_read_block(bus_device* bus, uint64_t addr, void* out);
bool memory_write_block(bus_device* bus, uint64_t addr, void* in);

uint8_t* memory_lock_block(bus_device* device, uint64_t block);
void memory_unlock_block(bus_device* device, uint64_t block);
