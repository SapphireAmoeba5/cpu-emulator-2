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

    .device_read_8 = memory_read_8,
    .device_read_4 = memory_read_4,
    .device_read_2 = memory_read_2,
    .device_read_1 = memory_read_1,
    .device_read_n = memory_read_n,
    .device_read_block = memory_read_block,

    .device_write_8 = memory_write_8,
    .device_write_4 = memory_write_4,
    .device_write_2 = memory_write_2,
    .device_write_1 = memory_write_1,
    .device_write_n = memory_write_n,
    .device_write_block = memory_write_block,
};

typedef struct memory {
    const bus_device_vtable* vtable;
    uint8_t* data;
    size_t length;
} memory;

memory* memory_create() {
    memory* mem = malloc(sizeof(memory));

    if(mem == NULL) {
        return NULL;
    }

    mem->vtable = &memory_vtable;

    return mem;
}

bool memory_init(bus_device* bus, size_t length) {
    memory* mem = (memory*)bus;

    mem->length = length;
    mem->data = malloc(length);

    if(mem->data == NULL) {
        return false;
    }

    return true;
}

bool memory_destroy(bus_device* bus) {
    memory* mem = (memory*)bus;
    free(mem->data);
    return false;
}

bool memory_read_8(bus_device* bus, uint64_t addr, uint64_t* out) {
    memory* mem = (memory*)bus;
    memcpy(out, &mem->data[addr], sizeof(*out));
    return true;
}
bool memory_read_4(bus_device* bus, uint64_t addr, uint32_t* out) {
    memory* mem = (memory*)bus;
    memcpy(out, &mem->data[addr], sizeof(*out));
    return true;
}
bool memory_read_2(bus_device* bus, uint64_t addr, uint16_t* out) {
    memory* mem = (memory*)bus;
    memcpy(out, &mem->data[addr], sizeof(*out));
    return true;
}
bool memory_read_1(bus_device* bus, uint64_t addr, uint8_t* out) {
    memory* mem = (memory*)bus;
    memcpy(out, &mem->data[addr], sizeof(*out));
    return true;
}

bool memory_read_n(bus_device *bus, uint64_t addr, void *out, uint64_t n) {
    memory* mem = (memory*)bus;
    memcpy(out, &mem->data[addr], n);
    return true;
}

bool memory_read_block(bus_device *bus, uint64_t addr, void *out) {
    memory* mem = (memory*)bus;
    memcpy(out, &mem->data[addr], BLOCK_SIZE);
    return true;
}

bool memory_write_8(bus_device* bus, uint64_t addr, uint64_t value) {
    memory* mem = (memory*)bus;
    memcpy(&mem->data[addr], &value, sizeof(value));
    return true;
}
bool memory_write_4(bus_device* bus, uint64_t addr, uint32_t value) {
    memory* mem = (memory*)bus;
    memcpy(&mem->data[addr], &value, sizeof(value));
    return true;

}
bool memory_write_2(bus_device* bus, uint64_t addr, uint16_t value) {
    memory* mem = (memory*)bus;
    memcpy(&mem->data[addr], &value, sizeof(value));
    return true;

}
bool memory_write_1(bus_device* bus, uint64_t addr, uint8_t value) {
    memory* mem = (memory*)bus;
    memcpy(&mem->data[addr], &value, sizeof(value));
    return true;
}

bool memory_write_n(bus_device *bus, uint64_t addr, void *in, uint64_t n) {
    memory* mem = (memory*)bus;
    memcpy(&mem->data[addr], in, n);
    return true;
}

bool memory_write_block(bus_device *bus, uint64_t addr, void *in) {
    memory* mem = (memory*)bus;
    memcpy(&mem->data[addr], in, BLOCK_SIZE);
    return true;
}
