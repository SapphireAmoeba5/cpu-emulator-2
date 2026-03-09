#pragma once

#include <stdint.h>
#include <limits.h>

typedef struct free_list {
    uint64_t base;
    uint64_t size;
    struct free_list* next;
} free_list;

inline static void freelist_init(free_list* list) {
    list->base = 0;
    list->size = UINT64_MAX;
    list->next = nullptr;
}
void freelist_deinit(free_list* list);

/// Attempts to allocate `size` bytes with alignment `align`.
/// On success returns the base address of the allocation and sets `*success` to true
/// On failure sets `*success` to false
uint64_t freelist_allocate(free_list* list, uint64_t size, uint64_t align, bool* success);

void freelist_pretty_print(free_list* list);
