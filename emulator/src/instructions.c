#include "instructions.h"
#include "cpu.h"
#include "memory.h"

#include <assert.h>
#include <inttypes.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define INT_SUB_OVERFLOW_CHECK(A, B) __builtin_sub_overflow_p((A), (B), 0)

#define REG_OPERATION(OP)                                                      \
    cpu->ip += 2;                                                              \
    uint8_t transfer_byte = instruction[1];                                    \
    uint8_t dest;                                                              \
    uint8_t src;                                                               \
    parse_reg_transfer_byte(transfer_byte, &dest, &src);                       \
    cpu->registers[dest] = cpu->registers[src];

#define IMM_OPERATION(OP)                                                      \
    cpu->ip += 2;                                                              \
    uint8_t dest;                                                              \
    int size;                                                                  \
    bool sign_extend;                                                          \
    parse_imm_transfer_byte(instruction[1], &dest, &size, &sign_extend);       \
    cpu->ip += 1 << size;                                                      \
    uint64_t value = 0;                                                        \
    memcpy(&value, &instruction[2], 1 << size);                                \
    switch (size) {                                                            \
    case 0: {                                                                  \
        if (sign_extend) {                                                     \
            value = (int64_t)((int8_t)value);                                  \
            cpu->registers[dest].r OP value;                                   \
        } else {                                                               \
            cpu->registers[dest].b OP value;                                   \
        }                                                                      \
        break;                                                                 \
    }                                                                          \
    case 1: {                                                                  \
        if (sign_extend) {                                                     \
            value = (int64_t)((int16_t)value);                                 \
            cpu->registers[dest].r OP value;                                   \
        } else {                                                               \
            cpu->registers[dest].s OP value;                                   \
        }                                                                      \
        break;                                                                 \
    }                                                                          \
    case 2: {                                                                  \
        if (sign_extend) {                                                     \
            value = (int64_t)((int32_t)value);                                 \
            cpu->registers[dest].r OP value;                                   \
        } else {                                                               \
            cpu->registers[dest].w OP value;                                   \
        }                                                                      \
        break;                                                                 \
    }                                                                          \
    case 3: {                                                                  \
        cpu->registers[dest].r OP value;                                       \
        break;                                                                 \
    }                                                                          \
    }

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

#define checked_div(result, a, b)                                              \
    _Generic((a),                                                              \
        int8_t: checked_div_i8,                                                \
        int16_t: checked_div_i16,                                              \
        int32_t: checked_div_i32,                                              \
        int64_t: checked_div_i64,                                              \
        uint8_t: checked_div_u8,                                               \
        uint16_t: checked_div_u16,                                             \
        uint32_t: checked_div_u32,                                             \
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
    printf("Invalid instruction opcode: %02x (dec: %d)\n",
           (uint32_t)instruction[0], (int32_t)instruction[0]);
    exit(1);
}

void halt(Cpu* cpu, uint8_t instructions[16]) {
    cpu->ip += 1;
    printf("REACHED HALT\n");
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

    if (index == 0x80) {
        printf("Cycle: %lu\n", cpu->clock_count);
        for (int i = 0; i < 32; i++) {
            uint64_t value = cpu->registers[i].r;
            printf("r%lu = %016lx (%ld)\n", (uint64_t)i, value, (int64_t)value);
        }

        printf("ip: %lu/%lu\nsp: %lu\n", cpu->ip,
               memory_size_bytes(cpu->memory), cpu->sp);

        printf("%b\n", cpu->flags);

        cpu->exit = true;
    }
    else if(index == 0x81) {
        cpu->registers[0].r -= 1;
        cpu->flags &= ~FLAG_ZERO;
        if(cpu->registers[0].r == 0) {
            cpu->flags |= FLAG_ZERO;
        }
    }
    else if(index == 0x82) {
        printf("DEBUG PRINT %llu\n", cpu->clock_count);
    }
}

static inline void parse_reg_transfer_byte(uint8_t byte, uint8_t* dest,
                                           uint8_t* src) {
    *dest = (byte >> 4) & 0x0f;
    *src = byte & 0x0f;
}

/// `dest`, `size` and `sign_extend` must be non-null and valid pointers
/// `size` will contain one of 0, 1, 2, 3
static inline void parse_imm_transfer_byte(uint8_t byte, uint8_t* dest,
                                           int* size, bool* sign_extend) {
    *dest = (byte >> 4) & 0x0f;
    *size = (byte >> 2) & 0b11;
    *sign_extend = (byte >> 1) & 1;
}

