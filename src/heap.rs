#![allow(dead_code)]
// heap.rs - min-heap (priority queue) for scheduling
// this is the BRAIN of the scheduler. it always knows which task is the most urgent
// its called a "min-heap" because the smallest (most urgent) value is always at the top
// think of it like a tournament bracket but upside down - the winner (most urgent) sits at the root
//
// the heap is stored as a flat array but CONCEPTUALLY its a binary tree:
//       [most urgent]          <- root, index 0
//        /          \
//   [2nd most]   [3rd most]    <- index 1, index 2
//    /     \      /     \
//  [4th] [5th]  [6th]  [7th]   <- index 3, 4, 5, 6
//
// the trick to navigate without pointers:
//   parent of index i     = (i - 1) / 2
//   left child of index i = 2 * i + 1
//   right child of i      = 2 * i + 2
//
// ordered by urgency: rank -> deadline -> duration -> arrival order
// lower values = more urgent = closer to top

// max entries the heap can hold
const HEAP_CAP: usize = 1024;

// this is what actually gets stored in the heap
// its a lightweight version of a task - just the fields we need for sorting
// the full task details live in the hash table, we look them up by task_id when needed
#[derive(Clone, Copy)]
pub struct HeapEntry {
    pub rank: i32,      // 0=CRITICAL, 1=HIGH, 2=MED, 3=LOW (flipped from priority so lower = more urgent)
    pub deadline: i32,  // absolute deadline (arrival + relative deadline) for tiebreaking
    pub duration: i32,  // how long the task takes, shorter wins if everything else is tied
    pub order: i32,     // arrival order for fifo tiebreak, if EVERYTHING is the same whoever came first wins
    pub task_id: i32,   // which task this entry represents, used to look up full details in hash table
}

impl HeapEntry {
    // blank entry with all zeros, used to fill the array at initialization
    pub fn empty() -> HeapEntry {
        HeapEntry {
            rank: 0,
            deadline: 0,
            duration: 0,
            order: 0,
            task_id: 0,
        }
    }
}

pub struct Heap {
    data: [HeapEntry; HEAP_CAP],  // the array that holds all entries (the "tree" stored flat)
    count: usize,                  // how many entries are currently in the heap
}

// compare two entries - returns true if a is more urgent than b
// this is the core comparison that decides who goes first
// it checks 4 things in order, only moving to the next one if theres a tie:
//   1. priority rank -> CRITICAL (0) beats HIGH (1) beats MED (2) beats LOW (3)
//   2. deadline -> task that must finish sooner wins (tighter deadline = more urgent)
//   3. duration -> shorter task wins (get it done quick, clears the backlog faster)
//   4. arrival order -> whoever showed up first wins (FIFO fairness, so nothing starves)
fn is_higher(a: &HeapEntry, b: &HeapEntry) -> bool {
    if a.rank != b.rank {
        return a.rank < b.rank; // lower rank = higher priority, CRITICAL (0) beats LOW (3)
    }
    if a.deadline != b.deadline {
        return a.deadline < b.deadline; // earlier deadline = more urgent
    }
    if a.duration != b.duration {
        return a.duration < b.duration; // shorter task = quicker to finish = less blocking
    }
    return a.order < b.order;  // first come first served if literally everything else matches
}

impl Heap {
    // create an empty heap, array filled with blank entries, count at 0
    pub fn new() -> Heap {
        Heap {
            data: [HeapEntry::empty(); HEAP_CAP],
            count: 0,
        }
    }

    // check if the heap has no entries
    // used in the main loop to know if theres anything ready to schedule
    pub fn is_empty(&self) -> bool {
        return self.count == 0;
    }

    // insert a new entry into the heap
    // puts it at the bottom of the tree then "sifts up" to find its correct position
    // like a new employee who might be really good - they start at the bottom and get promoted
    // O(log n) because the tree has log2(n) levels, worst case we go from bottom to top
    pub fn insert(&mut self, entry: HeapEntry) -> i32 {
        if self.count >= HEAP_CAP {
            println!("  [heap] full, cant insert task {}", entry.task_id);
            return -1;
        }

        // stick it at the very end of the array (bottom of the tree)
        self.data[self.count] = entry;
        // now bubble it up to where it belongs based on urgency
        self.sift_up(self.count);
        self.count = self.count + 1;
        return 0;
    }

    // remove and return the most urgent entry (the root of the tree, index 0)
    // this is what the scheduler calls to get the next task to run
    // after removing the root we need to fix the heap:
    //   1. take the last element and put it at the root
    //   2. sift it down because its probably not urgent enough to be at the top
    // also O(log n) for the same reason - worst case sift from top to bottom
    pub fn extract_min(&mut self) -> HeapEntry {
        if self.count == 0 {
            // shouldnt happen if we check is_empty first, but just in case
            return HeapEntry::empty();
        }

        let min = self.data[0]; // save the root (most urgent entry)

        // move the last element to the root position
        // this breaks the heap property so we need to fix it
        self.count = self.count - 1;
        if self.count > 0 {
            self.data[0] = self.data[self.count]; // last element goes to root
            self.sift_down(0);                     // push it down to where it belongs
        }

        return min; // return the most urgent entry we saved
    }

    // sift_up - bubble an element UP the tree until the heap property is restored
    // called after insert() puts a new element at the bottom
    // compares with parent, if more urgent than parent -> swap and keep going up
    // stops when we reach the root (idx=0) or find a parent thats more urgent than us
    //
    // parent of any index is at (index - 1) / 2
    // example: index 5 -> parent is (5-1)/2 = 2
    //          index 1 -> parent is (1-1)/2 = 0 (the root)
    fn sift_up(&mut self, mut idx: usize) {
        while idx > 0 {
            let parent = (idx - 1) / 2;

            // if current element is more urgent than its parent, they need to swap
            if is_higher(&self.data[idx], &self.data[parent]) {
                let tmp = self.data[idx];
                self.data[idx] = self.data[parent];
                self.data[parent] = tmp;
                idx = parent; // move up to the parent position and check again
            } else {
                break; // parent is more urgent than us, we're in the right spot
            }
        }
    }

    // sift_down - push an element DOWN the tree until the heap property is restored
    // called after extract_min() puts a random element at the root
    // compares with both children, swaps with the MORE urgent child, keeps going down
    // stops when we have no children or both children are less urgent than us
    //
    // left child of any index is at  2 * index + 1
    // right child of any index is at 2 * index + 2
    // example: index 0 -> left=1, right=2
    //          index 3 -> left=7, right=8
    fn sift_down(&mut self, mut idx: usize) {
        loop {
            let left = 2 * idx + 1;   // left child index
            let right = 2 * idx + 2;  // right child index
            let mut smallest = idx;    // assume current is the most urgent for now

            // check if left child exists and is more urgent than current
            if left < self.count && is_higher(&self.data[left], &self.data[smallest]) {
                smallest = left;
            }

            // check if right child exists and is more urgent than current (or left)
            if right < self.count && is_higher(&self.data[right], &self.data[smallest]) {
                smallest = right;
            }

            // if one of the children is more urgent, swap with it and continue down
            if smallest != idx {
                let tmp = self.data[idx];
                self.data[idx] = self.data[smallest];
                self.data[smallest] = tmp;
                idx = smallest; // move down to where we swapped and check again
            } else {
                break; // we're more urgent than both children, we're in the right spot
            }
        }
    }
}
