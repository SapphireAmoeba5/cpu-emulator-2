#include "decode.h"
#include "address_bus.h"
#include "cpu.h"
#include <assert.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define DEBUG_EXIT()                                                           \
    printf("TODO: Exceptions\n");                                              \
    exit(1)

inline static bool cache(Cpu* cpu) {
    cpu->cached_address = cpu->ip - (cpu->ip % BLOCK_SIZE);
    if (!cpu_read_block(cpu, cpu->cached_address, &cpu->cache[0])) {
        cpu->cached_address = 0xffffffffffffffff;
        return false;
    }
    return true;
}

inline static bool fetch(Cpu* cpu, uint8_t* byte) {
    if (!cpu_read_1(cpu, cpu->ip, byte)) {
        return false;
    }
    cpu->ip += 1;
    return true;
    // if (cpu->ip < cpu->cached_address ||
    //     cpu->ip >= cpu->cached_address + BLOCK_SIZE) {
    //     if (!cache(cpu)) {
    //         if (!cpu_read_1(cpu, cpu->ip, byte)) {
    //             return false;
    //         }
    //         return true;
    //     }
    // }
    //
    // uint64_t i = cpu->ip - cpu->cached_address;
    // *byte = cpu->cache[i];
    // cpu->ip += 1;
    //
    // return true;
}

inline static bool fetch_2(Cpu* cpu, uint16_t* out) {
    if (!cpu_read_2(cpu, cpu->ip, out)) {
        return false;
    }
    cpu->ip += 2;
    return true;
    //
    // if (cpu->ip < cpu->cached_address) {
    //
    // } else if (cpu->ip + 2 > cpu->cached_address + BLOCK_SIZE) {
    // }
    //
    // int i = cpu->ip - cpu->cached_address;
    // memcpy(out, &cpu->cache[i], sizeof(*out));
    // cpu->ip += 2;
    // return true;
}

inline static bool fetch_4(Cpu* cpu, uint32_t* out) {
    if (!cpu_read_4(cpu, cpu->ip, out)) {
        return false;
    }
    cpu->ip += 4;
    return true;
}

inline static bool fetch_8(Cpu* cpu, uint64_t* out) {
    if (!cpu_read_8(cpu, cpu->ip, out)) {
        return false;
    }
    cpu->ip += 8;
    return true;
}

// clang-format off
iop ops[] = 
{
    /* 0x00 */ op_halt, op_int, op_invl, op_invl, op_invl, op_mov, op_mov, op_mov, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl,
    /* 0x10 */ op_invl, op_invl, op_invl, op_invl, op_invl, op_add, op_add, op_add, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl,
    /* 0x20 */ op_invl, op_invl, op_invl, op_invl, op_invl, op_sub, op_sub, op_sub, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl,
    /* 0x30 */ op_invl, op_invl, op_invl, op_invl, op_invl, op_mul, op_mul, op_mul, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl,
    /* 0x40 */ op_invl, op_invl, op_invl, op_invl, op_invl, op_div, op_div, op_div, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl,
    /* 0x50 */ op_invl, op_invl, op_invl, op_invl, op_invl, op_idiv, op_idiv, op_idiv, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl,
    /* 0x60 */ op_invl, op_invl, op_invl, op_invl, op_invl, op_and, op_and, op_and, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl,
    /* 0x70 */ op_invl, op_invl, op_invl, op_invl, op_invl, op_or, op_or, op_or, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl,
    /* 0x80 */ op_invl, op_invl, op_invl, op_invl, op_invl, op_xor, op_xor, op_xor, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl,
    /* 0x90 */ op_invl, op_invl, op_invl, op_invl, op_invl, op_cmp, op_cmp, op_cmp, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl,
    /* 0xa0 */ op_invl, op_invl, op_invl, op_invl, op_invl, op_test, op_test, op_test, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl,
    /* 0xb0 */ op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl,
    /* 0xc0 */ op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl,
    /* 0xd0 */ op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl,
    /* 0xe0 */ op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl,
    /* 0xf0 */ op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl,
};
// clang-format on

// Return the opcode's operation
static inline iop get_op(uint8_t opcode) {
    switch (opcode) {
    case 0x00:
        return op_halt;
    case 0x01:
        return op_int;
    case 0x05:
    case 0x06:
    case 0x07:
        return op_mov;
    case 0x15:
    case 0x16:
    case 0x17:
        return op_add;
    case 0x25:
    case 0x26:
    case 0x27:
        return op_sub;
    case 0x35:
    case 0x36:
    case 0x37:
        return op_mul;
    case 0x45:
    case 0x46:
    case 0x47:
        return op_div;
    case 0x55:
    case 0x56:
    case 0x57:
        return op_idiv;
    case 0x65:
    case 0x66:
    case 0x67:
        return op_and;
    case 0x75:
    case 0x76:
    case 0x77:
        return op_or;
    case 0x85:
    case 0x86:
    case 0x87:
        return op_xor;
    case 0x95:
    case 0x96:
    case 0x97:
        return op_cmp;
    case 0xa5:
    case 0xa6:
    case 0xa7:
        return op_test;

    default:
        return op_invl;
    }
}

/// Same as `get_op` but implemented with a lookup table
static inline iop get_op2(uint8_t opcode) { return ops[opcode]; }

inline static bool decode_reg_operand(Cpu* cpu, instruction* instr) {
    uint8_t transfer_byte;
    if (!fetch(cpu, &transfer_byte)) {
        return false;
    }
    uint8_t src = transfer_byte & 0x0f;
    uint8_t dest = (transfer_byte >> 4) & 0x0f;

    instr->dest = &cpu->registers[dest].r;
    instr->src = cpu->registers[src].r;
    return true;
}

inline static bool decode_imm_operand(Cpu* cpu, instruction* instr) {
    uint8_t transfer_byte;
    if (!fetch(cpu, &transfer_byte)) {
        return false;
    }

    uint8_t dest = (transfer_byte >> 4) & 0x0f;
    uint8_t size = (transfer_byte >> 2) & 0x03;

    instr->dest = &cpu->registers[dest].r;

    switch (size) {
    case 0:
        return fetch(cpu, (uint8_t*)&instr->src);
    case 1:
        return fetch_2(cpu, (uint16_t*)&instr->src);
    case 2:
        return fetch_4(cpu, (uint32_t*)&instr->src);
    case 3:
        return fetch_8(cpu, (uint64_t*)&instr->src);
    default:
        UNREACHABLE();
    }
}

bool cpu_decode(Cpu* cpu, instruction* instr) {
    memset(instr, 0, sizeof(*instr));
    uint8_t opcode;
    if (!fetch(cpu, &opcode)) {
        return false;
    }

    printf("OPCODE: %d\n", opcode);

    instr->op = get_op2(opcode);
    switch (opcode) {
    case 0x00:
        return true;
    case 0x01:
        if (!fetch(cpu, (uint8_t*)&instr->src)) {
            return false;
        }
        return true;
    case 0x05:
    case 0x15:
    case 0x25:
    case 0x35:
    case 0x45:
    case 0x55:
    case 0x65:
    case 0x75:
    case 0x85:
    case 0x95:
    case 0xa5:
        if (!decode_reg_operand(cpu, instr)) {
            return false;
        }
        return true;

    case 0x06:
    case 0x16:
    case 0x26:
    case 0x36:
    case 0x46:
    case 0x56:
    case 0x66:
    case 0x76:
    case 0x86:
    case 0x96:
    case 0xa6:
        if (!decode_imm_operand(cpu, instr)) {
            return false;
        }
        return true;

    default:
        return false;
    }
}
