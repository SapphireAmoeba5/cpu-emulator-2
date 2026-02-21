#include <stdatomic.h>

typedef struct {
    atomic_flag flag;
}spinlock;

/// Spins until a lock is achieved
void spinlock_lock(spinlock* lock);
void spinlock_unlock(spinlock* lock);
