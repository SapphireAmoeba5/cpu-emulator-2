#include "spinlock.h"
#include <stdatomic.h>

// Spins until a lock is achieved
void spinlock_lock(spinlock* lock) {
    // Spin until this returns false
    while(atomic_flag_test_and_set_explicit(&lock->flag, memory_order_acquire));
}
void spinlock_unlock(spinlock* lock) {
    atomic_flag_clear_explicit(&lock->flag, memory_order_release);
}
