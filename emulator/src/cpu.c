#include "cpu.h"
#include "address_bus.h"
#include "data_cache.h"
#include "decode.h"
#include "execute.h"
#include "instruction_cache.h"
#include "memory.h"
#include <stdbool.h>
#include <stdckdint.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

bool cpu_write_8(Cpu* cpu, uint64_t data, uint64_t address) {
    printf("Address: %llu\n", address);
    if (address % 8 != 0) {
        return false;
    }

    uint64_t aligned = align_to_block_boundary(address);
    int cache_line = get_cache_line(aligned);

    // Keep the dirty flag set since we are about to write here anyways
    if (cpu->data_cache.addresses[cache_line] != aligned) {
        if (cpu->data_cache.dirty[cache_line]) {
            if (!addr_bus_write_block(cpu->bus,
                                      cpu->data_cache.addresses[cache_line],
                                      &cpu->data_cache.lines[cache_line])) {
                cpu->data_cache.dirty[cache_line] = false;
                cpu->data_cache.addresses[cache_line] = UNOCCUPIED_LINE;
                return false;
            }
        }

        if (!addr_bus_read_block(cpu->bus, aligned,
                                 &cpu->data_cache.lines[cache_line])) {
            cpu->data_cache.dirty[cache_line] = false;
            cpu->data_cache.addresses[cache_line] = UNOCCUPIED_LINE;
            return false;
        }
        cpu->data_cache.addresses[cache_line] = aligned;
    }

    uint64_t offset = address - aligned;
    // No bounds checks, garunteed to fit
    memcpy(&cpu->data_cache.lines[cache_line][offset], &data, 8);
    return true;
}
bool cpu_write_4(Cpu* cpu, uint32_t data, uint64_t address) {
    if (address % 4 != 0) {
        return false;
    }

    uint64_t aligned = align_to_block_boundary(address);
    int cache_line = get_cache_line(aligned);

    // Keep the dirty flag set since we are about to write here anyways
    if (cpu->data_cache.addresses[cache_line] != aligned) {
        if (cpu->data_cache.dirty[cache_line]) {
            if (!addr_bus_write_block(cpu->bus,
                                      cpu->data_cache.addresses[cache_line],
                                      &cpu->data_cache.lines[cache_line])) {
                cpu->data_cache.dirty[cache_line] = false;
                cpu->data_cache.addresses[cache_line] = UNOCCUPIED_LINE;
                return false;
            }
        }

        if (!addr_bus_read_block(cpu->bus, aligned,
                                 &cpu->data_cache.lines[cache_line])) {
            cpu->data_cache.dirty[cache_line] = false;
            cpu->data_cache.addresses[cache_line] = UNOCCUPIED_LINE;
            return false;
        }
        cpu->data_cache.addresses[cache_line] = aligned;
    }

    uint64_t offset = address - aligned;
    // No bounds checks, garunteed to fit
    memcpy(&cpu->data_cache.lines[cache_line][offset], &data, 4);
    return true;
}

bool cpu_write_2(Cpu* cpu, uint16_t data, uint64_t address) {
    if (address % 2 != 0) {
        return false;
    }

    uint64_t aligned = align_to_block_boundary(address);
    int cache_line = get_cache_line(aligned);

    // Keep the dirty flag set since we are about to write here anyways
    if (cpu->data_cache.addresses[cache_line] != aligned) {
        if (cpu->data_cache.dirty[cache_line]) {
            if (!addr_bus_write_block(cpu->bus,
                                      cpu->data_cache.addresses[cache_line],
                                      &cpu->data_cache.lines[cache_line])) {
                cpu->data_cache.dirty[cache_line] = false;
                cpu->data_cache.addresses[cache_line] = UNOCCUPIED_LINE;
                return false;
            }
        }

        if (!addr_bus_read_block(cpu->bus, aligned,
                                 &cpu->data_cache.lines[cache_line])) {
            cpu->data_cache.dirty[cache_line] = false;
            cpu->data_cache.addresses[cache_line] = UNOCCUPIED_LINE;
            return false;
        }
        cpu->data_cache.addresses[cache_line] = aligned;
    }

    uint64_t offset = address - aligned;
    // No bounds checks, garunteed to fit
    memcpy(&cpu->data_cache.lines[cache_line][offset], &data, 2);
    return true;
}

bool cpu_write_1(Cpu* cpu, uint8_t data, uint64_t address) {
    uint64_t aligned = align_to_block_boundary(address);
    int cache_line = get_cache_line(aligned);

    // Keep the dirty flag set since we are about to write here anyways
    if (cpu->data_cache.addresses[cache_line] != aligned) {
        if (cpu->data_cache.dirty[cache_line]) {
            if (!addr_bus_write_block(cpu->bus,
                                      cpu->data_cache.addresses[cache_line],
                                      &cpu->data_cache.lines[cache_line])) {
                cpu->data_cache.dirty[cache_line] = false;
                cpu->data_cache.addresses[cache_line] = UNOCCUPIED_LINE;
                return false;
            }
        }

        if (!addr_bus_read_block(cpu->bus, aligned,
                                 &cpu->data_cache.lines[cache_line])) {
            cpu->data_cache.dirty[cache_line] = false;
            cpu->data_cache.addresses[cache_line] = UNOCCUPIED_LINE;
            return false;
        }
        cpu->data_cache.addresses[cache_line] = aligned;
    }

    uint64_t offset = address - aligned;
    // No bounds checks, garunteed to fit
    memcpy(&cpu->data_cache.lines[cache_line][offset], &data, 1);
    return true;
}

