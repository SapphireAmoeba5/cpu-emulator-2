#include "job_queue.h"
#include <assert.h>

void job_queue_init(job_queue* queue) { memset(queue, 0, sizeof(*queue)); }

void job_queue_deinit(job_queue* queue) {
    job_queue_node* current = queue->head;

    while (current != nullptr) {
        job_queue_node* next = current->next;
        free(current);
        current = next;
    }
}

bool job_queue_is_empty(job_queue* queue) { return queue->head == nullptr; }

job_data job_queue_dequeue(job_queue* queue) {
    job_queue_node* next_head = queue->head->next;

    job_data job = queue->head->job;
    free(queue->head);

    queue->head = next_head;

    return job;
}

void job_queue_enqueue(job_queue* queue, const job_data* job) {
    job_queue_node* next_tail = calloc(1, sizeof(*next_tail));
    assert(next_tail->next == nullptr);

    next_tail->job = *job;

    if (queue->head == nullptr) {
        queue->head = next_tail;
        queue->tail = next_tail;
    } else {
        queue->tail->next = next_tail;
        queue->tail = next_tail;
    }
}
