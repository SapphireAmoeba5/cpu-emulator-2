#include "execute.h"
#include "cpu.h"
#include "decode.h"
#include <stdio.h>
#include <stdlib.h>

// #define LOOKUP_TABLE_IMPL

static void intpt(Cpu* cpu, int index) {
    if (index == 0x80) {
        printf("Cycle: %llu\n", cpu->clock_count);
        for (int i = 0; i < 16; i++) {
            uint64_t value = cpu->registers[i].r;
            printf("r%llu = %016llx (%lld)\n", (uint64_t)i, value,
                   (int64_t)value);
        }

        printf("ip: %llu\nsp: %llu\n", cpu->ip, cpu->sp);

        printf("ZR | CR | OF | SN\n");

        if (cpu->flags & FLAG_ZERO) {
            printf("1  | ");

        } else {
            printf("0  | ");
        }
        if (cpu->flags & FLAG_CARRY) {

            printf("1  | ");
        } else {

            printf("0  | ");
        }
        if (cpu->flags & FLAG_OVERFLOW) {

            printf("1  | ");
        } else {

            printf("0  | ");
        }
        if (cpu->flags & FLAG_SIGN) {

            printf("1\n");
        } else {

            printf("0\n");
        }

        cpu->exit = true;
    } else if (index == 0x81) {
        cpu->registers[0].r -= 1;
        cpu->flags &= ~FLAG_ZERO;
        if (cpu->registers[0].r == 0) {
            cpu->flags |= FLAG_ZERO;
        }
    } else if (index == 0x82) {
        printf("DEBUG PRINT %llu\n", cpu->clock_count);
    }
}

static inline uint64_t do_add(Cpu* cpu, uint64_t left, uint64_t right) {
    uint64_t sum = left + right;

    uint64_t tmp;
    bool carry = __builtin_add_overflow(left, right, &tmp);
    bool overflow =
        __builtin_add_overflow((int64_t)left, (int64_t)right, (int64_t*)&tmp);
    bool zero = sum == 0;
    bool sign = (int64_t)sum < 0;

    cpu->flags &= ~(FLAG_ZERO | FLAG_CARRY | FLAG_OVERFLOW | FLAG_SIGN);
    if (carry) {
        cpu->flags |= FLAG_CARRY;
    }
    if (overflow) {
        cpu->flags |= FLAG_OVERFLOW;
    }
    if (zero) {
        cpu->flags |= FLAG_ZERO;
    }
    if (sign) {
        cpu->flags |= FLAG_SIGN;
    }

    return sum;
}

static inline uint64_t do_sub(Cpu* cpu, uint64_t left, uint64_t right) {
    uint64_t difference = left - right;

    uint64_t tmp;
    bool carry = __builtin_sub_overflow(left, right, &tmp);
    bool overflow =
        __builtin_sub_overflow((int64_t)left, (int64_t)right, (int64_t*)&tmp);
    bool zero = difference == 0;
    bool sign = (int64_t)difference < 0;

    cpu->flags &= ~(FLAG_ZERO | FLAG_CARRY | FLAG_OVERFLOW | FLAG_SIGN);
    if (carry) {
        cpu->flags |= FLAG_CARRY;
    }
    if (overflow) {
        cpu->flags |= FLAG_OVERFLOW;
    }
    if (zero) {
        cpu->flags |= FLAG_ZERO;
    }
    if (sign) {
        cpu->flags |= FLAG_SIGN;
    }

    return difference;
}

static inline uint64_t do_mul(Cpu* cpu, uint64_t left, uint64_t right) {
    uint64_t sum = left * right;

    uint64_t tmp;
    bool carry = __builtin_mul_overflow(left, right, &tmp);
    bool overflow =
        __builtin_mul_overflow((int64_t)left, (int64_t)right, (int64_t*)&tmp);
    bool zero = sum == 0;
    bool sign = (int64_t)sum < 0;

    cpu->flags &= ~(FLAG_ZERO | FLAG_CARRY | FLAG_OVERFLOW | FLAG_SIGN);
    if (carry) {
        cpu->flags |= FLAG_CARRY;
    }
    if (overflow) {
        cpu->flags |= FLAG_OVERFLOW;
    }
    if (zero) {
        cpu->flags |= FLAG_ZERO;
    }
    if (sign) {
        cpu->flags |= FLAG_SIGN;
    }

    return sum;
}

static inline uint64_t do_div(Cpu* cpu, uint64_t left, uint64_t right) {
    uint64_t sum = left / right;

    cpu->flags &= ~(FLAG_ZERO | FLAG_CARRY | FLAG_OVERFLOW | FLAG_SIGN);

    return sum;
}

