#include "block.h"
#include <stdlib.h>

static void resize(block* buf) {
    uint32_t new_cap = (buf->cap + 1) * 2;
    buf->cap = new_cap;

    instruction* instr = realloc(buf->instructions, new_cap * sizeof(instruction));
    buf->instructions = instr;
}

block instruction_buf_create(uint32_t cap) {
    block buf = {.cap = cap, .len = 0};
    buf.instructions = malloc(sizeof(instruction) * cap);
    return buf;
}

void instruction_buf_append(block* buf, instruction* instr) {
    if (buf->len >= buf->cap) {
        resize(buf);
    }

    buf->instructions[buf->len++] = *instr;
}
