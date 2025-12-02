#include "cpu.h"
#include "instructions.h"
#include "memory.h"
#include <stdbool.h>
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
    /* 0x00 */ halt, intpt, invl, invl, invl, mov_reg, mov_imm_or_mem, str, invl, invl, invl, invl, invl, invl, invl, invl,
    /* 0x10 */ invl, invl,  invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl,
    /* 0x20 */ invl, invl,  invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl,
    /* 0x30 */ invl, invl,  invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl,
    /* 0x40 */ invl, invl,  invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl,
    /* 0x50 */ invl, invl,  invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl,
    /* 0x60 */ invl, invl,  invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl,
    /* 0x70 */ invl, invl,  invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl,
    /* 0x80 */ invl, invl,  invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl,
    /* 0x90 */ invl, invl,  invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl,
    /* 0xa0 */ invl, invl,  invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl,
    /* 0xb0 */ invl, invl,  invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl,
    /* 0xc0 */ invl, invl,  invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl,
    /* 0xd0 */ invl, invl,  invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl,
    /* 0xe0 */ invl, invl,  invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl,
    /* 0xf0 */ invl, invl,  invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl, invl,
};
// clang-format on

// Caches the next BUF_SIZE 8byte words into the instruction buffer
static void cache_instructions(Cpu* cpu) {
    // Align to the previous aligned 8byte address
    uint64_t cached_address = cpu->ip - (cpu->ip % 8);
    cpu->cached_address = cached_address;

    for (int i = 0; i < BUF_SIZE; i++) {
        uint64_t address = cached_address + (i * 8);
        uint64_t value = memory_read_8(cpu->memory, address);
        cpu->instruction_buffer[i] = value;
    }
}

bool cpu_write_8(Cpu* cpu, uint64_t data, size_t address) {
    if (address % 8 != 0) {
        printf("CPU fault unaligned write\n");
        return false;
    }

    memory_write_8(cpu->memory, data, address);

    return true;
}
bool cpu_write_4(Cpu* cpu, uint32_t data, size_t address) {
    if (address % 4 != 0) {
        printf("CPU fault unaligned write\n");
        return false;
    }

    memory_write_4(cpu->memory, data, address);

    return true;
}

bool cpu_write_2(Cpu* cpu, uint16_t data, size_t address) {
    if (address % 2 != 0) {
        printf("CPU fault unaligned write\n");
        return false;
    }

    memory_write_2(cpu->memory, data, address);

    return true;
}

bool cpu_write_1(Cpu* cpu, uint8_t data, size_t address) {
    memory_write_1(cpu->memory, data, address);

    return true;
}

bool cpu_read_8(Cpu* cpu, size_t address, uint64_t* value) {
    if (address % 8 != 0) {
        printf("CPU fault unaligned read\n");
        return false;
    }

    *value = memory_read_8(cpu->memory, address);

    return true;
}

bool cpu_read_4(Cpu* cpu, size_t address, uint32_t* value) {
    if (address % 4 != 0) {
        printf("CPU fault unaligned read\n");
        return false;
    }

    *value = memory_read_4(cpu->memory, address);

    return true;
}

bool cpu_read_2(Cpu* cpu, size_t address, uint16_t* value) {
    if (address % 2 != 0) {
        printf("CPU fault unaligned read\n");
        return false;
    }

    *value = memory_read_2(cpu->memory, address);

    return true;
}

bool cpu_read_1(Cpu* cpu, size_t address, uint8_t* value) {
    *value = memory_read_1(cpu->memory, address);

    return true;
}

void cpu_create(Cpu* cpu, Memory* memory) {
    memset(cpu, 0, sizeof(Cpu));

    cpu->memory = memory;
    cache_instructions(cpu);
}

void cpu_destroy(Cpu* cpu) {
    // So far nothing
    return;
}

static void cpu_clock(Cpu* cpu) {
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
