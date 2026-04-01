#![allow(dead_code)]
// queue.rs - circular queue for task arrival buffer
// this is the first stop for tasks when they arrive at the scheduler
// its a FIFO (first in first out) buffer - like a line at tims, first person in line gets served first
// we only store task IDs here, not the full task struct, because the full task is huge
// and we can just look up the details in the hash table later when we need them

// max number of items the queue can hold at once
// matches MAX_TASKS because worst case every task arrives at the same time
const QUEUE_CAP: usize = 1024;

// the queue struct itself
// this is a "circular" queue which means when we reach the end of the array we wrap back to index 0
// imagine the array as a circle, not a line - the rear can wrap around and use empty slots at the front
// this way we never have to shift elements around which would be slow
pub struct Queue {
    data: [i32; QUEUE_CAP], // the actual array that holds task ids
                             // same thing as last time could have used Vec but prof said i gotta do it manually
    front: usize,            // index of the first element in the queue (the one that gets dequeued next)
    rear: usize,             // index of the NEXT empty slot (where the next enqueue goes)
    count: usize,            // how many elements are currently in the queue
}

// storing task ids and not full tasks because a task struct is MASSIVE (has name array, all the fields etc)
// but a task id is just one i32 (4 bytes). way more memory efficient
// when we need the full task details we just look it up in the hash table by id

impl Queue {
    // create a brand new empty queue
    // front and rear both start at 0, count is 0, array is all zeros
    pub fn new() -> Queue {
        Queue {
            data: [0; QUEUE_CAP],
            front: 0,
            rear: 0,
            count: 0,
        }
    }

    // add a task id to the back of the queue
    // returns 0 on success, -1 if the queue is full
    // this is O(1) - constant time, always the same speed no matter how many items are in the queue
    pub fn enqueue(&mut self, task_id: i32) -> i32 {
        if self.count >= QUEUE_CAP {
            println!("  [queue] full, cant enqueue task {}", task_id);
            return -1;
        }

        self.data[self.rear] = task_id; // put the task id into the slot that rear is pointing at
        // so like if rear is at index 3, we put the id at data[3], then move rear to index 4
        // the modulo (%) is the key trick here - it makes the queue circular
        // if rear is at 1023 (last index) and we add 1, we get 1024 % 1024 = 0, wrapping back to the start
        // this is what makes it a CIRCULAR queue instead of a regular one
        self.rear = (self.rear + 1) % QUEUE_CAP;
        self.count = self.count + 1;
        return 0;
    }

    // remove and return the task id from the front of the queue
    // returns the task id, or -1 if the queue is empty
    // also O(1) - same constant time trick, just move the front pointer forward
    // we dont actually "delete" the old data, we just move past it. it gets overwritten eventually
    pub fn dequeue(&mut self) -> i32 {
        if self.count == 0 {
            return -1; // nothing to dequeue, queue is empty
        }

        let val = self.data[self.front]; // grab the value at the front
        // move front forward with the same wrap-around trick
        // if front was at 1023 and we add 1, we get 0 again
        self.front = (self.front + 1) % QUEUE_CAP;
        self.count = self.count - 1;
        return val;
    }

    // check if the queue has no elements in it
    // used in the main simulation loop to know when to stop draining into the heap
    pub fn is_empty(&self) -> bool {
        return self.count == 0;
    }

}

// both enqueue and dequeue are O(1) which is the whole point of using a circular queue
// no shifting, no searching, just pointer math and modulo. fast as it gets