typedef enum {
    PCRel = 0,
    SPRel = 1,
    // Base + Index * Scale
    BIS = 2,
    Addr = 3,

} addr_mode;

/// `dest`, `addr_mode`, and `size` must not be NULL and be valid pointers
/// `size` will be assigned 0, 1, 2, or 3 which encodes the byte width as a
/// power of two
static inline void parse_transfer_byte(uint8_t byte, uint8_t* dest,
                                       addr_mode* addr_mode, uint8_t* size) {
    *dest = (byte >> 4) & 0x0f;
    *addr_mode = (byte >> 2) & 0b11;
    *size = byte & 0b11;
}

// TODO: Make the passed in `instruction` start at the beginning of where the
// function should start computing
static inline uint64_t get_pc_rel_addr(Cpu* cpu, uint8_t instruction[16]) {
    cpu->ip += 4;
    int32_t off = 0;
    memcpy(&off, &instruction[2], sizeof(off));
    uint64_t address = (uint64_t)((int64_t)cpu->ip + (int64_t)off);

    return address;
}

static inline bool load_pc_rel(Cpu* cpu, uint8_t instruction[16], uint8_t size,
                               uint64_t* value) {
    uint64_t address = get_pc_rel_addr(cpu, instruction);

    switch (size) {
    case 0:
        // Only works on little endian
        if (!cpu_read_1(cpu, address, (uint8_t*)value)) {
            return false;
        }
        break;
    case 1:
        // Only works on little endian
        if (!cpu_read_2(cpu, address, (uint16_t*)value)) {
            return false;
        }
        break;
    case 2:
        // Only works on little endian
        if (!cpu_read_4(cpu, address, (uint32_t*)value)) {
            return false;
        }
        break;
    case 3:
        // Only works on little endian
        if (!cpu_read_8(cpu, address, value)) {
            return false;
        }
        break;
    default:
        __builtin_unreachable();
    }

    return true;
}

static inline uint64_t get_sp_rel_addr(Cpu* cpu, uint8_t instruction[16]) {
    cpu->ip += 1;
}

/// `value` must be non null and a valid pointer
/// `size` must be either 0, 1, 2, or 3
static inline bool load_from_mem(Cpu* cpu, uint64_t address, int size,
                                 uint64_t* value) {
    switch (size) {
    case 0: {
        uint8_t byte;
        if (!cpu_read_1(cpu, address, &byte)) {
            return false;
        }
        *value = byte;
        break;
    }
    case 1: {
        uint16_t word;
        if (!cpu_read_2(cpu, address, &word)) {
            return false;
        }
        *value = word;
        break;
    }
    case 2: {
        uint32_t dword;
        if (!cpu_read_4(cpu, address, &dword)) {
            return false;
        }
        *value = dword;
        break;
    }
    case 3: {
        if (!cpu_read_8(cpu, address, value)) {
            return false;
        }
    }
    }
    return true;
}

void mov_reg(Cpu* cpu, uint8_t instruction[16]) {
    cpu->ip += 2;
    uint8_t transfer_byte = instruction[1];
    uint8_t dest;
    uint8_t src;
    parse_reg_transfer_byte(transfer_byte, &dest, &src);
    cpu->registers[dest] = cpu->registers[src];
}

void mov_mem(Cpu* cpu, uint8_t instruction[16]) {
    // Opcode and the transfer byte
    cpu->ip += 2;

    uint8_t dest;
    addr_mode addr_mode;
    uint8_t size;

    uint64_t value = 0;
    parse_transfer_byte(instruction[1], &dest, &addr_mode, &size);

    switch (addr_mode) {
    case PCRel:
        if (!load_pc_rel(cpu, instruction, size, &value)) {
            return;
        }
        break;
    case BIS: {
        assert(1 == 0);
        break;
    }

    case SPRel: {
        assert(1 == 0);
        break;
    }

    case Addr: {
        cpu->ip += 8;
        // The address is always 8 bytes
        uint64_t address;
        // Only works on little endian systems
        memcpy(&address, &instruction[2], sizeof(address));
        load_from_mem(cpu, address, size, &value);

        break;
    }
    }

    switch (size) {
    case 0:
        cpu->registers[dest].b = value;
        break;
    case 1:
        cpu->registers[dest].s = value;
        break;
    case 2:
        cpu->registers[dest].w = value;
        break;
    case 3:
        cpu->registers[dest].r = value;
        break;
    }
}

