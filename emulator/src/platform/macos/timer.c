#include "timer.h"

void timer_start(timer* timer) {
    mach_timebase_info_data_t timebase;
    mach_timebase_info(&timebase);
    timer->scale = (double)timebase.numer / (double)timebase.denom / 1e9;
    timer->start = mach_absolute_time();
}

double timer_elapsed_seconds(timer* timer) {
    uint64_t end = mach_absolute_time();

    double seconds = (double)(end - timer->start) * timer->scale;

    return seconds;
}
