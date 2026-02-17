#pragma once

#include <mach/mach_time.h>


typedef struct {
    // Implementation defined
    double scale;
    uint64_t start;
} timer;

void timer_start(timer* timer);
double timer_elapsed_seconds(timer* timer);
