#pragma once

#include "cpu.h"
#include "instruction.h"

error_t cpu_execute(Cpu* cpu, instruction* instr);
