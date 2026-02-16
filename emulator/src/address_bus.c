#include "address_bus.h"
#include "bus_device.h"
#include "devices/memory.h"
#include <assert.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

inline static bool dispatch_init(bus_device* device, size_t length) {
    switch(device->type) {
        case device_memory:
            return memory_init(device, length);
        case device_custom:
            return device->vtable->device_init(device, length);
    }
}

inline static bool dispatch_destroy(bus_device* device) {
    switch(device->type) {
        case device_memory:
            return memory_destroy(device);
        case device_custom:
            return device->vtable->device_destroy(device);
    }

}

inline static bool dispatch_read_8(bus_device* bus, uint64_t off, uint64_t* out) {
    switch(bus->type) {
        case device_memory:
            return memory_read_8(bus, off, out);
        case device_custom:
            return bus->vtable->device_read_8(bus, off, out);
    }
}

inline static bool dispatch_read_4(bus_device* bus, uint64_t off, uint32_t* out) {
    switch(bus->type) {
        case device_memory:
            return memory_read_4(bus, off, out);
        case device_custom:
            return bus->vtable->device_read_4(bus, off, out);
    }

}
inline static bool dispatch_read_2(bus_device* bus, uint64_t off, uint16_t* out) {
    switch(bus->type) {
        case device_memory:
            return memory_read_2(bus, off, out);
        case device_custom:
            return bus->vtable->device_read_2(bus, off, out);
    }

}
inline static bool dispatch_read_1(bus_device* bus, uint64_t off, uint8_t* out) {
    switch(bus->type) {
        case device_memory:
            return memory_read_1(bus, off, out);
        case device_custom:
            return bus->vtable->device_read_1(bus, off, out);
    }

}
inline static bool dispatch_read_n(bus_device* bus, uint64_t off, void* out, uint64_t n) {
    switch(bus->type) {
        case device_memory:
            return memory_read_n(bus, off, out, n);
        case device_custom:
            return bus->vtable->device_read_8(bus, off, out);
    }

}
inline static bool dispatch_read_block(bus_device* device, uint64_t off, void* out) {
    switch(device->type) {
        case device_memory:
            return memory_read_block(device, off, out);
        case device_custom:
            return device->vtable->device_read_8(device, off, out);
    }

}

inline static bool dispatch_write_8(bus_device* device, uint64_t off, uint64_t value) {
    switch(device->type) {
        case device_memory:
            return memory_write_8(device, off, value);
        case device_custom:
            return device->vtable->device_write_8(device, off, value);
    }

}

inline static bool dispatch_write_4(bus_device* device, uint64_t off, uint32_t value) {
    switch(device->type) {
        case device_memory:
            return memory_write_4(device, off, value);
        case device_custom:
            return device->vtable->device_write_4(device, off, value);
    }

}
inline static bool dispatch_write_2(bus_device* device, uint64_t off, uint16_t value) {
    switch(device->type) {
        case device_memory:
            return memory_write_2(device, off, value);
        case device_custom:
            return device->vtable->device_write_2(device, off, value);
    }
}
inline static bool dispatch_write_1(bus_device* device, uint64_t off, uint8_t value) {
    switch(device->type) {
        case device_memory:
            return memory_write_1(device, off, value);
        case device_custom:
            return device->vtable->device_write_1(device, off, value);
    }

}
inline static bool dispatch_write_n(bus_device* device, uint64_t off, void* in, uint64_t n) {
    switch(device->type) {
        case device_memory:
            return memory_write_n(device, off, in, n);
        case device_custom:
            return device->vtable->device_write_n(device, off, in, n);
    }

}
inline static bool dispatch_write_block(bus_device* device, uint64_t off, void* in) {
    switch(device->type) {
        case device_memory:
            return memory_write_block(device, off, in);
        case device_custom:
            return device->vtable->device_write_block(device, off, in);
    }

}


inline static void shift_right(address_bus* bus, uint64_t at) {
    assert(bus->num_devices < MAX_DEVICES);

    for (int i = bus->num_devices; i > at; i--) {
        bus->ranges[i] = bus->ranges[i - 1];
        bus->devices[i] = bus->devices[i - 1];
    }
}

