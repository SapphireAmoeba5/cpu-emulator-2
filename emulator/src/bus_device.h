#pragma once

#include <stddef.h>
#include <stdint.h>

typedef struct bus_device bus_device;

typedef enum {
    device_custom,
    device_memory,
} device_type;

typedef struct {
    /// Called when the device gets added to the address bus
    ///
    /// Params:
    /// `device` - The bus device
    /// `requested` - A multiple of `BLOCK_SIZE` that the device needs
    bool(*device_init)(bus_device* device, uint64_t* requested);
    // Called when the device get's removed from the address bus
    bool(*device_destroy)(bus_device* device);

    bool(*device_read_block)(bus_device*, uint64_t block, void* out);
    bool(*device_write_block)(bus_device*, uint64_t block, void* in);

    uint8_t*(*device_lock_block)(bus_device* device, uint64_t block);
    void(*device_unlock_block)(bus_device* device, uint64_t block);
} bus_device_vtable;

#define BUS_DEVICE_IMPL \
    const bus_device_vtable* vtable; \
    device_type type;

typedef struct bus_device {
    BUS_DEVICE_IMPL
}bus_device;

