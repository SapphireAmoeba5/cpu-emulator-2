#pragma once

#include "cpu.h"
#include <stdint.h>

// Operation for the instruction
typedef enum {
    // Invalid op
    op_invl,
    op_halt,
    op_int,
    op_mov,
    op_add,
    op_sub,
    op_mul,
    op_div,
    op_idiv,
    op_and,
    op_or,
    op_xor,
    op_cmp,
    op_test,
}iop;

typedef struct {
    iop op;
    uint64_t* dest;
    uint64_t src;
} instruction;

bool cpu_decode(Cpu* cpu, instruction* instr);
