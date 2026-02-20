#include "decode.h"
#include "cpu.h"
#include "instruction.h"
#include <assert.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

constexpr uint8_t OPCODE_EXTENSION = 0x0f;

/// Appends the opcode extension byte onto an opcode
#define EXT(opcode) (((uint16_t)OPCODE_EXTENSION << 8) | ((uint16_t)(opcode)))

#define DEBUG_EXIT()                                                           \
    printf("TODO: Exceptions\n");                                              \
    exit(1)

typedef enum {
    PCRel = 0,
    SPRel = 1,
    // Base + Index * Scale
    BIS = 2,
    Addr = 3,

} addr_mode;

// On error returns `MEMORY_ERROR` otherwise returns `NO_ERROR`
inline static error_t fetch(Cpu* cpu, uint8_t* byte) {
    if (!cache_read_1(&cpu->instruction_cache, cpu->bus,
                      cpu->registers[IP_INDEX].r, byte)) {
        return MEMORY_ERROR;
    }
    cpu->registers[IP_INDEX].r += 1;
    return NO_ERROR;
}

inline static error_t fetch_2(Cpu* cpu, uint16_t* out) {
    if (!cache_read_2(&cpu->instruction_cache, cpu->bus,
                      cpu->registers[IP_INDEX].r, out)) {
        return MEMORY_ERROR;
    }
    cpu->registers[IP_INDEX].r += 2;
    return NO_ERROR;
}

inline static error_t fetch_4(Cpu* cpu, uint32_t* out) {
    if (!cache_read_4(&cpu->instruction_cache, cpu->bus,
                      cpu->registers[IP_INDEX].r, out)) {
        return MEMORY_ERROR;
    }
    cpu->registers[IP_INDEX].r += 4;
    return NO_ERROR;
}

inline static error_t fetch_8(Cpu* cpu, uint64_t* out) {
    if (!cache_read_8(&cpu->instruction_cache, cpu->bus,
                      cpu->registers[IP_INDEX].r, out)) {
        return MEMORY_ERROR;
    }
    cpu->registers[IP_INDEX].r += 8;
    return NO_ERROR;
}

// clang-format off
iop ops[] = 
{
    /* 0x00 */ op_halt, op_int, op_ret, op_invl, op_invl, op_mov, op_mov, op_mov, op_str, op_mov, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl,
    /* 0x10 */ op_mov, op_mov, op_mov, op_mov, op_mov, op_mov, op_mov, op_mov, op_mov, op_mov, op_mov, op_mov, op_mov, op_mov, op_mov, op_call,
    /* 0x20 */ op_mov, op_add, op_sub, op_mul, op_div, op_idiv, op_and, op_or, op_xor, op_cmp, op_test, op_invl, op_invl, op_invl, op_invl, op_invl,
    /* 0x30 */ op_mov, op_add, op_sub, op_mul, op_div, op_idiv, op_and, op_or, op_xor, op_cmp, op_test, op_invl, op_invl, op_invl, op_invl, op_invl,
    /* 0x40 */ op_mov, op_add, op_sub, op_mul, op_div, op_idiv, op_and, op_or, op_xor, op_cmp, op_test, op_invl, op_invl, op_invl, op_invl, op_invl,
    /* 0x50 */ op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl,
    /* 0x60 */ op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl,
    /* 0x70 */ op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl,
    /* 0x80 */ op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl,
    /* 0x90 */ op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl,
    /* 0xa0 */ op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl,
    /* 0xb0 */ op_call, op_call, op_call, op_call, op_call, op_call, op_call, op_call, op_call, op_call, op_call, op_call, op_call, op_call, op_call, op_call,
    /* 0xc0 */ op_mov, op_mov, op_mov, op_mov, op_mov, op_mov, op_mov, op_mov, op_mov, op_mov, op_mov, op_mov, op_mov, op_mov, op_mov, op_mov,
    /* 0xd0 */ op_push, op_push, op_push, op_push, op_push, op_push, op_push, op_push, op_push, op_push, op_push, op_push, op_push, op_push, op_push, op_push,
    /* 0xe0 */ op_pop, op_pop, op_pop, op_pop, op_pop, op_pop, op_pop, op_pop, op_pop, op_pop, op_pop, op_pop, op_pop, op_pop, op_pop, op_pop,
    /* 0xf0 */ op_rdt, op_rdt, op_rdt, op_rdt, op_rdt, op_rdt, op_rdt, op_rdt, op_rdt, op_rdt, op_rdt, op_rdt, op_rdt, op_rdt, op_rdt, op_rdt,
};

