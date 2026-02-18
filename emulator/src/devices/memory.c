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

bool memory_init(bus_device* bus, uint64_t* requested_range) {
    memory* mem = (memory*)bus;
    *requested_range = mem->length / BLOCK_SIZE;
    return true;
}

bool memory_destroy(bus_device* bus) {
    memory* mem = (memory*)bus;
    free(mem->data);
    mem->length = 0;
    return false;
}

bool memory_read_block(bus_device *bus, uint64_t block, void *out) {
    memory* mem = (memory*)bus;
    memcpy(out, &mem->data[block * BLOCK_SIZE], BLOCK_SIZE);
    return true;
}

bool memory_write_block(bus_device *bus, uint64_t block, void *in) {
    memory* mem = (memory*)bus;
    memcpy(&mem->data[block * BLOCK_SIZE], in, BLOCK_SIZE);
    return true;
}
