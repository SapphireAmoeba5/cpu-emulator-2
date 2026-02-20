#include "cpu.h"
#include "address_bus.h"
#include "bus_device.h"
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
#ifdef USE_CACHE
    return cache_write_8(&cpu->data_cache, cpu->bus, address, data);
#else
    uint64_t block_boundary = align_to_block_boundary(address);
    uint64_t offset = address - block_boundary;

    bus_device* device;
    block_range range;
    uint8_t* buf = addr_bus_lock_block(cpu->bus, address, &device, &range);

    if (buf == nullptr) {
        return false;
    }

    if (address + 8 > block_boundary + BLOCK_SIZE) {
        uint64_t next_boundary = align_to_block_boundary(address + 8);
        int remaining = next_boundary - address;
        memcpy(&buf[offset], &data, remaining);
        addr_bus_unlock_block(cpu->bus, address, device, range);

        buf = addr_bus_lock_block(cpu->bus, next_boundary, &device, &range);

        if (buf == nullptr) {
            return false;
        }

        memcpy(&buf[0], (char*)&data + remaining, 8 - remaining);
        addr_bus_unlock_block(cpu->bus, next_boundary, device, range);
    } else {
        memcpy(&buf[offset], &data, 8);
        addr_bus_unlock_block(cpu->bus, address, device, range);
    }
    return true;
#endif
}
bool cpu_write_4(Cpu* cpu, uint32_t data, uint64_t address) {
#ifdef USE_CACHE
    return cache_write_4(&cpu->data_cache, cpu->bus, address, data);
#else
    uint64_t block_boundary = align_to_block_boundary(address);
    uint64_t offset = address - block_boundary;

    bus_device* device;
    block_range range;
    uint8_t* buf = addr_bus_lock_block(cpu->bus, address, &device, &range);

    if (buf == nullptr) {
        return false;
    }

    if (address + 4 > block_boundary + BLOCK_SIZE) {
        uint64_t next_boundary = align_to_block_boundary(address + 4);
        int remaining = next_boundary - address;
        memcpy(&buf[offset], &data, remaining);
        addr_bus_unlock_block(cpu->bus, address, device, range);

        buf = addr_bus_lock_block(cpu->bus, next_boundary, &device, &range);

        if (buf == nullptr) {
            return false;
        }

        memcpy(&buf[0], (char*)&data + remaining, 4 - remaining);
        addr_bus_unlock_block(cpu->bus, next_boundary, device, range);
    } else {
        memcpy(&buf[offset], &data, 4);
        addr_bus_unlock_block(cpu->bus, address, device, range);
    }
    return true;
#endif
}

bool cpu_write_2(Cpu* cpu, uint16_t data, uint64_t address) {
#ifdef USE_CACHE
    return cache_write_1(&cpu->data_cache, cpu->bus, address, data);
#else
    uint64_t block_boundary = align_to_block_boundary(address);
    uint64_t offset = address - block_boundary;

    bus_device* device;
    block_range range;
    uint8_t* buf = addr_bus_lock_block(cpu->bus, address, &device, &range);

    if (buf == nullptr) {
        return false;
    }

    if (address + 2 > block_boundary + BLOCK_SIZE) {
        uint64_t next_boundary = align_to_block_boundary(address + 2);
        memcpy(&buf[offset], &data, 1);
        addr_bus_unlock_block(cpu->bus, address, device, range);

        buf = addr_bus_lock_block(cpu->bus, next_boundary, &device, &range);

        if (buf == nullptr) {
            return false;
        }

        memcpy(&buf[0], (char*)&data + 1, 1);
        addr_bus_unlock_block(cpu->bus, next_boundary, device, range);
    } else {
        memcpy(&buf[offset], &data, 2);
        addr_bus_unlock_block(cpu->bus, address, device, range);
    }
    return true;
#endif
}

bool cpu_write_1(Cpu* cpu, uint8_t data, uint64_t address) {
#ifdef USE_CACHE
    return cache_write_1(&cpu->data_cache, cpu->bus, address, data);
#else
    uint64_t block_boundary = align_to_block_boundary(address);
    uint64_t offset = address - block_boundary;

    bus_device* device;
    block_range range;
    uint8_t* buf = addr_bus_lock_block(cpu->bus, address, &device, &range);

    if (buf == nullptr) {
        return false;
    }

    memcpy(&buf[offset], &data, 1);

    addr_bus_unlock_block(cpu->bus, address, device, range);

    return true;
#endif
}

