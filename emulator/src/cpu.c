#include "cpu.h"
#include "address_bus.h"
#include "decode.h"
#include "execute.h"
#include "instruction_cache.h"
#include "memory.h"
#include <__stddef_unreachable.h>
#include <stdbool.h>
#include <stdckdint.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

bool cpu_write_8(Cpu* cpu, uint64_t data, uint64_t address) {
    return addr_bus_write_8(cpu->bus, address, data);
}
bool cpu_write_4(Cpu* cpu, uint32_t data, uint64_t address) {
    return addr_bus_write_4(cpu->bus, address, data);
}

bool cpu_write_2(Cpu* cpu, uint16_t data, uint64_t address) {
    return addr_bus_write_2(cpu->bus, address, data);
}

bool cpu_write_1(Cpu* cpu, uint8_t data, uint64_t address) {
    return addr_bus_write_1(cpu->bus, address, data);
}

bool cpu_read_8(Cpu* cpu, uint64_t address, uint64_t* value) {
    return addr_bus_read_8(cpu->bus, address, value);
}

bool cpu_read_4(Cpu* cpu, uint64_t address, uint32_t* value) {
    return addr_bus_read_4(cpu->bus, address, value);
}

bool cpu_read_2(Cpu* cpu, uint64_t address, uint16_t* value) {
    return addr_bus_read_2(cpu->bus, address, value);
}

bool cpu_read_1(Cpu* cpu, uint64_t address, uint8_t* value) {
    return addr_bus_read_1(cpu->bus, address, value);
}

bool cpu_read_n(Cpu* cpu, uint64_t address, void* out, uint64_t n) {
    return addr_bus_read_n(cpu->bus, address, out, n);
}
bool cpu_read_block(Cpu* cpu, uint64_t address, void* out) {
    return addr_bus_read_block(cpu->bus, address, out);
}

bool cpu_push(Cpu* cpu, uint64_t value) {
    cpu->registers[SP_INDEX].r -= 8;
    return cpu_write_8(cpu, value, cpu->registers[SP_INDEX].r);
}

bool cpu_pop(Cpu* cpu, uint64_t* out) {
    if (!cpu_read_8(cpu, cpu->registers[SP_INDEX].r, out)) {
        return false;
    }
    cpu->registers[SP_INDEX].r += 8;
    return true;
}

void cpu_create(Cpu* cpu, address_bus* bus) {
    memset(cpu, 0, sizeof(Cpu));

    cpu->cache = instr_cache_create();
    cpu->bus = bus;
    timer_start(&cpu->timer);
}

// Does not free the address bus, that is owned by the caller to cpu_create
void cpu_destroy(Cpu* cpu) {
    // TODO: Free the instruction cache
    return;
}

void cpu_run(Cpu* cpu) {
    while (!cpu->exit) {
        block* buf = instr_cache_get(&cpu->cache, cpu->registers[IP_INDEX].r);

        if (buf->len == 0) {
            // TODO: Check if it is faster to always cache up to the maximum
            // cache size, ignoring branches and manually check in the execution
            // loop if the IP has been modified.
            // This could potentially lower the overhead of caches since we won't need
            bool branches = false;

            uint64_t block_start = cpu->registers[IP_INDEX].r;
            while (!branches && buf->len < MAX_CACHE_BLOCK) {
                uint64_t start = cpu->registers[IP_INDEX].r;
                instruction instr;
                error_t err = cpu_decode(cpu, &instr, &branches);
                if (err != NO_ERROR) {
                    if (buf->len == 0) {
                        printf("Cache error: %d\n", err);
                        cpu->exit = true;
                    }
                    break;
                }
                uint64_t size = cpu->registers[IP_INDEX].r - start;
                instr.instruction_size = size;
                instruction_buf_append(buf, &instr);
            }
            cpu->registers[IP_INDEX].r = block_start;
        }

        uint32_t i = 0;
        while (i < buf->len && !cpu->halt && !cpu->exit) {
            cpu->clock_count++;
            instruction* instr = &buf->instructions[i];
            cpu->registers[IP_INDEX].r += instr->instruction_size;
            error_t err = cpu_execute(cpu, instr);
            if (err != NO_ERROR) {
                printf("ERROR EXECUTING %d\n", err);
                abort();
            }
            i++;
        }
    }
}
