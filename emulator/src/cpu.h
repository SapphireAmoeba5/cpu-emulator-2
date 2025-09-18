#pragma once

#include "memory.h"
#include <stdint.h>
#include <stdbool.h>

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
    reg registers[32];
    // Instruction and stack pointer
    uint64_t ip;
    uint64_t sp;
    bool halt;
    bool exit;

    // Stores N instructions. Updated when IP leaves the bounds that were cached
    uint64_t cached_address;
    uint64_t instruction_buffer[BUF_SIZE];
} Cpu;

void cpu_create(Cpu* cpu, Memory* memory);
void cpu_run(Cpu* cpu);

