#include "cpu.h"
#include "address_bus.h"
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
    /* 0x00 */ halt, intpt, invl, invl, invl, mov_reg, mov_imm, mov_mem, str, lea, invl, invl, invl, invl, invl, invl,
    /* 0x10 */ jmp,  jz,   invl, invl, invl, add_reg, add_imm, add_mem, invl, invl, invl, invl, invl, invl, invl, invl,
    /* 0x20 */ jnz,  jc,  invl, invl, invl, sub_reg, sub_imm, sub_mem, invl, invl, invl, invl, invl, invl, invl, invl,
    /* 0x30 */ jnc, jo,  invl, invl, invl, mul_reg, mul_imm, mul_mem, invl, invl, invl, invl, invl, invl, invl, invl,
    /* 0x40 */ jno, js,  invl, invl, invl, div_reg,  div_imm, div_mem, invl, invl, invl, invl, invl, invl, invl, invl,
    /* 0x50 */ jns, ja,  invl, invl, invl, idiv_reg, idiv_imm, idiv_mem, invl, invl, invl, invl, invl, invl, invl, invl,
    /* 0x60 */ jbe, jg,  invl, invl, invl, and_reg, and_imm, and_mem, invl, invl, invl, invl, invl, invl, invl, invl,
    /* 0x70 */ jle, jge,  invl, invl, invl, or_reg, or_imm, or_mem, invl, invl, invl, invl, invl, invl, invl, invl,
    /* 0x80 */ jl, invl,  invl, invl, invl, xor_reg, xor_imm, xor_mem, invl, invl, invl, invl, invl, invl, invl, invl,
    /* 0x90 */ invl, invl,  invl, invl, invl, cmp_reg, cmp_imm, cmp_mem, invl, invl, invl, invl, invl, invl, invl, invl,
    /* 0xa0 */ invl, invl,  invl, invl, invl, test_reg, test_imm, test_mem, invl, invl, invl, invl, invl, invl, invl, invl,
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
        uint64_t value = 0;
        bool success = addr_bus_read_8(cpu->bus, address, &value);
        // uint64_t value = memory_read_8(cpu->memory, address);
        cpu->instruction_buffer[i] = value;
    }
}

bool cpu_write_8(Cpu* cpu, uint64_t data, size_t address) {
    // if (false && address % 8 != 0) {
    //     printf("CPU fault unaligned write\n");
    //     return false;
    // }

    return addr_bus_write_8(cpu->bus, address, data);
    // memory_write_8(cpu->memory, data, address);
}
bool cpu_write_4(Cpu* cpu, uint32_t data, size_t address) {
    if (address % 4 != 0) {
        printf("CPU fault unaligned write\n");
        return false;
    }

    return addr_bus_write_4(cpu->bus, address, data);
}

bool cpu_write_2(Cpu* cpu, uint16_t data, size_t address) {
    if (address % 2 != 0) {
        printf("CPU fault unaligned write\n");
        return false;
    }

    return addr_bus_write_2(cpu->bus, address, data);
}

bool cpu_write_1(Cpu* cpu, uint8_t data, size_t address) {
    return addr_bus_write_1(cpu->bus, address, data);
}

bool cpu_read_8(Cpu* cpu, size_t address, uint64_t* value) {
    if (address % 8 != 0) {
        printf("CPU fault unaligned read\n");
        return false;
    }

    return addr_bus_read_8(cpu->bus, address, value);
}

bool cpu_read_4(Cpu* cpu, size_t address, uint32_t* value) {
    if (address % 4 != 0) {
        printf("CPU fault unaligned read\n");
        return false;
    }

    return addr_bus_read_4(cpu->bus, address, value);
}

bool cpu_read_2(Cpu* cpu, size_t address, uint16_t* value) {
    if (address % 2 != 0) {
        printf("CPU fault unaligned read\n");
        return false;
    }

    return addr_bus_read_2(cpu->bus, address, value);
}

bool cpu_read_1(Cpu* cpu, size_t address, uint8_t* value) {
    return addr_bus_read_1(cpu->bus, address, value);
}

void cpu_create(Cpu* cpu, address_bus* bus) {
    memset(cpu, 0, sizeof(Cpu));

    cpu->bus = bus;
    cache_instructions(cpu);
}

// Does not free the address bus, that is owned by the caller to cpu_create
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
