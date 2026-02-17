#pragma once

#include "address_bus.h"
#include "instruction_cache.h"
#include "timer.h"
#include <stdbool.h>
#include <stdint.h>

#ifdef NDEBUG
#define UNREACHABLE() __builtin_unreachable()
#else
#include <stdlib.h>
#define UNREACHABLE()                                                          \
    printf("%s:%d Unreachable code reached!\n", __FILE__, __LINE__);           \
    abort()
#endif

// Maximum amount of instructions to cache if there were no branch points
// There should be a sweet spot that prevents slowdown from caching more
// instructions than get executed in a block, and from caching too few that we
// need to query the cache more often
constexpr uint64_t MAX_CACHE_BLOCK = 32;

constexpr uint64_t CLOCK_HZ = 500000000;

#define BIT(n) 1 << (n)

typedef enum {
    NO_ERROR = -1,
    MEMORY_ERROR,
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

    // If the CPU is halted
    bool halt;
    // If the CPU should delete itself
    bool exit;
    // CPU flags
    flags_t flags;

    instruction_cache cache;
    timer timer;

    // If the cache is valid
    bool valid_fetch_cache;
    // Where the cache is
    uint64_t fetch_cache_address;
    // This is the cache that is used when fetching instructions
    uint8_t fetch_cache[BLOCK_SIZE];
} Cpu;

bool cpu_write_8(Cpu* cpu, uint64_t data, uint64_t address);
bool cpu_write_4(Cpu* cpu, uint32_t data, uint64_t address);
bool cpu_write_2(Cpu* cpu, uint16_t data, uint64_t address);
bool cpu_write_1(Cpu* cpu, uint8_t data, uint64_t address);

/// `value` must not be NULL
bool cpu_read_8(Cpu* cpu, uint64_t address, uint64_t* value);
/// `value` must not be NULL
bool cpu_read_4(Cpu* cpu, uint64_t address, uint32_t* value);
/// `value` must not be NULL
bool cpu_read_2(Cpu* cpu, uint64_t address, uint16_t* value);
/// `value` must not be NULL
bool cpu_read_1(Cpu* cpu, uint64_t address, uint8_t* value);
bool cpu_read_n(Cpu* cpu, uint64_t address, void* out, uint64_t n);
bool cpu_read_block(Cpu* cpu, uint64_t address, void* out);

bool cpu_push(Cpu* cpu, uint64_t value);
bool cpu_pop(Cpu* cpu, uint64_t* out);

void cpu_create(Cpu* cpu, address_bus* bus);
void cpu_destroy(Cpu* cpu);
void cpu_run(Cpu* cpu);
