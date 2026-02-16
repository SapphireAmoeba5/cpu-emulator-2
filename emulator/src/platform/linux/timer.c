#include "timer.h"

void timer_start(timer* timer) {
    clock_gettime(CLOCK_MONOTONIC_COARSE, &timer->timespec);
}

double timer_elapsed_seconds(timer* timer) {
    struct timespec* start = &timer->timespec;
    struct timespec end;
    clock_gettime(CLOCK_MONOTONIC_COARSE, &end);
    double elapsed =
        (end.tv_sec - start->tv_sec) + (end.tv_nsec - start->tv_nsec) / 1e9;

    return elapsed;
}
