#pragma once
#include "instruction.h"
#include <stdint.h>

typedef struct {
    // The instructions in this block
    instruction* instructions;
    uint32_t len;
    uint32_t cap;
} block;

block instruction_buf_create(uint32_t cap);
// Copies the instruction to the end of the buffer
void instruction_buf_append(block* buf, instruction* instr);
