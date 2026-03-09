#pragma once

#include <stdlib.h>
#include <string.h>

typedef void(*job_t)(void*); 

typedef struct {
    void* arg;
    job_t job;
} job_data;

/// Implemented using a simple linked list
typedef struct job_queue_node {
    job_data job;
    struct job_queue_node* next;
} job_queue_node;

typedef struct {
    job_queue_node* head;
    job_queue_node* tail;
} job_queue;

void job_queue_init(job_queue* queue);
/// It is UB to use this queue after calling this unless you call job_queue_init again
void job_queue_deinit(job_queue* queue);

/// Returns true if the queue is empty, if the queue isn't empty is returns
/// false
bool job_queue_is_empty(job_queue* queue);

/// Undefined behavior if `queue` is empty
job_data job_queue_dequeue(job_queue* queue);

/// Enqueues `job` to the front of the queue.
/// `job` is copied into the queue
void job_queue_enqueue(job_queue* queue, const job_data* job);
