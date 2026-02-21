#pragma once

#include "cpu.h"
#include "instruction.h"

void cpu_execute(Cpu* cpu, instruction* instr);
