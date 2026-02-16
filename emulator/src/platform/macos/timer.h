#pragma once

#include <mach/mach_time.h>


typedef struct {
    // Implementation defined
    mach_timebase_info_data_t timebase;
    uint64_t start;
} timer;

void timer_start(timer* timer);
double timer_elapsed_seconds(timer* timer);
