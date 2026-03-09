#pragma once

#include "job_queue.h"
#include "util/types.h"
#include "threads.h"

constexpr u64 THREAD_POOL_SIZE = 8;

typedef struct {
    cnd_t cnd;
    mtx_t cnd_mtx;


    // If true, tells the threads to exit once they have been woken up
    bool exit;
    job_queue jobs;
} thread_data;

typedef struct {
    thrd_t threads[THREAD_POOL_SIZE];

    thread_data data;
} thread_pool;

void thread_pool_init(thread_pool* pool);
void thread_pool_deinit(thread_pool *pool);

/// Enqueues `*job` to the job queue then signals a thread to start working on the job
/// `job` is copied into the queue
void thread_pool_queue_job(thread_pool* pool, job_t job, void* arg);
