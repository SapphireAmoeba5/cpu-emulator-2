#include "instructions.h"
#include "cpu.h"
#include "memory.h"

#include <inttypes.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

// Split into two different structs because the encoding is slightly different
typedef struct dest_byte {
    uint8_t reg_index;
    uint8_t size;
} destbyte;

typedef struct src_byte {
    uint8_t reg_index;
} srcbyte;

destbyte get_dest_byte(uint8_t byte) {
    destbyte destbyte;
    destbyte.size = (byte & 0b110) >> 1;
    destbyte.reg_index = byte >> 3;
    return destbyte;
}

srcbyte get_src_byte(uint8_t byte) {
    srcbyte srcbyte;
    srcbyte.reg_index = byte >> 3;
    return srcbyte;
}

#define checked_div(result, a, b)  \
    _Generic((a),                  \
        int8_t: checked_div_i8,    \
        int16_t: checked_div_i16,  \
        int32_t: checked_div_i32,  \
        int64_t: checked_div_i64,  \
        uint8_t: checked_div_u8,   \
        uint16_t: checked_div_u16, \
        uint32_t: checked_div_u32, \
        uint64_t: checked_div_u64)(result, a, b)

bool checked_div_i8(int8_t* result, int8_t a, int8_t b) {
    if (a == INT8_MIN && b == -1 || b == 0) {
        return true;
    } else {
        *result = a / b;
        return false;
    }
}

bool checked_div_i16(int16_t* result, int16_t a, int16_t b) {
    if (a == INT16_MIN && b == -1 || b == 0) {
        return true;
    } else {
        *result = a / b;
        return false;
    }
}

bool checked_div_i32(int32_t* result, int32_t a, int32_t b) {
    if (a == INT32_MIN && b == -1 || b == 0) {
        return true;
    } else {
        *result = a / b;
        return false;
    }
}

bool checked_div_i64(int64_t* result, int64_t a, int64_t b) {
    if (a == INT64_MIN && b == -1 || b == 0) {
        return true;
    } else {
        *result = a / b;
        return false;
    }
}

bool checked_div_u8(uint8_t* result, uint8_t a, uint8_t b) {
    if (b == 0) {
        return true;
    } else {
        *result = a / b;
        return false;
    }
}

bool checked_div_u16(uint16_t* result, uint16_t a, uint16_t b) {
    if (b == 0) {
        return true;
    } else {
        *result = a / b;
        return false;
    }
}

bool checked_div_u32(uint32_t* result, uint32_t a, uint32_t b) {
    if (b == 0) {
        return true;
    } else {
        *result = a / b;
        return false;
    }
}

bool checked_div_u64(uint64_t* result, uint64_t a, uint64_t b) {
    if (b == 0) {
        return true;
    } else {
        *result = a / b;
        return false;
    }
}

void invl(Cpu* cpu, uint8_t instruction[16]) {
    printf("Invalid instruction opcode: %d\n", (uint32_t)instruction[0]);
    exit(1);
}

void halt(Cpu* cpu, uint8_t instructions[16]) {
    cpu->ip += 1;
    // printf("Halted after %lu clock cycles\n", cpu->clock_count);
    // for(int i = 0; i < 32; i++) {
    //     uint64_t value = cpu->registers[i].r;
    //     printf("r%lu = %lu\n", (uint64_t)i, value);
    // }
    cpu->halt = true;
}

void intpt(Cpu* cpu, uint8_t instruction[16]) {
    cpu->ip += 2;

    uint8_t index = instruction[1];

    if (index == 80) {
        printf("Cycle: %lu\n", cpu->clock_count);
        for (int i = 0; i < 32; i++) {
            uint64_t value = cpu->registers[i].r;
            printf("r%lu = %016lx (%ld)\n", (uint64_t)i, value, (int64_t)value);
        }

        printf("ip: %lu/%lu\nsp: %lu\n", cpu->ip, memory_size_bytes(cpu->memory), cpu->sp);

        cpu->exit = true;
    }
}

