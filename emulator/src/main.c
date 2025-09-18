#include <stdckdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <threads.h>
#include <time.h>

#include "cpu.h"
#include "memory.h"

int main(void) { 
    Memory* memory = malloc(sizeof(Memory));
    memory_create(memory, 1024 * 1024 * 100 / 8);

    FILE* file = fopen("output.bin", "r");
    fseek(file, 0, SEEK_END);
    size_t size = ftell(file);
    rewind(file);

    uint8_t* buffer = malloc(size);
    fread(buffer, 1, size, file);

    fclose(file);

    for(int i = 0; i < size; i++) {
        memory_write_1(memory, buffer[i], i);
    }

    Cpu cpu;
    cpu_create(&cpu, memory);

    auto start = clock();
    cpu_run(&cpu);
    auto end = clock();

    double duration = (double)(end - start) / CLOCKS_PER_SEC;

    printf("Time taken: %f\n", duration);
}
