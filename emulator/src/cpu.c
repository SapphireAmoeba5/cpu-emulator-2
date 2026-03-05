#include "cpu.h"
#include "address_bus.h"
#include "bus_device.h"
#include "decode.h"
#include "execute.h"
#include "instruction_cache.h"
#include "interrupt_flags.h"
#include "memory.h"
#include <setjmp.h>
#include <stdatomic.h>
#include <stdbool.h>
#include <stdckdint.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

void cpu_write_8(Cpu* cpu, uint64_t data, uint64_t address) {
    if (!address_bus_write_n(cpu->bus, address, &data, 8)) {
        cpu_except(cpu, BUS_ERROR);
    }
}
void cpu_write_4(Cpu* cpu, uint32_t data, uint64_t address) {
    if (!address_bus_write_n(cpu->bus, address, &data, 4)) {
        cpu_except(cpu, BUS_ERROR);
    }
}

void cpu_write_2(Cpu* cpu, uint16_t data, uint64_t address) {
    if (!address_bus_write_n(cpu->bus, address, &data, 2)) {
        cpu_except(cpu, BUS_ERROR);
    }
}

void cpu_write_1(Cpu* cpu, uint8_t data, uint64_t address) {
    if (!address_bus_write_n(cpu->bus, address, &data, 1)) {
        cpu_except(cpu, BUS_ERROR);
    }
}

void cpu_read_8(Cpu* cpu, uint64_t address, uint64_t* value) {
    if (!address_bus_read_n(cpu->bus, address, value, 8)) {
        cpu_except(cpu, BUS_ERROR);
    }
}

void cpu_read_4(Cpu* cpu, uint64_t address, uint32_t* value) {
    if (!address_bus_read_n(cpu->bus, address, value, 4)) {
        cpu_except(cpu, BUS_ERROR);
    }
}

void cpu_read_2(Cpu* cpu, uint64_t address, uint16_t* value) {
    if (!address_bus_read_n(cpu->bus, address, value, 2)) {
        cpu_except(cpu, BUS_ERROR);
    }
}

void cpu_read_1(Cpu* cpu, uint64_t address, uint8_t* value) {
    if (!address_bus_read_n(cpu->bus, address, value, 1)) {
        cpu_except(cpu, BUS_ERROR);
    }
}

void cpu_push(Cpu* cpu, uint64_t value) {
    cpu->registers[SP_INDEX].r -= 8;
    cpu_write_8(cpu, value, cpu->registers[SP_INDEX].r);
}

void cpu_pop(Cpu* cpu, uint64_t* out) {
    cpu_read_8(cpu, cpu->registers[SP_INDEX].r, out);
    cpu->registers[SP_INDEX].r += 8;
}

void cpu_create(Cpu* cpu, address_bus* bus) {
    memset(cpu, 0, sizeof(Cpu));

    memset(&cpu->iflag_mask, 0xff, sizeof(Cpu));
    cpu->interrupt_enable = 0;
    cpu->cache = instr_cache_create();
    cpu->bus = bus;
    timer_start(&cpu->timer);
}

/// Flushes the caches and resets the instruction pointer back to zero
inline static void cpu_reset(Cpu* cpu) {
    instr_cache_clear(&cpu->cache);
    cpu->interrupt_enable = 0;
    cpu->registers[IP_INDEX].r = 0;
    memset(&cpu->iflag_mask, 0xff, sizeof(cpu->iflag_mask));
}

inline static bool cpu_pending_interrupt(Cpu* cpu) {
    return atomic_load_explicit(&cpu->pending_interrupt, memory_order_relaxed);
}

bool push_interrupt_state(Cpu* cpu) {
    struct {
        uint64_t ip;
        flags_t flags;
    } state;

    // Ensure no padding bytes
    static_assert(sizeof(state) == sizeof(uint64_t) + sizeof(flags_t));

    state.ip = cpu->registers[IP_INDEX].r;
    state.flags = cpu->flags;

    cpu->registers[SP_INDEX].r -= sizeof(state);
    if (!address_bus_write_n(cpu->bus, cpu->registers[SP_INDEX].r, &state,
                             sizeof(state))) {
        // Reset the cpu if the write fails
        cpu_reset(cpu);
        return false;
    }
    return true;
}

