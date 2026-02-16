#pragma once

#include <time.h>

typedef struct {
    // Implementation defined
    struct timespec timespec;
} timer;

void timer_start(timer* timer);
double timer_elapsed_seconds(timer* timer);
