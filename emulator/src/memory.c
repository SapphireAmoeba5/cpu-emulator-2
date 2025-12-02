#include "memory.h"
#include <stdatomic.h>
#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>

#define MEMORY_CHECK_ALIGNMENT

void memory_create(Memory* memory, size_t size) {
    memory->length = size;
    memory->data = calloc(size, sizeof(AtomicU64));
}

void memory_destroy(Memory* memory) {
    free(memory->data);
}

void memory_clear(Memory* memory, uint64_t clearValue) {
    for (int i = 0; i < memory->length; i++) {
        atomic_store_explicit(&memory->data[i], clearValue, memory_order_relaxed);
    }
}

size_t memory_size_bytes(Memory* mem) {
    return mem->length * sizeof(AtomicU64);
}

uint64_t memory_read_8(Memory* memory, size_t address) {
#ifdef MEMORY_CHECK_ALIGNMENT
    if (address % 8 != 0) {
        printf("%zu is not aligned to 8 bytes\n", address);
        exit(1);
    }
#endif

    size_t index = address / 8;
    auto value = atomic_load_explicit(&memory->data[index], memory_order_relaxed);
    return value;
}

uint32_t memory_read_4(Memory* memory, size_t address) {
#ifdef MEMORY_CHECK_ALIGNMENT
    if (address % 4 != 0) {
        printf("%zu is not aligned to 4 bytes\n", address);
        exit(1);
    }
#endif

    size_t index = address / 8;
    uint64_t value = atomic_load_explicit(&memory->data[index], memory_order_relaxed);
    uint32_t shifted = value >> (address % 8) * 8;
    return shifted;
}

uint16_t memory_read_2(Memory* memory, size_t address) {
#ifdef MEMORY_CHECK_ALIGNMENT
    if (address % 2 != 0) {
        printf("%zu is not aligned to 2 bytes\n", address);
        exit(1);
    }
#endif

    size_t index = address / 8;
    uint64_t value = atomic_load_explicit(&memory->data[index], memory_order_relaxed);
    uint16_t shifted = value >> (address % 8) * 8;
    return shifted;
}

uint8_t memory_read_1(Memory* memory, size_t address) {
    size_t index = address / 8;
    uint64_t value = atomic_load_explicit(&memory->data[index], memory_order_relaxed);
    uint8_t shifted = value >> (address % 8) * 8;
    return shifted;
}

// ========== WRITE ===============
void memory_write_8(Memory* memory, uint64_t value, size_t address) {
#ifdef MEMORY_CHECK_ALIGNMENT
    if (address % 8 != 0) {
        printf("%zu is not aligned to 8 bytes\n", address);
        exit(1);
    }
#endif

    size_t index = address / 8;
    atomic_store_explicit(&memory->data[index], value, memory_order_relaxed);
}

void memory_write_4(Memory* memory, uint32_t to_write, size_t address) {
#ifdef MEMORY_CHECK_ALIGNMENT
    if (address % 4 != 0) {
        printf("%zu is not aligned to 4 bytes\n", address);
        exit(1);
    }
#endif

    const uint64_t shift = (address % 8) * 8;
    size_t index = address / 8;
    uint64_t old_value;
    uint64_t new_value;
    const uint64_t mask = 0xffffffff;
    do {
        old_value = atomic_load_explicit(&memory->data[index], memory_order_relaxed);
        new_value = (old_value & ~(mask << shift)) | ((uint64_t)to_write << shift);
    } while (!atomic_compare_exchange_weak_explicit(&memory->data[index], &old_value, new_value, memory_order_relaxed, memory_order_relaxed));
}
void memory_write_2(Memory* memory, uint16_t to_write, size_t address) {
#ifdef MEMORY_CHECK_ALIGNMENT
    if (address % 2 != 0) {
        printf("%zu is not aligned to 2 bytes\n", address);
        exit(1);
    }
#endif

    size_t index = address / 8;
    uint64_t old_value;
    uint64_t new_value;
    const uint64_t mask = 0xffff;
    do {
        const uint64_t shift = address % 8 * 8;
        old_value = atomic_load_explicit(&memory->data[index], memory_order_relaxed);
        new_value = (old_value & ~(mask << shift)) | ((uint64_t)to_write << shift);
    } while (!atomic_compare_exchange_weak_explicit(&memory->data[index], &old_value, new_value, memory_order_relaxed, memory_order_relaxed));
}
void memory_write_1(Memory* memory, uint8_t to_write, size_t address) {
    size_t index = address / 8;
    uint64_t old_value;
    uint64_t new_value;
    uint64_t mask = 0xff;
    do {
        const uint64_t shift = address % 8 * 8;
        old_value = atomic_load_explicit(&memory->data[index], memory_order_relaxed);
        new_value = (old_value & ~(mask << shift)) | ((uint64_t)to_write << shift);
    } while (!atomic_compare_exchange_weak_explicit(&memory->data[index], &old_value, new_value, memory_order_relaxed, memory_order_relaxed));
}
