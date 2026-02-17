#pragma once

#include <stdint.h>
// Operation for the instruction
typedef enum : uint8_t {
    // Invalid op
    op_invl,
    op_halt,
    op_int,
    op_mov,
    op_str,
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
    op_push,
    op_pop,
    op_rdt,
    op_call,
    op_ret,
    // Number of variants
    op_LENGTH,
} iop;

typedef enum : uint8_t {
    op_src_immediate,
    op_src_dereference_reg,
    op_src_dereference_mem,
    // The source value is the calculated address
    op_src_calculate_address,
} op_src;

/// This encodes the conditions for operations to be executed. This is how
/// conditional moves and conditional jumps are implemented
typedef enum : uint8_t {
    // This condition is always true no matter what
    cd_true,
    cd_zero,
    cd_nzero,
    cd_carry,
    cd_ncarry,
    cd_overflow,
    cd_noverflow,
    cd_sign,
    cd_nsign,
    cd_above,
    cd_be,
    cd_greater,
    cd_le,
    cd_ge,
    cd_less,

} condition;
typedef struct {
    union {
        uint64_t* dest;
    };
    // If op_src is none, this stores the immediate value of the src,
    // if op_src is dereference_reg this is src (a pointer to a registe or some
    // other value) If op_src is dereference_mem this is the displacement
    union {
        uint64_t immediate;
        uint64_t displacement;
        uint64_t* src;
    };
    // Memory operand size
    uint8_t size;

    // Only used if op_src is dereference_mem
    uint8_t base_id;
    uint8_t index_id;
    uint8_t scale;

    // The operation between the destination and source
    // Is automatically set by a lookup table
    iop op;
    // The operation used to get the correct source value
    op_src op_src;
    // The the condition that has to be true to do the operation
    // Is automatically set by a lookup table
    condition cond;
    // The number of bytes the instruction takes up
    uint8_t instruction_size;
} instruction;
