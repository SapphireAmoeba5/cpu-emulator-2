#pragma once
#include <stdatomic.h>
#include <stddef.h>
#include <stdint.h>
typedef _Atomic uint64_t AtomicU64;

typedef struct Memory {
    AtomicU64* data;
    size_t length; // length in elements, not bytes
} Memory;


// Size is in units of 8 bytes
void memory_create(Memory* memory, size_t size);
void memory_destroy(Memory* memory);

void memory_clear(Memory* memory, uint64_t clearValue);

size_t memory_size_bytes(Memory* mem);

void memory_write_1(Memory* memory, uint8_t byte, size_t address);
void memory_write_2(Memory* memory, uint16_t byte, size_t address);
void memory_write_4(Memory* memory, uint32_t byte, size_t address);
void memory_write_8(Memory* memory, uint64_t byte, size_t address);

uint8_t memory_read_1(Memory* memory, size_t address);
uint16_t memory_read_2(Memory* memory, size_t address);
uint32_t memory_read_4(Memory* memory, size_t address);
uint64_t memory_read_8(Memory* memory, size_t address);
