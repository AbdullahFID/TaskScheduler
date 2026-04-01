#![allow(dead_code)]
// queue.rs - circular queue for task arrival buffer
// stores task ids, not full tasks (look up details in hash table)

const QUEUE_CAP: usize = 1024;

pub struct Queue {
    data: [i32; QUEUE_CAP], // same thing as last time could have strings this but prof said i gotta do it manually
    front: usize,
    rear: usize,
    count: usize,
}

// storing tasks and not tasks id because tasks is MASSIVE but task id in comparison is not (memory efficient)

impl Queue {
    pub fn new() -> Queue {
        Queue {
            data: [0; QUEUE_CAP],
            front: 0,
            rear: 0,
            count: 0,
        }
    }

    // add task id to the back of the queue
    pub fn enqueue(&mut self, task_id: i32) -> i32 {
        if self.count >= QUEUE_CAP {
            println!("  [queue] full, cant enqueue task {}", task_id);
            return -1;
        }

        self.data[self.rear] = task_id; // put data into what rear is pointing at so like if it was all 0 and u put 5 into slot 1 (index 0) then u move rear to slot 2 (index 1)
        // so basically replace rear value with the data and move once more to the right
        // wrap around if we hit the end
        self.rear = (self.rear + 1) % QUEUE_CAP; // this is what matters if its 1023 and u add +1 then the quotient is remainder none so wrap
        self.count = self.count + 1;
        return 0;
    }

    // remove and return task id from the front (same shit but backwards)
    pub fn dequeue(&mut self) -> i32 {
        if self.count == 0 {
            return -1; // empty
        }

        let val = self.data[self.front];
        // move front forward, wrapping around
        self.front = (self.front + 1) % QUEUE_CAP;
        self.count = self.count - 1;
        return val;
    }

    pub fn is_empty(&self) -> bool {
        return self.count == 0;
    }

}

// O(1) btw
