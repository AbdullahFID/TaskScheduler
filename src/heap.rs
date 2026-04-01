#![allow(dead_code)]
// heap.rs - min-heap (priority queue) for scheduling
// ordered by urgency: rank -> deadline -> duration -> arrival order
// lower values = more urgent = closer to top

const HEAP_CAP: usize = 1024;

#[derive(Clone, Copy)]
pub struct HeapEntry {
    pub rank: i32,      // 0=CRITICAL, 1=HIGH, 2=MED, 3=LOW
    pub deadline: i32,  // absolute deadline for sorting
    pub duration: i32,
    pub order: i32,     // arrival order for fifo tiebreak
    pub task_id: i32,
}

impl HeapEntry {
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
    data: [HeapEntry; HEAP_CAP],
    count: usize,
}

// compare two entries - returns true if a is more urgent than b
// checks rank first, then deadline, then duration, then arrival order
fn is_higher(a: &HeapEntry, b: &HeapEntry) -> bool {
    if a.rank != b.rank {
        return a.rank < b.rank; // CRITICAL (0) beats LOW (3)
    }
    if a.deadline != b.deadline {
        return a.deadline < b.deadline; // tighter deadline wins
    }
    if a.duration != b.duration {
        return a.duration < b.duration; // shorter task wins
    }
    return a.order < b.order;  // arrived first wins (FIFO)
}

// 1. Priority rank —> CRITICAL always wins over LOW
// 2. Deadline —> if same priority, tighter deadline wins
// 3. Duration —> if same deadline, shorter task wins (quick win)
// 4. Arrival order —> if literally everything matches, whoever arrived first wins (FIFO fairness)

impl Heap {
    pub fn new() -> Heap {
        Heap {
            data: [HeapEntry::empty(); HEAP_CAP],
            count: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        return self.count == 0;
    }

    // insert a new entry into the heap
    pub fn insert(&mut self, entry: HeapEntry) -> i32 {
        if self.count >= HEAP_CAP {
            println!("  [heap] full, cant insert task {}", entry.task_id);
            return -1;
        }

        // put it at the end
        self.data[self.count] = entry;
        // sift it up to the right spot
        self.sift_up(self.count);
        self.count = self.count + 1;
        return 0;
    }

    // remove and return the most urgent entry (root)
    pub fn extract_min(&mut self) -> HeapEntry {
        if self.count == 0 {
            // shouldnt happen if we check is_empty first
            return HeapEntry::empty();
        }

        let min = self.data[0];

        // move last element to root and sift down
        self.count = self.count - 1;
        if self.count > 0 {
            self.data[0] = self.data[self.count];
            self.sift_down(0);
        }

        return min;
    }

    // bubble an element up until heap property is restored
    fn sift_up(&mut self, mut idx: usize) {
        while idx > 0 {
            let parent = (idx - 1) / 2;

            // if current is more urgent than parent, swap them
            if is_higher(&self.data[idx], &self.data[parent]) {
                let tmp = self.data[idx];
                self.data[idx] = self.data[parent];
                self.data[parent] = tmp;
                idx = parent;
            } else {
                break;
            }
        }
    }

    // push an element down until heap property is restored
    fn sift_down(&mut self, mut idx: usize) {
        loop {
            let left = 2 * idx + 1;
            let right = 2 * idx + 2;
            let mut smallest = idx;

            // check left child
            if left < self.count && is_higher(&self.data[left], &self.data[smallest]) {
                smallest = left;
            }

            // check right child
            if right < self.count && is_higher(&self.data[right], &self.data[smallest]) {
                smallest = right;
            }

            // if a child is more urgent, swap and keep going
            if smallest != idx {
                let tmp = self.data[idx];
                self.data[idx] = self.data[smallest];
                self.data[smallest] = tmp;
                idx = smallest;
            } else {
                break;
            }
        }
    }
}
