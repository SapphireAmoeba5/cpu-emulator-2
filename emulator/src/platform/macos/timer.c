#include "timer.h"


void timer_start(timer* timer) {
    mach_timebase_info(&timer->timebase);
    timer->start = mach_absolute_time();
}

double timer_elapsed_seconds(timer* timer) {
    uint64_t end = mach_absolute_time();

    double nanoseconds = (double)(end - timer->start) * timer->timebase.numer / timer->timebase.denom;
    double seconds = nanoseconds / 1e9;

    return seconds;
}