// Operations for the extended opcodes
iop ext_ops[] =
{
    /* 0x00 */ op_mov, op_mov, op_mov, op_mov, op_mov, op_mov, op_mov, op_mov, op_mov, op_mov, op_mov, op_mov, op_mov, op_mov, op_mov, op_mov,
    /* 0x10 */ op_mov, op_mov, op_mov, op_mov, op_mov, op_mov, op_mov, op_mov, op_mov, op_mov, op_mov, op_mov, op_sysinfo, op_invl, op_invl, op_invl,
    /* 0x20 */ op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl,
    /* 0x30 */ op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl,
    /* 0x40 */ op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl,
    /* 0x50 */ op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl,
    /* 0x60 */ op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl,
    /* 0x70 */ op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl,
    /* 0x80 */ op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl,
    /* 0x90 */ op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl,
    /* 0xa0 */ op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl,
    /* 0xb0 */ op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl,
    /* 0xc0 */ op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl,
    /* 0xd0 */ op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl, op_invl,
    /* 0xe0 */ op_mov, op_mov, op_mov, op_mov, op_mov, op_mov, op_mov, op_mov, op_mov, op_mov, op_mov, op_mov, op_mov, op_mov, op_mov, op_mov,
    /* 0xf0 */ op_mov, op_mov, op_mov, op_mov, op_mov, op_mov, op_mov, op_mov, op_mov, op_mov, op_mov, op_mov, op_mov, op_mov, op_mov, op_mov,
};

condition conditions[] =
{
    /* 0x00 */ cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true,
    /* 0x10 */ cd_true, cd_zero, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true,
    /* 0x20 */ cd_nzero, cd_carry, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true,
    /* 0x30 */ cd_ncarry, cd_overflow, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true,
    /* 0x40 */ cd_noverflow, cd_sign, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true,
    /* 0x50 */ cd_nsign, cd_above, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true,
    /* 0x60 */ cd_be, cd_greater, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true,
    /* 0x70 */ cd_le, cd_ge, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true,
    /* 0x80 */ cd_less, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true,
    /* 0x90 */ cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true,
    /* 0xa0 */ cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true,
    /* 0xb0 */ cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true,
    /* 0xc0 */ cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true,
    /* 0xd0 */ cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true,
    /* 0xe0 */ cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true,
    /* 0xf0 */ cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true,
};

condition ext_conditions[] = {
    /* 0x00 */ cd_nzero, cd_zero, cd_carry, cd_ncarry, cd_overflow, cd_noverflow, cd_sign, cd_nsign, cd_above, cd_be, cd_greater, cd_le, cd_ge, cd_less, cd_nzero, cd_zero, 
    /* 0x10 */ cd_carry, cd_ncarry, cd_overflow, cd_noverflow, cd_sign, cd_nsign, cd_above, cd_be, cd_greater, cd_le, cd_ge, cd_less, cd_true, cd_true, cd_true, cd_true, 
    /* 0x20 */ cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, 
    /* 0x30 */ cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, 
    /* 0x40 */ cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, 
    /* 0x50 */ cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, 
    /* 0x60 */ cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, 
    /* 0x70 */ cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, 
    /* 0x80 */ cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, 
    /* 0x90 */ cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, 
    /* 0xa0 */ cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, 
    /* 0xb0 */ cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, 
    /* 0xc0 */ cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, 
    /* 0xd0 */ cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, 
    /* 0xe0 */ cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, 
    /* 0xf0 */ cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, cd_true, 
};

