#include <errno.h>
#include <stdatomic.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include "address_bus.h"
#include "bus_device.h"
#include "cpu.h"
#include "memory.h"
#include "timer.h"

/// Reads the file at PATH, and returns an allocated bufer and writes the length
/// of the buffer to the pointer referenced by `length`
///
/// The returned buffer is owned by the caller
///
/// # Errors
/// Returns NULL if reading failed and `length` will contain an unspecified
/// value
void* readFile(const char* path, uint64_t* length) {
    FILE* file = fopen(path, "rb");

    if (file == NULL) {
        return NULL;
    }

    fseek(file, 0, SEEK_END);
    long size = ftell(file);
    rewind(file);

    *length = size;
    void* buf = malloc(size);

    unsigned long n = 0;

    while (n != size) {
        if (ferror(file) != 0 && errno != EINTR) {
            fclose(file);
            free(buf);
            return NULL;
        } else if (feof(file) != 0) {
            break;
        }

        n = fread(buf, size, 1, file);
    }

    fclose(file);
    return buf;
}

int main(void) {
    address_bus bus;

    address_bus_init(&bus);

    if(!address_bus_add_memory(&bus, 0, 1 * 1024 * 1024)) {
        printf("Failed to add memory!\n");
        return 1;
    }
    address_bus_finalize_mapping(&bus);

    uint64_t length;
    uint8_t* program = readFile("output.bin", &length);

    if(!address_bus_write_n(&bus, 0, program, length)) {
        printf("Failed to load program\n");
        return 1;
    }

    Cpu cpu;
    cpu_create(&cpu, &bus);
    timer timer;
    timer_start(&timer);
    cpu_run(&cpu);
    double elapsed = timer_elapsed_seconds(&timer);

    printf("Time taken: %f\n", elapsed);

    cpu_destroy(&cpu);

    // addr_bus_destroy(&bus);
}