/// Overwrites `ranges` and `devices` at index `i` with the given values and
/// calls init on the device
inline static void put_device(address_bus* bus, size_t i, addr_range range,
                              bus_device* device) {
    size_t length = range.range + 1;
    bus->ranges[i] = range;
    bus->devices[i] = device;
    // TODO: If this function returns false, don't insert it into the list
    dispatch_init(device, length);
}

void addr_bus_init(address_bus* bus) { memset(bus, 0, sizeof(address_bus)); }
void addr_bus_destroy(address_bus* bus) {
    for(int i = 0; i < bus->num_devices; i++) {
        bus_device* device = bus->devices[i];
        dispatch_destroy(device);
        free(device);
    }
}

bool addr_bus_add_device(address_bus* bus, addr_range range,
                         bus_device* device) {
    if (device == NULL || bus->num_devices >= MAX_DEVICES) {
        return false;
    }

    if (bus->num_devices == 0) {
        put_device(bus, bus->num_devices, range, device);
        bus->num_devices += 1;
        return true;
    }

    if (range.address < bus->ranges[0].address) {
        if (!intersects(range, bus->ranges[0])) {
            shift_right(bus, 0);
            put_device(bus, 0, range, device);
            bus->num_devices += 1;
            return true;
        } else {
            return false;
        }
    }

    for (int i = 0; i < bus->num_devices; i++) {
        addr_range a = bus->ranges[i];

        if (intersects(range, a)) {
            return false;
        }

        if (i + 1 == bus->num_devices) {
            put_device(bus, bus->num_devices, range, device);
            bus->num_devices += 1;
            return true;
        }

        addr_range b = bus->ranges[i + 1];

        if (range.address >= a.address && range.address <= b.address) {
            if (!intersects(range, b)) {
                shift_right(bus, i + 1);
                put_device(bus, i + 1, range, device);
                bus->num_devices += 1;
                return true;
            } else {
                return false;
            }
        }
    }

    return false;
}

void addr_bus_pretty_print(address_bus* bus) {
    printf("%zu devices:\n", bus->num_devices);
    for (int i = 0; i < bus->num_devices; i++) {
        addr_range range = bus->ranges[i];

        printf("%016llx %016llx (%llu %llu)\n", range.address,
               range.address + range.range, range.address,
               range.address + range.range);
    }
}

bool addr_bus_intersects(address_bus* bus, addr_range range) {
    for (int i = 0; i < bus->num_devices; i++) {
        if (intersects(range, bus->ranges[i])) {
            return true;
        }
    }

    return false;
}

inline static bool address_intersects(addr_range range, uint64_t addr) {
    return addr >= range.address && addr <= range.address + range.range;
}

bool addr_bus_read_8(address_bus* bus, uint64_t addr, uint64_t* out) {
    for(int i = 0; i < bus->num_devices; i++) {
        addr_range range = bus->ranges[i];
        bus_device* device = bus->devices[i];

        if(address_intersects(range, addr)) {
            if(addr + 7 > range.address + range.range) {
                return false;
            }
            size_t off = addr - range.address;
            return dispatch_read_8(device, off, out);
        }
    }

    return false;
}
bool addr_bus_read_4(address_bus* bus, uint64_t addr, uint32_t* out) {
    for(int i = 0; i < bus->num_devices; i++) {
        addr_range range = bus->ranges[i];
        bus_device* device = bus->devices[i];

        if(address_intersects(range, addr)) {
            if(addr + 3 > range.address + range.range) {
                return false;
            }
            size_t off = addr - range.address;
            return dispatch_read_4(device, off, out);
        }
    }

    return false;
}
bool addr_bus_read_2(address_bus* bus, uint64_t addr, uint16_t* out) {
    for(int i = 0; i < bus->num_devices; i++) {
        addr_range range = bus->ranges[i];
        bus_device* device = bus->devices[i];

        if(address_intersects(range, addr)) {
            if(addr + 1 > range.address + range.range) {
                return false;
            }
            size_t off = addr - range.address;
            return dispatch_read_2(device, off, out);
        }
    }

    return false;
}
bool addr_bus_read_1(address_bus* bus, uint64_t addr, uint8_t* out) {
    for(int i = 0; i < bus->num_devices; i++) {
        addr_range range = bus->ranges[i];
        bus_device* device = bus->devices[i];

        if(address_intersects(range, addr)) {
            size_t off = addr - range.address;
            return dispatch_read_1(device, off, out);
        }
    }

    return false;
}

