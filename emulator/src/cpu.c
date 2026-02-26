#include "cpu.h"
#include "address_bus.h"
#include "bus_device.h"
#include "decode.h"
#include "execute.h"
#include "instruction_cache.h"
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

static void intpt(Cpu* cpu, int index) {
    if (index == 0x80) {
        printf("Cycle: %llu\n", cpu->clock_count);
        for (int i = 0; i < 16; i++) {
            uint64_t value = cpu->registers[i].r;
            printf("r%llu = %016llx (%lld)\n", (uint64_t)i, value,
                   (int64_t)value);
        }

        printf("ip: %llu\nsp: %llu\n", cpu->registers[IP_INDEX].r,
               cpu->registers[SP_INDEX].r);

        printf("ZF | CF | OF | SF\n");

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

        double elapsed = timer_elapsed_seconds(&cpu->timer);
        double instructions_per_second = cpu->clock_count / elapsed;
        double mips = instructions_per_second / 1e6;
        printf("MIPS: %f\n", mips);

        cpu->exit = true;
    } else if (index == 0x82) {
        printf("DEBUG PRINT %llu\n", cpu->clock_count);
    }
}

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

    cpu->cache = instr_cache_create();
    cpu->bus = bus;
    timer_start(&cpu->timer);
}

void cpu_except(Cpu* cpu, error_t error) {
    longjmp(cpu->interrupt_jmp, (int)error + 1);
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
        int interrupt_id = code - 1;
        printf("Interrupt: %d\n", interrupt_id);
        intpt(cpu, interrupt_id);

        cpu->exit = true;
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
        }
    }
}