// clang-format on

/// Same as `get_op` but implemented with a lookup table
static inline iop get_op2(uint8_t opcode) { return ops[opcode]; }

inline static error_t decode_reg_operand(Cpu* cpu, instruction* instr) {
    uint8_t transfer_byte;
    if (fetch(cpu, &transfer_byte) != NO_ERROR) {
        return MEMORY_ERROR;
    }
    uint8_t src = transfer_byte & 0x0f;
    uint8_t dest = (transfer_byte >> 4) & 0x0f;

    instr->dest = &cpu->registers[dest].r;
    instr->src = &cpu->registers[src].r;
    return NO_ERROR;
}

inline static error_t decode_imm_operand(Cpu* cpu, instruction* instr) {
    uint8_t transfer_byte;
    if (fetch(cpu, &transfer_byte) != NO_ERROR) {
        return MEMORY_ERROR;
    }

    uint8_t dest = (transfer_byte >> 4) & 0x0f;
    uint8_t size = (transfer_byte >> 2) & 0x03;

    instr->dest = &cpu->registers[dest].r;

    switch (size) {
    case 0:
        return fetch(cpu, (uint8_t*)&instr->immediate);
    case 1:
        return fetch_2(cpu, (uint16_t*)&instr->immediate);
    case 2:
        return fetch_4(cpu, (uint32_t*)&instr->immediate);
    case 3:
        return fetch_8(cpu, (uint64_t*)&instr->immediate);
    default:
        UNREACHABLE();
    }
}

inline static error_t decode_sp_rel_addr(Cpu* cpu, instruction* instr) {
    uint8_t byte;
    if (fetch(cpu, &byte) != NO_ERROR) {
        return MEMORY_ERROR;
    }

    uint8_t scale = 1 << ((byte >> 2) & 0x03);

    uint8_t ignore_bit = byte & 1;
    uint8_t disp_width = (byte >> 1) & 1;

    instr->base_id = SP_INDEX;
    instr->index_id = INVALID_ID;
    instr->scale = scale;

    if (!ignore_bit) {
        uint8_t index_reg = (byte >> 4) & 0x0f;
        instr->index_id = index_reg;
    }

    if (disp_width == 0) {
        int32_t tmp = 0;
        if (fetch_4(cpu, (uint32_t*)&tmp) != NO_ERROR) {
            return MEMORY_ERROR;
        }
        instr->displacement = (int64_t)tmp;
    } else {
        int16_t tmp = 0;
        if (fetch_2(cpu, (uint16_t*)&tmp) != NO_ERROR) {
            return MEMORY_ERROR;
        }
        instr->displacement = (int64_t)tmp;
    }

    return NO_ERROR;
}

inline static error_t decode_bis_address(Cpu* cpu, instruction* instr) {
    uint8_t byte;
    if (fetch(cpu, &byte) != NO_ERROR) {
        return MEMORY_ERROR;
    }

    uint8_t scale = 1 << ((byte >> 2) & 0x03);

    uint8_t ignore_bit = byte & 1;
    uint8_t disp_width = (byte >> 1) & 1;

    instr->base_id = INVALID_ID;
    instr->index_id = INVALID_ID;
    instr->scale = scale;

    if (!ignore_bit) {
        uint8_t base_reg = (byte >> 4) & 0b1111;
        instr->base_id = base_reg;
    } else {
        uint8_t second_byte;
        if (fetch(cpu, &second_byte) != NO_ERROR) {
            return MEMORY_ERROR;
        }
        instr->base_id = (second_byte >> 4) & 0b1111;
        instr->index_id = second_byte & 0b1111;
    }

    if (disp_width == 0) {
        int32_t tmp = 0;
        if (fetch_4(cpu, (uint32_t*)&tmp) != NO_ERROR) {
            return MEMORY_ERROR;
        }
        instr->displacement = (int64_t)tmp;
    } else {
        int16_t tmp = 0;
        if (fetch_2(cpu, (uint16_t*)&tmp) != NO_ERROR) {
            return MEMORY_ERROR;
        }
        instr->displacement = (int64_t)tmp;
    }

    return NO_ERROR;
}