void mov_reg_reg(Cpu* cpu, uint8_t instruction[16]) {
    cpu->ip += 3;

    destbyte dest_byte = get_dest_byte(instruction[1]);

    srcbyte src_byte = get_src_byte(instruction[2]);

    reg* dest = &cpu->registers[dest_byte.reg_index];
    reg* src = &cpu->registers[src_byte.reg_index];

    switch (dest_byte.size) {
    case 0:
        dest->b = src->b;
        break;
    case 1:
        dest->s = src->s;
        break;
    case 2:
        dest->w = src->w;
        break;
    case 3:
        dest->r = src->r;
    }

    // printf("mov r%d, r%d\n", (uint32_t)dest_byte.reg_index, (uint32_t)src_byte.reg_index);
}

void mov_reg_imm8(Cpu* cpu, uint8_t instruction[16]) {
    cpu->ip += 3;
    destbyte dest_byte = get_dest_byte(instruction[1]);

    uint8_t value = instruction[2];

    cpu->registers[dest_byte.reg_index].b = value;

    // printf("mov b%d, %d\n", (uint32_t)dest_byte.reg_index, (uint32_t)value);
}
void mov_reg_imm16(Cpu* cpu, uint8_t instruction[16]) {
    cpu->ip += 4;

    destbyte dest_byte = get_dest_byte(instruction[1]);

    uint16_t value;
    memcpy(&value, &instruction[2], sizeof(uint16_t));

    cpu->registers[dest_byte.reg_index].s = value;

    // printf("mov s%d, %d\n", (uint32_t)dest_byte.reg_index, (uint32_t)value);
}
void mov_reg_imm32(Cpu* cpu, uint8_t instruction[16]) {
    cpu->ip += 6;

    destbyte dest_byte = get_dest_byte(instruction[1]);

    uint32_t value;
    memcpy(&value, &instruction[2], sizeof(uint32_t));

    cpu->registers[dest_byte.reg_index].w = value;

    // printf("mov w%d, %d\n", (uint32_t)dest_byte.reg_index, (uint32_t)value);
}
void mov_reg_imm64(Cpu* cpu, uint8_t instruction[16]) {
    cpu->ip += 10;

    destbyte dest_byte = get_dest_byte(instruction[1]);

    uint64_t value;
    memcpy(&value, &instruction[2], sizeof(uint64_t));

    cpu->registers[dest_byte.reg_index].r = value;

    // printf("mov r%d, %" PRIu64 "\n", (uint32_t)dest_byte.reg_index, value);
}

void add_reg_reg(Cpu* cpu, uint8_t instruction[16]) {
    cpu->ip += 3;

    destbyte dest_byte = get_dest_byte(instruction[1]);
    srcbyte src_byte = get_src_byte(instruction[2]);

    reg* dest = &cpu->registers[dest_byte.reg_index];
    reg* src = &cpu->registers[src_byte.reg_index];

    switch (dest_byte.size) {
    case 0:
        dest->b += src->b;
        break;
    case 1:
        dest->s += src->s;
        break;
    case 2:
        dest->w += src->w;
        break;
    case 3:
        dest->r += src->r;
    }
}
void add_reg_imm8(Cpu* cpu, uint8_t instruction[16]) {
    cpu->ip += 3;
    destbyte dest_byte = get_dest_byte(instruction[1]);

    uint8_t value = instruction[2];

    cpu->registers[dest_byte.reg_index].b += value;
}
void add_reg_imm16(Cpu* cpu, uint8_t instruction[16]) {
    cpu->ip += 4;

    destbyte dest_byte = get_dest_byte(instruction[1]);

    uint16_t value;
    memcpy(&value, &instruction[2], sizeof(uint16_t));

    cpu->registers[dest_byte.reg_index].s += value;
}
void add_reg_imm32(Cpu* cpu, uint8_t instruction[16]) {
    cpu->ip += 6;

    destbyte dest_byte = get_dest_byte(instruction[1]);

    uint32_t value;
    memcpy(&value, &instruction[2], sizeof(uint32_t));

    cpu->registers[dest_byte.reg_index].w += value;
}
void add_reg_imm64(Cpu* cpu, uint8_t instruction[16]) {
    cpu->ip += 10;

    destbyte dest_byte = get_dest_byte(instruction[1]);

    uint64_t value;
    memcpy(&value, &instruction[2], sizeof(uint64_t));

    cpu->registers[dest_byte.reg_index].r += value;
}