void mov_imm(Cpu* cpu, uint8_t instruction[16]) {
    cpu->ip += 2;
    uint8_t dest;
    int size;
    bool sign_extend;
    parse_imm_transfer_byte(instruction[1], &dest, &size, &sign_extend);
    cpu->ip += 1 << size;
    uint64_t value = 0;
    memcpy(&value, &instruction[2], 1 << size);
    switch (size) {
    case 0: {
        if (sign_extend) {
            value = (int64_t)((int8_t)value);
            cpu->registers[dest].r = value;
        } else {
            cpu->registers[dest].b = value;
        }
        break;
    }
    case 1: {
        if (sign_extend) {
            value = (int64_t)((int16_t)value);
            cpu->registers[dest].r = value;
        } else {
            cpu->registers[dest].s = value;
        }
        break;
    }
    case 2: {
        if (sign_extend) {
            value = (int64_t)((int32_t)value);
            cpu->registers[dest].r = value;
        } else {
            cpu->registers[dest].w = value;
        }
        break;
    }
    case 3: {
        cpu->registers[dest].r = value;
        break;
    }
    }
}

void sub_reg(Cpu* cpu, uint8_t instruction[16]) {
    cpu->ip += 2;
    uint8_t transfer_byte = instruction[1];
    uint8_t dest;
    uint8_t src;
    parse_reg_transfer_byte(transfer_byte, &dest, &src);
    uint64_t left = cpu->registers[dest].r;
    uint64_t right = cpu->registers[src].r;
    uint64_t difference;

    cpu->flags &= ~(FLAG_ZERO | FLAG_CARRY | FLAG_OVERFLOW | FLAG_SIGN);

    if(__builtin_sub_overflow(left, right, &difference)) {
        cpu->flags |= FLAG_CARRY;
    }

    int64_t tmp;
    if(__builtin_sub_overflow((int64_t)left, (int64_t)right, &tmp)) {
        cpu->flags |= FLAG_OVERFLOW;
    }


    if(difference == 0) {
        cpu->flags |= FLAG_ZERO;
    }
    if((int64_t)difference < 0) {
        cpu->flags |= FLAG_SIGN;
    }

    cpu->registers[dest].r = difference;
}
void sub_mem(Cpu* cpu, uint8_t instruction[16]) {

}
void sub_imm(Cpu* cpu, uint8_t instruction[16]) {

}

void str(Cpu* cpu, uint8_t instruction[16]) {
    cpu->ip += 2;

    uint8_t dest;
    addr_mode addr_mode;
    uint8_t size;

    parse_transfer_byte(instruction[1], &dest, &addr_mode, &size);

    uint64_t address;
    switch (addr_mode) {
    case PCRel:
        address = get_pc_rel_addr(cpu, instruction);
        break;
    case BIS:
        assert(1 == 0);
        break;
    case SPRel:
        assert(1 == 0);
        break;
    case Addr:
        // The immediate value for the str instruction is always 8 bytes
        cpu->ip += 8;

        memcpy(&address, &instruction[2], sizeof(address));
        break;
    }

    switch (size) {
    case 0:
        cpu_write_1(cpu, cpu->registers[dest].b, address);
        break;
    case 1:
        cpu_write_2(cpu, cpu->registers[dest].s, address);
        break;
    case 2:
        cpu_write_4(cpu, cpu->registers[dest].w, address);
        break;
    case 3:
        cpu_write_8(cpu, cpu->registers[dest].r, address);
        break;
    default:
        __builtin_unreachable();
    }
}

void jmp(Cpu* cpu, uint8_t instruction[16]) {
    cpu->ip += 5;
    int32_t offset;
    memcpy(&offset, &instruction[1], sizeof(offset));
    cpu->ip += (int64_t)offset;
}

void jnz(Cpu* cpu, uint8_t instruction[16]) {
    cpu->ip += 5;
    if((cpu->flags & FLAG_ZERO) == 0) {
        int32_t offset;
        memcpy(&offset, &instruction[1], sizeof(offset));
        cpu->ip += (int64_t)offset;
    }
}
