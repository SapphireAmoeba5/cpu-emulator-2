#include "bus_device.h"

typedef struct test_device {
    BUS_DEVICE_IMPL 
    uint64_t size;
} test_device;

void test_device_init(test_device* dev, uint64_t size);
