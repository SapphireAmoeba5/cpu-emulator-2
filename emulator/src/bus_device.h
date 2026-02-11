#pragma once

#include <stddef.h>
#include <stdint.h>

typedef struct bus_device bus_device;

typedef struct {
    /// Called when the device gets added to the address bus
    ///
    /// Params:
    /// `device` - The bus device
    /// `length` - The length of the address range associated with this device
    bool(*device_init)(bus_device* device, size_t length);
    // Called when the device get's removed from the address bus
    bool(*device_destroy)(bus_device* device);

    bool(*device_read_8)(bus_device* bus, uint64_t off, uint64_t* out);
    bool(*device_read_4)(bus_device* bus, uint64_t off, uint32_t* out);
    bool(*device_read_2)(bus_device* bus, uint64_t off, uint16_t* out);
    bool(*device_read_1)(bus_device* bus, uint64_t off, uint8_t* out);
    bool(*device_read_n)(bus_device*, uint64_t off, void* out, uint64_t n);
    bool(*device_read_block)(bus_device*, uint64_t off, void* out);

    bool(*device_write_8)(bus_device* bus, uint64_t off, uint64_t value);
    bool(*device_write_4)(bus_device* bus, uint64_t off, uint32_t value);
    bool(*device_write_2)(bus_device* bus, uint64_t off, uint16_t value);
    bool(*device_write_1)(bus_device* bus, uint64_t off, uint8_t value);
    bool(*device_write_n)(bus_device*, uint64_t off, void* in, uint64_t n);
    bool(*device_write_block)(bus_device*, uint64_t off, void* in);
} bus_device_vtable;

typedef struct bus_device {
    const bus_device_vtable* vtable; 
}bus_device;