static inline uint64_t do_idiv(Cpu* cpu, int64_t left, int64_t right) {
    int64_t sum = left / right;

    cpu->flags &= ~(FLAG_ZERO | FLAG_CARRY | FLAG_OVERFLOW | FLAG_SIGN);

    return (uint64_t)sum;
}

static inline void do_bitwise_flags(Cpu* cpu, uint64_t result) {
    cpu->flags &= ~(FLAG_ZERO | FLAG_CARRY | FLAG_OVERFLOW | FLAG_SIGN);

    if (result == 0) {
        cpu->flags |= FLAG_ZERO;
    }
    if ((int64_t)result < 0) {
        cpu->flags |= FLAG_SIGN;
    }
}

uint64_t deref_reg(Cpu* cpu, uint8_t reg) {
    if (reg == SP_ID) {
        return cpu->sp;
    } else if (reg == IP_ID) {
        return cpu->ip;
    } else {
        return cpu->registers[reg].r;
    }
}

uint64_t calculate_addr(Cpu* cpu, instruction* instr) {
    uint64_t base = 0;
    uint64_t index = 0;
    uint64_t base_scale = instr->scale;
    uint64_t index_scale = 1;
    uint64_t disp = instr->displacement;

    if (instr->base_id != INVALID_ID) {
        base = deref_reg(cpu, instr->base_id);
    }
    if (instr->index_id != INVALID_ID) {
        index = deref_reg(cpu, instr->index_id);
        // If there is an index register then the scale is applied to it
        base_scale = 1;
        index_scale = instr->scale;
    }

    return (base * base_scale) + (index * index_scale) + disp;
}

error_t deref_memory(Cpu* cpu, instruction* instr, uint64_t* value) {
    uint64_t address = calculate_addr(cpu, instr);

    switch (instr->size) {
    case 0:
        if (!cpu_read_1(cpu, address, (uint8_t*)value)) {
            return MEMORY_ERROR;
        }
        return NO_ERROR;
    case 1:
        if (!cpu_read_2(cpu, address, (uint16_t*)value)) {
            return MEMORY_ERROR;
        }
        return NO_ERROR;
    case 2:
        if (!cpu_read_4(cpu, address, (uint32_t*)value)) {
            return MEMORY_ERROR;
        }
        return NO_ERROR;
    case 3:
        if (!cpu_read_8(cpu, address, value)) {
            return MEMORY_ERROR;
        }
        return NO_ERROR;
    default:
        UNREACHABLE();
    }
}

static bool is_condition(Cpu* cpu, condition cond) {
    switch (cond) {
    case cd_true:
        return true;
    case cd_zero:
        return cpu->flags & FLAG_ZERO;
    case cd_nzero:
        return (cpu->flags & FLAG_ZERO) == 0;
    case cd_carry:
        return cpu->flags & FLAG_CARRY;
    case cd_ncarry:
        return (cpu->flags & FLAG_CARRY) == 0;
    case cd_overflow:
        return cpu->flags & FLAG_OVERFLOW;
    case cd_noverflow:
        return (cpu->flags & FLAG_OVERFLOW) == 0;
    case cd_sign:
        return (cpu->flags & FLAG_SIGN);
    case cd_nsign:
        return (cpu->flags & FLAG_SIGN) == 0;
    case cd_above:
        return (cpu->flags & FLAG_CARRY) == 0 && (cpu->flags & FLAG_ZERO) == 0;
    case cd_be:
        return cpu->flags & FLAG_CARRY || cpu->flags & FLAG_ZERO;
    case cd_greater:
        return (cpu->flags & FLAG_ZERO) == 0 &&
               ((cpu->flags & FLAG_SIGN) == 0) ==
                   ((cpu->flags & FLAG_OVERFLOW) == 0);
    case cd_le:
        return cpu->flags & FLAG_ZERO ||
               ((cpu->flags & FLAG_SIGN) == 0) !=
                   ((cpu->flags & FLAG_OVERFLOW) == 0);
    case cd_ge:
        return ((cpu->flags & FLAG_SIGN) == 0 == (cpu->flags & FLAG_OVERFLOW) ==
                0);
    case cd_less:
        return (cpu->flags & FLAG_SIGN) == 0 != (cpu->flags & FLAG_OVERFLOW) ==
               0;
    }
}

static error_t handle_invl(Cpu* cpu, instruction* instr, uint64_t src) {
    printf("Invalid operation called. This is a bug\n");
    abort();
}

static error_t handle_halt(Cpu* cpu, instruction* instr, uint64_t src) {
    printf("CPU HALTED\n");
    cpu->halt = true;
    return NO_ERROR;
}

