#include "instruction_cache.h"
#include "cpu.h"
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef XXH64_hash_t hash_t;

constexpr XXH64_hash_t SEED = 992893;

instruction_cache instr_cache_create() {
    instruction_cache cache = {0};
    cache.cap = 1 << 10;
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
    entry->buf = instruction_buf_create(MAX_CACHE_BLOCK);
    return entry;
}

static inline uint64_t get_index(uint64_t key, uint64_t cap) {
    uint64_t index = key & (cap - 1);
    
    return index;
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
