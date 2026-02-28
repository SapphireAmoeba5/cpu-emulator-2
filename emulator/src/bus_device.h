#pragma once

#include <stddef.h>
#include <stdint.h>

typedef struct bus_device bus_device;

typedef struct {
    /// Called when the device gets added to the address bus
    ///
    /// Params:
    /// `device` - The bus device
    /// `requested` - A multiple of `BLOCK_SIZE` that the device needs
    bool(*device_init)(bus_device* device, uint64_t* requested);
    // Called when the device get's removed from the address bus
    bool(*device_destroy)(bus_device* device);

    void(*device_read_n)(bus_device* device, uint64_t address, void* dst, uint64_t n);
    void(*device_write_n)(bus_device* device, uint64_t address, void* src, uint64_t n);

} bus_device_vtable;

#define BUS_DEVICE_IMPL \
    const bus_device_vtable* vtable; \

typedef struct bus_device {
    BUS_DEVICE_IMPL
}bus_device;