bool cpu_read_8(Cpu* cpu, uint64_t address, uint64_t* value) {
#ifdef USE_CACHE
    return cache_read_8(&cpu->data_cache, cpu->bus, address, value);
#else
    uint64_t block_boundary = align_to_block_boundary(address);
    uint64_t offset = address - block_boundary;
    bus_device* device;
    block_range range;
    uint8_t* buf = addr_bus_lock_block(cpu->bus, address, &device, &range);

    if (buf == nullptr) {
        return false;
    }

    if (address + 8 > block_boundary + BLOCK_SIZE) {
        uint64_t next_boundary = align_to_block_boundary(address + 8);
        int remaining = next_boundary - address;
        memcpy(value, &buf[offset], remaining);
        addr_bus_unlock_block(cpu->bus, address, device, range);

        buf = addr_bus_lock_block(cpu->bus, next_boundary, &device, &range);

        if (buf == nullptr) {
            return false;
        }

        memcpy((char*)value + remaining, &buf[0], 8 - remaining);
        addr_bus_unlock_block(cpu->bus, next_boundary, device, range);
    } else {

        memcpy(value, &buf[offset], 8);
        addr_bus_unlock_block(cpu->bus, address, device, range);
    }
    return true;
#endif
}

bool cpu_read_4(Cpu* cpu, uint64_t address, uint32_t* value) {
#ifdef USE_CACHE
    return cache_read_4(&cpu->data_cache, cpu->bus, address, value);
#else
    uint64_t block_boundary = align_to_block_boundary(address);
    uint64_t offset = address - block_boundary;
    bus_device* device;
    block_range range;
    uint8_t* buf = addr_bus_lock_block(cpu->bus, address, &device, &range);

    if (buf == nullptr) {
        return false;
    }

    if (address + 4 > block_boundary + BLOCK_SIZE) {
        uint64_t next_boundary = align_to_block_boundary(address + 4);
        int remaining = next_boundary - address;
        memcpy(value, &buf[offset], remaining);
        addr_bus_unlock_block(cpu->bus, address, device, range);

        buf = addr_bus_lock_block(cpu->bus, next_boundary, &device, &range);

        if (buf == nullptr) {
            return false;
        }

        memcpy((char*)value + remaining, &buf[0], 4 - remaining);
        addr_bus_unlock_block(cpu->bus, next_boundary, device, range);
    } else {

        memcpy(value, &buf[offset], 4);
        addr_bus_unlock_block(cpu->bus, address, device, range);
    }
    return true;
#endif
}

bool cpu_read_2(Cpu* cpu, uint64_t address, uint16_t* value) {
#ifdef USE_CACHE
    return cache_read_2(&cpu->data_cache, cpu->bus, address, value);
#else
    uint64_t block_boundary = align_to_block_boundary(address);
    uint64_t offset = address - block_boundary;
    bus_device* device;
    block_range range;
    uint8_t* buf = addr_bus_lock_block(cpu->bus, address, &device, &range);

    if (buf == nullptr) {
        return false;
    }

    if (address + 2 > block_boundary + BLOCK_SIZE) {
        uint64_t next_boundary = align_to_block_boundary(address + 2);
        memcpy(value, &buf[offset], 1);
        addr_bus_unlock_block(cpu->bus, address, device, range);

        buf = addr_bus_lock_block(cpu->bus, next_boundary, &device, &range);

        if (buf == nullptr) {
            return false;
        }

        memcpy((char*)value + 1, &buf[0], 1);
        addr_bus_unlock_block(cpu->bus, next_boundary, device, range);
    } else {

        memcpy(value, &buf[offset], 2);
        addr_bus_unlock_block(cpu->bus, address, device, range);
    }
    return true;
#endif
}

bool cpu_read_1(Cpu* cpu, uint64_t address, uint8_t* value) {
#ifdef USE_CACHE
    return cache_read_1(&cpu->data_cache, cpu->bus, address, value);
#else
    uint64_t block_boundary = align_to_block_boundary(address);
    uint64_t offset = address - block_boundary;
    bus_device* device;
    block_range range;
    uint8_t* buf = addr_bus_lock_block(cpu->bus, address, &device, &range);

    if (buf == nullptr) {
        return false;
    }

    memcpy(value, &buf[offset], 1);
    addr_bus_unlock_block(cpu->bus, address, device, range);

    return true;
#endif
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