inline static error_t decode_pc_rel(Cpu* cpu, instruction* instr) {
    instr->base_id = INVALID_ID;
    instr->index_id = INVALID_ID;
    // The PC relative displacement is constant so we can set both registers
    // to be invalid here
    int32_t off = 0;
    if (fetch_4(cpu, (uint32_t*)&off) != NO_ERROR) {
        return MEMORY_ERROR;
    }
    // Encode this as a displacement because the address from this is
    // effectively a constant
    instr->displacement = (int64_t)cpu->registers[IP_INDEX].r + (int64_t)off;

    return NO_ERROR;
}

inline static error_t decode_mem_operand(Cpu* cpu, instruction* instr) {
    uint8_t byte;
    if (fetch(cpu, &byte) != NO_ERROR) {
        return MEMORY_ERROR;
    }
    int dest = (byte >> 4) & 0x0f;
    addr_mode mode = (byte >> 2) & 0x03;
    int size = byte & 0x03;
    instr->size = size;
    instr->dest = &cpu->registers[dest].r;

    switch (mode) {
    case PCRel:
        return decode_pc_rel(cpu, instr);
    case BIS:
        return decode_bis_address(cpu, instr);
    case SPRel:
        return decode_sp_rel_addr(cpu, instr);
    case Addr:
        instr->index_id = INVALID_ID;
        instr->base_id = INVALID_ID;
        instr->scale = 1;
        return fetch_8(cpu, &instr->displacement);
    }
}

