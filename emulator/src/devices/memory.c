#include "memory.h"

#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

#include "address_bus.h"
#include "bus_device.h"

const bus_device_vtable memory_vtable = {
    .device_init = memory_init,
    .device_destroy = memory_destroy,

    .device_read_block = memory_read_block,
    .device_write_block = memory_write_block,

    .device_lock_block = memory_lock_block,
    .device_unlock_block = memory_unlock_block,
};

memory* memory_create(uint64_t blocks) {
    memory* mem = malloc(sizeof(memory));
    mem->vtable = &memory_vtable;
    mem->type = device_memory;

    if(mem == NULL) {
        return NULL;
    }

    mem->data = malloc(blocks * BLOCK_SIZE);
    mem->length = blocks * BLOCK_SIZE;

    if(mem->data == NULL) {
        free(mem);
        return NULL;
    }

    return mem;
}

bool memory_init(bus_device* device, uint64_t* requested_range) {
    memory* mem = (memory*)device;
    *requested_range = mem->length / BLOCK_SIZE;
    return true;
}

bool memory_destroy(bus_device* device) {
    memory* mem = (memory*)device;
    free(mem->data);
    mem->length = 0;
    return false;
}

bool memory_read_block(bus_device *device, uint64_t block, void *out) {
    memory* mem = (memory*)device;
    memcpy(out, &mem->data[block * BLOCK_SIZE], BLOCK_SIZE);
    return true;
}

bool memory_write_block(bus_device *device, uint64_t block, void *in) {
    memory* mem = (memory*)device;
    memcpy(&mem->data[block * BLOCK_SIZE], in, BLOCK_SIZE);
    return true;
}

uint8_t* memory_lock_block(bus_device* device, uint64_t block) {
    // TODO: Make sure in the future we add locks here
    memory* mem = (memory*)device;
    uint64_t off = block * BLOCK_SIZE;

    return &mem->data[off];
}

void memory_unlock_block(bus_device* device, uint64_t block) {
    // Nothing for now
}