void pop_interrupt_state(Cpu* cpu) {
    struct {
        uint64_t ip;
        flags_t flags;
    } state;

    // Ensure no padding bytes
    static_assert(sizeof(state) == sizeof(uint64_t) + sizeof(flags_t));

    if (!address_bus_read_n(cpu->bus, cpu->registers[SP_INDEX].r, &state,
                            sizeof(state))) {
        // Reset the cpu if the read fails
        cpu_reset(cpu);
        return;
    }
    cpu->registers[SP_INDEX].r += sizeof(state);
    cpu->registers[IP_INDEX].r = state.ip;
    cpu->flags = state.flags;
}

/// Set's the instruction pointer to the interrupt handler associated with the
/// current interrupt vector, and pushes the cpu state to the stack
void cpu_call_interrupt(Cpu* cpu, u8 vector) {
    u64 handler;
    if (!address_bus_read_n(cpu->bus, cpu->idtr + (vector * 8), &handler, 8)) {
        cpu_reset(cpu);
        return;
    }
    if (push_interrupt_state(cpu)) {
        cpu->registers[IP_INDEX].r = handler;
    }
}

void cpu_except(Cpu* cpu, error_t error) {
    cpu->software_interrupt_index = (u8)error;
    longjmp(cpu->interrupt_jmp, 1);
}

// Does not free the address bus, that is owned by the caller to cpu_create
void cpu_destroy(Cpu* cpu) {
    // TODO: Free the instruction cache
    return;
}

void cpu_run(Cpu* cpu) {
    // Any inerrupts will jump here
    int code = setjmp(cpu->interrupt_jmp);
    if (code != 0) {
        printf("Calling interrupt %d\n", cpu->software_interrupt_index);
        cpu_call_interrupt(cpu, cpu->software_interrupt_index);
    }

    while (!cpu->exit) {
        block* buf = instr_cache_get(&cpu->cache, cpu->registers[IP_INDEX].r);

        if (buf->len == 0) {
            bool branches = false;

            uint64_t block_start = cpu->registers[IP_INDEX].r;
            while (!branches && buf->len < MAX_CACHE_BLOCK) {
                uint64_t start = cpu->registers[IP_INDEX].r;
                instruction instr;
                error_t err = cpu_decode(cpu, &instr, &branches);
                if (err != NO_ERROR) {
                    if (buf->len == 0) {
                        cpu_except(cpu, DECODE_ERROR);
                    }
                    break;
                }
                uint64_t size = cpu->registers[IP_INDEX].r - start;
                instr.instruction_size = size;
                instruction_buf_append(buf, &instr);
            }

            cpu->registers[IP_INDEX].r = block_start;
        }

        uint64_t block_start = cpu->registers[IP_INDEX].r;
        // Don't do a cache lookup while we are still executing the same block
        // of code
        while (cpu->registers[IP_INDEX].r == block_start) {
            uint64_t i = 0;
            while (i < buf->len) {
                instruction* instr = &buf->instructions[i];
                cpu->registers[IP_INDEX].r += instr->instruction_size;
                cpu_execute(cpu, instr);
                i++;
            }
            cpu->clock_count += i;

            if (cpu->interrupt_enable && cpu_pending_interrupt(cpu)) {
                spinlock_lock(&cpu->iflag_lock);

                if (iflag_non_zero(&cpu->iflags)) {
                    u8 interrupt_index = iflag_trailing_zeros(&cpu->iflags);
                    iflag_unset_bit(&cpu->iflags, interrupt_index);
                } else {
                    atomic_store_explicit(&cpu->pending_interrupt, false,
                                          memory_order_relaxed);
                }

                spinlock_unlock(&cpu->iflag_lock);
            }
        }
    }
}
