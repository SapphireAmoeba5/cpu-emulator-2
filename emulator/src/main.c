#include <asm-generic/errno-base.h>
#include <asm-generic/errno.h>
#include <errno.h>
#include <stdatomic.h>
#include <stdckdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <threads.h>
#include <time.h>

#include "cpu.h"
#include "memory.h"

/// Reads the file at PATH, and returns an allocated bufer and writes the length of the buffer to the pointer referenced by `length`
///
/// The returned buffer is owned by the caller
///
/// # Errors
/// Returns NULL if reading failed and `length` will contain an unspecified value
void* readFile(const char* path, uint64_t* length) {
    FILE* file = fopen(path, "r");
    
    if (file == NULL) {
        return NULL;
    }

    fseek(file, 0, SEEK_END);
    long size = ftell(file);
    rewind(file);

    *length = size;
    void* buf = malloc(size);

    unsigned long n = 0;

    while(n != size) {
        if(ferror(file) != 0 && errno != EINTR) {
            fclose(file);
            free(buf);
            return NULL;
        }
        else if(feof(file) != 0) {
            break;
        }

        n = fread(buf, size, 1, file);
    }

    fclose(file);
    return buf;
}

int main(void) { 
    Memory* memory = malloc(sizeof(Memory));
    memory_create(memory, 1024 * 1024 * 100 / 8);

    uint64_t length;
    uint8_t* program = readFile("output.bin", &length);

    for(int i = 0; i < length; i++) {
        memory_write_1(memory, program[i], i);
    }

    printf("Done writing\n");

    Cpu cpu;
    cpu_create(&cpu, memory);
    auto start = clock();
    cpu_run(&cpu);
    auto end = clock();

    double duration = (double)(end - start) / CLOCKS_PER_SEC;

    printf("Time taken: %f\n", duration);
}
