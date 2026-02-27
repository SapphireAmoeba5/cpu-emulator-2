#include "free_list.h"
#include "util/align.h"
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

uint64_t freelist_allocate(free_list* list, uint64_t size, uint64_t align,
                           bool* success) {
    if(size == 0) {
        *success = true;
        return 0;
    }

    while (list != nullptr) {
        uint64_t address = align_next(list->base, align);
        uint64_t offset_from_base = address - list->base;
        uint64_t size_free = list->size - offset_from_base;

        // Compare `list->size` and `size_free`
        // `size_free` will underflow if `address` rounds to to an address beyond
        // `list->base + list->size`
        if (list->size >= size && size_free >= size) {
            printf("Allocated at address: %llx\n", address);
            *success = true;

            if (list->base == address) {
                list->base += size;
                list->size -= size;

                // Remove entries with a size of zero
                if(list->size == 0 && list->next != nullptr) {
                    // Store it here so we can free it later
                    free_list* next = list->next; 

                    list->base = next->base;
                    list->size = next->size;
                    list->next = next->next;

                    free(next);
                }

            } else {
                uint64_t total_block_size = list->size;
                list->size = offset_from_base;

                free_list* next = malloc(sizeof(free_list));
                next->next = nullptr;
                next->base = address + size;
                next->size = total_block_size - list->size - size;

                list->next = next;
            }

            return address;
        } else {
            // Continue searching
            list = list->next;
        }
    }

    *success = false;
    return 0;
}

void freelist_pretty_print(free_list* list) {
    uint64_t total = 0;
    while (list != nullptr) {
        uint64_t base = list->base;
        uint64_t size = list->size;
        uint64_t end = base + size;

        printf("%016llx %016llx (%llu)\n", base, end, size);

        total += size;
        list = list->next;
    }

    printf("Total: %llu bytes\n", total);
    printf("Used: %llu bytes\n", UINT64_MAX - total);
}