error_t cpu_decode(Cpu* cpu, instruction* instr, bool* branch_point) {
    memset(instr, 0, sizeof(*instr));
    uint16_t opcode = 0;
    if (fetch(cpu, (uint8_t*)&opcode) != NO_ERROR) {
        return MEMORY_ERROR;
    }

    if (opcode == OPCODE_EXTENSION) {
        opcode <<= 8;

        // Fetch the real opcode
        if (fetch(cpu, (uint8_t*)&opcode) != NO_ERROR) {
            return MEMORY_ERROR;
        }
        // Clear the upper 8 bits and use only the lower 4 bits to index into
        // the extended opcode tables
        instr->op = ext_ops[opcode & 0xff];
        instr->cond = ext_conditions[opcode & 0xff];
    } else {
        instr->op = ops[opcode];
        instr->cond = conditions[opcode];
    }

    // PUSH and POP instruction respecively
    if (opcode >= 0xd0 && opcode <= 0xdf || opcode >= 0xe0 && opcode <= 0xef) {
        uint8_t reg_id = opcode & 0x0f;
        instr->dest = &cpu->registers[reg_id].r;
        return NO_ERROR;
    }
    // JMP and CALL with a register operand encoded in the lower 4 bits
    else if (opcode >= 0xc0 && opcode <= 0xcf ||
             opcode >= 0xb0 && opcode <= 0xbf) {
        *branch_point = true;
        instr->op_src = op_src_dereference_reg;
        uint8_t reg_id = opcode & 0x0f;
        instr->src = &cpu->registers[reg_id].r;
        instr->dest = &cpu->registers[IP_INDEX].r;
        return NO_ERROR;
    }
    // RDT (read timer) instructions (encoded with the lowest 4 bits of the
    // operand)
    else if (opcode >= 0xf0 && opcode <= 0xff) {
        instr->op_src = op_src_immediate;
        uint8_t reg_id = opcode & 0x0f;
        instr->dest = &cpu->registers[reg_id].r;
        return NO_ERROR;
    }
    // RDSP (load stack pointer) instruction
    else if (opcode >= EXT(0xf0) && opcode <= EXT(0xff)) {
        uint8_t reg_id = opcode & 0x0f;
        instr->op_src = op_src_dereference_reg;
        instr->src = &cpu->registers[SP_INDEX].r;
        instr->dest = &cpu->registers[reg_id].r;
        return NO_ERROR;
    }
    // STSP (store stack pointer) instruction
    else if (opcode >= EXT(0xe0) && opcode <= EXT(0xef)) {
        uint8_t reg_id = opcode & 0x0f;
        instr->op_src = op_src_dereference_reg;
        instr->src = &cpu->registers[reg_id].r;
        instr->dest = &cpu->registers[SP_INDEX].r;
        return NO_ERROR;
    }

    switch (opcode) {
    case EXT(0x1c):
        return NO_ERROR; // These instructions are just opcodes, nothing else to decode
    // The interrupt instruction
    case 0x01:
        instr->op_src = op_src_immediate;
        return fetch(cpu, (uint8_t*)&instr->immediate);
    case 0x00:
    case 0x02: // RET instruction
        // These instructions are usually have some special operation and don't
        // need anything else to be modified
        *branch_point = true;
        return NO_ERROR;

    // Branch instructions that take in a constant PC relative displacement
    case 0x10:
    case 0x11:
    case 0x12:
    case 0x13:
    case 0x14:
    case 0x15:
    case 0x16:
    case 0x17:
    case 0x18:
    case 0x19:
    case 0x1a:
    case 0x1b:
    case 0x1c:
    case 0x1d:
    case 0x1e:
    case 0x1f: // 0x1F is the Call rel32 instruction
        *branch_point = true;
        // The address calculated from this is always constant
        instr->op_src = op_src_immediate;
        instr->dest = &cpu->registers[IP_INDEX].r;
        return decode_pc_rel(cpu, instr);
    // Data transfer instructions between registers
    case 0x20:
    case 0x21:
    case 0x22:
    case 0x23:
    case 0x24:
    case 0x25:
    case 0x26:
    case 0x27:
    case 0x28:
    case 0x29:
    case 0x2a:
    case EXT(0x00):
    case EXT(0x01):
    case EXT(0x02):
    case EXT(0x03):
    case EXT(0x04):
    case EXT(0x05):
    case EXT(0x06):
    case EXT(0x07):
    case EXT(0x08):
    case EXT(0x09):
    case EXT(0x0a):
    case EXT(0x0b):
    case EXT(0x0c):
    case EXT(0x0d):
        instr->op_src = op_src_dereference_reg;
        return decode_reg_operand(cpu, instr);
    // Data transfer instructions between a register and immediate
    case 0x30:
    case 0x31:
    case 0x32:
    case 0x33:
    case 0x34:
    case 0x35:
    case 0x36:
    case 0x37:
    case 0x38:
    case 0x39:
    case 0x3a:
        instr->op_src = op_src_immediate;
        return decode_imm_operand(cpu, instr);
    // Data transfer instructions between a register and memory location
    case 0x40:
    case 0x41:
    case 0x42:
    case 0x43:
    case 0x44:
    case 0x45:
    case 0x46:
    case 0x47:
    case 0x48:
    case 0x49:
    case 0x4a:
    case EXT(0x0e):
    case EXT(0x0f):
    case EXT(0x10):
    case EXT(0x11):
    case EXT(0x12):
    case EXT(0x13):
    case EXT(0x14):
    case EXT(0x15):
    case EXT(0x16):
    case EXT(0x17):
    case EXT(0x18):
    case EXT(0x19):
    case EXT(0x1a):
    case EXT(0x1b):
        instr->op_src = op_src_dereference_mem;
        return decode_mem_operand(cpu, instr);

    // STR and LEA instructions
    case 0x09:
    case 0x08:
        instr->op_src = op_src_calculate_address;
        return decode_mem_operand(cpu, instr);

    default:
        return DECODE_ERROR;
    }
}
