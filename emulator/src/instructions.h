#pragma once

#include "cpu.h"
#include <stdint.h>


// Invalid instruction opcode
void invl(Cpu* cpu, uint8_t instruction[16]);

void halt(Cpu* cpu, uint8_t instruction[16]);
void intpt(Cpu* cpu, uint8_t instruction[16]);

void mov_reg_reg(Cpu* cpu, uint8_t instruction[16]);
void mov_reg_imm8(Cpu* cpu, uint8_t instruction[16]);
void mov_reg_imm16(Cpu* cpu, uint8_t instruction[16]);
void mov_reg_imm32(Cpu* cpu, uint8_t instruction[16]);
void mov_reg_imm64(Cpu* cpu, uint8_t instruction[16]);

void add_reg_reg(Cpu* cpu, uint8_t instruction[16]);
void add_reg_imm8(Cpu* cpu, uint8_t instruction[16]);
void add_reg_imm16(Cpu* cpu, uint8_t instruction[16]);
void add_reg_imm32(Cpu* cpu, uint8_t instruction[16]);
void add_reg_imm64(Cpu* cpu, uint8_t instruction[16]);

void sub_reg_reg(Cpu* cpu, uint8_t instruction[16]);
void sub_reg_imm8(Cpu* cpu, uint8_t instruction[16]);
void sub_reg_imm16(Cpu* cpu, uint8_t instruction[16]);
void sub_reg_imm32(Cpu* cpu, uint8_t instruction[16]);
void sub_reg_imm64(Cpu* cpu, uint8_t instruction[16]);

void mul_reg_reg(Cpu* cpu, uint8_t instruction[16]);
void mul_reg_imm8(Cpu* cpu, uint8_t instruction[16]);
void mul_reg_imm16(Cpu* cpu, uint8_t instruction[16]);
void mul_reg_imm32(Cpu* cpu, uint8_t instruction[16]);
void mul_reg_imm64(Cpu* cpu, uint8_t instruction[16]);

void div_reg_reg(Cpu* cpu, uint8_t instruction[16]);
void div_reg_imm8(Cpu* cpu, uint8_t instruction[16]);
void div_reg_imm16(Cpu* cpu, uint8_t instruction[16]);
void div_reg_imm32(Cpu* cpu, uint8_t instruction[16]);
void div_reg_imm64(Cpu* cpu, uint8_t instruction[16]);

void div_reg_reg(Cpu* cpu, uint8_t instruction[16]);
void div_reg_imm8(Cpu* cpu, uint8_t instruction[16]);
void div_reg_imm16(Cpu* cpu, uint8_t instruction[16]);
void div_reg_imm32(Cpu* cpu, uint8_t instruction[16]);
void div_reg_imm64(Cpu* cpu, uint8_t instruction[16]);

void idiv_reg_reg(Cpu* cpu, uint8_t instruction[16]);
void idiv_reg_imm8(Cpu* cpu, uint8_t instruction[16]);
void idiv_reg_imm16(Cpu* cpu, uint8_t instruction[16]);
void idiv_reg_imm32(Cpu* cpu, uint8_t instruction[16]);
void idiv_reg_imm64(Cpu* cpu, uint8_t instruction[16]);

void and_reg_reg(Cpu* cpu, uint8_t instruction[16]);
void and_reg_imm8(Cpu* cpu, uint8_t instruction[16]);
void and_reg_imm16(Cpu* cpu, uint8_t instruction[16]);
void and_reg_imm32(Cpu* cpu, uint8_t instruction[16]);
void and_reg_imm64(Cpu* cpu, uint8_t instruction[16]);

void or_reg_reg(Cpu* cpu, uint8_t instruction[16]);
void or_reg_imm8(Cpu* cpu, uint8_t instruction[16]);
void or_reg_imm16(Cpu* cpu, uint8_t instruction[16]);
void or_reg_imm32(Cpu* cpu, uint8_t instruction[16]);
void or_reg_imm64(Cpu* cpu, uint8_t instruction[16]);

void xor_reg_reg(Cpu* cpu, uint8_t instruction[16]);
void xor_reg_imm8(Cpu* cpu, uint8_t instruction[16]);
void xor_reg_imm16(Cpu* cpu, uint8_t instruction[16]);
void xor_reg_imm32(Cpu* cpu, uint8_t instruction[16]);
void xor_reg_imm64(Cpu* cpu, uint8_t instruction[16]);
