#include "thread_pool.h"
#include "job_queue.h"
#include "util/common.h"
#include <stdio.h>
#include <threads.h>

static int thread_pool_worker(void* arg) {
    thread_data* data = (thread_data*)arg;


    while (1) {
        mtx_lock(&data->cnd_mtx);

        while (job_queue_is_empty(&data->jobs)) {
            cnd_wait(&data->cnd, &data->cnd_mtx);

            if (data->exit) {
                mtx_unlock(&data->cnd_mtx);
                return 0;
            }
        }
        // The mutex is locked at this point, it is safe to dequeue data
        job_data next_job = job_queue_dequeue(&data->jobs);

        // Now we own this job and no other thread can access it so it is safe
        // to unlock the mutex
        mtx_unlock(&data->cnd_mtx);

        // Execute the job
        next_job.job(next_job.arg);
    }

    UNREACHABLE_SAFE("This should never be run\n");
}

void thread_pool_init(thread_pool* pool) {
    job_queue_init(&pool->data.jobs);

    mtx_init(&pool->data.cnd_mtx, mtx_plain);
    cnd_init(&pool->data.cnd);

    void* arg = &pool->data;
    for (u64 i = 0; i < THREAD_POOL_SIZE; i++) {
        thrd_create(&pool->threads[i], thread_pool_worker, arg);
    }
}

void thread_pool_deinit(thread_pool* pool) {
    mtx_lock(&pool->data.cnd_mtx);

    pool->data.exit = true;
    cnd_broadcast(&pool->data.cnd);

    mtx_unlock(&pool->data.cnd_mtx);

    for (u64 i = 0; i < THREAD_POOL_SIZE; i++) {
        thrd_t thread = pool->threads[i];
        thrd_join(thread, NULL);
    }

    mtx_destroy(&pool->data.cnd_mtx);
    cnd_destroy(&pool->data.cnd);

    job_queue_deinit(&pool->data.jobs);
}

void thread_pool_queue_job(thread_pool* pool, job_t job, void* arg) {
    mtx_lock(&pool->data.cnd_mtx);

    job_data job_data;
    job_data.arg = arg;
    job_data.job = job;
    job_queue_enqueue(&pool->data.jobs, &job_data);

    cnd_signal(&pool->data.cnd);

    mtx_unlock(&pool->data.cnd_mtx);
}