void sub_reg_reg(Cpu* cpu, uint8_t instruction[16]) {
    cpu->ip += 3;

    destbyte dest_byte = get_dest_byte(instruction[1]);
    srcbyte src_byte = get_src_byte(instruction[2]);

    reg* dest = &cpu->registers[dest_byte.reg_index];
    reg* src = &cpu->registers[src_byte.reg_index];

    switch (dest_byte.size) {
    case 0:
        dest->b -= src->b;
        break;
    case 1:
        dest->s -= src->s;
        break;
    case 2:
        dest->w -= src->w;
        break;
    case 3:
        dest->r -= src->r;
    }
}
void sub_reg_imm8(Cpu* cpu, uint8_t instruction[16]) {
    cpu->ip += 3;
    destbyte dest_byte = get_dest_byte(instruction[1]);

    uint8_t value = instruction[2];

    cpu->registers[dest_byte.reg_index].b -= value;
}
void sub_reg_imm16(Cpu* cpu, uint8_t instruction[16]) {
    cpu->ip += 4;

    destbyte dest_byte = get_dest_byte(instruction[1]);

    uint16_t value;
    memcpy(&value, &instruction[2], sizeof(uint16_t));

    cpu->registers[dest_byte.reg_index].s -= value;
}
void sub_reg_imm32(Cpu* cpu, uint8_t instruction[16]) {
    cpu->ip += 6;

    destbyte dest_byte = get_dest_byte(instruction[1]);

    uint32_t value;
    memcpy(&value, &instruction[2], sizeof(uint32_t));

    cpu->registers[dest_byte.reg_index].w -= value;
}
void sub_reg_imm64(Cpu* cpu, uint8_t instruction[16]) {
    cpu->ip += 10;

    destbyte dest_byte = get_dest_byte(instruction[1]);

    uint64_t value;
    memcpy(&value, &instruction[2], sizeof(uint64_t));

    cpu->registers[dest_byte.reg_index].r -= value;
}

void mul_reg_reg(Cpu* cpu, uint8_t instruction[16]) {
    cpu->ip += 3;

    destbyte dest_byte = get_dest_byte(instruction[1]);
    srcbyte src_byte = get_src_byte(instruction[2]);

    reg* dest = &cpu->registers[dest_byte.reg_index];
    reg* src = &cpu->registers[src_byte.reg_index];

    switch (dest_byte.size) {
    case 0:
        dest->b *= src->b;
        break;
    case 1:
        dest->s *= src->s;
        break;
    case 2:
        dest->w *= src->w;
        break;
    case 3:
        dest->r *= src->r;
    }
}
void mul_reg_imm8(Cpu* cpu, uint8_t instruction[16]) {
    cpu->ip += 3;
    destbyte dest_byte = get_dest_byte(instruction[1]);

    uint8_t value = instruction[2];

    cpu->registers[dest_byte.reg_index].b *= value;
}
void mul_reg_imm16(Cpu* cpu, uint8_t instruction[16]) {
    cpu->ip += 4;

    destbyte dest_byte = get_dest_byte(instruction[1]);

    uint16_t value;
    memcpy(&value, &instruction[2], sizeof(uint16_t));

    cpu->registers[dest_byte.reg_index].s *= value;
}
void mul_reg_imm32(Cpu* cpu, uint8_t instruction[16]) {
    cpu->ip += 6;

    destbyte dest_byte = get_dest_byte(instruction[1]);

    uint32_t value;
    memcpy(&value, &instruction[2], sizeof(uint32_t));

    cpu->registers[dest_byte.reg_index].w *= value;
}
void mul_reg_imm64(Cpu* cpu, uint8_t instruction[16]) {
    cpu->ip += 10;

    destbyte dest_byte = get_dest_byte(instruction[1]);

    uint64_t value;
    memcpy(&value, &instruction[2], sizeof(uint64_t));

    cpu->registers[dest_byte.reg_index].r *= value;
}

