#pragma once

#include "xxhash.h"
#include <stdint.h>
#include <stddef.h>
#include "instruction.h"


typedef struct {
    // The instructions in this block
    instruction* instructions;
    uint32_t len;
    uint32_t cap;
} block;

block instruction_buf_create(uint32_t cap);
// Copies the instruction to the end of the buffer
void instruction_buf_append(block* buf, instruction* instr);

typedef struct {
    uint64_t key; 
    block buf;
} bucket_entry;

typedef struct {
    bucket_entry* entries;
    uint32_t len;
    uint32_t cap;
} bucket;

typedef struct {
    // bucket* buckets;
    bucket* buckets;
    // Number of buckets
    uint64_t cap;
    // Number of items
    uint64_t len;
} instruction_cache;

instruction_cache instr_cache_create();
block* instr_cache_get(instruction_cache* cache, uint64_t address);
