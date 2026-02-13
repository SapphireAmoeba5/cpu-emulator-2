#include "execute.h"
#include "decode.h"
#include <stdio.h>

static void intpt(Cpu* cpu, int index) {
    if (index == 0x80) {
        printf("Cycle: %llu\n", cpu->clock_count);
        for (int i = 0; i < 16; i++) {
            uint64_t value = cpu->registers[i].r;
            printf("r%llu = %016llx (%lld)\n", (uint64_t)i, value,
                   (int64_t)value);
        }

        printf("ip: %llu\nsp: %llu\n", cpu->ip, cpu->sp);

        printf("ZR | CR | OF | SN\n");

        if (cpu->flags & FLAG_ZERO) {
            printf("1  | ");

        } else {
            printf("0  | ");
        }
        if (cpu->flags & FLAG_CARRY) {

            printf("1  | ");
        } else {

            printf("0  | ");
        }
        if (cpu->flags & FLAG_OVERFLOW) {

            printf("1  | ");
        } else {

            printf("0  | ");
        }
        if (cpu->flags & FLAG_SIGN) {

            printf("1\n");
        } else {

            printf("0\n");
        }

        printf("%b\n", cpu->flags);

        cpu->exit = true;
    } else if (index == 0x81) {
        cpu->registers[0].r -= 1;
        cpu->flags &= ~FLAG_ZERO;
        if (cpu->registers[0].r == 0) {
            cpu->flags |= FLAG_ZERO;
        }
    } else if (index == 0x82) {
        printf("DEBUG PRINT %llu\n", cpu->clock_count);
    }
}

void cpu_execute(Cpu* cpu, instruction* instr) {
    switch (instr->op) {
    case op_halt:
        printf("CPU HALTED\n");
        cpu->halt = true;
        return;
    case op_int:
        intpt(cpu, instr->src);
        cpu->exit = true;
        return;
    case op_mov:
        printf("Value: %p :: %llu\n", instr->dest, ((uint64_t)instr->dest - (uint64_t)&cpu->registers[0]) / 8);
        *instr->dest = instr->src;
        return;
    case op_invl:
        UNREACHABLE();
        return;
    default:
        UNREACHABLE();
        return;
    }
}