static error_t handle_int(Cpu* cpu, instruction* instr, uint64_t src) {
    intpt(cpu, src);
    return NO_ERROR;
}

static error_t handle_mov(Cpu* cpu, instruction* instr, uint64_t src) {
    *instr->dest = src;
    return NO_ERROR;
}
static error_t handle_add(Cpu* cpu, instruction* instr, uint64_t src) {
    *instr->dest = do_add(cpu, *instr->dest, src);
    return NO_ERROR;
}
static error_t handle_sub(Cpu* cpu, instruction* instr, uint64_t src) {
    *instr->dest = do_sub(cpu, *instr->dest, src);
    return NO_ERROR;
}
static error_t handle_mul(Cpu* cpu, instruction* instr, uint64_t src) {
    *instr->dest = do_mul(cpu, *instr->dest, src);
    return NO_ERROR;
}
static error_t handle_div(Cpu* cpu, instruction* instr, uint64_t src) {
    if (src == 0) {
        return MATH_ERROR;
    }

    *instr->dest = do_div(cpu, *instr->dest, src);
    return NO_ERROR;
}
static error_t handle_idiv(Cpu* cpu, instruction* instr, uint64_t src) {
    if (src == 0) {
        return MATH_ERROR;
    }

    *instr->dest = do_idiv(cpu, *instr->dest, src);
    return NO_ERROR;
}
static error_t handle_and(Cpu* cpu, instruction* instr, uint64_t src) {
    *instr->dest &= src;
    do_bitwise_flags(cpu, *instr->dest);
    return NO_ERROR;
}
static error_t handle_or(Cpu* cpu, instruction* instr, uint64_t src) {
    *instr->dest |= src;
    do_bitwise_flags(cpu, *instr->dest);
    return NO_ERROR;
}
static error_t handle_xor(Cpu* cpu, instruction* instr, uint64_t src) {
    *instr->dest ^= src;
    do_bitwise_flags(cpu, *instr->dest);
    return NO_ERROR;
}
static error_t handle_cmp(Cpu* cpu, instruction* instr, uint64_t src) {
    do_sub(cpu, *instr->dest, src);
    return NO_ERROR;
}
static error_t handle_test(Cpu* cpu, instruction* instr, uint64_t src) {
    uint64_t result = *instr->dest & src;
    do_bitwise_flags(cpu, result);
    return NO_ERROR;
}

error_t (*op_handlers[op_LENGTH])(Cpu* cpu, instruction* instr,
                                  uint64_t src) = {
    [op_invl] = handle_invl, [op_halt] = handle_halt, [op_int] = handle_int,
    [op_mov] = handle_mov,   [op_add] = handle_add,   [op_sub] = handle_sub,
    [op_mul] = handle_mul,   [op_div] = handle_div,   [op_idiv] = handle_idiv,
    [op_and] = handle_and,   [op_or] = handle_or,     [op_xor] = handle_xor,
    [op_cmp] = handle_cmp,   [op_test] = handle_test,
};

error_t cpu_execute(Cpu* cpu, instruction* instr) {
    uint64_t src = 0;

    switch (instr->op_src) {
    case op_src_immediate:
        src = instr->immediate;
        break;
    case op_src_dereference_reg:
        src = deref_reg(cpu, instr->src_reg_id);
        break;
    case op_src_calculate_address:
        src = calculate_addr(cpu, instr);
        break;
    case op_src_dereference_mem:
        if (deref_memory(cpu, instr, &src) != NO_ERROR) {
            return MEMORY_ERROR;
        }
        break;
    }

    if (!is_condition(cpu, instr->cond)) {
        return NO_ERROR;
    }

#ifdef LOOKUP_TABLE_IMPL
    return op_handlers[instr->op](cpu, instr, src);
#else
    switch (instr->op) {
    case op_halt:
        return handle_halt(cpu, instr, src);
    case op_int:
            printf("INTER\n");
        return handle_int(cpu, instr, src);
    case op_mov:
        return handle_mov(cpu, instr, src);
    case op_add:
        return handle_add(cpu, instr, src);
    case op_sub:
        return handle_sub(cpu, instr, src);
    case op_mul:
        return handle_mul(cpu, instr, src);
    case op_div:
        return handle_div(cpu, instr, src);
    case op_idiv:
        return handle_idiv(cpu, instr, src);
    case op_and:
        return handle_and(cpu, instr, src);
    case op_or:
        return handle_or(cpu, instr, src);
    case op_xor:
        return handle_xor(cpu, instr, src);
    case op_cmp:
        return handle_cmp(cpu, instr, src);
    case op_test:
        return handle_test(cpu, instr, src);
    case op_LENGTH:
    case op_invl:
        UNREACHABLE();
    }
#endif
}