bool cpu_read_8(Cpu* cpu, uint64_t address, uint64_t* value) {
    if (address % 8 != 0) {
        return false;
    }

    uint64_t aligned = align_to_block_boundary(address);
    int cache_line = get_cache_line(aligned);

    if (cpu->data_cache.addresses[cache_line] != aligned) {
        bool dirty = cpu->data_cache.dirty[cache_line];
        cpu->data_cache.dirty[cache_line] = false;
        if (dirty) {
            if (!addr_bus_write_block(cpu->bus,
                                      cpu->data_cache.addresses[cache_line],
                                      &cpu->data_cache.lines[cache_line])) {
                cpu->data_cache.addresses[cache_line] = UNOCCUPIED_LINE;
                return false;
            }
        }

        if (!addr_bus_read_block(cpu->bus, aligned,
                                 &cpu->data_cache.lines[cache_line])) {
            cpu->data_cache.addresses[cache_line] = UNOCCUPIED_LINE;
            return false;
        }
        cpu->data_cache.addresses[cache_line] = aligned;
    }

    uint64_t offset = address - aligned;
    // No bounds checks, garunteed to fit
    memcpy(value, &cpu->data_cache.lines[cache_line][offset], 8);
    return true;
}

bool cpu_read_4(Cpu* cpu, uint64_t address, uint32_t* value) {
    if (address % 4 != 0) {
        return false;
    }

    uint64_t aligned = align_to_block_boundary(address);
    int cache_line = get_cache_line(aligned);

    if (cpu->data_cache.addresses[cache_line] != aligned) {
        bool dirty = cpu->data_cache.dirty[cache_line];
        cpu->data_cache.dirty[cache_line] = false;
        if (dirty) {
            if (!addr_bus_write_block(cpu->bus,
                                      cpu->data_cache.addresses[cache_line],
                                      &cpu->data_cache.lines[cache_line])) {
                cpu->data_cache.addresses[cache_line] = UNOCCUPIED_LINE;
                return false;
            }
        }

        if (!addr_bus_read_block(cpu->bus, aligned,
                                 &cpu->data_cache.lines[cache_line])) {
            cpu->data_cache.addresses[cache_line] = UNOCCUPIED_LINE;
            return false;
        }
        cpu->data_cache.addresses[cache_line] = aligned;
    }

    uint64_t offset = address - aligned;
    // No bounds checks, garunteed to fit
    memcpy(value, &cpu->data_cache.lines[cache_line][offset], 4);
    return true;
}

bool cpu_read_2(Cpu* cpu, uint64_t address, uint16_t* value) {
    if (address % 2 != 0) {
        return false;
    }

    uint64_t aligned = align_to_block_boundary(address);
    int cache_line = get_cache_line(aligned);

    if (cpu->data_cache.addresses[cache_line] != aligned) {
        bool dirty = cpu->data_cache.dirty[cache_line];
        cpu->data_cache.dirty[cache_line] = false;
        if (dirty) {
            if (!addr_bus_write_block(cpu->bus,
                                      cpu->data_cache.addresses[cache_line],
                                      &cpu->data_cache.lines[cache_line])) {
                cpu->data_cache.addresses[cache_line] = UNOCCUPIED_LINE;
                return false;
            }
        }

        if (!addr_bus_read_block(cpu->bus, aligned,
                                 &cpu->data_cache.lines[cache_line])) {
            cpu->data_cache.addresses[cache_line] = UNOCCUPIED_LINE;
            return false;
        }
        cpu->data_cache.addresses[cache_line] = aligned;
    }

    uint64_t offset = address - aligned;
    // No bounds checks, garunteed to fit
    memcpy(value, &cpu->data_cache.lines[cache_line][offset], 2);
    return true;
}

bool cpu_read_1(Cpu* cpu, uint64_t address, uint8_t* value) {
    uint64_t aligned = align_to_block_boundary(address);
    int cache_line = get_cache_line(aligned);

    if (cpu->data_cache.addresses[cache_line] != aligned) {
        bool dirty = cpu->data_cache.dirty[cache_line];
        cpu->data_cache.dirty[cache_line] = false;
        if (dirty) {
            if (!addr_bus_write_block(cpu->bus,
                                      cpu->data_cache.addresses[cache_line],
                                      &cpu->data_cache.lines[cache_line])) {
                cpu->data_cache.addresses[cache_line] = UNOCCUPIED_LINE;
                return false;
            }
        }

        if (!addr_bus_read_block(cpu->bus, aligned,
                                 &cpu->data_cache.lines[cache_line])) {
            cpu->data_cache.addresses[cache_line] = UNOCCUPIED_LINE;
            return false;
        }
        cpu->data_cache.addresses[cache_line] = aligned;
    }

    uint64_t offset = address - aligned;
    // No bounds checks, garunteed to fit
    memcpy(value, &cpu->data_cache.lines[cache_line][offset], 1);
    return true;
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

    memset(&cpu->data_cache.addresses, UNOCCUPIED_LINE,
           sizeof(cpu->data_cache.addresses));
    memset(&cpu->instruction_cache.addresses, UNOCCUPIED_LINE,
           sizeof(cpu->instruction_cache.addresses));

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

        uint64_t block_start = cpu->registers[IP_INDEX].r;
        // Don't do a cache lookup while we are still executing the same block
        // of code
        while (cpu->registers[IP_INDEX].r == block_start && !cpu->halt &&
               !cpu->exit) {
            uint64_t i = 0;
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
}
