#include "cpu.h"
#include "instructions.h"
#include "memory.h"
#include <stdckdint.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

#define TABLE_IMPL

typedef void (*instruction_f)(Cpu* cpu, uint8_t instruction[16]);

// clang-format off
static instruction_f instructions[256] = 
{
    /* 0x00 */ halt, intpt, invl, invl, invl, mov_reg_reg, mov_reg_imm8, mov_reg_imm16, mov_reg_imm32, mov_reg_imm64, invl, invl, invl, invl, invl, invl,
    /* 0x10 */ invl, invl, invl, invl, invl, add_reg_reg, add_reg_imm8, add_reg_imm16, add_reg_imm32, add_reg_imm64, invl, invl, invl, invl, invl, invl,
    /* 0x20 */ invl, invl, invl, invl, invl, sub_reg_reg, sub_reg_imm8, sub_reg_imm16, sub_reg_imm32, sub_reg_imm64, invl, invl, invl, invl, invl, invl,
    /* 0x30 */ invl, invl, invl, invl, invl, mul_reg_reg, mul_reg_imm8, mul_reg_imm16, mul_reg_imm32, mul_reg_imm64, invl, invl, invl, invl, invl, invl,
    /* 0x40 */ invl, invl, invl, invl, invl, div_reg_reg, div_reg_imm8, div_reg_imm16, div_reg_imm32, div_reg_imm64, invl, invl, invl, invl, invl, invl,
    /* 0x50 */ invl, invl, invl, invl, invl, idiv_reg_reg, idiv_reg_imm8, idiv_reg_imm16, idiv_reg_imm32, idiv_reg_imm64, invl, invl, invl, invl, invl, invl,
    /* 0x60 */ invl, invl, invl, invl, invl, and_reg_reg, and_reg_imm8, and_reg_imm16, and_reg_imm32, and_reg_imm64, invl, invl, invl, invl, invl, invl,
    /* 0x70 */ invl, invl, invl, invl, invl, or_reg_reg, or_reg_imm8, or_reg_imm16, or_reg_imm32, or_reg_imm64, invl, invl, invl, invl, invl, invl,
    /* 0x80 */ invl, invl, invl, invl, invl, xor_reg_reg, xor_reg_imm8, xor_reg_imm16, xor_reg_imm32, xor_reg_imm64, invl, invl, invl, invl, invl, invl,
    /* 0x90 */ invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl,
    /* 0xa0 */ invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl,
    /* 0xb0 */ invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl,
    /* 0xc0 */ invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl,
    /* 0xd0 */ invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl,
    /* 0xe0 */ invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl,
    /* 0xf0 */ invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl,
};
// clang-format on

// Caches the next BUF_SIZE 8byte words into the instruction buffer
void cache_instructions(Cpu* cpu) {
    // Align to the previous aligned 8byte address
    uint64_t cached_address = cpu->ip - (cpu->ip % 8);
    cpu->cached_address = cached_address;

    for (int i = 0; i < BUF_SIZE; i++) {
        uint64_t address = cached_address + (i * 8);
        uint64_t value = memory_read_8(cpu->memory, address);
        cpu->instruction_buffer[i] = value;
    }
}
 
void cpu_create(Cpu* cpu, Memory* memory) {
    memset(cpu, 0, sizeof(Cpu));

    cpu->memory = memory;
    cache_instructions(cpu);
}

void cpu_clock(Cpu* cpu) {
    cpu->clock_count++;

    if (cpu->ip + 16 >= cpu->cached_address + SIZE_BYTES) {
        cache_instructions(cpu);
    }

    uint64_t offset = cpu->ip - cpu->cached_address;
    uint8_t* instruction = ((uint8_t*)cpu->instruction_buffer) + offset;

    uint8_t opcode = instruction[0];
    instructions[opcode](cpu, instruction);
}

void cpu_run(Cpu* cpu) {
    while (!cpu->exit) {
        if (!cpu->halt) {
            cpu_clock(cpu);
        }
    }
}
