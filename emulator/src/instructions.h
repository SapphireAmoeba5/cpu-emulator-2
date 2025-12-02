#pragma once

#include "cpu.h"
#include <stdint.h>


// Invalid instruction opcode
void invl(Cpu* cpu, uint8_t instruction[16]);

void halt(Cpu* cpu, uint8_t instruction[16]);
void intpt(Cpu* cpu, uint8_t instruction[16]);

void mov_reg(Cpu* cpu, uint8_t instruction[16]);
void mov_imm_or_mem(Cpu* cpu, uint8_t instruction[16]);

void str(Cpu* cpu, uint8_t instruction[16]);
