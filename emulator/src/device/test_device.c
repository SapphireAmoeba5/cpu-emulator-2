#include "test_device.h"
#include "bus_device.h"
#include "util/common.h"
#include <stdio.h>
#include <string.h>

const static bus_device_vtable vtable;

void test_device_init(test_device* dev, uint64_t size) {
    dev->vtable = &vtable;
    dev->size = size;    
}

bool test_device_addr_bus_init(bus_device* def, uint64_t* requested) {
    printf("Initializing test device\n");
    test_device* td = (test_device*)def;
    *requested = td->size;
    return true;
}

bool test_device_destroy(bus_device* dev) {
    printf("Destroying test device\n");
    UNUSED(dev);
    return true;
}

void test_device_read_n(bus_device* device, uint64_t address, void* dst, uint64_t n) {
    test_device* td = (test_device*)device;
    UNUSED(td);

    memset(dst, 0xff, n);

    printf("Reading %llu bytes at %llx (%llu)\n", n, address, address);
}

void test_device_write_n(bus_device* device, uint64_t address, void* dst, uint64_t n) {
    test_device* td = (test_device*)device;
    (void)td;

    printf("Writing %llu bytes at %llx (%llu)\n", n, address, address);
}


const static bus_device_vtable vtable = {
    .device_init = test_device_addr_bus_init,
    .device_destroy = test_device_destroy,
    .device_read_n = test_device_read_n,
    .device_write_n = test_device_write_n,
};
