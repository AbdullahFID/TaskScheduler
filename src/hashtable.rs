#![allow(dead_code)]
// hashtable.rs - hash table with separate chaining for task lookup
// uses an array-based node pool instead of heap allocation
// this is similar to how you'd do it in C with a pre-allocated array

use crate::task::Task;

const TABLE_SIZE: usize = 256;   // number of buckets
const POOL_SIZE: usize = 1024;   // max number of nodes we can store

#[derive(Clone, Copy)]
struct HashNode {
    task: Task,
    next: i32,     // index of next node in chain, -1 = end
    used: i32,     // 1 = occupied, 0 = free
}

impl HashNode {
    fn empty() -> HashNode {
        HashNode {
            task: Task::empty(),
            next: -1,
            used: 0,
        }
    }
}

pub struct HashTable {
    buckets: [i32; TABLE_SIZE],     // head index for each bucket, -1 = empty
    pool: [HashNode; POOL_SIZE],    // node storage (like malloc'd array in C)
    pool_next: usize,               // next free slot
}

// simple hash function - just mod by table size
fn hash_id(id: i32) -> usize {
    let val = if id < 0 { -id } else { id };
    return (val as usize) % TABLE_SIZE;
}

impl HashTable {
    pub fn new() -> HashTable {
        HashTable {
            buckets: [-1; TABLE_SIZE],
            pool: [HashNode::empty(); POOL_SIZE],
            pool_next: 0,
        }
    }

    // grab a free slot from the pool
    fn alloc_node(&mut self) -> i32 {
        // first try to find a freed slot
        let mut i = 0;
        while i < self.pool_next {
            if self.pool[i].used == 0 {
                return i as i32;
            }
            i = i + 1;
        }

        // otherwise use the next fresh slot
        if self.pool_next >= POOL_SIZE {
            return -1; // pool is full
        }
        let idx = self.pool_next;
        self.pool_next = self.pool_next + 1;
        return idx as i32;
    }

    // insert a task into the hash table
    pub fn insert(&mut self, task: Task) -> i32 {
        // check for duplicate first
        if self.lookup(task.id).is_some() {
            println!("  [ht] duplicate id {}, skipping", task.id);
            return -1;
        }

        let slot = self.alloc_node();
        if slot < 0 {
            println!("  [ht] pool full, cant insert task {}", task.id);
            return -1;
        }

        let idx = slot as usize;
        let bucket = hash_id(task.id);

        // fill in the node
        self.pool[idx].task = task;
        self.pool[idx].used = 1;

        // link it at the head of the bucket chain
        self.pool[idx].next = self.buckets[bucket];
        self.buckets[bucket] = slot;

        return 0;
    }

    // look up a task by id - returns a copy if found
    pub fn lookup(&self, id: i32) -> Option<Task> {
        let bucket = hash_id(id);
        let mut cur = self.buckets[bucket];

        while cur >= 0 {
            let node = &self.pool[cur as usize];
            if node.used == 1 && node.task.id == id {
                return Some(node.task);
            }
            cur = node.next;
        }

        return None;
    }

    // update the status field for a task
    pub fn update_status(&mut self, id: i32, new_status: i32) -> i32 {
        let bucket = hash_id(id);
        let mut cur = self.buckets[bucket];

        while cur >= 0 {
            let idx = cur as usize;
            if self.pool[idx].used == 1 && self.pool[idx].task.id == id {
                self.pool[idx].task.status = new_status;
                return 0;
            }
            cur = self.pool[idx].next;
        }

        return -1; // not found
    }

    // delete a task by id
    pub fn delete(&mut self, id: i32) -> i32 {
        let bucket = hash_id(id);
        let mut prev: i32 = -1;
        let mut cur = self.buckets[bucket];

        while cur >= 0 {
            let idx = cur as usize;
            if self.pool[idx].used == 1 && self.pool[idx].task.id == id {
                // unlink from chain
                if prev < 0 {
                    // its the head of the bucket
                    self.buckets[bucket] = self.pool[idx].next;
                } else {
                    self.pool[prev as usize].next = self.pool[idx].next;
                }
                // mark as free
                self.pool[idx].used = 0;
                self.pool[idx].next = -1;
                return 0;
            }
            prev = cur;
            cur = self.pool[idx].next;
        }

        return -1; // not found
    }
}
