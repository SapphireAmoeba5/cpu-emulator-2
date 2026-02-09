#pragma once

#include "cpu.h"
#include <stdint.h>


// Invalid instruction opcode
void invl(Cpu* cpu, uint8_t instruction[16]);

void halt(Cpu* cpu, uint8_t instruction[16]);
void intpt(Cpu* cpu, uint8_t instruction[16]);

void mov_reg(Cpu* cpu, uint8_t instruction[16]);
void mov_mem(Cpu* cpu, uint8_t instruction[16]);
void mov_imm(Cpu* cpu, uint8_t instruction[16]);

void sub_reg(Cpu* cpu, uint8_t instruction[16]);
void sub_mem(Cpu* cpu, uint8_t instruction[16]);
void sub_imm(Cpu* cpu, uint8_t instruction[16]);

void add_reg(Cpu* cpu, uint8_t instruction[16]);
void add_mem(Cpu* cpu, uint8_t instruction[16]);
void add_imm(Cpu* cpu, uint8_t instruction[16]);

void mul_reg(Cpu* cpu, uint8_t instruction[16]);
void mul_mem(Cpu* cpu, uint8_t instruction[16]);
void mul_imm(Cpu* cpu, uint8_t instruction[16]);

void div_reg(Cpu* cpu, uint8_t instruction[16]);
void div_mem(Cpu* cpu, uint8_t instruction[16]);
void div_imm(Cpu* cpu, uint8_t instruction[16]);

void idiv_reg(Cpu* cpu, uint8_t instruction[16]);
void idiv_mem(Cpu* cpu, uint8_t instruction[16]);
void idiv_imm(Cpu* cpu, uint8_t instruction[16]);

void and_reg(Cpu* cpu, uint8_t instruction[16]);
void and_mem(Cpu* cpu, uint8_t instruction[16]);
void and_imm(Cpu* cpu, uint8_t instruction[16]);

void or_reg(Cpu* cpu, uint8_t instruction[16]);
void or_mem(Cpu* cpu, uint8_t instruction[16]);
void or_imm(Cpu* cpu, uint8_t instruction[16]);

void xor_reg(Cpu* cpu, uint8_t instruction[16]);
void xor_mem(Cpu* cpu, uint8_t instruction[16]);
void xor_imm(Cpu* cpu, uint8_t instruction[16]);

void cmp_reg(Cpu* cpu, uint8_t instruction[16]);
void cmp_mem(Cpu* cpu, uint8_t instruction[16]);
void cmp_imm(Cpu* cpu, uint8_t instruction[16]);

void test_reg(Cpu* cpu, uint8_t instruction[16]);
void test_mem(Cpu* cpu, uint8_t instruction[16]);
void test_imm(Cpu* cpu, uint8_t instruction[16]);

void str(Cpu* cpu, uint8_t instruction[16]);

void jmp(Cpu* cpu, uint8_t instruction[16]);
void jz(Cpu* cpu, uint8_t instruction[16]);
void jnz(Cpu* cpu, uint8_t instruction[16]);
void jc(Cpu* cpu, uint8_t instruction[16]);
void jnc(Cpu* cpu, uint8_t instruction[16]);
void jo(Cpu* cpu, uint8_t instruction[16]);
void jno(Cpu* cpu, uint8_t instruction[16]);
void js(Cpu* cpu, uint8_t instruction[16]);
void jns(Cpu* cpu, uint8_t instruction[16]);
void ja(Cpu* cpu, uint8_t instruction[16]);
void jbe(Cpu* cpu, uint8_t instruction[16]);

void jg(Cpu* cpu, uint8_t instruction[16]);
void jle(Cpu* cpu, uint8_t instruction[16]);

void jge(Cpu* cpu, uint8_t instruction[16]);
void jl(Cpu* cpu, uint8_t instruction[16]);
