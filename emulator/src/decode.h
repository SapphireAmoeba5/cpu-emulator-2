#pragma once

#include "cpu.h"
#include "instruction.h"
#include <stdint.h>

// Invalid register ID
constexpr uint8_t INVALID_ID = 255;

error_t cpu_decode(Cpu* cpu, instruction* instr, bool* branch_point);
