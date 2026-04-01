#![allow(dead_code)]
// hashtable.rs - hash table with separate chaining for task lookup
// this is the "memory" of the scheduler - it stores the full details of every task
// and lets you look up any task by its id in O(1) average time
//
// why we need this: the heap only stores lightweight entries (id + sort keys)
// but when the scheduler picks a task to run, it needs the FULL info (name, duration, etc)
// so we ask the hash table "hey whats the deal with task #7?" and it tells us everything
//
// uses an array-based node pool instead of heap allocation (no malloc/Box/Vec)
// this is similar to how you'd do it in C with a pre-allocated array
// prof wanted everything manual so here we are

use crate::task::Task;

// TABLE_SIZE is how many "buckets" we have - think of it as 256 drawers in a filing cabinet
// when a task comes in we do math to figure out which drawer it goes in
const TABLE_SIZE: usize = 256;

// POOL_SIZE is the total number of nodes (slots) we can store across all buckets combined
// this is our pre-allocated memory pool, like calling malloc(1024 * sizeof(node)) upfront in C
const POOL_SIZE: usize = 1024;

// each node in the hash table holds one task plus a pointer to the next node in the chain
// "chain" because multiple tasks can end up in the same bucket (called a collision)
// the next field is an INDEX into the pool array, not a real pointer (-1 means end of chain)
#[derive(Clone, Copy)]
struct HashNode {
    task: Task,    // the full task data (id, name, priority, deadline, everything)
    next: i32,     // index of next node in this bucket's chain, -1 = end of chain
    used: i32,     // 1 = this slot has real data, 0 = this slot is free/empty
}

impl HashNode {
    // create an empty node, used to fill the pool array at the start
    fn empty() -> HashNode {
        HashNode {
            task: Task::empty(),
            next: -1,
            used: 0,
        }
    }
}

// the hash table itself
// buckets array: each index is a "drawer" that points to the first node in that drawer's chain
// pool array: where all the actual nodes live, its one big pre-allocated chunk
// pool_next: tracks how far into the pool we've gone (next unused slot)
pub struct HashTable {
    buckets: [i32; TABLE_SIZE],     // 256 buckets, each holds the index of the first node in its chain
                                     // -1 means that bucket is empty (no tasks hashed to it)
    pool: [HashNode; POOL_SIZE],    // the actual storage for all nodes
                                     // think of it like a big array we "allocate" from manually
                                     // this is basically our own mini memory allocator
    pool_next: usize,               // next fresh slot in the pool that hasnt been used yet
}

// the hash function - turns a task id into a bucket index (0 to 255)
// its super simple: just take the absolute value and mod by table size
// example: task id 473 -> 473 % 256 = 217 -> goes in bucket 217
// example: task id 7   -> 7 % 256 = 7     -> goes in bucket 7
// example: task id 263 -> 263 % 256 = 7   -> ALSO goes in bucket 7 (collision!)
// collisions are fine, thats what the chains (linked lists) are for
fn hash_id(id: i32) -> usize {
    let val = if id < 0 { -id } else { id }; // absolute value so negative ids dont break things
    return (val as usize) % TABLE_SIZE;
}

impl HashTable {
    // create a fresh empty hash table
    // all 256 buckets point to -1 (empty), pool is all empty nodes
    pub fn new() -> HashTable {
        HashTable {
            buckets: [-1; TABLE_SIZE],          // every bucket starts empty
            pool: [HashNode::empty(); POOL_SIZE], // every slot starts unused
            pool_next: 0,                        // havent used any slots yet
        }
    }

