#pragma once

#include "memory.h"
#include <stdint.h>
#include <stdbool.h>

#define BIT(n) 1 << (n)

typedef uint16_t flags_t ;

constexpr flags_t FLAG_ZERO = BIT(0);
constexpr flags_t FLAG_CARRY = BIT(1);
constexpr flags_t FLAG_OVERFLOW = BIT(2);
constexpr flags_t FLAG_SIGN = BIT(3);

// Number of instructions to read ahead
constexpr uint64_t N = 50;
// Size of the instruction buffer in uin64_t's. Add's one to account for alignment when populating the instruction buffer
constexpr uint64_t BUF_SIZE = (N + 1) * 2;
constexpr uint64_t SIZE_BYTES = BUF_SIZE * sizeof(uint64_t);

typedef union reg {
    uint8_t b;
    uint16_t s;
    uint32_t w;
    uint64_t r;
} reg;

typedef struct Cpu {
    Memory* memory;
    // Total number of clocks
    uint64_t clock_count;
    // 32 general purpose registers
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

    // Stores N instructions. Updated when IP leaves the bounds that were cached
    uint64_t cached_address;
    uint64_t instruction_buffer[BUF_SIZE];
} Cpu;

bool cpu_write_8(Cpu* cpu, uint64_t data, size_t address);
bool cpu_write_4(Cpu* cpu, uint32_t data, size_t address);
bool cpu_write_2(Cpu* cpu, uint16_t data, size_t address);
bool cpu_write_1(Cpu* cpu, uint8_t data, size_t address);

/// `value` must not be NULL
bool cpu_read_8(Cpu* cpu, size_t address, uint64_t* value);
/// `value` must not be NULL
bool cpu_read_4(Cpu* cpu, size_t address,  uint32_t* value);
/// `value` must not be NULL
bool cpu_read_2(Cpu* cpu, size_t address,  uint16_t* value);
/// `value` must not be NULL
bool cpu_read_1(Cpu* cpu, size_t address,  uint8_t* value);

void cpu_create(Cpu* cpu, Memory* memory);
void cpu_destroy(Cpu* cpu);
void cpu_run(Cpu* cpu);
