#pragma once

/// Compiler memory barrier
#define BARRIER() asm volatile("" ::: "memory")