    // grab a free slot from the pool - this is our manual memory allocator
    // first it scans through already-used slots to find any that were freed (used == 0)
    // if it cant find a recycled slot, it grabs the next fresh one from pool_next
    // returns the index of the free slot, or -1 if the pool is completely full
    fn alloc_node(&mut self) -> i32 {
        // first try to find a previously freed slot (recycling)
        let mut i = 0;
        while i < self.pool_next {
            if self.pool[i].used == 0 {
                return i as i32; // found a free one, reuse it
            }
            i = i + 1;
        }

        // no recycled slots available, grab a fresh one from the end
        if self.pool_next >= POOL_SIZE {
            return -1; // pool is completely full, cant store anything more
        }
        let idx = self.pool_next;
        self.pool_next = self.pool_next + 1; // advance to next fresh slot for next time
        return idx as i32;
    }

    // insert a task into the hash table
    // 1. check if this task id already exists (no duplicates allowed)
    // 2. allocate a node from the pool
    // 3. hash the id to figure out which bucket
    // 4. link the new node at the HEAD of that bucket's chain
    //
    // inserting at the head is easiest: new node points to old head, bucket points to new node
    // example: bucket[7] was -> [task 7] -> [task 263] -> -1
    //          insert task 519 (519 % 256 = 7, same bucket)
    //          bucket[7] now -> [task 519] -> [task 7] -> [task 263] -> -1
    //
    // O(1) average time (ignoring the duplicate check which is also O(1) average)
    pub fn insert(&mut self, task: Task) -> i32 {
        // make sure we dont already have this id
        if self.lookup(task.id).is_some() {
            println!("  [ht] duplicate id {}, skipping", task.id);
            return -1;
        }

        // get a free slot from our pool
        let slot = self.alloc_node();
        if slot < 0 {
            println!("  [ht] pool full, cant insert task {}", task.id);
            return -1;
        }

        let idx = slot as usize;
        let bucket = hash_id(task.id); // figure out which bucket this task belongs in

        // fill in the node with the task data
        self.pool[idx].task = task;
        self.pool[idx].used = 1; // mark as occupied

        // link at the head of the bucket chain
        // the new node's "next" points to whatever was previously the head
        // then the bucket itself points to the new node (making it the new head)
        self.pool[idx].next = self.buckets[bucket]; // new node -> old head
        self.buckets[bucket] = slot;                  // bucket -> new node

        return 0; // success
    }

    // look up a task by its id - returns a copy of the task if found, None if not
    // this is the main reason the hash table exists: fast lookup by id
    // the heap says "task 7 is most urgent" and we ask the hash table "give me task 7's details"
    //
    // how it works:
    // 1. hash the id to find the bucket
    // 2. walk the chain in that bucket checking each node's task id
    // 3. return the first match, or None if we reach the end of the chain
    //
    // O(1) average time because most buckets have very few items in their chain
    // worst case is O(n) if everything hashes to the same bucket but that basically never happens
    pub fn lookup(&self, id: i32) -> Option<Task> {
        let bucket = hash_id(id);          // which bucket to look in
        let mut cur = self.buckets[bucket]; // start at the head of that bucket's chain

        // walk the chain until we find the task or hit the end (-1)
        while cur >= 0 {
            let node = &self.pool[cur as usize];
            if node.used == 1 && node.task.id == id {
                return Some(node.task); // found it, return a copy
            }
            cur = node.next; // move to next node in the chain
        }

        return None; // walked the entire chain, task not found
    }

    // update the status field for a task (QUEUED -> READY -> RUNNING -> DONE)
    // same walking logic as lookup but instead of returning the task we modify it in place
    // this gets called every time a task moves to a new lifecycle stage
    // returns 0 on success, -1 if the task wasnt found
    pub fn update_status(&mut self, id: i32, new_status: i32) -> i32 {
        let bucket = hash_id(id);
        let mut cur = self.buckets[bucket];

        while cur >= 0 {
            let idx = cur as usize;
            if self.pool[idx].used == 1 && self.pool[idx].task.id == id {
                self.pool[idx].task.status = new_status; // update the status
                return 0; // success
            }
            cur = self.pool[idx].next;
        }

        return -1; // not found
    }
}
