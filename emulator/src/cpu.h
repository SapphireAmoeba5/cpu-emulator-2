#pragma once

#include "address_bus.h"
#include <stdbool.h>
#include <stdint.h>

#ifdef NDEBUG
#define UNREACHABLE() __builtin_unreachable()
#else
#include <stdlib.h>
#define UNREACHABLE()                                                          \
    printf("%s:%d Unreachable code reached!\n", __FILE__, __LINE__);           \
    abort()
#endif

#define BIT(n) 1 << (n)

typedef enum {
    NO_ERROR = -1,
    MEMORY_ERROR,
    DECODE_ERROR,
    MATH_ERROR,
} error_t;

typedef uint16_t flags_t;

constexpr flags_t FLAG_ZERO = BIT(0);
constexpr flags_t FLAG_CARRY = BIT(1);
constexpr flags_t FLAG_OVERFLOW = BIT(2);
constexpr flags_t FLAG_SIGN = BIT(3);

typedef union reg {
    uint8_t b;
    uint16_t s;
    uint32_t w;
    uint64_t r;
} reg;

typedef struct Cpu {
    address_bus* bus;
    // Total number of clocks
    uint64_t clock_count;
    // 16 general purpose registers
    reg registers[16];
    // Instruction and stack pointer
    uint64_t ip;
    uint64_t sp;

    // If the CPU is halted
    bool halt;
    // If the CPU should delete itself
    bool exit;
    // CPU flags
    flags_t flags;

    // The address that has been cached
    uint64_t cached_address;
    // Stores a local cache of BLOCK_SIZE bytes containing instructions
    uint8_t cache[BLOCK_SIZE];

} Cpu;

bool cpu_write_8(Cpu* cpu, uint64_t data, uint64_t address);
bool cpu_write_4(Cpu* cpu, uint32_t data, uint64_t address);
bool cpu_write_2(Cpu* cpu, uint16_t data, uint64_t address);
bool cpu_write_1(Cpu* cpu, uint8_t data, uint64_t address);

/// `value` must not be NULL
bool cpu_read_8(Cpu* cpu, uint64_t address, uint64_t* value);
/// `value` must not be NULL
bool cpu_read_4(Cpu* cpu, uint64_t address, uint32_t* value);
/// `value` must not be NULL
bool cpu_read_2(Cpu* cpu, uint64_t address, uint16_t* value);
/// `value` must not be NULL
bool cpu_read_1(Cpu* cpu, uint64_t address, uint8_t* value);
bool cpu_read_n(Cpu* cpu, uint64_t address, void* out, uint64_t n);
bool cpu_read_block(Cpu* cpu, uint64_t address, void* out);

void cpu_create(Cpu* cpu, address_bus* bus);
void cpu_destroy(Cpu* cpu);
void cpu_run(Cpu* cpu);
