#include "instruction_cache.h"
#include "cpu.h"
#include "instruction.h"
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef XXH64_hash_t hash_t;

constexpr XXH64_hash_t SEED = 992893;

static void resize(block* buf) {
    uint32_t new_cap = (buf->cap + 1) * 2;
    buf->cap = new_cap;

    instruction* instr = realloc(buf->instructions, new_cap * sizeof(instruction));
    buf->instructions = instr;
}

block instruction_buf_create(uint32_t cap) {
    block buf = {0};
    return buf;
}

void instruction_buf_append(block* buf, instruction* instr) {
    if (buf->len >= buf->cap) {
        resize(buf);
    }

    buf->instructions[buf->len++] = *instr;
}

instruction_cache instr_cache_create() {
    instruction_cache cache = {0};
    cache.cap = 128;
    cache.buckets = calloc(1, cache.cap * (sizeof(*cache.buckets)));
    return cache;
}

bucket_entry* add_new_entry_to_bucket(bucket* bucket, uint64_t key) {
    if (bucket->len >= bucket->cap) {
        bucket->cap = (bucket->cap + 1) * 2;
        bucket->entries = realloc(bucket->entries, bucket->cap * sizeof(bucket_entry));
    }

    bucket_entry* entry = &bucket->entries[bucket->len++];
    entry->key = key;
    memset(&entry->buf, 0, sizeof(entry->buf));
    entry->buf.cap = MAX_CACHE_BLOCK;
    entry->buf.instructions = malloc(MAX_CACHE_BLOCK * sizeof(instruction));
    return entry;
}

static uint64_t get_index(uint64_t key, uint64_t cap) {
    return XXH64(&key, sizeof(key), SEED) % cap;
}

block* instr_cache_get(instruction_cache* cache, uint64_t address) {
    uint64_t index = get_index(address, cache->cap);
    bucket* bucket = &cache->buckets[index];


    for (int i = 0; i < bucket->len; i++) {
        if (bucket->entries[i].key == address) {
            return &bucket->entries[i].buf;
        }
    }
    return &add_new_entry_to_bucket(bucket, address)->buf;
}