void div_reg_reg(Cpu* cpu, uint8_t instruction[16]) {
    cpu->ip += 3;

    destbyte dest_byte = get_dest_byte(instruction[1]);
    srcbyte src_byte = get_src_byte(instruction[2]);

    reg* dest = &cpu->registers[dest_byte.reg_index];
    reg* src = &cpu->registers[src_byte.reg_index];

    switch (dest_byte.size) {
    case 0:
        if (checked_div(&dest->b, dest->b, src->b)) {
            printf("Invalid division\n");
            exit(1);
        }
        break;
    case 1:
        if (checked_div(&dest->s, dest->s, src->s)) {
            printf("Invalid division\n");
            exit(1);
        }
        break;
    case 2:
        if (checked_div(&dest->w, dest->w, src->w)) {
            printf("Invalid division\n");
            exit(1);
        }
        break;
    case 3:
        if (checked_div(&dest->r, dest->r, src->r)) {
            printf("Invalid division\n");
            exit(1);
        }
        break;
    }
}
void div_reg_imm8(Cpu* cpu, uint8_t instruction[16]) {
    cpu->ip += 3;
    destbyte dest_byte = get_dest_byte(instruction[1]);

    uint8_t value = instruction[2];

    reg* dest = &cpu->registers[dest_byte.reg_index];
    if (checked_div(&dest->b, dest->b, value)) {
        printf("Invalid division\n");
        exit(1);
    }
}
void div_reg_imm16(Cpu* cpu, uint8_t instruction[16]) {
    cpu->ip += 4;

    destbyte dest_byte = get_dest_byte(instruction[1]);

    uint16_t value;
    memcpy(&value, &instruction[2], sizeof(uint16_t));

    reg* dest = &cpu->registers[dest_byte.reg_index];
    if (checked_div(&dest->s, dest->s, value)) {
        printf("Invalid division\n");
        exit(1);
    }
}
void div_reg_imm32(Cpu* cpu, uint8_t instruction[16]) {
    cpu->ip += 6;

    destbyte dest_byte = get_dest_byte(instruction[1]);

    uint32_t value;
    memcpy(&value, &instruction[2], sizeof(uint32_t));

    reg* dest = &cpu->registers[dest_byte.reg_index];
    if (checked_div(&dest->w, dest->w, value)) {
        printf("Invalid division\n");
        exit(1);
    }
}
void div_reg_imm64(Cpu* cpu, uint8_t instruction[16]) {
    cpu->ip += 10;

    destbyte dest_byte = get_dest_byte(instruction[1]);

    uint64_t value;
    memcpy(&value, &instruction[2], sizeof(uint64_t));

    reg* dest = &cpu->registers[dest_byte.reg_index];
    if (checked_div(&dest->r, dest->r, value)) {
        printf("Invalid division %lu\n", value);
        exit(1);
    }
}

