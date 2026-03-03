#pragma once

#include "address_bus.h"
#include "instruction_cache.h"
#include "interrupt_flags.h"
#include "sync/spinlock.h"
#include "timer.h"
#include <assert.h>
#include <setjmp.h>
#include <stdatomic.h>
#include <stdbool.h>
#include <stdint.h>

#include "util/common.h"
#include "util/types.h"

#ifdef __APPLE__
// On MacOS, the normal setjmp/longjmp have the overhead of saving and
// reloading the signal mask every time they are called. We don't need to
// preserve signal masks so these are the best choice for speed
#define setjmp(a) _setjmp(a)
#define longjmp(a, b) _longjmp(a, b)
#endif

// Maximum amount of instructions to cache if there were no branch points
// There should be a sweet spot that prevents slowdown from caching more
// instructions than get executed in a block, and from caching too few that we
// need to query the cache more often
constexpr uint64_t MAX_CACHE_BLOCK = 32;

constexpr uint64_t CLOCK_HZ = 500000000;

typedef enum {
    NO_ERROR = -1,
    BUS_ERROR,
    DECODE_ERROR,
    MATH_ERROR,
} error_t;

typedef uint16_t flags_t;

constexpr flags_t FLAG_ZERO = BIT(0);
constexpr flags_t FLAG_CARRY = BIT(1);
constexpr flags_t FLAG_OVERFLOW = BIT(2);
constexpr flags_t FLAG_SIGN = BIT(3);

constexpr uint64_t SP_INDEX = 16;
constexpr uint64_t IP_INDEX = 17;

typedef union reg {
    uint8_t b;
    uint16_t s;
    uint32_t w;
    uint64_t r;
} reg;

typedef struct Cpu {
    address_bus* bus;
    // Total number of clocks
    uint64_t clock_count;
    // 16 general purpose registers plus the stack pointer and instruction
    // pointer
    reg registers[16 + 2];
    // Interrupt descriptor register
    uint64_t idtr;

    // Jumps to the exception handler code when there is any exception
    jmp_buf interrupt_jmp;
    // If the CPU is halted
    bool halt;
    // If the CPU should delete itself
    bool exit;
    // CPU flags
    flags_t flags;

    instruction_cache cache;
    timer timer;

    // The index used for software interrupts triggered by the `INT` instruction
    // or by exceptions such as divide by error or bus errors
    u8 software_interrupt_index;
    // The fields in this anonymous struct are protected by the spinlock.
    // The CPU thread may read from `pending_interrupt` without a lock but to
    // write to it, it must lock the spinlock to ensure coherency between it and `iflags`
    struct {
        // This lock must be obtained before modifying the iflags and
        // `pending_interrupt`
        spinlock iflag_lock;
        // Set to true when another thread requests an interrupt
        atomic_bool pending_interrupt;
        iflag iflags;
    };

    // This is a private variable that only the CPU thread will touch.
    // Each bit in this mask corresponds to a pending interrupt bit in `iflags`,
    // if the result of a bitwise and with `iflag_mask` and `iflags` causes a
    // bit to be 0, that interrupt will not be fired, if it a 1 that interrupt
    // will be fired
    // Initialized to all ones
    iflag iflag_mask;

    // If non-zero, interrupts are enabled, if zero, then maskable-interrupts
    // are disabled It is a uint64_t so that it can be set and unset with the
    // existing code for `op_mov`
    // Initialized to zero (interrupts disabled)
    uint64_t interrupt_enable;
} Cpu;

/// Longjumps to `interrupt_jmp` on exception
/// May only be called by CPU thread
void cpu_write_8(Cpu* cpu, uint64_t data, uint64_t address);
/// Longjumps to `interrupt_jmp` on exception
/// May only be called by CPU thread
void cpu_write_4(Cpu* cpu, uint32_t data, uint64_t address);
/// Longjumps to `interrupt_jmp` on exception
/// May only be called by CPU thread
void cpu_write_2(Cpu* cpu, uint16_t data, uint64_t address);
/// Longjumps to `interrupt_jmp` on exception
/// May only be called by CPU thread
void cpu_write_1(Cpu* cpu, uint8_t data, uint64_t address);

/// `value` must not be NULL
/// Longjumps to `interrupt_jmp` on exception
/// May only be called by CPU thread
void cpu_read_8(Cpu* cpu, uint64_t address, uint64_t* value);
/// `value` must not be NULL
/// Longjumps to `interrupt_jmp` on exception
/// May only be called by CPU thread
void cpu_read_4(Cpu* cpu, uint64_t address, uint32_t* value);
/// `value` must not be NULL
/// Longjumps to `interrupt_jmp` on exception
/// May only be called by CPU thread
void cpu_read_2(Cpu* cpu, uint64_t address, uint16_t* value);
/// `value` must not be NULL
/// Longjumps to `interrupt_jmp` on exception
/// May only be called by CPU thread
void cpu_read_1(Cpu* cpu, uint64_t address, uint8_t* value);

/// Must be called by the CPU thread
void cpu_call_interrupt(Cpu* cpu, u8 vector);

/// May only be called by CPU thread
void cpu_push(Cpu* cpu, uint64_t value);
/// May only be called by CPU thread
void cpu_pop(Cpu* cpu, uint64_t* out);

/// Long jumps to the Cpu's exception handler
void cpu_except(Cpu* cpu, error_t error);

void cpu_create(Cpu* cpu, address_bus* bus);
void cpu_destroy(Cpu* cpu);
void cpu_run(Cpu* cpu);