bool addr_bus_read_n(address_bus* bus, uint64_t addr, void* out, uint64_t n) {
    for(int i = 0; i < bus->num_devices; i++) {
        addr_range range = bus->ranges[i];
        bus_device* device = bus->devices[i];

        if(address_intersects(range, addr)) {
            if(addr + n - 1 > range.address + range.range) {
                return false;
            }
            size_t off = addr - range.address;
            return dispatch_read_n(device, off, out, n);
        }
    }
    return false;
}

bool addr_bus_read_block(address_bus* bus, uint64_t addr, void* out) {
    for(int i = 0; i < bus->num_devices; i++) {
        addr_range range = bus->ranges[i];
        bus_device* device = bus->devices[i];

        if(address_intersects(range, addr)) {
            if(addr + BLOCK_SIZE - 1 > range.address + range.range) {
                return false;
            }
            size_t off = addr - range.address;
            return dispatch_read_block(device, off, out);
        }
    }
    return false;
}

bool addr_bus_write_8(address_bus* bus, uint64_t addr, uint64_t value) {
    for(int i = 0; i < bus->num_devices; i++) {
        addr_range range = bus->ranges[i];
        bus_device* device = bus->devices[i];

        if(address_intersects(range, addr)) {
            if(addr + 7 > range.address + range.range) {
                printf("OUT OF RANGE!\n");
                return false;
            }
            size_t off = addr - range.address;
            return dispatch_write_8(device, off, value);
        }
    }
    printf("No devices found\n");
    return false;
}
bool addr_bus_write_4(address_bus* bus, uint64_t addr, uint32_t value) {
    for(int i = 0; i < bus->num_devices; i++) {
        addr_range range = bus->ranges[i];
        bus_device* device = bus->devices[i];

        if(address_intersects(range, addr)) {
            if(addr + 3 > range.address + range.range) {
                return false;
            }
            size_t off = addr - range.address;
            return dispatch_write_4(device, off, value);
        }
    }
    return false;
}
bool addr_bus_write_2(address_bus* bus, uint64_t addr, uint16_t value) {
    for(int i = 0; i < bus->num_devices; i++) {
        addr_range range = bus->ranges[i];
        bus_device* device = bus->devices[i];

        if(address_intersects(range, addr)) {
            if(addr + 1 > range.address + range.range) {
                return false;
            }
            size_t off = addr - range.address;
            return dispatch_write_2(device, off, value);
        }
    }
    return false;
}
bool addr_bus_write_1(address_bus* bus, uint64_t addr, uint8_t value) {
    for(int i = 0; i < bus->num_devices; i++) {
        addr_range range = bus->ranges[i];
        bus_device* device = bus->devices[i];

        if(address_intersects(range, addr)) {
            size_t off = addr - range.address;
            return dispatch_write_1(device, off, value);
        }
    }
    return false;
}

bool addr_bus_write_n(address_bus* bus, uint64_t addr, void* in, uint64_t n) {
    for(int i = 0; i < bus->num_devices; i++) {
        addr_range range = bus->ranges[i];
        bus_device* device = bus->devices[i];

        if(address_intersects(range, addr)) {
            if(addr + n - 1 > range.address + range.range) {
                return false;
            }
            size_t off = addr - range.address;
            return dispatch_write_n(device, off, in, n);
        }
    }
    return false;
}

bool addr_bus_write_block(address_bus* bus, uint64_t addr, void* in) {
    for(int i = 0; i < bus->num_devices; i++) {
        addr_range range = bus->ranges[i];
        bus_device* device = bus->devices[i];

        if(address_intersects(range, addr)) {
            if(addr + BLOCK_SIZE - 1 > range.address + range.range) {
                return false;
            }
            size_t off = addr - range.address;
            return dispatch_write_block(device, off, in);
        }
    }
    return false;
}