void idiv_reg_reg(Cpu* cpu, uint8_t instruction[16]) {
    cpu->ip += 3;

    destbyte dest_byte = get_dest_byte(instruction[1]);
    srcbyte src_byte = get_src_byte(instruction[2]);

    reg* dest = &cpu->registers[dest_byte.reg_index];
    reg* src = &cpu->registers[src_byte.reg_index];

    switch (dest_byte.size) {
    case 0:
        if(checked_div_i8((int8_t*)&dest->b, (int8_t)dest->b, (int8_t)src->b)) {
            printf("Invalid division\n");
            exit(1);
        }
        break;
    case 1:
        if(checked_div_i16((int16_t*)&dest->s, (int16_t)dest->s, (int16_t)src->s)) {
            printf("Invalid division\n");
            exit(1);
        }
        break;
    case 2:
        if(checked_div_i32((int32_t*)&dest->w, (int32_t)dest->w, (int32_t)src->w)) {
            printf("Invalid division\n");
            exit(1);
        }
        break;
    case 3:
        if(checked_div_i64((int64_t*)&dest->r, (int64_t)dest->r, (int64_t)src->r)) {
            printf("Invalid division\n");
            exit(1);
        }
    }
}
void idiv_reg_imm8(Cpu* cpu, uint8_t instruction[16]) {
    cpu->ip += 3;
    destbyte dest_byte = get_dest_byte(instruction[1]);

    uint8_t value = instruction[2];

    if (value == 0) {
        printf("Zero dividend\n");
        exit(1);
    }
    cpu->registers[dest_byte.reg_index].b /= value;
}
void idiv_reg_imm16(Cpu* cpu, uint8_t instruction[16]) {
    cpu->ip += 4;

    destbyte dest_byte = get_dest_byte(instruction[1]);

    uint16_t value;
    memcpy(&value, &instruction[2], sizeof(uint16_t));

    if (value == 0) {
        printf("Zero dividend\n");
        exit(1);
    }
    cpu->registers[dest_byte.reg_index].s /= value;
}
void idiv_reg_imm32(Cpu* cpu, uint8_t instruction[16]) {
    cpu->ip += 6;

    destbyte dest_byte = get_dest_byte(instruction[1]);

    uint32_t value;
    memcpy(&value, &instruction[2], sizeof(uint32_t));

    if (value == 0) {
        printf("Zero dividend\n");
        exit(1);
    }
    cpu->registers[dest_byte.reg_index].w /= value;
}
void idiv_reg_imm64(Cpu* cpu, uint8_t instruction[16]) {
    cpu->ip += 10;

    destbyte dest_byte = get_dest_byte(instruction[1]);

    uint64_t value;
    memcpy(&value, &instruction[2], sizeof(uint64_t));

    if (value == 0) {
        printf("Zero dividend\n");
        exit(1);
    }

    cpu->registers[dest_byte.reg_index].r = (int64_t)cpu->registers[dest_byte.reg_index].r / (int64_t)value;
}

void and_reg_reg(Cpu* cpu, uint8_t instruction[16]) {
    cpu->ip += 3;

    destbyte dest_byte = get_dest_byte(instruction[1]);
    srcbyte src_byte = get_src_byte(instruction[2]);

    reg* dest = &cpu->registers[dest_byte.reg_index];
    reg* src = &cpu->registers[src_byte.reg_index];

    switch (dest_byte.size) {
    case 0:
        dest->b &= src->b;
        break;
    case 1:
        dest->s &= src->s;
        break;
    case 2:
        dest->w &= src->w;
        break;
    case 3:
        dest->r &= src->r;
    }
}
void and_reg_imm8(Cpu* cpu, uint8_t instruction[16]) {
    cpu->ip += 3;
    destbyte dest_byte = get_dest_byte(instruction[1]);

    uint8_t value = instruction[2];

    cpu->registers[dest_byte.reg_index].b &= value;
}
void and_reg_imm16(Cpu* cpu, uint8_t instruction[16]) {
    cpu->ip += 4;

    destbyte dest_byte = get_dest_byte(instruction[1]);

    uint16_t value;
    memcpy(&value, &instruction[2], sizeof(uint16_t));

    cpu->registers[dest_byte.reg_index].s &= value;
}
void and_reg_imm32(Cpu* cpu, uint8_t instruction[16]) {
    cpu->ip += 6;

    destbyte dest_byte = get_dest_byte(instruction[1]);

    uint32_t value;
    memcpy(&value, &instruction[2], sizeof(uint32_t));

    cpu->registers[dest_byte.reg_index].w &= value;
}
void and_reg_imm64(Cpu* cpu, uint8_t instruction[16]) {
    cpu->ip += 10;

    destbyte dest_byte = get_dest_byte(instruction[1]);

    uint64_t value;
    memcpy(&value, &instruction[2], sizeof(uint64_t));

    cpu->registers[dest_byte.reg_index].r &= value;
}

void or_reg_reg(Cpu* cpu, uint8_t instruction[16]) {
    cpu->ip += 3;

    destbyte dest_byte = get_dest_byte(instruction[1]);
    srcbyte src_byte = get_src_byte(instruction[2]);

    reg* dest = &cpu->registers[dest_byte.reg_index];
    reg* src = &cpu->registers[src_byte.reg_index];

    switch (dest_byte.size) {
    case 0:
        dest->b |= src->b;
        break;
    case 1:
        dest->s |= src->s;
        break;
    case 2:
        dest->w |= src->w;
        break;
    case 3:
        dest->r |= src->r;
    }
}

void or_reg_imm8(Cpu* cpu, uint8_t instruction[16]) {
    cpu->ip += 3;
    destbyte dest_byte = get_dest_byte(instruction[1]);

    uint8_t value = instruction[2];

    cpu->registers[dest_byte.reg_index].b |= value;
}
void or_reg_imm16(Cpu* cpu, uint8_t instruction[16]) {
    cpu->ip += 4;

    destbyte dest_byte = get_dest_byte(instruction[1]);

    uint16_t value;
    memcpy(&value, &instruction[2], sizeof(uint16_t));

    cpu->registers[dest_byte.reg_index].s |= value;
}
void or_reg_imm32(Cpu* cpu, uint8_t instruction[16]) {
    cpu->ip += 6;

    destbyte dest_byte = get_dest_byte(instruction[1]);

    uint32_t value;
    memcpy(&value, &instruction[2], sizeof(uint32_t));

    cpu->registers[dest_byte.reg_index].w |= value;
}
void or_reg_imm64(Cpu* cpu, uint8_t instruction[16]) {
    cpu->ip += 10;

    destbyte dest_byte = get_dest_byte(instruction[1]);

    uint64_t value;
    memcpy(&value, &instruction[2], sizeof(uint64_t));

    cpu->registers[dest_byte.reg_index].r |= value;
}

void xor_reg_reg(Cpu* cpu, uint8_t instruction[16]) {
    cpu->ip += 3;

    destbyte dest_byte = get_dest_byte(instruction[1]);
    srcbyte src_byte = get_src_byte(instruction[2]);

    reg* dest = &cpu->registers[dest_byte.reg_index];
    reg* src = &cpu->registers[src_byte.reg_index];

    switch (dest_byte.size) {
    case 0:
        dest->b ^= src->b;
        break;
    case 1:
        dest->s ^= src->s;
        break;
    case 2:
        dest->w ^= src->w;
        break;
    case 3:
        dest->r ^= src->r;
    }
}

void xor_reg_imm8(Cpu* cpu, uint8_t instruction[16]) {
    cpu->ip += 3;
    destbyte dest_byte = get_dest_byte(instruction[1]);

    uint8_t value = instruction[2];

    cpu->registers[dest_byte.reg_index].b ^= value;
}
void xor_reg_imm16(Cpu* cpu, uint8_t instruction[16]) {
    cpu->ip += 4;

    destbyte dest_byte = get_dest_byte(instruction[1]);

    uint16_t value;
    memcpy(&value, &instruction[2], sizeof(uint16_t));

    cpu->registers[dest_byte.reg_index].s ^= value;
}
void xor_reg_imm32(Cpu* cpu, uint8_t instruction[16]) {
    cpu->ip += 6;

    destbyte dest_byte = get_dest_byte(instruction[1]);

    uint32_t value;
    memcpy(&value, &instruction[2], sizeof(uint32_t));

    cpu->registers[dest_byte.reg_index].w ^= value;
}
void xor_reg_imm64(Cpu* cpu, uint8_t instruction[16]) {
    cpu->ip += 10;

    destbyte dest_byte = get_dest_byte(instruction[1]);

    uint64_t value;
    memcpy(&value, &instruction[2], sizeof(uint64_t));

    cpu->registers[dest_byte.reg_index].r ^= value;
}
